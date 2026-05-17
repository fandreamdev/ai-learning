"""响应辅助函数。"""

from __future__ import annotations

from typing import TypeVar

from app.schemas.common import ApiResponse, PageData

T = TypeVar("T")


def success[T](data: T | None = None, message: str = "success") -> ApiResponse[T]:
    """构造成功响应。"""
    return ApiResponse[T](code=0, message=message, data=data)


def paginated[T](
    items: list[T],
    total: int,
    page: int = 1,
    page_size: int = 20,
) -> ApiResponse[PageData[T]]:
    """构造分页响应。"""
    return ApiResponse[PageData[T]](
        code=0,
        message="success",
        data=PageData[T](items=items, total=total, page=page, page_size=page_size),
    )
