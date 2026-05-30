//! 错误处理中间件
//!
//! 提供统一的错误处理和响应格式

use crate::error::AppError;
use axum::{
    extract::Request,
    response::Response,
    middleware::Next,
};
use tracing::{error, warn};

/// 统一错误处理中间件
///
/// 将所有错误转换为统一的 JSON 响应格式
pub async fn error_handler_middleware(
    request: Request,
    next: Next,
) -> Response {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        next.run(request),
    )
    .await;

    match result {
        Ok(response) => response,
        Err(_) => {
            // 请求超时
            tracing::warn!("Request timeout");
            let error = AppError::InternalError("Request timeout".to_string());
            error.into_response()
        }
    }
}

/// 请求恢复中间件
///
/// 捕获 panic 并返回 500 错误
pub async fn recovery_middleware(
    request: Request,
    next: Next,
) -> Response {
    let result = std::panic::catch_unwind(|| {
        futures::executor::block_on(next.run(request))
    });

    match result {
        Ok(response) => response,
        Err(_) => {
            error!("Panic recovered in request handler");
            let error = AppError::InternalError("Internal server error".to_string());
            error.into_response()
        }
    }
}
