"""Ticket Pydantic Schema。

对照 ``specs/week1/0001-spec.md`` §5.2 的 API 契约。
"""

from __future__ import annotations

from datetime import datetime
from enum import StrEnum

from pydantic import BaseModel, ConfigDict, Field, field_validator


class TicketStatus(StrEnum):
    """Ticket 状态枚举。"""

    open = "open"
    in_progress = "in_progress"
    done = "done"
    closed = "closed"


class TicketPriority(StrEnum):
    """Ticket 优先级枚举。"""

    low = "low"
    medium = "medium"
    high = "high"
    urgent = "urgent"


# 标签校验配置（spec §3.4）
TAG_MIN_LEN = 1
TAG_MAX_LEN = 20
TAG_MAX_COUNT = 10


def _normalize_tag_list(value: object) -> list[str] | None:
    """对 tags 列表做规范化：strip、转小写、去重、保序、长度校验。

    - 输入 None / 缺省 → 原样返回，让上层决定空数组语义
    - 输入非 list → 抛 ValueError 由 Pydantic 转 422
    - 每项长度 1~20，超出 10 项总数交由 Field(max_length=10) 处理
    """
    if value is None:
        return None
    if not isinstance(value, list):
        raise ValueError("tags 必须为字符串数组")

    seen: set[str] = set()
    result: list[str] = []
    for raw in value:
        if not isinstance(raw, str):
            raise ValueError("tags 元素必须为字符串")
        item = raw.strip().lower()
        if not item:
            raise ValueError("tag 不能为空字符串")
        if len(item) < TAG_MIN_LEN or len(item) > TAG_MAX_LEN:
            raise ValueError(f"tag 长度必须在 {TAG_MIN_LEN}~{TAG_MAX_LEN} 之间")
        if item in seen:
            continue
        seen.add(item)
        result.append(item)
    return result


class TicketBase(BaseModel):
    """新建/编辑共享的字段。"""

    title: str = Field(min_length=1, max_length=200, description="标题")
    description: str | None = Field(default=None, description="描述")
    priority: TicketPriority = Field(default=TicketPriority.medium, description="优先级")
    assignee: str | None = Field(default=None, max_length=100, description="负责人")
    tags: list[str] = Field(
        default_factory=list,
        max_length=TAG_MAX_COUNT,
        description="标签列表（小写、去重、最多 10 项、每项 1~20 字符）",
    )

    @field_validator("tags", mode="before")
    @classmethod
    def _validate_tags(cls, v: object) -> list[str]:
        normalized = _normalize_tag_list(v)
        return normalized or []

    @field_validator("title")
    @classmethod
    def _strip_title(cls, v: str) -> str:
        stripped = v.strip()
        if not stripped:
            raise ValueError("title 不能为空")
        return stripped


class TicketCreate(TicketBase):
    """POST /tickets 入参。"""


class TicketUpdate(BaseModel):
    """PUT /tickets/{id} 入参：所有字段可选，部分更新。"""

    title: str | None = Field(default=None, min_length=1, max_length=200)
    description: str | None = None
    priority: TicketPriority | None = None
    assignee: str | None = Field(default=None, max_length=100)
    tags: list[str] | None = Field(default=None, max_length=TAG_MAX_COUNT)
    status: TicketStatus | None = None

    @field_validator("tags", mode="before")
    @classmethod
    def _validate_tags(cls, v: object) -> list[str] | None:
        return _normalize_tag_list(v)

    @field_validator("title")
    @classmethod
    def _strip_title(cls, v: str | None) -> str | None:
        if v is None:
            return None
        stripped = v.strip()
        if not stripped:
            raise ValueError("title 不能为空")
        return stripped


class StatusUpdate(BaseModel):
    """PATCH /tickets/{id}/status 入参。"""

    status: TicketStatus


class TicketRead(BaseModel):
    """API 出参。"""

    id: int
    title: str
    description: str | None = None
    status: TicketStatus
    priority: TicketPriority
    assignee: str | None = None
    tags: list[str] = Field(default_factory=list)
    created_at: datetime
    updated_at: datetime

    model_config = ConfigDict(from_attributes=True)
