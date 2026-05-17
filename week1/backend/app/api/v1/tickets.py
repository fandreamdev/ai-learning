"""Ticket CRUD 路由。

挂载路径: ``/api/v1/tickets``。
对照 spec §5.2 的 API 契约。
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, Query, Response, status
from sqlalchemy.orm import Session

from app.api.deps import get_db_session
from app.api.v1._query_utils import parse_multi_csv
from app.core.exceptions import ValidationError
from app.schemas.common import ApiResponse, PageData
from app.schemas.ticket import (
    SortBy,
    SortOrder,
    StatusUpdate,
    TicketCreate,
    TicketListQuery,
    TicketPriority,
    TicketRead,
    TicketStatus,
    TicketUpdate,
)
from app.services import ticket_service
from app.utils.responses import paginated, success

router = APIRouter(prefix="/tickets", tags=["tickets"])


def _parse_enum_list(
    raw: list[str] | None,
    enum_cls: type[TicketStatus] | type[TicketPriority],
    field_name: str,
) -> list[TicketStatus] | list[TicketPriority]:
    """把字符串列表转成枚举列表，遇非法值抛 ValidationError(40001)。"""
    items = parse_multi_csv(raw)
    result: list[TicketStatus] | list[TicketPriority] = []
    for s in items:
        try:
            result.append(enum_cls(s))  # type: ignore[arg-type]
        except ValueError as exc:
            raise ValidationError(f"{field_name}: 非法取值 {s!r}") from exc
    return result


@router.get(
    "",
    response_model=ApiResponse[PageData[TicketRead]],
    summary="分页查询 Ticket 列表",
)
def list_tickets(
    status: list[str] | None = Query(
        default=None,
        description="按状态筛选；支持逗号分隔或重复参数",
    ),
    priority: list[str] | None = Query(
        default=None,
        description="按优先级筛选；支持逗号分隔或重复参数",
    ),
    assignee: str | None = Query(default=None, description="按负责人精确筛选"),
    tag: str | None = Query(default=None, description="按单个标签筛选"),
    keyword: str | None = Query(
        default=None,
        min_length=2,
        description="对标题/描述模糊搜索（最少 2 字符）",
    ),
    sort_by: SortBy = Query(default=SortBy.created_at, description="排序字段"),
    sort_order: SortOrder = Query(default=SortOrder.desc, description="排序方向"),
    page: int = Query(default=1, ge=1, description="页码，从 1 开始"),
    page_size: int = Query(default=20, ge=1, le=100, description="每页条数 1~100"),
    db: Session = Depends(get_db_session),
) -> ApiResponse[PageData[TicketRead]]:
    statuses = _parse_enum_list(status, TicketStatus, "status")
    priorities = _parse_enum_list(priority, TicketPriority, "priority")
    query = TicketListQuery(
        statuses=statuses,
        priorities=priorities,
        assignee=assignee or None,
        tag=tag or None,
        keyword=keyword or None,
        sort_by=sort_by,
        sort_order=sort_order,
        page=page,
        page_size=page_size,
    )
    items, total = ticket_service.list_tickets(db, query)
    return paginated(
        [TicketRead.model_validate(o) for o in items],
        total=total,
        page=query.page,
        page_size=query.page_size,
    )


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
