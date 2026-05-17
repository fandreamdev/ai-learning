"""TicketListQuery 与 parse_multi_csv 单元测试。"""

from __future__ import annotations

import pytest
from app.api.v1._query_utils import parse_multi_csv
from app.schemas.ticket import SortBy, SortOrder, TicketListQuery, TicketPriority, TicketStatus
from pydantic import ValidationError

# ---------- parse_multi_csv ----------


def test_parse_multi_csv_none_returns_empty() -> None:
    assert parse_multi_csv(None) == []
    assert parse_multi_csv([]) == []


def test_parse_multi_csv_single_value() -> None:
    assert parse_multi_csv(["open"]) == ["open"]


def test_parse_multi_csv_comma_separated() -> None:
    assert parse_multi_csv(["open,in_progress"]) == ["open", "in_progress"]


def test_parse_multi_csv_repeated_param() -> None:
    assert parse_multi_csv(["open", "in_progress"]) == ["open", "in_progress"]


def test_parse_multi_csv_mixed() -> None:
    assert parse_multi_csv(["open,in_progress", "done"]) == [
        "open",
        "in_progress",
        "done",
    ]


def test_parse_multi_csv_dedup_preserves_order() -> None:
    assert parse_multi_csv(["open", "open,in_progress", "in_progress"]) == [
        "open",
        "in_progress",
    ]


def test_parse_multi_csv_strips_whitespace() -> None:
    assert parse_multi_csv([" open , in_progress "]) == ["open", "in_progress"]


def test_parse_multi_csv_skips_empty_segments() -> None:
    assert parse_multi_csv(["open,,in_progress"]) == ["open", "in_progress"]


# ---------- TicketListQuery ----------


def test_default_query() -> None:
    q = TicketListQuery()
    assert q.statuses == []
    assert q.priorities == []
    assert q.sort_by == SortBy.created_at
    assert q.sort_order == SortOrder.desc
    assert q.page == 1
    assert q.page_size == 20


def test_query_with_filters() -> None:
    q = TicketListQuery(
        statuses=[TicketStatus.open, TicketStatus.in_progress],
        priorities=[TicketPriority.urgent],
        assignee="zhang",
        tag="bug",
        keyword="login",
    )
    assert len(q.statuses) == 2
    assert q.assignee == "zhang"


def test_query_page_must_be_at_least_1() -> None:
    with pytest.raises(ValidationError):
        TicketListQuery(page=0)


def test_query_page_size_min() -> None:
    with pytest.raises(ValidationError):
        TicketListQuery(page_size=0)


def test_query_page_size_max() -> None:
    with pytest.raises(ValidationError):
        TicketListQuery(page_size=101)


def test_query_invalid_sort_by() -> None:
    with pytest.raises(ValidationError):
        TicketListQuery(sort_by="title")  # type: ignore[arg-type]


def test_query_invalid_sort_order() -> None:
    with pytest.raises(ValidationError):
        TicketListQuery(sort_order="random")  # type: ignore[arg-type]
