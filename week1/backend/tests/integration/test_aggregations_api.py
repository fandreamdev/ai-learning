"""GET /api/v1/tags 与 /api/v1/assignees 集成测试。"""

from __future__ import annotations

from fastapi.testclient import TestClient


def test_tags_empty_when_no_data(client: TestClient) -> None:
    resp = client.get("/api/v1/tags")
    assert resp.status_code == 200
    body = resp.json()
    assert body["code"] == 0
    assert body["data"] == []


def test_tags_aggregated_unique_sorted(client: TestClient) -> None:
    client.post(
        "/api/v1/tickets",
        json={"title": "a", "tags": ["bug", "frontend"]},
    )
    client.post(
        "/api/v1/tickets",
        json={"title": "b", "tags": ["bug", "backend"]},
    )
    resp = client.get("/api/v1/tags")
    assert resp.status_code == 200
    assert resp.json()["data"] == ["backend", "bug", "frontend"]


def test_assignees_empty_when_no_data(client: TestClient) -> None:
    resp = client.get("/api/v1/assignees")
    assert resp.status_code == 200
    assert resp.json()["data"] == []


def test_assignees_aggregated_unique_sorted(client: TestClient) -> None:
    client.post("/api/v1/tickets", json={"title": "a", "assignee": "zhang"})
    client.post("/api/v1/tickets", json={"title": "b", "assignee": "li"})
    client.post("/api/v1/tickets", json={"title": "c", "assignee": "zhang"})
    client.post("/api/v1/tickets", json={"title": "d"})
    resp = client.get("/api/v1/assignees")
    assert resp.status_code == 200
    assert resp.json()["data"] == ["li", "zhang"]
