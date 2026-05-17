"""聚合查询业务逻辑。

提供 ``/api/v1/tags`` 与 ``/api/v1/assignees`` 接口所需的聚合数据，
直接从 ``tickets`` 表汇总，避免引入额外的标签/用户表。
"""

from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.orm import Session

from app.models.ticket import Ticket


def list_tags(db: Session) -> list[str]:
    """聚合所有 ticket.tags 的并集，去重 + 字典序排序。"""
    rows = db.scalars(select(Ticket.tags)).all()
    seen: set[str] = set()
    result: list[str] = []
    for tags in rows:
        if not tags:
            continue
        for tag in tags:
            if not isinstance(tag, str):
                continue
            if tag not in seen:
                seen.add(tag)
                result.append(tag)
    result.sort()
    return result


def list_assignees(db: Session) -> list[str]:
    """聚合所有非空负责人，去重 + 字典序排序。"""
    rows = db.scalars(select(Ticket.assignee).where(Ticket.assignee.is_not(None)).distinct()).all()
    return sorted(name for name in rows if name)
