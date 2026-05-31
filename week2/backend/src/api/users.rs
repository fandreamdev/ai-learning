//! 用户 API 处理器
//!
//! 处理用户管理相关的 HTTP 请求

use crate::error::{AppError, AppResult};
use crate::models::{ChangePasswordRequest, UpdateUserRequest, UserPublic};
use crate::repositories::UserRepo;
use crate::state::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct UserListParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 列出用户
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserListParams>,
) -> AppResult<Json<Vec<UserPublic>>> {
    let repo = UserRepo::new(state.db.clone());
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    let (users, _) = repo.list(page, page_size).await?;
    let public_users: Vec<UserPublic> = users.iter().map(UserPublic::from).collect();

    Ok(Json(public_users))
}

/// 获取用户
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<UserPublic>> {
    let repo = UserRepo::new(state.db.clone());
    let user = repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    Ok(Json(UserPublic::from(&user)))
}

/// 更新用户
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> AppResult<Json<UserPublic>> {
    let repo = UserRepo::new(state.db.clone());
    let mut user = repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    if let Some(username) = request.username {
        // 检查用户名是否已被使用
        if let Some(existing) = repo.find_by_username(&username).await? {
            if existing.id != user_id {
                return Err(AppError::AlreadyExists("用户名已被使用".to_string()));
            }
        }
        user.username = username;
    }

    if let Some(email) = request.email {
        // 检查邮箱是否已被使用
        if let Some(existing) = repo.find_by_email(&email).await? {
            if existing.id != user_id {
                return Err(AppError::AlreadyExists("邮箱已被使用".to_string()));
            }
        }
        user.email = email;
    }

    if let Some(role) = request.role {
        user.role = role;
    }

    if let Some(is_active) = request.is_active {
        user.is_active = is_active;
    }

    user.updated_at = chrono::Utc::now();
    let user = repo.update(&user).await?;

    Ok(Json(UserPublic::from(&user)))
}

/// 删除用户
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let repo = UserRepo::new(state.db.clone());

    // 检查用户是否存在
    repo.find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    repo.delete(user_id).await?;
    Ok(Json(()))
}

/// 修改密码
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<ChangePasswordRequest>,
) -> AppResult<Json<()>> {
    let repo = UserRepo::new(state.db.clone());
    let user = repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    // 验证旧密码
    let password_utils = crate::utils::password::PasswordUtils::new(&state.config.security);
    if !password_utils.verify_password(&request.old_password, &user.password_hash) {
        return Err(AppError::ValidationError("旧密码不正确".to_string()));
    }

    // 哈希新密码
    let new_hash = password_utils.hash_password(&request.new_password)?;

    repo.update_password(user_id, &new_hash).await?;
    Ok(Json(()))
}
