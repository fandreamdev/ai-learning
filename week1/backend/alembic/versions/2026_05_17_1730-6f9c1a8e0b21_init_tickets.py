"""init tickets table

Revision ID: 6f9c1a8e0b21
Revises:
Create Date: 2026-05-17 17:30:00

本迁移代表 ``tickets`` 表的 baseline。与 spec §10.1 DDL 1:1 对齐。

对于业务库 ``project_alpha``：表由阶段 0 ``init.sql`` 直接建立，
应使用 ``alembic stamp head`` 将本迁移标记为"已应用"，避免重复 DDL。

对于测试库 ``project_alpha_test``：执行 ``alembic upgrade head`` 即可。
"""

from __future__ import annotations

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import mysql

# revision identifiers, used by Alembic.
revision: str = "6f9c1a8e0b21"
down_revision: str | None = None
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        "tickets",
        sa.Column(
            "id",
            mysql.INTEGER(unsigned=True),
            autoincrement=True,
            nullable=False,
            comment="主键",
        ),
        sa.Column(
            "title",
            sa.String(length=200),
            nullable=False,
            comment="Ticket 标题",
        ),
        sa.Column(
            "description",
            sa.Text(),
            nullable=True,
            comment="详细描述",
        ),
        sa.Column(
            "status",
            sa.Enum(
                "open",
                "in_progress",
                "done",
                "closed",
                name="ticket_status",
            ),
            nullable=False,
            server_default="open",
            comment="状态",
        ),
        sa.Column(
            "priority",
            sa.Enum(
                "low",
                "medium",
                "high",
                "urgent",
                name="ticket_priority",
            ),
            nullable=False,
            server_default="medium",
            comment="优先级",
        ),
        sa.Column(
            "assignee",
            sa.String(length=100),
            nullable=True,
            comment="负责人",
        ),
        sa.Column(
            "tags",
            sa.JSON(),
            nullable=True,
            comment="标签列表",
        ),
        sa.Column(
            "created_at",
            sa.DateTime(),
            nullable=False,
            server_default=sa.text("CURRENT_TIMESTAMP"),
            comment="创建时间",
        ),
        sa.Column(
            "updated_at",
            sa.DateTime(),
            nullable=False,
            server_default=sa.text("CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP"),
            comment="更新时间",
        ),
        sa.PrimaryKeyConstraint("id"),
        comment="Ticket 工单表",
        mysql_engine="InnoDB",
        mysql_charset="utf8mb4",
        mysql_collate="utf8mb4_unicode_ci",
    )

    # 普通索引
    op.create_index("idx_status", "tickets", ["status"], unique=False)
    op.create_index("idx_priority", "tickets", ["priority"], unique=False)
    op.create_index("idx_assignee", "tickets", ["assignee"], unique=False)
    op.create_index("idx_created_at", "tickets", ["created_at"], unique=False)
    op.create_index("idx_updated_at", "tickets", ["updated_at"], unique=False)

    # 全文索引（Alembic autogenerate 对 FULLTEXT 支持有限，这里使用原生 SQL）
    op.execute("CREATE FULLTEXT INDEX ft_title_desc ON tickets (title, description)")


def downgrade() -> None:
    op.execute("DROP INDEX ft_title_desc ON tickets")
    op.drop_index("idx_updated_at", table_name="tickets")
    op.drop_index("idx_created_at", table_name="tickets")
    op.drop_index("idx_assignee", table_name="tickets")
    op.drop_index("idx_priority", table_name="tickets")
    op.drop_index("idx_status", table_name="tickets")
    op.drop_table("tickets")
