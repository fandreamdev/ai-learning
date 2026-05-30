//! 用户 API 处理器
//!
//! 处理用户管理相关的 HTTP 请求

use crate::error::{AppError, AppResult};
use crate::models::{ChangePasswordRequest, UpdateUserRequest, UserPublic};
use crate::services::auth_service::AuthService;
use crate::state::AppState;
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;

/// 列出用户
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<uuid::Uuid>,
) -> AppResult<Json<Vec<UserPublic>>> {
    // TODO: 实现用户列表查询
    Ok(Json(vec![]))
}

/// 获取用户
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<uuid::Uuid>,
) -> AppResult<Json<UserPublic>> {
    // TODO: 实现获取单个用户
    Err(AppError::not_found("用户"))
}

/// 更新用户
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<uuid::Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> AppResult<Json<UserPublic>> {
    // TODO: 实现更新用户
    Err(AppError::not_found("用户"))
}

/// 删除用户
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<uuid::Uuid>,
) -> AppResult<Json<()>> {
    // TODO: 实现删除用户
    Err(AppError::not_found("用户"))
}

/// 修改密码
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<uuid::Uuid>,
    Json(request): Json<ChangePasswordRequest>,
) -> AppResult<Json<()>> {
    let auth_service = AuthService::new(
        state.config.clone(),
        state.db.clone(),
    );

    auth_service.change_password(user_id, request).await?;
    Ok(Json(()))
}
