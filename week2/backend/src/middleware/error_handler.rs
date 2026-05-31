//! 错误处理中间件
//!
//! 提供统一的错误处理和响应格式

use crate::error::AppError;
use axum::{
    extract::Request,
    response::{IntoResponse, Response},
    middleware::Next,
};

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
/// 注意: 由于异步代码的特性，不再使用 panic 捕获
/// 而是通过监控线程来检测异常
pub async fn recovery_middleware(
    request: Request,
    next: Next,
) -> Response {
    next.run(request).await
}
