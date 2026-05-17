"""ticket_service.list_tickets 单元测试。"""

from __future__ import annotations

from app.schemas.ticket import (
    SortBy,
    SortOrder,
    TicketCreate,
    TicketListQuery,
    TicketPriority,
    TicketStatus,
)
from app.services import ticket_service
from sqlalchemy.orm import Session


def _seed(db: Session) -> None:
    """在测试事务中预置 5 条 Ticket 数据，覆盖各种状态/优先级/标签/负责人。"""
    samples = [
        TicketCreate(
            title="fix login captcha",
            description="captcha not refreshing on login page",
            priority=TicketPriority.high,
            assignee="zhang",
            tags=["bug", "frontend"],
        ),
        TicketCreate(
            title="add export button",
            description="users want to export tickets to csv",
            priority=TicketPriority.medium,
            assignee="li",
            tags=["feat"],
        ),
        TicketCreate(
            title="optimize query",
            description="list endpoint is slow",
            priority=TicketPriority.low,
            assignee="wang",
            tags=["perf", "backend"],
        ),
        TicketCreate(
            title="critical security patch",
            description="urgent CVE",
            priority=TicketPriority.urgent,
            assignee="zhang",
            tags=["bug", "security"],
        ),
        TicketCreate(
            title="docs typo",
            description=None,
            priority=TicketPriority.low,
            assignee=None,
            tags=[],
        ),
    ]
    created = []
    for s in samples:
        created.append(ticket_service.create_ticket(db, s))

    # 把第二条改成 in_progress 状态以覆盖多状态过滤
    ticket_service.update_status(db, created[1].id, TicketStatus.in_progress)


# ---------- 列表 ----------


def test_list_no_filter_returns_all_desc(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(db_session, TicketListQuery())
    assert total == 5
    assert len(items) == 5
    # created_at desc + id desc：最新创建的排第一
    assert items[0].id > items[-1].id


def test_list_filter_by_single_status(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(
        db_session,
        TicketListQuery(statuses=[TicketStatus.in_progress]),
    )
    assert total == 1
    assert items[0].title == "add export button"


def test_list_filter_by_multiple_statuses(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(
        db_session,
        TicketListQuery(
            statuses=[TicketStatus.open, TicketStatus.in_progress],
        ),
    )
    assert total == 5  # 全部


def test_list_filter_by_priority(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(
        db_session,
        TicketListQuery(priorities=[TicketPriority.urgent]),
    )
    assert total == 1
    assert items[0].priority == "urgent"


def test_list_filter_by_assignee(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(
        db_session,
        TicketListQuery(assignee="zhang"),
    )
    assert total == 2
    assert {t.assignee for t in items} == {"zhang"}


def test_list_filter_by_tag(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(db_session, TicketListQuery(tag="bug"))
    assert total == 2
    for t in items:
        assert "bug" in (t.tags or [])


def test_list_keyword_matches_title(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(db_session, TicketListQuery(keyword="captcha"))
    assert total == 1
    assert items[0].title.startswith("fix login")


def test_list_keyword_matches_description(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(db_session, TicketListQuery(keyword="csv"))
    assert total == 1
    assert items[0].title == "add export button"


def test_list_combined_filters(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(
        db_session,
        TicketListQuery(
            priorities=[TicketPriority.urgent],
            keyword="cve",
            assignee="zhang",
        ),
    )
    assert total == 1
    assert items[0].title == "critical security patch"


def test_list_pagination(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(db_session, TicketListQuery(page=1, page_size=2))
    assert total == 5
    assert len(items) == 2

    page2, _ = ticket_service.list_tickets(db_session, TicketListQuery(page=2, page_size=2))
    page3, _ = ticket_service.list_tickets(db_session, TicketListQuery(page=3, page_size=2))
    assert len(page2) == 2
    assert len(page3) == 1
    # 互不重复
    ids = {t.id for t in items + page2 + page3}
    assert len(ids) == 5


def test_list_sort_by_updated_at_asc(db_session: Session) -> None:
    _seed(db_session)
    items, _ = ticket_service.list_tickets(
        db_session,
        TicketListQuery(sort_by=SortBy.updated_at, sort_order=SortOrder.asc),
    )
    times = [t.updated_at for t in items]
    assert times == sorted(times)


def test_list_empty_when_no_match(db_session: Session) -> None:
    _seed(db_session)
    items, total = ticket_service.list_tickets(db_session, TicketListQuery(assignee="nobody"))
    assert total == 0
    assert items == []
