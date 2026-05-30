//! 请求提取器模块
//!
//! 提供从请求中提取用户信息的提取器

use axum::http::request::Parts;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::UserSession;

/// 当前用户提取器
///
/// 从请求中提取当前登录用户的信息
#[derive(Clone)]
pub struct CurrentUser {
    pub user_id: Uuid,
    pub username: String,
    pub role: crate::models::UserRole,
}

impl CurrentUser {
    /// 从 Parts 中提取用户会话
    pub fn from_parts(parts: &Parts) -> AppResult<Self> {
        let session = parts
            .extensions
            .get::<UserSession>()
            .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

        Ok(Self {
            user_id: session.user_id,
            username: session.username.clone(),
            role: session.role.clone(),
        })
    }
}

/// 用户 ID 提取器
///
/// 仅提取用户 ID，不包含其他用户信息
#[derive(Clone)]
pub struct UserId(pub Uuid);
