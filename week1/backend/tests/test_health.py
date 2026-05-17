"""健康检查与统一异常响应的基础测试。"""

from __future__ import annotations

from fastapi.testclient import TestClient


def test_healthz(client: TestClient) -> None:
    """/healthz 返回统一成功格式。"""
    resp = client.get("/healthz")
    assert resp.status_code == 200
    body = resp.json()
    assert body == {"code": 0, "message": "success", "data": "ok"}


def test_readyz(client: TestClient) -> None:
    """/readyz 在数据库可达时返回 ready。"""
    resp = client.get("/readyz")
    assert resp.status_code == 200
    body = resp.json()
    assert body["code"] == 0
    assert body["data"] == "ready"


def test_unknown_route_returns_unified_404(client: TestClient) -> None:
    """未匹配的路由也应走统一响应格式。"""
    resp = client.get("/api/v1/no-such-route")
    assert resp.status_code == 404
    body = resp.json()
    assert body["code"] != 0
    assert body["data"] is None
    assert "message" in body


def test_openapi_docs_available(client: TestClient) -> None:
    """OpenAPI 元数据可访问。"""
    resp = client.get("/openapi.json")
    assert resp.status_code == 200
    body = resp.json()
    assert body["info"]["title"] == "ProjectAlpha API"
