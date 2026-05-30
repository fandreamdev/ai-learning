//! 错误处理中间件
//!
//! 提供统一的错误处理和响应格式

use crate::error::{AppError, ErrorResponse};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
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

/// 错误到响应的转换
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            // 认证错误
            AppError::AuthenticationFailed(msg) => (StatusCode::UNAUTHORIZED, 1001, msg.clone()),
            AppError::TokenExpired => (StatusCode::UNAUTHORIZED, 1002, "Token 已过期".to_string()),
            AppError::InvalidToken(msg) => (StatusCode::UNAUTHORIZED, 1003, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, 1004, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, 1005, msg.clone()),

            // 验证错误
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, 2001, msg.clone()),
            AppError::MissingParameter(msg) => (StatusCode::BAD_REQUEST, 2002, msg.clone()),
            AppError::InvalidFormat(msg) => (StatusCode::BAD_REQUEST, 2003, msg.clone()),

            // 资源错误
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, 3001, msg.clone()),
            AppError::AlreadyExists(msg) => (StatusCode::CONFLICT, 3002, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, 3003, msg.clone()),

            // 数据库错误
            AppError::DatabaseError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 4001, msg.clone())
            }
            AppError::ConnectionFailed(msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, 4002, msg.clone())
            }
            AppError::QueryTimeout(msg) => (StatusCode::GATEWAY_TIMEOUT, 4003, msg.clone()),

            // SQL 安全错误
            AppError::SqlSecurityError(msg) => (StatusCode::FORBIDDEN, 5001, msg.clone()),
            AppError::DmlForbidden(msg) => (StatusCode::FORBIDDEN, 5002, msg.clone()),
            AppError::DdlForbidden(msg) => (StatusCode::FORBIDDEN, 5003, msg.clone()),
            AppError::DangerousFunction(msg) => (StatusCode::FORBIDDEN, 5004, msg.clone()),
            AppError::SqlInjection(msg) => (StatusCode::FORBIDDEN, 5005, msg.clone()),

            // LLM 错误
            AppError::LlmError(msg) => (StatusCode::BAD_GATEWAY, 6001, msg.clone()),
            AppError::LlmUnavailable(msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, 6002, msg.clone())
            }
            AppError::LlmParseError(msg) => (StatusCode::BAD_GATEWAY, 6003, msg.clone()),
            AppError::TokenLimitExceeded(msg) => {
                (StatusCode::PAYLOAD_TOO_LARGE, 6004, msg.clone())
            }

            // 业务错误
            AppError::BusinessError(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, 7001, msg.clone())
            }
            AppError::UnsupportedOperation(msg) => {
                (StatusCode::NOT_IMPLEMENTED, 7002, msg.clone())
            }

            // 内部错误
            AppError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 9001, msg.clone())
            }
            AppError::ConfigError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 9002, msg.clone())
            }
            AppError::NotInitialized(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 9003, msg.clone())
            }
        };

        // 记录错误日志
        let log_level = if status.is_server_error() {
            error!("Server error: {} (code: {})", message, code)
        } else if status.is_client_error() {
            warn!("Client error: {} (code: {})", message, code)
        } else {
            tracing::info!("Request error: {} (code: {})", message, code)
        };

        let error_response = ErrorResponse::new(code, message);

        (status, Json(error_response)).into_response()
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

/// 404 处理
pub async fn not_found_handler() -> impl IntoResponse {
    let error = AppError::NotFound("Endpoint not found".to_string());
    error.into_response()
}

/// 方法不支持处理
pub async fn method_not_allowed_handler() -> impl IntoResponse {
    let error = AppError::UnsupportedOperation("Method not allowed".to_string());
    error.into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response() {
        let response = ErrorResponse::new(1001, "Test error");
        assert_eq!(response.code, 1001);
        assert_eq!(response.message, "Test error");
        assert!(response.details.is_none());
    }

    #[test]
    fn test_error_response_with_details() {
        let response = ErrorResponse::new(1001, "Test error")
            .with_details("Additional details");

        assert!(response.details.is_some());
        assert_eq!(response.details.unwrap(), "Additional details");
    }
}
