"""API v1 路由聚合。

各业务子路由统一挂载到 ``/api/v1`` 前缀下。
"""

from __future__ import annotations

from fastapi import APIRouter

from app.api.v1 import tickets

router = APIRouter(prefix="/api/v1")
router.include_router(tickets.router)
