"""日志配置。"""

from __future__ import annotations

import logging
import logging.config

from app.core.config import settings


def setup_logging() -> None:
    """配置全局日志。"""
    logging.config.dictConfig(
        {
            "version": 1,
            "disable_existing_loggers": False,
            "formatters": {
                "default": {
                    "format": "%(asctime)s [%(levelname)s] %(name)s: %(message)s",
                    "datefmt": "%Y-%m-%d %H:%M:%S",
                },
            },
            "handlers": {
                "console": {
                    "class": "logging.StreamHandler",
                    "formatter": "default",
                    "level": settings.log_level,
                },
            },
            "root": {
                "level": settings.log_level,
                "handlers": ["console"],
            },
            "loggers": {
                "uvicorn": {
                    "level": settings.log_level,
                    "propagate": False,
                    "handlers": ["console"],
                },
                "uvicorn.access": {
                    "level": settings.log_level,
                    "propagate": False,
                    "handlers": ["console"],
                },
            },
        }
    )
