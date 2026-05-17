"""ORM 模型聚合。

仅导入 :class:`Base`，避免循环依赖；
后续阶段将在此模块中导出 Ticket 等模型。
"""

from __future__ import annotations

from app.core.database import Base

__all__ = ["Base"]
