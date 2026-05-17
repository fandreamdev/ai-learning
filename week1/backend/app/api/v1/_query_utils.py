"""路由层共用查询参数辅助。"""

from __future__ import annotations


def parse_multi_csv(values: list[str] | None) -> list[str]:
    """把 ``["open,in_progress", "done"]`` 与 ``["open", "in_progress"]``
    都展平成 ``["open", "in_progress", "done"]``。

    - 同时支持"逗号分隔"与"重复参数"两种 query 写法
    - strip 空白；去重时保留首次出现顺序
    """
    if not values:
        return []
    items: list[str] = []
    seen: set[str] = set()
    for raw in values:
        for part in raw.split(","):
            cleaned = part.strip()
            if not cleaned or cleaned in seen:
                continue
            seen.add(cleaned)
            items.append(cleaned)
    return items
