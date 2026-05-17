"""ticket_service 单元测试（走真实 MySQL，事务回滚隔离）。

覆盖 plan §2.4 用例 UT-T2-01~09。
"""

from __future__ import annotations

import pytest
from app.core.exceptions import InvalidStatusTransition, TicketNotFound
from app.schemas.ticket import (
    TicketCreate,
    TicketPriority,
    TicketStatus,
    TicketUpdate,
)
from app.services import ticket_service
from sqlalchemy.orm import Session

# ---------- create ----------


def test_create_returns_persisted_ticket(db_session: Session) -> None:
    obj = ticket_service.create_ticket(
        db_session,
        TicketCreate(title="hello", priority=TicketPriority.high, tags=["bug"]),
    )
    assert obj.id is not None
    assert obj.title == "hello"
    assert obj.status == "open"  # 数据库默认值
    assert obj.priority == "high"
    assert obj.tags == ["bug"]
    assert obj.created_at is not None
    assert obj.updated_at is not None


def test_create_with_default_priority(db_session: Session) -> None:
    obj = ticket_service.create_ticket(db_session, TicketCreate(title="t"))
    assert obj.priority == "medium"
    assert obj.tags == []


# ---------- get ----------


def test_get_existing(db_session: Session) -> None:
    created = ticket_service.create_ticket(db_session, TicketCreate(title="x"))
    fetched = ticket_service.get_ticket(db_session, created.id)
    assert fetched.id == created.id


def test_get_missing_raises(db_session: Session) -> None:
    with pytest.raises(TicketNotFound):
        ticket_service.get_ticket(db_session, ticket_id=999_999)


# ---------- update ----------


def test_update_partial_keeps_other_fields(db_session: Session) -> None:
    created = ticket_service.create_ticket(
        db_session,
        TicketCreate(title="orig", priority=TicketPriority.low, tags=["a"]),
    )
    updated = ticket_service.update_ticket(db_session, created.id, TicketUpdate(title="new"))
    assert updated.title == "new"
    assert updated.priority == "low"
    assert updated.tags == ["a"]


def test_update_status_legal(db_session: Session) -> None:
    created = ticket_service.create_ticket(db_session, TicketCreate(title="t"))
    updated = ticket_service.update_ticket(
        db_session, created.id, TicketUpdate(status=TicketStatus.in_progress)
    )
    assert updated.status == "in_progress"


def test_update_status_illegal_raises(db_session: Session) -> None:
    created = ticket_service.create_ticket(db_session, TicketCreate(title="t"))
    # open -> done 非法
    with pytest.raises(InvalidStatusTransition):
        ticket_service.update_ticket(db_session, created.id, TicketUpdate(status=TicketStatus.done))


def test_update_missing_raises(db_session: Session) -> None:
    with pytest.raises(TicketNotFound):
        ticket_service.update_ticket(db_session, 999_999, TicketUpdate(title="x"))


# ---------- update_status ----------


@pytest.mark.parametrize(
    "from_status,to_status",
    [
        (TicketStatus.open, TicketStatus.in_progress),
        (TicketStatus.open, TicketStatus.closed),
        (TicketStatus.in_progress, TicketStatus.done),
        (TicketStatus.in_progress, TicketStatus.closed),
        (TicketStatus.done, TicketStatus.closed),
        (TicketStatus.closed, TicketStatus.open),
    ],
)
def test_status_transition_legal(
    db_session: Session, from_status: TicketStatus, to_status: TicketStatus
) -> None:
    obj = ticket_service.create_ticket(db_session, TicketCreate(title="t"))
    # 通过链式合法流转把 obj 推到 from_status
    if obj.status != from_status.value:
        # 直接绕过校验，模拟数据库初始状态
        obj.status = from_status.value
        db_session.commit()
    updated = ticket_service.update_status(db_session, obj.id, to_status)
    assert updated.status == to_status.value


@pytest.mark.parametrize(
    "from_status,to_status",
    [
        (TicketStatus.open, TicketStatus.done),
        (TicketStatus.in_progress, TicketStatus.open),
        (TicketStatus.done, TicketStatus.in_progress),
        (TicketStatus.done, TicketStatus.open),
        (TicketStatus.closed, TicketStatus.in_progress),
        (TicketStatus.closed, TicketStatus.done),
    ],
)
def test_status_transition_illegal(
    db_session: Session, from_status: TicketStatus, to_status: TicketStatus
) -> None:
    obj = ticket_service.create_ticket(db_session, TicketCreate(title="t"))
    if obj.status != from_status.value:
        obj.status = from_status.value
        db_session.commit()
    with pytest.raises(InvalidStatusTransition):
        ticket_service.update_status(db_session, obj.id, to_status)


# ---------- delete ----------


def test_delete_existing(db_session: Session) -> None:
    created = ticket_service.create_ticket(db_session, TicketCreate(title="t"))
    ticket_service.delete_ticket(db_session, created.id)
    with pytest.raises(TicketNotFound):
        ticket_service.get_ticket(db_session, created.id)


def test_delete_missing_raises(db_session: Session) -> None:
    with pytest.raises(TicketNotFound):
        ticket_service.delete_ticket(db_session, 999_999)
