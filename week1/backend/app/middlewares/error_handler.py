"""全局异常处理器。

将各类异常统一为 ``{code, message, data: null}`` 响应结构（spec §5.1 / §11）。
"""

from __future__ import annotations

import logging

from fastapi import FastAPI, Request, status
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from starlette.exceptions import HTTPException as StarletteHTTPException

from app.core.exceptions import BusinessException

logger = logging.getLogger(__name__)


def _payload(code: int, message: str) -> dict[str, object]:
    return {"code": code, "message": message, "data": None}


def register_exception_handlers(app: FastAPI) -> None:
    """注册全部异常处理器。"""

    @app.exception_handler(BusinessException)
    async def _business_exc_handler(_: Request, exc: BusinessException) -> JSONResponse:
        return JSONResponse(
            status_code=exc.http_status,
            content=_payload(exc.code, exc.message),
        )

    @app.exception_handler(RequestValidationError)
    async def _validation_exc_handler(_: Request, exc: RequestValidationError) -> JSONResponse:
        # 取首条字段错误作为对外消息，原始细节放入 detail
        errors = exc.errors()
        first_msg = "请求参数校验失败"
        if errors:
            err0 = errors[0]
            loc = ".".join(str(p) for p in err0.get("loc", ()) if p != "body")
            first_msg = f"{loc}: {err0.get('msg')}" if loc else str(err0.get("msg"))
        return JSONResponse(
            status_code=status.HTTP_400_BAD_REQUEST,
            content={
                "code": 40001,
                "message": first_msg,
                "data": None,
                "detail": jsonable_encoder(errors),
            },
        )

    @app.exception_handler(StarletteHTTPException)
    async def _http_exc_handler(_: Request, exc: StarletteHTTPException) -> JSONResponse:
        # 例如未匹配的路由会触发 404 — 仍走统一格式
        code_map = {
            status.HTTP_404_NOT_FOUND: 40404,
            status.HTTP_405_METHOD_NOT_ALLOWED: 40405,
        }
        return JSONResponse(
            status_code=exc.status_code,
            content=_payload(code_map.get(exc.status_code, exc.status_code * 100), str(exc.detail)),
        )

    @app.exception_handler(Exception)
    async def _unhandled_exc_handler(_: Request, exc: Exception) -> JSONResponse:
        logger.exception("Unhandled exception: %s", exc)
        return JSONResponse(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            content=_payload(50001, "服务器内部错误"),
        )
