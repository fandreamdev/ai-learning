//! 认证 API 处理器
//!
//! 处理用户认证相关的 HTTP 请求

use crate::error::AppResult;
use crate::models::{CreateUserRequest, LoginRequest};
use crate::services::auth_service::AuthService;
use crate::state::AppState;
use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 登录
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> AppResult<Json<crate::models::LoginResponse>> {
    let auth_service = AuthService::new(
        state.config.clone(),
        state.db.clone(),
    );

    let response = auth_service.login(request).await?;
    Ok(Json(response))
}

/// 注册
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<Json<crate::models::UserPublic>> {
    let auth_service = AuthService::new(
        state.config.clone(),
        state.db.clone(),
    );

    let user = auth_service.register(request).await?;
    Ok(Json(user))
}

/// 刷新 Token
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> AppResult<Json<crate::models::LoginResponse>> {
    let auth_service = AuthService::new(
        state.config.clone(),
        state.db.clone(),
    );

    let response = auth_service.refresh_token(&request.refresh_token).await?;
    Ok(Json(response))
}

/// 登出
pub async fn logout() -> &'static str {
    "Logged out successfully"
}
