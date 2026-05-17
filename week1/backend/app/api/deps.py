"""FastAPI 依赖注入函数集合。"""

from __future__ import annotations

from collections.abc import Generator

from sqlalchemy.orm import Session

from app.core.database import get_db


def get_db_session() -> Generator[Session]:
    """请求级 DB Session 依赖。

    用法::

        from fastapi import Depends
        from app.api.deps import get_db_session

        @router.get("/items")
        def list_items(db: Session = Depends(get_db_session)) -> ...: ...
    """
    yield from get_db()
