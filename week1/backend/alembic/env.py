"""Alembic 迁移环境配置。

从 :mod:`app.core.config` 读取数据库连接串，
并将 :class:`app.models.Base.metadata` 作为 ``target_metadata``，
以便后续阶段使用 ``--autogenerate``。
"""

from __future__ import annotations

from logging.config import fileConfig

from alembic import context

# 加载 app 包的设置与模型
from app.core.config import settings
from app.models import Base  # noqa: F401  # 确保模型注册到 Base.metadata
from sqlalchemy import engine_from_config, pool

# 这是 Alembic Config 对象，提供 .ini 文件中的访问能力
config = context.config

# 注入运行时数据库连接串（覆盖 alembic.ini 中的空值）
config.set_main_option("sqlalchemy.url", settings.sqlalchemy_url)

# 解析日志配置
if config.config_file_name is not None:
    fileConfig(config.config_file_name)

# autogenerate 需要的元数据
target_metadata = Base.metadata


def run_migrations_offline() -> None:
    """以离线模式运行迁移（仅产生 SQL，不连接数据库）。"""
    url = config.get_main_option("sqlalchemy.url")
    context.configure(
        url=url,
        target_metadata=target_metadata,
        literal_binds=True,
        dialect_opts={"paramstyle": "named"},
        compare_type=True,
    )

    with context.begin_transaction():
        context.run_migrations()


def run_migrations_online() -> None:
    """以在线模式运行迁移（实际连接数据库执行）。"""
    connectable = engine_from_config(
        config.get_section(config.config_ini_section, {}),
        prefix="sqlalchemy.",
        poolclass=pool.NullPool,
    )

    with connectable.connect() as connection:
        context.configure(
            connection=connection,
            target_metadata=target_metadata,
            compare_type=True,
        )

        with context.begin_transaction():
            context.run_migrations()


if context.is_offline_mode():
    run_migrations_offline()
else:
    run_migrations_online()
