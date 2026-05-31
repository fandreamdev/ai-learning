//! 错误处理模块
//!
//! 定义统一的错误类型和错误码

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// 应用错误类型
#[derive(Debug, Error)]
pub enum AppError {
    // ==================== 认证错误 (1xxx) ====================
    #[error("认证失败: {0}")]
    AuthenticationFailed(String),

    #[error("Token 已过期")]
    TokenExpired,

    #[error("Token 无效: {0}")]
    InvalidToken(String),

    #[error("权限不足: {0}")]
    Unauthorized(String),

    #[error("禁止访问: {0}")]
    Forbidden(String),

    // ==================== 验证错误 (2xxx) ====================
    #[error("参数验证失败: {0}")]
    ValidationError(String),

    #[error("缺少必需参数: {0}")]
    MissingParameter(String),

    #[error("参数格式错误: {0}")]
    InvalidFormat(String),

    // ==================== 资源错误 (3xxx) ====================
    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("资源已存在: {0}")]
    AlreadyExists(String),

    #[error("资源冲突: {0}")]
    Conflict(String),

    // ==================== 数据库错误 (4xxx) ====================
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("连接失败: {0}")]
    ConnectionFailed(String),

    #[error("查询超时: {0}")]
    QueryTimeout(String),

    // ==================== SQL 安全错误 (5xxx) ====================
    #[error("SQL 安全检查失败: {0}")]
    SqlSecurityError(String),

    #[error("禁止执行 DML 操作: {0}")]
    DmlForbidden(String),

    #[error("禁止执行 DDL 操作: {0}")]
    DdlForbidden(String),

    #[error("危险函数检测: {0}")]
    DangerousFunction(String),

    #[error("SQL 注入检测: {0}")]
    SqlInjection(String),

    // ==================== LLM 错误 (6xxx) ====================
    #[error("LLM 调用失败: {0}")]
    LlmError(String),

    #[error("LLM 服务不可用: {0}")]
    LlmUnavailable(String),

    #[error("LLM 响应解析失败: {0}")]
    LlmParseError(String),

    #[error("Token 超出限制: {0}")]
    TokenLimitExceeded(String),

    // ==================== 业务错误 (7xxx) ====================
    #[error("业务逻辑错误: {0}")]
    BusinessError(String),

    #[error("不支持的操作: {0}")]
    UnsupportedOperation(String),

    // ==================== 内部错误 (9xxx) ====================
    #[error("内部错误: {0}")]
    InternalError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("服务未初始化: {0}")]
    NotInitialized(String),
}

/// API 错误响应格式
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

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
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 4001, msg.clone()),
            AppError::ConnectionFailed(msg) => (StatusCode::SERVICE_UNAVAILABLE, 4002, msg.clone()),
            AppError::QueryTimeout(msg) => (StatusCode::GATEWAY_TIMEOUT, 4003, msg.clone()),

            // SQL 安全错误
            AppError::SqlSecurityError(msg) => (StatusCode::FORBIDDEN, 5001, msg.clone()),
            AppError::DmlForbidden(msg) => (StatusCode::FORBIDDEN, 5002, msg.clone()),
            AppError::DdlForbidden(msg) => (StatusCode::FORBIDDEN, 5003, msg.clone()),
            AppError::DangerousFunction(msg) => (StatusCode::FORBIDDEN, 5004, msg.clone()),
            AppError::SqlInjection(msg) => (StatusCode::FORBIDDEN, 5005, msg.clone()),

            // LLM 错误
            AppError::LlmError(msg) => (StatusCode::BAD_GATEWAY, 6001, msg.clone()),
            AppError::LlmUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, 6002, msg.clone()),
            AppError::LlmParseError(msg) => (StatusCode::BAD_GATEWAY, 6003, msg.clone()),
            AppError::TokenLimitExceeded(msg) => (StatusCode::PAYLOAD_TOO_LARGE, 6004, msg.clone()),

            // 业务错误
            AppError::BusinessError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, 7001, msg.clone()),
            AppError::UnsupportedOperation(msg) => (StatusCode::NOT_IMPLEMENTED, 7002, msg.clone()),

            // 内部错误
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 9001, msg.clone()),
            AppError::ConfigError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 9002, msg.clone()),
            AppError::NotInitialized(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 9003, msg.clone()),
        };

        let body = Json(ErrorResponse::new(code, message));

        (status, body).into_response()
    }
}

impl AppError {
    /// 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        AppError::ValidationError(msg.into())
    }

    /// 创建未找到错误
    pub fn not_found(resource: impl Into<String>) -> Self {
        AppError::NotFound(resource.into())
    }

    /// 创建数据库错误
    pub fn database(msg: impl Into<String>) -> Self {
        AppError::DatabaseError(msg.into())
    }

    /// 创建内部错误
    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::InternalError(msg.into())
    }
}

// ==================== 通用 Result 类型 ====================

/// 应用结果类型
pub type AppResult<T> = Result<T, AppError>;

/// 便捷的转换实现
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("记录不存在".to_string()),
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("HTTP client error: {:?}", err);
        AppError::LlmError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("JSON parse error: {:?}", err);
        AppError::InternalError(format!("JSON 解析失败: {}", err))
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = err
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("{} 验证失败", field))
                })
            })
            .collect();
        AppError::ValidationError(messages.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        let err = AppError::not_found("用户");
        let response: Response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::validation("用户名不能为空");
        let response: Response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
