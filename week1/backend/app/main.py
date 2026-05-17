"""FastAPI 应用入口。

通过 ``create_app()`` 工厂构建实例，便于测试与多实例部署。
"""

from __future__ import annotations

from fastapi import FastAPI

from app.api import v1 as api_v1
from app.api.health import router as health_router
from app.core.config import settings
from app.core.logging import setup_logging
from app.middlewares.cors import add_cors_middleware
from app.middlewares.error_handler import register_exception_handlers


def create_app() -> FastAPI:
    """构建并返回 FastAPI 实例。"""
    setup_logging()

    app = FastAPI(
        title="ProjectAlpha API",
        version="0.1.0",
        description="轻量级 Ticket 管理工具 API",
        docs_url="/docs",
        redoc_url="/redoc",
        openapi_url="/openapi.json",
        debug=settings.app_debug,
    )

    # 中间件
    add_cors_middleware(app)

    # 异常处理
    register_exception_handlers(app)

    # 路由
    app.include_router(health_router)  # /healthz /readyz (无前缀)
    app.include_router(api_v1.router)  # /api/v1/*

    return app


app: FastAPI = create_app()
