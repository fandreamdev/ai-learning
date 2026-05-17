"""GET /api/v1/tickets 集成测试。

覆盖 spec §5.2.1 的 9 个查询参数与 plan §3.4 的 IT-T3-01~07。
"""

from __future__ import annotations

from typing import Any

from fastapi.testclient import TestClient


def _create(client: TestClient, **payload: Any) -> dict[str, Any]:
    base = {
        "title": payload.get("title", "t"),
        "description": payload.get("description"),
        "priority": payload.get("priority", "medium"),
        "assignee": payload.get("assignee"),
        "tags": payload.get("tags", []),
    }
    body: dict[str, Any] = {k: v for k, v in base.items() if v is not None or k == "tags"}
    return dict(client.post("/api/v1/tickets", json=body).json()["data"])


def _seed_via_api(client: TestClient) -> None:
    _create(
        client,
        title="fix login captcha",
        description="captcha not refreshing",
        priority="high",
        assignee="zhang",
        tags=["bug", "frontend"],
    )
    second = _create(
        client,
        title="add export button",
        description="export to csv",
        priority="medium",
        assignee="li",
        tags=["feat"],
    )
    client.patch(f"/api/v1/tickets/{second['id']}/status", json={"status": "in_progress"})
    _create(
        client,
        title="optimize query",
        description="list endpoint is slow",
        priority="low",
        assignee="wang",
        tags=["perf", "backend"],
    )
    _create(
        client,
        title="critical security patch",
        description="urgent CVE",
        priority="urgent",
        assignee="zhang",
        tags=["bug", "security"],
    )


# ---------- IT-T3-01 / IT-T3-02 / IT-T3-02b ----------


def test_list_no_filter_returns_all(client: TestClient) -> None:
    _seed_via_api(client)
    resp = client.get("/api/v1/tickets")
    assert resp.status_code == 200
    body = resp.json()
    assert body["code"] == 0
    assert body["data"]["total"] == 4
    assert body["data"]["page"] == 1
    assert body["data"]["page_size"] == 20
    assert len(body["data"]["items"]) == 4


def test_list_filter_status_csv(client: TestClient) -> None:
    _seed_via_api(client)
    resp = client.get("/api/v1/tickets?status=open,in_progress")
    assert resp.status_code == 200
    body = resp.json()
    assert body["data"]["total"] == 4
    statuses = {t["status"] for t in body["data"]["items"]}
    assert statuses <= {"open", "in_progress"}


def test_list_filter_status_repeated(client: TestClient) -> None:
    _seed_via_api(client)
    resp = client.get("/api/v1/tickets?status=open&status=in_progress")
    assert resp.status_code == 200
    body = resp.json()
    assert body["data"]["total"] == 4


def test_list_two_writeforms_equivalent(client: TestClient) -> None:
    _seed_via_api(client)
    a = client.get("/api/v1/tickets?priority=high,urgent").json()["data"]
    b = client.get("/api/v1/tickets?priority=high&priority=urgent").json()["data"]
    assert a["total"] == b["total"]
    assert {t["id"] for t in a["items"]} == {t["id"] for t in b["items"]}


# ---------- IT-T3-03 / IT-T3-04 ----------


def test_list_combined_priority_keyword(client: TestClient) -> None:
    _seed_via_api(client)
    resp = client.get("/api/v1/tickets?priority=urgent&keyword=cve")
    assert resp.status_code == 200
    body = resp.json()
    assert body["data"]["total"] == 1
    assert body["data"]["items"][0]["title"] == "critical security patch"


def test_list_keyword_hits_description(client: TestClient) -> None:
    _seed_via_api(client)
    resp = client.get("/api/v1/tickets?keyword=csv")
    assert resp.status_code == 200
    items = resp.json()["data"]["items"]
    assert len(items) == 1
    assert items[0]["title"] == "add export button"


# ---------- 标签 / 负责人 ----------


def test_list_filter_by_tag(client: TestClient) -> None:
    _seed_via_api(client)
    resp = client.get("/api/v1/tickets?tag=bug")
    assert resp.status_code == 200
    items = resp.json()["data"]["items"]
    assert len(items) == 2
    for t in items:
        assert "bug" in t["tags"]


def test_list_filter_by_assignee(client: TestClient) -> None:
    _seed_via_api(client)
    resp = client.get("/api/v1/tickets?assignee=zhang")
    assert resp.status_code == 200
    items = resp.json()["data"]["items"]
    assert {t["assignee"] for t in items} == {"zhang"}


# ---------- IT-T3-05 / IT-T3-06 ----------


def test_list_pagination(client: TestClient) -> None:
    _seed_via_api(client)
    page1 = client.get("/api/v1/tickets?page=1&page_size=2").json()["data"]
    page2 = client.get("/api/v1/tickets?page=2&page_size=2").json()["data"]
    assert page1["total"] == 4
    assert len(page1["items"]) == 2
    assert len(page2["items"]) == 2
    assert {i["id"] for i in page1["items"]} & {i["id"] for i in page2["items"]} == set()


def test_list_page_size_too_large(client: TestClient) -> None:
    resp = client.get("/api/v1/tickets?page_size=200")
    assert resp.status_code == 400
    assert resp.json()["code"] == 40001


def test_list_page_size_zero(client: TestClient) -> None:
    resp = client.get("/api/v1/tickets?page_size=0")
    assert resp.status_code == 400


# ---------- IT-T3-07 ----------


def test_list_sort_by_updated_at_asc(client: TestClient) -> None:
    _seed_via_api(client)
    resp = client.get("/api/v1/tickets?sort_by=updated_at&sort_order=asc")
    assert resp.status_code == 200
    items = resp.json()["data"]["items"]
    times = [t["updated_at"] for t in items]
    assert times == sorted(times)


def test_list_invalid_sort_by(client: TestClient) -> None:
    resp = client.get("/api/v1/tickets?sort_by=title")
    assert resp.status_code == 400
    assert resp.json()["code"] == 40001


# ---------- 非法值 ----------


def test_list_invalid_status_returns_40001(client: TestClient) -> None:
    resp = client.get("/api/v1/tickets?status=foo")
    assert resp.status_code == 400
    assert resp.json()["code"] == 40001


def test_list_keyword_too_short(client: TestClient) -> None:
    # FastAPI Query(min_length=2) 兜底
    resp = client.get("/api/v1/tickets?keyword=a")
    assert resp.status_code == 400
