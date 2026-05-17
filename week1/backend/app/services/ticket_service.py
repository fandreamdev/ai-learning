"""Ticket 业务逻辑。

设计原则：
- 函数式风格，每个函数接受 ``Session`` 与领域入参，不依赖请求上下文
- 抛 ``BusinessException`` 子类，由全局异常处理器转换为 HTTP 响应
- 不做参数二次校验（已由 Pydantic Schema 保证）
"""

from __future__ import annotations

from sqlalchemy import ColumnElement, func, or_, select
from sqlalchemy.orm import Session

from app.core.constants import STATUS_TRANSITIONS
from app.core.exceptions import InvalidStatusTransition, TicketNotFound
from app.models.ticket import Ticket
from app.schemas.ticket import (
    SortBy,
    SortOrder,
    TicketCreate,
    TicketListQuery,
    TicketStatus,
    TicketUpdate,
)

# ---------------------------------------------------------------------------
# 内部辅助
# ---------------------------------------------------------------------------


def _validate_transition(current: str, target: TicketStatus) -> None:
    """校验状态流转是否合法（spec §3.3）。"""
    try:
        current_enum = TicketStatus(current)
    except ValueError as exc:
        raise InvalidStatusTransition(f"当前状态 {current!r} 非法") from exc

    if current_enum == target:
        return  # 同状态视为幂等

    allowed = STATUS_TRANSITIONS.get(current_enum, frozenset())
    if target not in allowed:
        raise InvalidStatusTransition(f"不能从 {current_enum.value} 流转到 {target.value}")


def _get_ticket_or_404(db: Session, ticket_id: int) -> Ticket:
    obj = db.get(Ticket, ticket_id)
    if obj is None:
        raise TicketNotFound(f"Ticket #{ticket_id} 不存在")
    return obj


# ---------------------------------------------------------------------------
# 公开 API
# ---------------------------------------------------------------------------


def create_ticket(db: Session, data: TicketCreate) -> Ticket:
    """创建 Ticket。"""
    ticket = Ticket(
        title=data.title,
        description=data.description,
        priority=data.priority.value,
        assignee=data.assignee,
        tags=list(data.tags),
        # status 走数据库默认值 "open"
    )
    db.add(ticket)
    db.commit()
    db.refresh(ticket)
    return ticket


def get_ticket(db: Session, ticket_id: int) -> Ticket:
    """获取单个 Ticket，找不到抛 TicketNotFound。"""
    return _get_ticket_or_404(db, ticket_id)


def update_ticket(db: Session, ticket_id: int, data: TicketUpdate) -> Ticket:
    """部分更新 Ticket。

    若 ``data`` 中包含 ``status``，会执行状态流转校验。
    """
    ticket = _get_ticket_or_404(db, ticket_id)

    payload = data.model_dump(exclude_unset=True)

    # 状态流转校验
    if "status" in payload and payload["status"] is not None:
        target = TicketStatus(payload["status"])
        _validate_transition(ticket.status, target)
        ticket.status = target.value

    # 其他字段
    for field in ("title", "description", "priority", "assignee", "tags"):
        if field not in payload:
            continue
        value = payload[field]
        if field == "priority" and value is not None:
            value = value.value if hasattr(value, "value") else value
        setattr(ticket, field, value)

    db.commit()
    db.refresh(ticket)
    return ticket


def update_status(db: Session, ticket_id: int, target: TicketStatus) -> Ticket:
    """专用状态切换接口。"""
    ticket = _get_ticket_or_404(db, ticket_id)
    _validate_transition(ticket.status, target)
    ticket.status = target.value
    db.commit()
    db.refresh(ticket)
    return ticket


def delete_ticket(db: Session, ticket_id: int) -> None:
    """删除 Ticket。"""
    ticket = _get_ticket_or_404(db, ticket_id)
    db.delete(ticket)
    db.commit()


def count_tickets(db: Session) -> int:
    """工具函数：统计 ticket 数量（主要供测试与调试使用）。"""
    result = db.scalar(select(func.count()).select_from(Ticket))
    return int(result or 0)


# ---------------------------------------------------------------------------
# 列表查询
# ---------------------------------------------------------------------------


def _build_list_filters(q: TicketListQuery) -> list[ColumnElement[bool]]:
    """根据查询参数生成 WHERE 子句列表。"""
    conds: list[ColumnElement[bool]] = []
    if q.statuses:
        conds.append(Ticket.status.in_([s.value for s in q.statuses]))
    if q.priorities:
        conds.append(Ticket.priority.in_([p.value for p in q.priorities]))
    if q.assignee:
        conds.append(Ticket.assignee == q.assignee)
    if q.tag:
        # MySQL: JSON_CONTAINS(tags, JSON_QUOTE('bug'))
        conds.append(func.json_contains(Ticket.tags, func.json_quote(q.tag)) == 1)
    if q.keyword:
        like = f"%{q.keyword}%"
        conds.append(or_(Ticket.title.like(like), Ticket.description.like(like)))
    return conds


def list_tickets(db: Session, q: TicketListQuery) -> tuple[list[Ticket], int]:
    """筛选 + 搜索 + 排序 + 分页地查询 Ticket 列表。

    返回 ``(items, total)``，便于上层组装分页结构。
    """
    conds = _build_list_filters(q)

    stmt = select(Ticket)
    count_stmt = select(func.count()).select_from(Ticket)
    if conds:
        stmt = stmt.where(*conds)
        count_stmt = count_stmt.where(*conds)

    # 主排序
    sort_col = Ticket.created_at if q.sort_by == SortBy.created_at else Ticket.updated_at
    sort_expr = sort_col.desc() if q.sort_order == SortOrder.desc else sort_col.asc()
    # 次要排序：id desc 保证稳定
    stmt = stmt.order_by(sort_expr, Ticket.id.desc())

    # 分页
    offset = (q.page - 1) * q.page_size
    stmt = stmt.offset(offset).limit(q.page_size)

    items = list(db.scalars(stmt).all())
    total = int(db.scalar(count_stmt) or 0)
    return items, total
