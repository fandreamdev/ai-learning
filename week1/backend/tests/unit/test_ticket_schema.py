"""Ticket Pydantic Schema 单元测试。

覆盖 spec §4 / §5.2.3 / §3.4 中的字段约束：
- title 必填、空白被 strip 后判空、长度 ≤ 200
- priority / status 必须为合法枚举
- tags 长度上限、单项长度、自动 strip + lower + 去重
- assignee 长度 ≤ 100
"""

from __future__ import annotations

import pytest
from app.schemas.ticket import (
    StatusUpdate,
    TicketCreate,
    TicketPriority,
    TicketStatus,
    TicketUpdate,
)
from pydantic import ValidationError

# ---------- TicketCreate ----------


def test_create_minimal_ok() -> None:
    obj = TicketCreate(title="hello")
    assert obj.title == "hello"
    assert obj.priority == TicketPriority.medium
    assert obj.tags == []
    assert obj.assignee is None
    assert obj.description is None


def test_create_full_ok() -> None:
    obj = TicketCreate(
        title="  bug fix  ",
        description="...",
        priority=TicketPriority.high,
        assignee="zhang",
        tags=["Bug", "frontend", "BUG", " backend "],
    )
    # title strip
    assert obj.title == "bug fix"
    # tags 转小写、strip、去重
    assert obj.tags == ["bug", "frontend", "backend"]


def test_create_title_required() -> None:
    with pytest.raises(ValidationError):
        TicketCreate()  # type: ignore[call-arg]


def test_create_title_blank_rejected() -> None:
    with pytest.raises(ValidationError):
        TicketCreate(title="   ")


def test_create_title_too_long() -> None:
    with pytest.raises(ValidationError):
        TicketCreate(title="x" * 201)


def test_create_priority_invalid() -> None:
    with pytest.raises(ValidationError):
        TicketCreate(title="a", priority="super_urgent")  # type: ignore[arg-type]


def test_create_assignee_too_long() -> None:
    with pytest.raises(ValidationError):
        TicketCreate(title="a", assignee="x" * 101)


def test_create_tags_too_many() -> None:
    with pytest.raises(ValidationError):
        TicketCreate(title="a", tags=[f"t{i}" for i in range(11)])


def test_create_tag_too_long() -> None:
    with pytest.raises(ValidationError):
        TicketCreate(title="a", tags=["a" * 21])


def test_create_tag_empty_rejected() -> None:
    with pytest.raises(ValidationError):
        TicketCreate(title="a", tags=[" "])


def test_create_tag_must_be_string() -> None:
    with pytest.raises(ValidationError):
        TicketCreate(title="a", tags=[123])  # type: ignore[list-item]


# ---------- TicketUpdate ----------


def test_update_all_optional() -> None:
    obj = TicketUpdate()
    assert obj.title is None
    assert obj.tags is None


def test_update_status_must_be_enum() -> None:
    with pytest.raises(ValidationError):
        TicketUpdate(status="weird")  # type: ignore[arg-type]


def test_update_partial_only_some_fields() -> None:
    obj = TicketUpdate(priority=TicketPriority.urgent)
    dumped = obj.model_dump(exclude_unset=True)
    assert dumped == {"priority": TicketPriority.urgent}


def test_update_tags_normalize() -> None:
    obj = TicketUpdate(tags=["A", "a", "B"])
    assert obj.tags == ["a", "b"]


def test_update_blank_title_rejected() -> None:
    with pytest.raises(ValidationError):
        TicketUpdate(title="   ")


# ---------- StatusUpdate ----------


def test_status_update_ok() -> None:
    payload = StatusUpdate(status=TicketStatus.in_progress)
    assert payload.status == TicketStatus.in_progress


def test_status_update_invalid() -> None:
    with pytest.raises(ValidationError):
        StatusUpdate(status="foo")  # type: ignore[arg-type]
