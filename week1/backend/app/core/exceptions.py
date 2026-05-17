"""自定义业务异常体系。

所有业务异常继承自 :class:`BusinessException`，由
:func:`app.middlewares.error_handler.register_exception_handlers`
统一捕获并转换为统一响应格式。
"""

from __future__ import annotations


class BusinessException(Exception):
    """业务异常基类。"""

    code: int = 50001
    message: str = "internal error"
    http_status: int = 500

    def __init__(self, message: str | None = None) -> None:
        if message is not None:
            self.message = message
        super().__init__(self.message)


class TicketNotFound(BusinessException):
    """Ticket 不存在。"""

    code = 40401
    message = "Ticket 不存在"
    http_status = 404


class InvalidStatusTransition(BusinessException):
    """状态流转不合法。"""

    code = 40002
    message = "状态流转不合法"
    http_status = 400


class ValidationError(BusinessException):
    """业务级校验失败（与 Pydantic 422 区分）。"""

    code = 40001
    message = "请求参数校验失败"
    http_status = 400
