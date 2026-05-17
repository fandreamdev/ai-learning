"""应用配置 - 使用 Pydantic Settings 从 .env 加载。"""

from __future__ import annotations

from functools import lru_cache

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """应用全局配置。

    所有字段优先从环境变量读取，其次从 ``.env`` 文件读取。
    字段名不区分大小写。
    """

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    # ---------- 应用 ----------
    app_env: str = Field(default="development", description="运行环境")
    app_host: str = Field(default="0.0.0.0", description="监听地址")
    app_port: int = Field(default=8000, description="监听端口")
    app_debug: bool = Field(default=True, description="调试模式")

    # ---------- 数据库 ----------
    db_host: str = Field(default="localhost")
    db_port: int = Field(default=3306)
    db_user: str = Field(default="root")
    db_password: str = Field(default="")
    db_name: str = Field(default="project_alpha")
    db_name_test: str = Field(default="project_alpha_test")
    database_url: str | None = Field(
        default=None, description="SQLAlchemy 连接串，留空则按上述变量拼接"
    )

    # ---------- CORS ----------
    cors_allow_origins: str = Field(
        default="http://localhost:5173,http://127.0.0.1:5173",
        description="允许的 Origin，逗号分隔",
    )

    # ---------- 日志 ----------
    log_level: str = Field(default="INFO")

    @property
    def sqlalchemy_url(self) -> str:
        """生成 SQLAlchemy 连接串。"""
        if self.database_url:
            return self.database_url
        return (
            f"mysql+pymysql://{self.db_user}:{self.db_password}"
            f"@{self.db_host}:{self.db_port}/{self.db_name}?charset=utf8mb4"
        )

    @property
    def cors_origins_list(self) -> list[str]:
        """解析 CORS 允许列表。"""
        return [o.strip() for o in self.cors_allow_origins.split(",") if o.strip()]


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    """返回单例配置对象（带 LRU 缓存）。"""
    return Settings()


settings = get_settings()
