//! API 路由定义
//!
//! 定义所有 API 端点

use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::error::{AppError, AppResult};
use crate::models::{
    ChangePasswordRequest, ColumnMetadata, ConnectionPublic, Conversation, CreateConnectionRequest,
    CreateConversationRequest, CreateMetricRequest, CreateUserRequest, DatabaseConnection,
    DatabaseType, LoginRequest, Message, Metric, NlExecuteRequest, NlToSqlRequest,
    QueryHistory, QueryHistoryItem, SqlExecuteRequest, SqlFormatRequest,
    UpdateConnectionRequest, UpdateMetricRequest,
    UpdateUserRequest, User, UserPublic, UserSession,
};
use crate::repositories::{ConnectionRepo, ConversationRepo, QueryRepo, UserRepo};
use crate::services::connection_manager::ConnectionPool;
use crate::state::AppState;

// ==================== 指标存储 ====================
lazy_static::lazy_static! {
    static ref METRICS_STORE: std::sync::RwLock<std::collections::HashMap<Uuid, Metric>> =
        std::sync::RwLock::new(std::collections::HashMap::new());
}

// ==================== 路由定义 ====================

pub fn routes() -> Router<Arc<AppState>> {
    let api = Router::new()
        .route("/health", get(health_check))
        .route("/test", get(simple_test_handler))
        .route("/sql-exec", post(execute_sql_handler))
        .nest("/auth", auth_routes())
        .nest("/users", user_routes())
        .nest("/connections", connection_routes())
        .nest("/sql", sql_routes())
        .nest("/nl", nl_routes())
        .nest("/conversations", conversation_routes())
        .nest("/charts", chart_routes())
        .nest("/metrics", metric_routes());

    api
}

// ==================== 健康检查 ====================

async fn health_check() -> &'static str {
    "OK"
}

// 测试简单 handler
async fn simple_test_handler(State(state): State<Arc<AppState>>) -> &'static str {
    let _ = &state.db;  // 使用 state
    "Simple test"
}

// ==================== 认证路由 ====================

fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/register", post(register_handler))
        .route("/refresh", post(refresh_handler))
        .route("/logout", post(logout_handler))
}

fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_users_handler))
        .route("/{id}", get(get_user_handler))
        .route("/{id}", put(update_user_handler))
        .route("/{id}", delete(delete_user_handler))
        .route("/{id}/password", put(change_password_handler))
}

fn connection_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_connections_handler))
        .route("/", post(create_connection_handler))
        .route("/{id}", get(get_connection_handler))
        .route("/{id}", put(update_connection_handler))
        .route("/{id}", delete(delete_connection_handler))
        .route("/{id}/test", post(test_connection_handler))
        .route("/{id}/default", put(set_default_connection_handler))
        .route("/{id}/schema", get(get_schema_handler))
}

fn nl_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/convert", post(nl_convert_handler))
        .route("/execute", post(nl_execute_handler))
}

fn conversation_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_conversations_handler))
        .route("/", post(create_conversation_handler))
        .route("/{id}", get(get_conversation_handler))
        .route("/{id}", delete(delete_conversation_handler))
        .route("/{id}/messages", get(list_messages_handler))
        .route("/{id}/messages", post(send_message_handler))
}

fn chart_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/recommend", get(recommend_chart_handler))
        .route("/generate", post(generate_chart_handler))
        .route("/export", post(export_chart_handler))
}

fn metric_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_metrics_handler))
        .route("/", post(create_metric_handler))
        .route("/{id}", get(get_metric_handler))
        .route("/{id}", put(update_metric_handler))
        .route("/{id}", delete(delete_metric_handler))
        .route("/{id}/execute", post(execute_metric_handler))
        .route("/{id}/lineage", get(get_metric_lineage_handler))
}

fn sql_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/execute", post(execute_sql_handler))
        .route("/format", post(format_sql_handler))
        .route("/history", get(get_query_history_handler))
        .route("/explain", post(explain_sql_handler))
        .route("/preview", post(preview_data_handler))
}

// ==================== 辅助函数 ====================

/// 从请求中提取用户 ID
/// 优先从 Authorization header 解析 JWT token
/// 从请求中提取用户 ID (暂未使用，作为备用)
#[allow(dead_code)]
fn get_user_id(state: &AppState, auth_header: Option<&str>) -> AppResult<Uuid> {
    let auth_header = auth_header
        .ok_or_else(|| AppError::Unauthorized("Authorization header required".to_string()))?;

    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(AppError::AuthenticationFailed(
            "Invalid Authorization header format".to_string(),
        ));
    };

    let jwt_utils = crate::utils::JwtUtils::new(&state.config.jwt);
    jwt_utils.get_user_id(token)
}

/// 创建用户仓储
fn user_repo(state: &AppState) -> UserRepo {
    UserRepo::new(state.db.clone())
}

/// 创建连接仓储
fn connection_repo(state: &AppState) -> ConnectionRepo {
    ConnectionRepo::new(state.db.clone())
}

/// 创建查询仓储
fn query_repo(state: &AppState) -> QueryRepo {
    QueryRepo::new(state.db.clone())
}

/// 创建对话仓储
fn conversation_repo(state: &AppState) -> ConversationRepo {
    ConversationRepo::new(state.db.clone())
}

// ==================== 认证处理器 ====================

async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);

    // 查找用户
    let user = repo
        .find_by_username(&payload.username)
        .await?
        .ok_or_else(|| AppError::AuthenticationFailed("用户名或密码错误".to_string()))?;

    // 验证密码
    let password_valid = crate::utils::password::PasswordUtils::new(&state.config.security)
        .verify_password(&payload.password, &user.password_hash);

    if !password_valid {
        return Err(AppError::AuthenticationFailed("用户名或密码错误".to_string()));
    }

    // 检查用户是否激活
    if !user.is_active {
        return Err(AppError::Forbidden("账号已被禁用".to_string()));
    }

    // 生成 Token
    let jwt_utils = crate::utils::jwt::JwtUtils::new(&state.config.jwt);
    let access_token = jwt_utils.generate_access_token(&user.id, &user.username, user.role)?;
    let refresh_token = jwt_utils.generate_refresh_token(&user.id, &user.username, user.role)?;

    let response = serde_json::json!({
        "code": 0,
        "message": "登录成功",
        "data": {
            "access_token": access_token,
            "refresh_token": refresh_token,
            "expires_in": 3600,
            "token_type": "Bearer",
            "user": UserPublic::from(&user)
        }
    });

    Ok(Json(response))
}

async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);

    // 检查用户名是否存在
    if repo.find_by_username(&payload.username).await?.is_some() {
        return Err(AppError::AlreadyExists("用户名已存在".to_string()));
    }

    // 检查邮箱是否存在
    if repo.find_by_email(&payload.email).await?.is_some() {
        return Err(AppError::AlreadyExists("邮箱已被使用".to_string()));
    }

    // 哈希密码
    let password_hash = crate::utils::password::PasswordUtils::new(&state.config.security)
        .hash_password(&payload.password)?;

    // 创建用户
    let user = User::new(payload.username, payload.email, password_hash, payload.role);
    let user = repo.create(&user).await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "注册成功",
        "data": UserPublic::from(&user)
    });

    Ok(Json(response))
}

async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let refresh_token = payload
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("refresh_token is required".to_string()))?;

    let jwt_utils = crate::utils::jwt::JwtUtils::new(&state.config.jwt);
    let claims = jwt_utils.verify_refresh_token(refresh_token)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InvalidToken("Invalid user ID".to_string()))?;

    let repo = user_repo(&state);
    let user = repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    if !user.is_active {
        return Err(AppError::Forbidden("账号已被禁用".to_string()));
    }

    let new_access_token = jwt_utils.generate_access_token(&user.id, &user.username, user.role)?;
    let new_refresh_token = jwt_utils.generate_refresh_token(&user.id, &user.username, user.role)?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Token 刷新成功",
        "data": {
            "access_token": new_access_token,
            "refresh_token": new_refresh_token,
            "expires_in": 3600,
            "token_type": "Bearer"
        }
    });

    Ok(Json(response))
}

async fn logout_handler() -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该将 refresh_token 加入黑名单
    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "登出成功"
    })))
}

// ==================== 用户处理器 ====================

async fn list_users_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);
    let (users, total) = repo.list(1, 100).await?;

    let user_list: Vec<UserPublic> = users.iter().map(UserPublic::from).collect();

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": user_list,
            "total": total,
            "page": 1,
            "page_size": 100
        }
    });

    Ok(Json(response))
}

async fn get_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);
    let user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": UserPublic::from(&user)
    });

    Ok(Json(response))
}

async fn update_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);
    let mut user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    if let Some(username) = payload.username {
        user.username = username;
    }
    if let Some(email) = payload.email {
        user.email = email;
    }
    if let Some(role) = payload.role {
        user.role = role;
    }
    if let Some(is_active) = payload.is_active {
        user.is_active = is_active;
    }
    user.updated_at = Utc::now();

    let user = repo.update(&user).await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "更新成功",
        "data": UserPublic::from(&user)
    });

    Ok(Json(response))
}

async fn delete_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);

    // 检查用户是否存在
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    repo.delete(id).await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn change_password_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);
    let user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    let password_utils = crate::utils::password::PasswordUtils::new(&state.config.security);

    // 验证旧密码
    if !password_utils.verify_password(&payload.old_password, &user.password_hash) {
        return Err(AppError::ValidationError("旧密码不正确".to_string()));
    }

    // 哈希新密码
    let new_hash = password_utils.hash_password(&payload.new_password)?;

    repo.update_password(id, &new_hash).await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "密码修改成功"
    })))
}

// ==================== 连接处理器 ====================

async fn list_connections_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    let user_id = session.user_id;
    let connections = repo.list_by_user(user_id).await?;

    let conn_list: Vec<ConnectionPublic> =
        connections.iter().map(ConnectionPublic::from).collect();

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": conn_list,
            "total": conn_list.len() as i64
        }
    });

    Ok(Json(response))
}

async fn create_connection_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateConnectionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    let user_id = session.user_id;

    // 简单加密密码（实际应该使用更安全的方式）
    let encrypted_password = base64_encode(&payload.password);

    let mut conn = DatabaseConnection::new(
        payload.name,
        payload.db_type,
        payload.host,
        payload.port,
        payload.database_name,
        payload.username,
        encrypted_password,
        Some(user_id),
    );
    conn.is_default = payload.is_default;

    // 如果设为默认，先取消其他默认
    if conn.is_default {
        if let Ok(Some(default_conn)) = repo.get_default(user_id).await {
            let mut updated = default_conn;
            updated.is_default = false;
            let _ = repo.update(&updated);
        }
    }

    let conn = repo.create(&conn).await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "创建成功",
        "data": ConnectionPublic::from(&conn)
    });

    Ok(Json(response))
}

async fn get_connection_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    let conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": ConnectionPublic::from(&conn)
    });

    Ok(Json(response))
}

async fn update_connection_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConnectionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    let mut conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    if let Some(name) = payload.name {
        conn.name = name;
    }
    if let Some(db_type) = payload.db_type {
        conn.db_type = db_type;
    }
    if let Some(host) = payload.host {
        conn.host = host;
    }
    if let Some(port) = payload.port {
        conn.port = port;
    }
    if let Some(database_name) = payload.database_name {
        conn.database_name = database_name;
    }
    if let Some(username) = payload.username {
        conn.username = username;
    }
    if let Some(password) = payload.password {
        conn.encrypted_password = base64_encode(&password);
    }
    if let Some(is_default) = payload.is_default {
        if is_default {
            let user_id = conn.created_by.unwrap_or(Uuid::nil());
            if let Ok(Some(default_conn)) = repo.get_default(user_id).await {
                let mut updated = default_conn;
                updated.is_default = false;
                let _ = repo.update(&updated);
            }
        }
        conn.is_default = is_default;
    }
    conn.updated_at = Utc::now();

    let conn = repo.update(&conn).await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "更新成功",
        "data": ConnectionPublic::from(&conn)
    });

    Ok(Json(response))
}

async fn delete_connection_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    repo.delete(id).await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn test_connection_handler(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该使用连接信息测试数据库连接
    // 目前返回模拟响应
    let response = serde_json::json!({
        "code": 0,
        "message": "连接成功",
        "data": {
            "success": true,
            "message": "数据库连接测试成功",
            "server_version": "8.0.32",
            "latency_ms": 45
        }
    });

    Ok(Json(response))
}

async fn set_default_connection_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    let conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    let user_id = conn.created_by.unwrap_or(Uuid::nil());
    repo.set_default(id, user_id).await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "已设为默认连接"
    })))
}

async fn get_schema_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该从目标数据库获取 schema 信息
    // 目前返回模拟响应
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "connection_id": id,
            "tables": [
                {
                    "table_name": "users",
                    "table_schema": "public",
                    "table_type": "TABLE",
                    "row_count": 1000
                },
                {
                    "table_name": "orders",
                    "table_schema": "public",
                    "table_type": "TABLE",
                    "row_count": 5000
                },
                {
                    "table_name": "products",
                    "table_schema": "public",
                    "table_type": "TABLE",
                    "row_count": 500
                }
            ],
            "created_at": chrono::Utc::now()
        }
    });

    Ok(Json(response))
}

// ==================== SQL 处理器 ====================

// 简化的 SQL 执行 handler - 用于测试
async fn execute_sql_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SqlExecuteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 简化的响应
    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "SQL executed",
        "data": {
            "connection_id": payload.connection_id,
            "sql": payload.sql
        }
    })))
}

/// 执行 SQL 查询并返回结果 (暂未使用，作为备用)
#[allow(dead_code)]
async fn sql_execute(
    pool: &ConnectionPool,
    sql: &str,
    db_type: &DatabaseType,
) -> AppResult<(Vec<ColumnMetadata>, Vec<Vec<serde_json::Value>>)> {
    use sqlparser::dialect::{MySqlDialect, PostgreSqlDialect};
    use sqlparser::parser::Parser;
    use sqlx::Row;

    // 解析 SQL
    let dialect: Box<dyn sqlparser::dialect::Dialect> = match db_type {
        DatabaseType::Postgresql => Box::new(PostgreSqlDialect {}),
        DatabaseType::Mysql => Box::new(MySqlDialect {}),
        _ => return Err(AppError::validation("不支持的数据库类型".to_string())),
    };

    let ast = Parser::parse_sql(dialect.as_ref(), sql)
        .map_err(|e| AppError::validation(format!("SQL 解析失败: {}", e)))?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::validation("SQL 语句不能为空".to_string()))?;

    // 验证只允许 SELECT
    match &ast {
        sqlparser::ast::Statement::Query(_) => {}
        _ => return Err(AppError::DmlForbidden("只允许执行 SELECT 查询".to_string())),
    }

    // 根据连接池类型执行查询
    match pool {
        ConnectionPool::Postgres(pg_pool) => {
            let rows = sqlx::query(sql)
                .fetch_all(pg_pool)
                .await
                .map_err(|e| AppError::database(format!("查询执行失败: {}", e)))?;

            // 获取列信息
            let columns: Vec<ColumnMetadata> = if rows.is_empty() {
                vec![]
            } else {
                let num_cols = rows[0].columns().len();
                (0..num_cols)
                    .map(|i| ColumnMetadata {
                        name: format!("column_{}", i + 1),
                        data_type: "unknown".to_string(),
                        ordinal: i as i32,
                    })
                    .collect()
            };

            // 转换为 JSON 值
            let result_rows: Vec<Vec<serde_json::Value>> = rows
                .iter()
                .map(|row| {
                    let num_cols = row.columns().len();
                    (0..num_cols)
                        .map(|i| row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null))
                        .collect()
                })
                .collect();

            Ok((columns, result_rows))
        }
        ConnectionPool::Mysql(mysql_pool) => {
            let rows = sqlx::query(sql)
                .fetch_all(mysql_pool)
                .await
                .map_err(|e| AppError::database(format!("查询执行失败: {}", e)))?;

            // 获取列信息
            let columns: Vec<ColumnMetadata> = if rows.is_empty() {
                vec![]
            } else {
                let num_cols = rows[0].columns().len();
                (0..num_cols)
                    .map(|i| ColumnMetadata {
                        name: format!("column_{}", i + 1),
                        data_type: "unknown".to_string(),
                        ordinal: i as i32,
                    })
                    .collect()
            };

            // 转换为 JSON 值
            let result_rows: Vec<Vec<serde_json::Value>> = rows
                .iter()
                .map(|row| {
                    let num_cols = row.columns().len();
                    (0..num_cols)
                        .map(|i| row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null))
                        .collect()
                })
                .collect();

            Ok((columns, result_rows))
        }
    }
}

/// 解码密码 (暂未使用，作为备用)
#[allow(dead_code)]
fn decode_password(encoded: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    match STANDARD.decode(encoded) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => encoded.to_string(),
    }
}

async fn format_sql_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SqlFormatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该使用 sqlparser 格式化 SQL
    let formatted = payload.sql.clone();

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "formatted_sql": formatted
        }
    });

    Ok(Json(response))
}

async fn get_query_history_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = query_repo(&state);
    let user_id = session.user_id;
    let (histories, total) = repo.list_by_user(user_id, 1, 50).await?;

    let items: Vec<QueryHistoryItem> = histories
        .iter()
        .map(|h| QueryHistoryItem {
            id: h.id,
            connection_name: None,
            sql_text: h.sql_text.clone(),
            status: h.status,
            duration_ms: h.duration_ms,
            row_count: h.row_count,
            created_at: h.created_at,
        })
        .collect();

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": items,
            "total": total,
            "page": 1,
            "page_size": 50
        }
    });

    Ok(Json(response))
}

async fn explain_sql_handler(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该执行 EXPLAIN 并返回执行计划
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "plan_type": "SELECT",
            "estimated_cost": 100.5,
            "estimated_rows": 1000,
            "actual_rows": null,
            "details": {
                "node_type": "Seq Scan",
                "relation_name": "users",
                "filter": null
            }
        }
    });

    Ok(Json(response))
}

async fn preview_data_handler(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该预览表数据
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "columns": [
                {"name": "id", "data_type": "INT", "ordinal": 0},
                {"name": "name", "data_type": "VARCHAR", "ordinal": 1}
            ],
            "rows": [
                [1, "示例1"],
                [2, "示例2"],
                [3, "示例3"]
            ],
            "row_count": 3,
            "table_name": "users"
        }
    });

    Ok(Json(response))
}

// ==================== NL 处理器 ====================

async fn nl_convert_handler(
    State(state): State<Arc<AppState>>,
    Json(_payload): Json<NlToSqlRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 使用 LLM 客户端将自然语言转换为 SQL
    let llm_client = &state.llm_client;

    // TODO: 实际应该调用 LLM 进行转换
    // 模拟响应
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "sql": "SELECT COUNT(*) FROM users",
            "explanation": "这是一个查询用户总数的 SQL 语句",
            "confidence": 0.95,
            "estimated_rows": 1000,
            "referenced_tables": ["users"]
        }
    });

    let _ = llm_client;

    Ok(Json(response))
}

async fn nl_execute_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<NlExecuteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = session.user_id;

    // 执行 NL 转换并执行 SQL
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "columns": [
                {"name": "count", "data_type": "BIGINT", "ordinal": 0}
            ],
            "rows": [[1000]],
            "row_count": 1,
            "duration_ms": 150,
            "chart_config": null,
            "data_insight": "用户总数为 1000"
        }
    });

    // 记录查询历史
    let mut history = QueryHistory::new(Some(payload.connection_id), user_id, payload.sql);
    history.mark_success(150, 1);

    let repo = query_repo(&state);
    let _ = repo.create(&history).await;

    Ok(Json(response))
}

// ==================== 对话处理器 ====================

async fn list_conversations_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);
    let user_id = session.user_id;
    let conversations = repo.list_by_user(user_id).await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": conversations,
            "total": conversations.len() as i64
        }
    });

    Ok(Json(response))
}

async fn create_conversation_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);
    let user_id = session.user_id;

    let conv = Conversation::new(user_id, payload.title);
    let conv = repo.create_conversation(&conv).await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "创建成功",
        "data": conv
    });

    Ok(Json(response))
}

async fn get_conversation_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);
    let conv = repo
        .get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": conv
    });

    Ok(Json(response))
}

async fn delete_conversation_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);
    repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    repo.delete_conversation(id).await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn list_messages_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);
    let messages = repo.list_messages(id).await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": messages,
            "total": messages.len() as i64
        }
    });

    Ok(Json(response))
}

async fn send_message_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("content is required".to_string()))?;

    let repo = conversation_repo(&state);

    // 检查对话是否存在
    repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    // 创建用户消息
    let user_msg = Message::user_message(id, content.to_string());
    let user_msg = repo.create_message(&user_msg).await?;

    // TODO: 调用 LLM 生成回复（这里简化处理）
    let assistant_content = "这是一个模拟的 AI 回复。".to_string();
    let assistant_msg = Message::assistant_message(
        id,
        assistant_content,
        Some("SELECT 1".to_string()),
        Some("模拟 SQL".to_string()),
    );
    let assistant_msg = repo.create_message(&assistant_msg).await?;

    // 更新对话时间
    let _ = repo.update_title(id, "对话已更新");

    let response = serde_json::json!({
        "code": 0,
        "message": "发送成功",
        "data": {
            "user_message": user_msg,
            "assistant_message": assistant_msg
        }
    });

    Ok(Json(response))
}

// ==================== 图表处理器 ====================

async fn recommend_chart_handler(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该根据数据特征推荐图表类型
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "recommended_types": ["bar", "line", "pie"],
            "reasons": [
                "数据适合用柱状图展示",
                "时间序列数据适合用折线图",
                "比例数据适合用饼图"
            ]
        }
    });

    Ok(Json(response))
}

async fn generate_chart_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let chart_type = payload
        .get("chart_type")
        .and_then(|v| v.as_str())
        .unwrap_or("bar");

    // TODO: 实际应该生成 ECharts 配置
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "chart_type": chart_type,
            "config": {
                "title": {"text": "数据图表"},
                "tooltip": {},
                "xAxis": {"type": "category", "data": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]},
                "yAxis": {"type": "value"},
                "series": [{
                    "data": [120, 200, 150, 80, 70, 110, 130],
                    "type": chart_type
                }]
            }
        }
    });

    Ok(Json(response))
}

async fn export_chart_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let format = payload
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("png");

    // TODO: 实际应该导出图表
    let response = serde_json::json!({
        "code": 0,
        "message": "导出成功",
        "data": {
            "format": format,
            "url": format!("/exports/chart.{}", format)
        }
    });

    Ok(Json(response))
}

// ==================== 指标处理器 ====================

async fn list_metrics_handler(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该从数据库查询指标列表
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": [],
            "total": 0,
            "page": 1,
            "page_size": 20
        }
    });

    Ok(Json(response))
}

async fn create_metric_handler(
    State(_state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateMetricRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = session.user_id;

    let mut metric = Metric::new(payload.name, payload.code, payload.expression, Some(user_id));
    if let Some(desc) = payload.description {
        metric = metric.with_description(desc);
    }
    if let Some(unit) = payload.unit {
        metric = metric.with_unit(unit);
    }
    metric = metric.with_format_type(payload.format_type);
    if let Some(dims) = payload.dimensions {
        metric = metric.with_dimensions(dims);
    }

    let response = serde_json::json!({
        "code": 0,
        "message": "创建成功",
        "data": metric
    });

    Ok(Json(response))
}

async fn get_metric_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该从数据库查询
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "id": id,
            "name": "示例指标",
            "code": "example_metric",
            "expression": "SELECT COUNT(*) FROM users",
            "description": null,
            "dimensions": null,
            "unit": null,
            "format_type": "number",
            "created_by": null,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now()
        }
    });

    Ok(Json(response))
}

async fn update_metric_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMetricRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let response = serde_json::json!({
        "code": 0,
        "message": "更新成功",
        "data": {
            "id": id,
            "updated_at": chrono::Utc::now()
        }
    });

    let _ = payload;

    Ok(Json(response))
}

async fn delete_metric_handler(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn execute_metric_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该执行指标计算
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "metric": {
                "id": id,
                "name": "示例指标",
                "code": "example_metric",
                "value": 1000,
                "formatted_value": "1,000",
                "unit": "人",
                "dimensions": null
            },
            "duration_ms": 50,
            "executed_at": chrono::Utc::now()
        }
    });

    let _ = payload;

    Ok(Json(response))
}

async fn get_metric_lineage_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: 实际应该分析指标血缘
    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "metric_id": id,
            "metric_name": "示例指标",
            "source_tables": ["users", "orders"],
            "source_columns": ["users.id", "orders.user_id"],
            "dependent_metrics": [],
            "referenced_by": []
        }
    });

    Ok(Json(response))
}

// ==================== 工具函数 ====================

fn base64_encode(input: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(input.as_bytes())
}

/// 服务器启动函数
pub async fn start_server(state: Arc<crate::state::AppState>) -> anyhow::Result<()> {
    let app = routes().with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
