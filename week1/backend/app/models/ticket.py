"""Ticket ORM 模型。

字段定义对照 ``specs/week1/0001-spec.md`` §4.1 与 §10.1。
"""

from __future__ import annotations

from datetime import datetime

from sqlalchemy import DateTime, Enum, Index, String, Text, func
from sqlalchemy.dialects.mysql import INTEGER, JSON
from sqlalchemy.orm import Mapped, mapped_column

from app.core.database import Base


class Ticket(Base):
    """工单实体。"""

    __tablename__ = "tickets"

    id: Mapped[int] = mapped_column(
        INTEGER(unsigned=True),
        primary_key=True,
        autoincrement=True,
        comment="主键",
    )
    title: Mapped[str] = mapped_column(
        String(200),
        nullable=False,
        comment="Ticket 标题",
    )
    description: Mapped[str | None] = mapped_column(
        Text,
        nullable=True,
        comment="详细描述",
    )
    status: Mapped[str] = mapped_column(
        Enum(
            "open",
            "in_progress",
            "done",
            "closed",
            name="ticket_status",
            native_enum=True,
        ),
        nullable=False,
        default="open",
        server_default="open",
        index=True,
        comment="状态",
    )
    priority: Mapped[str] = mapped_column(
        Enum(
            "low",
            "medium",
            "high",
            "urgent",
            name="ticket_priority",
            native_enum=True,
        ),
        nullable=False,
        default="medium",
        server_default="medium",
        index=True,
        comment="优先级",
    )
    assignee: Mapped[str | None] = mapped_column(
        String(100),
        nullable=True,
        index=True,
        comment="负责人",
    )
    tags: Mapped[list[str] | None] = mapped_column(
        JSON,
        nullable=True,
        default=list,
        comment="标签列表",
    )
    created_at: Mapped[datetime] = mapped_column(
        DateTime,
        nullable=False,
        server_default=func.now(),
        index=True,
        comment="创建时间",
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime,
        nullable=False,
        server_default=func.now(),
        onupdate=func.now(),
        index=True,
        comment="更新时间",
    )

    __table_args__ = (
        Index("ft_title_desc", "title", "description", mysql_prefix="FULLTEXT"),
        {
            "mysql_engine": "InnoDB",
            "mysql_charset": "utf8mb4",
            "mysql_collate": "utf8mb4_unicode_ci",
            "comment": "Ticket 工单表",
        },
    )

    def __repr__(self) -> str:
        return f"<Ticket id={self.id!r} title={self.title!r} status={self.status!r}>"
