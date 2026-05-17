"""pytest 共用 fixture。

设计要点：
- 使用独立的 ``project_alpha_test`` 数据库（已通过 ``alembic upgrade head`` 建表）。
- 每个用例在独立事务中执行，结束自动回滚，互不污染。
- ``client`` fixture 通过 ``app.dependency_overrides`` 让路由复用同一个 Session。
"""

from __future__ import annotations

from collections.abc import Generator

import pytest
from app.api.deps import get_db_session
from app.core.config import settings
from app.main import app
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.engine import Engine
from sqlalchemy.orm import Session


def _build_test_url() -> str:
    """生成测试库连接串（基于 settings.db_name_test 替换业务库名）。"""
    base = settings.sqlalchemy_url
    if settings.db_name_test in base:
        return base
    return base.replace(f"/{settings.db_name}?", f"/{settings.db_name_test}?", 1)


@pytest.fixture(scope="session")
def test_engine() -> Generator[Engine]:
    """整个测试 session 复用同一个 engine。"""
    eng = create_engine(_build_test_url(), pool_pre_ping=True, future=True)
    yield eng
    eng.dispose()


@pytest.fixture
def db_session(test_engine: Engine) -> Generator[Session]:
    """事务回滚式 DB Session — 每个用例独立隔离。

    使用 SAVEPOINT (``join_transaction_mode="create_savepoint"``) 让
    Service 层的 ``db.commit()`` 仅提交内层 SAVEPOINT，外层事务
    始终保持可回滚。
    """
    connection = test_engine.connect()
    transaction = connection.begin()
    session = Session(
        bind=connection,
        expire_on_commit=False,
        join_transaction_mode="create_savepoint",
    )
    try:
        yield session
    finally:
        session.close()
        if transaction.is_active:
            transaction.rollback()
        connection.close()


@pytest.fixture
def client(db_session: Session) -> Generator[TestClient]:
    """复用同一 db_session 的 TestClient，避免提交后无法回滚。"""

    def _override() -> Generator[Session]:
        # 注意：这里需要让路由通过 commit/refresh 后仍走同一连接，
        # 同时保留外层事务的回滚能力。SQLAlchemy 通过 SAVEPOINT 实现。
        yield db_session

    app.dependency_overrides[get_db_session] = _override
    with TestClient(app) as c:
        yield c
    app.dependency_overrides.clear()
