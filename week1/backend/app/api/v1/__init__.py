"""API v1 路由聚合。

后续阶段添加的 ticket / tag 路由都挂载到此 ``router`` 之下。
"""

from __future__ import annotations

from fastapi import APIRouter

router = APIRouter(prefix="/api/v1")
