"""SQLAlchemy 引擎、Session 工厂与 Declarative Base。"""

from __future__ import annotations

from collections.abc import Generator

from sqlalchemy import create_engine
from sqlalchemy.engine import Engine
from sqlalchemy.orm import DeclarativeBase, Session, sessionmaker

from app.core.config import settings


class Base(DeclarativeBase):
    """所有 ORM 模型的基类。"""


def _build_engine() -> Engine:
    """构建 SQLAlchemy Engine。"""
    return create_engine(
        settings.sqlalchemy_url,
        pool_pre_ping=True,
        pool_recycle=3600,
        future=True,
        echo=False,
    )


engine: Engine = _build_engine()

SessionLocal = sessionmaker(
    bind=engine,
    autoflush=False,
    autocommit=False,
    expire_on_commit=False,
    class_=Session,
)


def get_db() -> Generator[Session]:
    """请求级数据库 Session 生成器。

    用法（在 FastAPI 依赖中）::

        from fastapi import Depends
        from app.core.database import get_db

        def my_route(db: Session = Depends(get_db)) -> None: ...
    """
    session: Session = SessionLocal()
    try:
        yield session
    finally:
        session.close()
