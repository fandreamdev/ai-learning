//! 数据库连接 API 处理器
//!
//! 处理数据库连接管理相关的 HTTP 请求

use crate::error::{AppError, AppResult};
use crate::models::{
    ConnectionPublic, CreateConnectionRequest, DatabaseConnection, TestConnectionResponse,
    UpdateConnectionRequest,
};
use crate::repositories::ConnectionRepo;
use crate::services::connection_manager::ConnectionManager;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct ConnectionListParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 列出连接
pub async fn list_connections(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionListParams>,
) -> AppResult<Json<Vec<ConnectionPublic>>> {
    let repo = ConnectionRepo::new(state.db.clone());
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    let connections = repo.list_paginated(page, page_size).await?;
    let public_connections: Vec<ConnectionPublic> = connections
        .into_iter()
        .map(ConnectionPublic::from)
        .collect();

    Ok(Json(public_connections))
}

/// 创建连接
pub async fn create_connection(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateConnectionRequest>,
) -> AppResult<Json<ConnectionPublic>> {
    let repo = ConnectionRepo::new(state.db.clone());

    // 简单加密密码
    let encrypted_password = base64_encode(&request.password);

    let mut conn = DatabaseConnection::new(
        request.name,
        request.db_type,
        request.host,
        request.port,
        request.database_name,
        request.username,
        encrypted_password,
        None,
    );
    conn.is_default = request.is_default;

    // 如果设为默认，先取消其他默认
    if conn.is_default {
        repo.clear_default_for_user(conn.created_by).await?;
    }

    let conn = repo.create(&conn).await?;
    Ok(Json(ConnectionPublic::from(&conn)))
}

/// 获取连接
pub async fn get_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ConnectionPublic>> {
    let repo = ConnectionRepo::new(state.db.clone());
    let conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    Ok(Json(ConnectionPublic::from(&conn)))
}

/// 更新连接
pub async fn update_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateConnectionRequest>,
) -> AppResult<Json<ConnectionPublic>> {
    let repo = ConnectionRepo::new(state.db.clone());
    let mut conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    if let Some(name) = request.name {
        conn.name = name;
    }
    if let Some(db_type) = request.db_type {
        conn.db_type = db_type;
    }
    if let Some(host) = request.host {
        conn.host = host;
    }
    if let Some(port) = request.port {
        conn.port = port;
    }
    if let Some(database_name) = request.database_name {
        conn.database_name = database_name;
    }
    if let Some(username) = request.username {
        conn.username = username;
    }
    if let Some(password) = request.password {
        conn.encrypted_password = base64_encode(&password);
    }
    if let Some(is_default) = request.is_default {
        if is_default {
            repo.clear_default_for_user(conn.created_by).await?;
        }
        conn.is_default = is_default;
    }
    conn.updated_at = chrono::Utc::now();

    let conn = repo.update(&conn).await?;
    Ok(Json(ConnectionPublic::from(&conn)))
}

/// 删除连接
pub async fn delete_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let repo = ConnectionRepo::new(state.db.clone());
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    repo.delete(id).await?;
    Ok(Json(()))
}

/// 测试连接
pub async fn test_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<TestConnectionResponse>> {
    let repo = ConnectionRepo::new(state.db.clone());
    let conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    // 使用连接管理器测试连接
    let start = std::time::Instant::now();
    let test_result = state
        .connection_manager
        .test_connection(&conn)
        .await;

    let latency_ms = start.elapsed().as_millis() as i64;

    match test_result {
        Ok(version) => Ok(Json(TestConnectionResponse {
            success: true,
            message: "连接成功".to_string(),
            server_version: Some(version),
            latency_ms: Some(latency_ms),
        })),
        Err(e) => Ok(Json(TestConnectionResponse {
            success: false,
            message: format!("连接失败: {}", e),
            server_version: None,
            latency_ms: Some(latency_ms),
        })),
    }
}

/// 设置默认连接
pub async fn set_default_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let repo = ConnectionRepo::new(state.db.clone());
    let conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    let user_id = conn.created_by.unwrap_or(Uuid::nil());
    repo.set_default(id, user_id).await?;

    Ok(Json(()))
}

/// 获取 Schema 信息
pub async fn get_schema(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let repo = ConnectionRepo::new(state.db.clone());
    let conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    // 使用连接管理器获取 schema
    match state.connection_manager.get_schema(&conn).await {
        Ok(tables) => Ok(Json(serde_json::json!({
            "code": 0,
            "message": "Success",
            "data": {
                "connection_id": id,
                "tables": tables,
                "created_at": chrono::Utc::now()
            }
        }))),
        Err(e) => Err(AppError::database(format!("获取 Schema 失败: {}", e))),
    }
}

/// 辅助函数：对密码进行 base64 编码
fn base64_encode(input: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(input.as_bytes())
}
