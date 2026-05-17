"""聚合查询路由：``/api/v1/tags`` 与 ``/api/v1/assignees``。

对应 spec §5.3 的辅助接口。
"""

from __future__ import annotations

from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session

from app.api.deps import get_db_session
from app.schemas.common import ApiResponse
from app.services import aggregation_service
from app.utils.responses import success

router = APIRouter(tags=["aggregations"])


@router.get(
    "/tags",
    response_model=ApiResponse[list[str]],
    summary="获取所有已使用的标签",
)
def list_tags(db: Session = Depends(get_db_session)) -> ApiResponse[list[str]]:
    return success(aggregation_service.list_tags(db))


@router.get(
    "/assignees",
    response_model=ApiResponse[list[str]],
    summary="获取所有已使用的负责人",
)
def list_assignees(db: Session = Depends(get_db_session)) -> ApiResponse[list[str]]:
    return success(aggregation_service.list_assignees(db))
