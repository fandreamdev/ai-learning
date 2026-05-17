"""Ticket CRUD 路由。

挂载路径: ``/api/v1/tickets``。
对照 spec §5.2 的 API 契约（GET 列表留待阶段 3）。
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, Response, status
from sqlalchemy.orm import Session

from app.api.deps import get_db_session
from app.schemas.common import ApiResponse
from app.schemas.ticket import StatusUpdate, TicketCreate, TicketRead, TicketUpdate
from app.services import ticket_service
from app.utils.responses import success

router = APIRouter(prefix="/tickets", tags=["tickets"])


@router.post(
    "",
    response_model=ApiResponse[TicketRead],
    status_code=status.HTTP_201_CREATED,
    summary="创建 Ticket",
)
def create_ticket(
    data: TicketCreate,
    db: Session = Depends(get_db_session),
) -> ApiResponse[TicketRead]:
    obj = ticket_service.create_ticket(db, data)
    return success(TicketRead.model_validate(obj), message="created")


@router.get(
    "/{ticket_id}",
    response_model=ApiResponse[TicketRead],
    summary="获取 Ticket 详情",
)
def get_ticket(
    ticket_id: int,
    db: Session = Depends(get_db_session),
) -> ApiResponse[TicketRead]:
    obj = ticket_service.get_ticket(db, ticket_id)
    return success(TicketRead.model_validate(obj))


@router.put(
    "/{ticket_id}",
    response_model=ApiResponse[TicketRead],
    summary="更新 Ticket（部分字段）",
)
def update_ticket(
    ticket_id: int,
    data: TicketUpdate,
    db: Session = Depends(get_db_session),
) -> ApiResponse[TicketRead]:
    obj = ticket_service.update_ticket(db, ticket_id, data)
    return success(TicketRead.model_validate(obj))


@router.patch(
    "/{ticket_id}/status",
    response_model=ApiResponse[TicketRead],
    summary="切换 Ticket 状态",
)
def patch_ticket_status(
    ticket_id: int,
    payload: StatusUpdate,
    db: Session = Depends(get_db_session),
) -> ApiResponse[TicketRead]:
    obj = ticket_service.update_status(db, ticket_id, payload.status)
    return success(TicketRead.model_validate(obj))


@router.delete(
    "/{ticket_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="删除 Ticket",
)
def delete_ticket(
    ticket_id: int,
    db: Session = Depends(get_db_session),
) -> Response:
    ticket_service.delete_ticket(db, ticket_id)
    return Response(status_code=status.HTTP_204_NO_CONTENT)
