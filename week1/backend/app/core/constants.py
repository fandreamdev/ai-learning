"""业务常量。"""

from __future__ import annotations

from app.schemas.ticket import TicketStatus

# 状态流转表（spec §3.3）
STATUS_TRANSITIONS: dict[TicketStatus, frozenset[TicketStatus]] = {
    TicketStatus.open: frozenset({TicketStatus.in_progress, TicketStatus.closed}),
    TicketStatus.in_progress: frozenset({TicketStatus.done, TicketStatus.closed}),
    TicketStatus.done: frozenset({TicketStatus.closed}),
    TicketStatus.closed: frozenset({TicketStatus.open}),
}
