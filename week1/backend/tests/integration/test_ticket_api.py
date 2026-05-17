"""Ticket API 集成测试。

覆盖 5 个 CRUD 端点的成功/异常路径，
所有用例通过 conftest 中的 ``client`` fixture 复用同一个回滚事务。
"""

from __future__ import annotations

from fastapi.testclient import TestClient


def _create_payload(**override: object) -> dict[str, object]:
    base: dict[str, object] = {
        "title": "fix login captcha",
        "description": "captcha not refreshing",
        "priority": "high",
        "assignee": "zhang",
        "tags": ["bug", "frontend"],
    }
    base.update(override)
    return base


# ---------- POST /tickets ----------


def test_create_returns_201(client: TestClient) -> None:
    resp = client.post("/api/v1/tickets", json=_create_payload())
    assert resp.status_code == 201
    body = resp.json()
    assert body["code"] == 0
    data = body["data"]
    assert data["id"] > 0
    assert data["title"] == "fix login captcha"
    assert data["status"] == "open"
    assert data["priority"] == "high"
    assert data["tags"] == ["bug", "frontend"]


def test_create_title_required(client: TestClient) -> None:
    resp = client.post("/api/v1/tickets", json={})
    assert resp.status_code == 400
    body = resp.json()
    assert body["code"] == 40001


def test_create_too_many_tags(client: TestClient) -> None:
    resp = client.post(
        "/api/v1/tickets",
        json=_create_payload(tags=[f"t{i}" for i in range(11)]),
    )
    assert resp.status_code == 400
    assert resp.json()["code"] == 40001


# ---------- GET /tickets/{id} ----------


def test_get_detail_ok(client: TestClient) -> None:
    created = client.post("/api/v1/tickets", json=_create_payload()).json()["data"]
    resp = client.get(f"/api/v1/tickets/{created['id']}")
    assert resp.status_code == 200
    body = resp.json()
    assert body["code"] == 0
    assert body["data"]["id"] == created["id"]


def test_get_detail_not_found(client: TestClient) -> None:
    resp = client.get("/api/v1/tickets/999999")
    assert resp.status_code == 404
    assert resp.json()["code"] == 40401


# ---------- PUT /tickets/{id} ----------


def test_update_partial(client: TestClient) -> None:
    created = client.post("/api/v1/tickets", json=_create_payload()).json()["data"]
    resp = client.put(
        f"/api/v1/tickets/{created['id']}",
        json={"title": "updated title"},
    )
    assert resp.status_code == 200
    data = resp.json()["data"]
    assert data["title"] == "updated title"
    # 其他字段不变
    assert data["priority"] == "high"
    assert data["tags"] == ["bug", "frontend"]


def test_update_via_put_with_legal_status(client: TestClient) -> None:
    created = client.post("/api/v1/tickets", json=_create_payload()).json()["data"]
    resp = client.put(
        f"/api/v1/tickets/{created['id']}",
        json={"status": "in_progress"},
    )
    assert resp.status_code == 200
    assert resp.json()["data"]["status"] == "in_progress"


def test_update_via_put_with_illegal_status(client: TestClient) -> None:
    created = client.post("/api/v1/tickets", json=_create_payload()).json()["data"]
    # open -> done 非法
    resp = client.put(
        f"/api/v1/tickets/{created['id']}",
        json={"status": "done"},
    )
    assert resp.status_code == 400
    assert resp.json()["code"] == 40002


def test_update_not_found(client: TestClient) -> None:
    resp = client.put("/api/v1/tickets/999999", json={"title": "x"})
    assert resp.status_code == 404
    assert resp.json()["code"] == 40401


# ---------- PATCH /tickets/{id}/status ----------


def test_patch_status_legal(client: TestClient) -> None:
    created = client.post("/api/v1/tickets", json=_create_payload()).json()["data"]
    resp = client.patch(
        f"/api/v1/tickets/{created['id']}/status",
        json={"status": "in_progress"},
    )
    assert resp.status_code == 200
    assert resp.json()["data"]["status"] == "in_progress"


def test_patch_status_illegal(client: TestClient) -> None:
    created = client.post("/api/v1/tickets", json=_create_payload()).json()["data"]
    resp = client.patch(
        f"/api/v1/tickets/{created['id']}/status",
        json={"status": "done"},
    )
    assert resp.status_code == 400
    assert resp.json()["code"] == 40002


def test_patch_status_not_found(client: TestClient) -> None:
    resp = client.patch("/api/v1/tickets/999999/status", json={"status": "closed"})
    assert resp.status_code == 404
    assert resp.json()["code"] == 40401


# ---------- DELETE /tickets/{id} ----------


def test_delete_ok(client: TestClient) -> None:
    created = client.post("/api/v1/tickets", json=_create_payload()).json()["data"]
    resp = client.delete(f"/api/v1/tickets/{created['id']}")
    assert resp.status_code == 204
    # 再次获取应 404
    follow = client.get(f"/api/v1/tickets/{created['id']}")
    assert follow.status_code == 404


def test_delete_not_found(client: TestClient) -> None:
    resp = client.delete("/api/v1/tickets/999999")
    assert resp.status_code == 404
    assert resp.json()["code"] == 40401
