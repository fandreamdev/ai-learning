"""健康检查路由。

- ``/healthz``  存活探针：仅返回应用是否运行
- ``/readyz``   就绪探针：包含数据库连通性检测
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, status
from sqlalchemy import text
from sqlalchemy.orm import Session

from app.api.deps import get_db_session
from app.schemas.common import ApiResponse

router = APIRouter(tags=["health"])


@router.get("/healthz", response_model=ApiResponse[str])
def healthz() -> ApiResponse[str]:
    """存活检测。"""
    return ApiResponse[str](data="ok")


@router.get("/readyz", response_model=ApiResponse[str])
def readyz(db: Session = Depends(get_db_session)) -> ApiResponse[str]:
    """就绪检测：附带一次数据库 ping。"""
    db.execute(text("SELECT 1"))
    return ApiResponse[str](data="ready")


__all__ = ["router", "status"]
