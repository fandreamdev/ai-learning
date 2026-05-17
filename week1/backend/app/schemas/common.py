"""统一响应结构。

- :class:`ApiResponse` 对应 spec §5.1 的成功/失败信封
- :class:`PageData`    对应分页结果的 ``data`` 子结构
"""

from __future__ import annotations

from typing import TypeVar

from pydantic import BaseModel, Field

T = TypeVar("T")


class ApiResponse[T](BaseModel):
    """统一 API 响应信封。"""

    code: int = Field(default=0, description="业务码：0 表示成功")
    message: str = Field(default="success", description="业务描述")
    data: T | None = Field(default=None, description="业务数据")


class PageData[T](BaseModel):
    """分页响应数据。"""

    items: list[T] = Field(default_factory=list)
    total: int = 0
    page: int = 1
    page_size: int = 20
