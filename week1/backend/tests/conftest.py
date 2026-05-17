"""pytest 共用 fixture。"""

from __future__ import annotations

from collections.abc import Generator

import pytest
from app.main import app
from fastapi.testclient import TestClient


@pytest.fixture(scope="session")
def client() -> Generator[TestClient]:
    """提供 FastAPI 测试客户端。"""
    with TestClient(app) as c:
        yield c
