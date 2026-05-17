"""ORM 模型聚合。"""

from __future__ import annotations

from app.core.database import Base
from app.models.ticket import Ticket

__all__ = ["Base", "Ticket"]
