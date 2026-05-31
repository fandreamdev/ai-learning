//! API 路由定义
//!
//! 定义所有 API 端点

use axum::{
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use sqlx::{Column, Row, TypeInfo};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::error::{AppError, AppResult};
use crate::models::{
    ChangePasswordRequest, ColumnMetadata, ConnectionPublic, Conversation, CreateConnectionRequest,
    CreateConversationRequest, CreateMetricRequest, CreateUserRequest, DatabaseConnection,
    DatabaseType, FormatType, LoginRequest, Message, MessageRole, Metric, NlExecuteRequest, NlToSqlRequest,
    QueryHistory, QueryHistoryItem, SendMessageRequest, SqlExecuteRequest, SqlFormatRequest,
    UpdateConnectionRequest, UpdateMetricRequest,
    UpdateUserRequest, User, UserPublic, UserSession,
};
use crate::repositories::{ConnectionRepo, ConversationRepo, QueryRepo, UserRepo};
use crate::services::chart_generator::ChartGenerator;
use crate::services::connection_manager::{ConnectionConfig, ConnectionPool};
use crate::state::AppState;

/// 预览数据请求
#[derive(Debug, serde::Deserialize)]
struct PreviewDataRequest {
    connection_id: Uuid,
    table_name: String,
    #[serde(default = "default_preview_limit")]
    limit: i32,
}

fn default_preview_limit() -> i32 {
    100
}

/// 图表推荐请求
#[derive(Debug, serde::Deserialize)]
struct ChartRecommendRequest {
    columns: Vec<ColumnMetadata>,
    rows: Vec<Vec<serde_json::Value>>,
}

/// 图表生成请求
#[derive(Debug, serde::Deserialize)]
struct ChartGenerateRequest {
    columns: Vec<ColumnMetadata>,
    rows: Vec<Vec<serde_json::Value>>,
    chart_type: String,
}

/// 图表导出请求
#[derive(Debug, serde::Deserialize)]
struct ChartExportRequest {
    config: serde_json::Value,
    format: Option<String>,
    filename: Option<String>,
}

/// 指标列表请求
#[derive(Debug, serde::Deserialize)]
struct MetricListRequest {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
    query: Option<String>,
}

fn default_page() -> i32 { 1 }
fn default_page_size() -> i32 { 20 }

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

    if state.token_blacklist.contains(refresh_token).await {
        return Err(AppError::InvalidToken("Refresh token has been revoked".to_string()));
    }

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

async fn logout_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 将 refresh_token 加入黑名单
    if let Some(refresh_token) = payload.get("refresh_token").and_then(|v| v.as_str()) {
        state.token_blacklist.add(refresh_token).await;
    }

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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = get_connection_config(&state, id).await?;
    let start = std::time::Instant::now();

    let test_result = match config.db_type {
        DatabaseType::Postgresql => {
            state.connection_manager.test_connection_pg(&config).await
        }
        DatabaseType::Mysql => {
            state.connection_manager.test_connection_mysql(&config).await
        }
        _ => Err(AppError::validation("不支持的数据库类型".to_string())),
    };

    let latency_ms = start.elapsed().as_millis() as i64;

    match test_result {
        Ok(version) => Ok(Json(serde_json::json!({
            "code": 0,
            "message": "连接成功",
            "data": {
                "success": true,
                "message": "数据库连接测试成功",
                "server_version": version,
                "latency_ms": latency_ms
            }
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "code": 0,
            "message": "连接失败",
            "data": {
                "success": false,
                "message": format!("连接失败: {}", e),
                "server_version": null,
                "latency_ms": latency_ms
            }
        }))),
    }
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = get_connection_config(&state, id).await?;
    let tables = state.connection_manager.get_schema(&config).await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "connection_id": id,
            "tables": tables,
            "created_at": chrono::Utc::now()
        }
    });

    Ok(Json(response))
}

// ==================== SQL 处理器 ====================

async fn execute_sql_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<SqlExecuteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = get_connection_config(&state, payload.connection_id).await?;
    let pool = state.connection_manager.get_pool(&config).await?;
    let start = std::time::Instant::now();

    let mut history = QueryHistory::new(
        Some(payload.connection_id),
        session.user_id,
        payload.sql.clone(),
    );

    let (columns, rows) = match sql_execute(&pool, &payload.sql, &config.db_type).await {
        Ok(result) => result,
        Err(e) => {
            history.mark_failed(e.to_string());
            let _ = query_repo(&state).create(&history).await;
            return Err(e);
        }
    };

    let duration_ms = start.elapsed().as_millis() as i64;
    let row_count = rows.len() as i64;
    history.mark_success(duration_ms, row_count);
    let history = query_repo(&state).create(&history).await.unwrap_or(history);

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "query_id": history.id,
            "columns": columns,
            "rows": rows,
            "row_count": row_count,
            "duration_ms": duration_ms,
            "execution_plan": null
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

    // 解析 SQL
    {
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
    }

    // 根据连接池类型执行查询
    match pool {
        ConnectionPool::Postgres(pg_pool) => {
            let rows = sqlx::query(sql)
                .fetch_all(pg_pool)
                .await
                .map_err(|e| AppError::database(format!("查询执行失败: {}", e)))?;

            Ok(extract_pg_rows_data(rows))
        }
        ConnectionPool::Mysql(mysql_pool) => {
            let rows = sqlx::query(sql)
                .fetch_all(mysql_pool)
                .await
                .map_err(|e| AppError::database(format!("查询执行失败: {}", e)))?;

            Ok(extract_mysql_rows_data(rows))
        }
    }
}

/// 解码密码
#[allow(dead_code)]
fn decode_password(encoded: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    match STANDARD.decode(encoded) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => encoded.to_string(),
    }
}

/// 从连接 ID 获取连接配置
async fn get_connection_config(
    state: &AppState,
    connection_id: Uuid,
) -> AppResult<ConnectionConfig> {
    let repo = connection_repo(state);
    let conn = repo
        .find_by_id(connection_id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    Ok(ConnectionConfig {
        id: conn.id,
        name: conn.name,
        db_type: conn.db_type,
        host: conn.host.clone(),
        port: conn.port,
        database_name: conn.database_name.clone(),
        username: conn.username.clone(),
        password: decode_password(&conn.encrypted_password),
    })
}

async fn format_sql_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SqlFormatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    use sqlparser::dialect::{PostgreSqlDialect, GenericDialect};
    use sqlparser::parser::Parser;

    let dialect = match payload.dialect.as_str() {
        "mysql" => Box::new(GenericDialect {}) as Box<dyn sqlparser::dialect::Dialect>,
        "postgresql" | "postgres" => Box::new(PostgreSqlDialect {}) as Box<dyn sqlparser::dialect::Dialect>,
        _ => Box::new(GenericDialect {}) as Box<dyn sqlparser::dialect::Dialect>,
    };

    let statements = Parser::parse_sql(dialect.as_ref(), &payload.sql)
        .map_err(|e| AppError::validation(format!("SQL 解析失败: {}", e)))?;

    let formatted = statements
        .first()
        .map(|s| s.to_string())
        .unwrap_or(payload.sql);

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
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SqlExecuteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = get_connection_config(&state, payload.connection_id).await?;
    let pool_type = state.connection_manager.get_pool(&config).await?;

    let explain_sql = format!("EXPLAIN {}", payload.sql);
    let start = std::time::Instant::now();

    let result = match pool_type {
        ConnectionPool::Postgres(pg_pool) => {
            let row: (String,) = sqlx::query_as(&explain_sql)
                .fetch_one(&pg_pool)
                .await
                .map_err(|e| AppError::database(format!("EXPLAIN 失败: {}", e)))?;
            row.0
        }
        ConnectionPool::Mysql(mysql_pool) => {
            let row: (String,) = sqlx::query_as(&explain_sql)
                .fetch_one(&mysql_pool)
                .await
                .map_err(|e| AppError::database(format!("EXPLAIN 失败: {}", e)))?;
            row.0
        }
    };

    let duration_ms = start.elapsed().as_millis() as i64;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "plan_type": "SELECT",
            "estimated_cost": null,
            "estimated_rows": null,
            "actual_rows": null,
            "duration_ms": duration_ms,
            "details": {
                "raw": result
            }
        }
    });

    Ok(Json(response))
}

async fn preview_data_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PreviewDataRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let table_name = validate_table_identifier(&payload.table_name)?;
    let limit = payload.limit.clamp(1, 1000);
    let config = get_connection_config(&state, payload.connection_id).await?;
    let pool_type = state.connection_manager.get_pool(&config).await?;

    let sql = format!("SELECT * FROM {} LIMIT {}", table_name, limit);

    let (columns, result_rows) = match pool_type {
        ConnectionPool::Postgres(pg_pool) => {
            let rows = sqlx::query(&sql)
                .fetch_all(&pg_pool)
                .await
                .map_err(|e| AppError::database(format!("预览表数据失败: {}", e)))?;
            extract_pg_rows_data(rows)
        }
        ConnectionPool::Mysql(mysql_pool) => {
            let rows = sqlx::query(&sql)
                .fetch_all(&mysql_pool)
                .await
                .map_err(|e| AppError::database(format!("预览表数据失败: {}", e)))?;
            extract_mysql_rows_data(rows)
        }
    };

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "columns": columns,
            "rows": result_rows,
            "row_count": result_rows.len() as i64,
            "table_name": table_name
        }
    });

    Ok(Json(response))
}

fn validate_table_identifier(identifier: &str) -> AppResult<String> {
    let re = regex::Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)?$")
        .map_err(|e| AppError::internal(format!("表名校验器初始化失败: {}", e)))?;
    if re.is_match(identifier) {
        Ok(identifier.to_string())
    } else {
        Err(AppError::ValidationError("Invalid table name".to_string()))
    }
}

// ==================== NL 处理器 ====================

async fn nl_convert_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NlToSqlRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let llm_client = &state.llm_client;

    // 获取 schema 上下文
    let schema_context = if let Ok(config) = get_connection_config(&state, payload.connection_id).await {
        match state.connection_manager.get_schema(&config).await {
            Ok(tables) => {
                let table_info: Vec<String> = tables
                    .iter()
                    .map(|t| format!("{}.{} ({})", t.table_schema, t.table_name, t.table_type))
                    .collect();
                Some(table_info.join(", "))
            }
            Err(_) => None,
        }
    } else {
        None
    };

    // 调用 LLM 进行转换
    let result = llm_client
        .convert_nl_to_sql(&payload.question, schema_context.as_deref())
        .await?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "sql": result.sql,
            "explanation": result.explanation,
            "confidence": result.confidence,
            "estimated_rows": result.estimated_rows,
            "referenced_tables": result.referenced_tables
        }
    });

    Ok(Json(response))
}

async fn nl_execute_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<NlExecuteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = session.user_id;
    let config = get_connection_config(&state, payload.connection_id).await?;
    let pool = state.connection_manager.get_pool(&config).await?;
    let start = std::time::Instant::now();

    let mut history = QueryHistory::new(Some(payload.connection_id), user_id, payload.sql.clone());
    let (columns, rows) = match sql_execute(&pool, &payload.sql, &config.db_type).await {
        Ok(result) => result,
        Err(e) => {
            history.mark_failed(e.to_string());
            let _ = query_repo(&state).create(&history).await;
            return Err(e);
        }
    };

    let duration_ms = start.elapsed().as_millis() as i64;
    let row_count = rows.len() as i64;
    history.mark_success(duration_ms, row_count);
    let _ = query_repo(&state).create(&history).await;

    let chart_config = if let Some(chart_type) = payload.chart_type.as_deref() {
        let generator = ChartGenerator::new();
        match generator.switch_chart_type(&columns, &rows, chart_type) {
            Ok(config) => Some(config.echarts_config),
            Err(_) => None,
        }
    } else {
        None
    };

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "columns": columns,
            "rows": rows,
            "row_count": row_count,
            "duration_ms": duration_ms,
            "chart_config": chart_config,
            "data_insight": null
        }
    });

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
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);

    // 检查对话是否存在
    repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    // 创建用户消息
    let user_msg = Message::user_message(id, payload.content.clone());
    let user_msg = repo.create_message(&user_msg).await?;

    // 获取对话历史用于上下文
    let messages = repo.list_messages(id).await?;
    let history_context: String = messages
        .iter()
        .take(10)
        .map(|m| format!("{}: {}", if m.role == MessageRole::User { "用户" } else { "助手" }, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    // 调用 LLM 生成回复
    let llm_client = &state.llm_client;
    let prompt = format!(
        "你是一个专业的 SQL 助手。用户正在与你对话。\n\n历史对话:\n{}\n\n当前用户消息: {}",
        history_context, payload.content
    );

    let llm_response = llm_client.call_llm(&prompt, "gpt-4o-mini").await;

    let (assistant_content, sql_result, explanation) = match llm_response {
        Ok(response) => {
            // 尝试从响应中提取 SQL
            let sql = extract_sql_from_text(&response);
            let explanation = if sql.is_some() {
                "已根据您的问题生成 SQL 查询".to_string()
            } else {
                response.clone()
            };
            (response, sql, Some(explanation))
        }
        Err(e) => {
            (format!("抱歉，我无法处理您的请求: {}", e), None, None)
        }
    };

    let assistant_msg = Message::assistant_message(id, assistant_content, sql_result, explanation);
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

/// 从 PostgreSQL 行提取列和数据
#[allow(dead_code)]
fn extract_pg_rows_data(rows: Vec<sqlx::postgres::PgRow>) -> (Vec<ColumnMetadata>, Vec<Vec<serde_json::Value>>) {
    let columns: Vec<ColumnMetadata> = if rows.is_empty() {
        vec![]
    } else {
        rows[0]
            .columns()
            .iter()
            .enumerate()
            .map(|(i, col)| ColumnMetadata {
                name: col.name().to_string(),
                data_type: col.type_info().name().to_string(),
                ordinal: i as i32,
            })
            .collect()
    };

    let result_rows: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| {
            (0..row.columns().len())
                .map(|i| pg_value_to_json(row, i))
                .collect()
        })
        .collect();

    (columns, result_rows)
}

/// 从 MySQL 行提取列和数据
#[allow(dead_code)]
fn extract_mysql_rows_data(rows: Vec<sqlx::mysql::MySqlRow>) -> (Vec<ColumnMetadata>, Vec<Vec<serde_json::Value>>) {
    let columns: Vec<ColumnMetadata> = if rows.is_empty() {
        vec![]
    } else {
        rows[0]
            .columns()
            .iter()
            .enumerate()
            .map(|(i, col)| ColumnMetadata {
                name: col.name().to_string(),
                data_type: col.type_info().name().to_string(),
                ordinal: i as i32,
            })
            .collect()
    };

    let result_rows: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| {
            (0..row.columns().len())
                .map(|i| mysql_value_to_json(row, i))
                .collect()
        })
        .collect();

    (columns, result_rows)
}

fn pg_value_to_json(row: &sqlx::postgres::PgRow, index: usize) -> serde_json::Value {
    if let Ok(v) = row.try_get::<serde_json::Value, _>(index) {
        return v;
    }
    if let Ok(v) = row.try_get::<String, _>(index) {
        return serde_json::Value::String(v);
    }
    if let Ok(v) = row.try_get::<i64, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<i32, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<f64, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<f32, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<bool, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(index) {
        return serde_json::Value::String(v.to_string());
    }
    if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(index) {
        return serde_json::Value::String(v.to_rfc3339());
    }
    serde_json::Value::Null
}

fn mysql_value_to_json(row: &sqlx::mysql::MySqlRow, index: usize) -> serde_json::Value {
    if let Ok(v) = row.try_get::<serde_json::Value, _>(index) {
        return v;
    }
    if let Ok(v) = row.try_get::<String, _>(index) {
        return serde_json::Value::String(v);
    }
    if let Ok(v) = row.try_get::<i64, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<i32, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<f64, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<f32, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<bool, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(index) {
        return serde_json::Value::String(v.to_string());
    }
    serde_json::Value::Null
}

/// 从文本中提取 SQL 语句
fn extract_sql_from_text(text: &str) -> Option<String> {
    // 尝试提取 ```sql ... ``` 块
    let re = regex::Regex::new(r"```sql\s*([\s\S]*?)\s*```").ok()?;
    if let Some(caps) = re.captures(text) {
        if let Some(sql_match) = caps.get(1) {
            return Some(sql_match.as_str().trim().to_string());
        }
    }

    // 尝试提取 ``` ... ``` 块
    let re = regex::Regex::new(r"```\s*([\s\S]*?)\s*```").ok()?;
    if let Some(caps) = re.captures(text) {
        if let Some(code_match) = caps.get(1) {
            let content = code_match.as_str().trim();
            if content.to_uppercase().contains("SELECT") {
                return Some(content.to_string());
            }
        }
    }

    // 尝试提取 SELECT ... 结尾的语句
    let re = regex::Regex::new(r"(?i)(SELECT[\s\S]+?)(?:\n\n|$)").ok()?;
    if let Some(caps) = re.captures(text) {
        if let Some(sql_match) = caps.get(1) {
            return Some(sql_match.as_str().trim().to_string());
        }
    }

    None
}

// ==================== 图表处理器 ====================

async fn recommend_chart_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<ChartRecommendRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let generator = ChartGenerator::new();
    let recommendation = generator.recommend(&payload.columns, &payload.rows)?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "recommended": recommendation.recommended.as_str(),
            "recommended_types": recommendation.alternatives.iter().map(|t| t.as_str()).collect::<Vec<_>>(),
            "reasons": recommendation.reasons,
            "chart_config": recommendation.chart_config
        }
    });

    Ok(Json(response))
}

async fn generate_chart_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<ChartGenerateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let generator = ChartGenerator::new();
    let config = generator.switch_chart_type(
        &payload.columns,
        &payload.rows,
        &payload.chart_type,
    )?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "chart_type": config.chart_type,
            "config": config.echarts_config
        }
    });

    Ok(Json(response))
}

async fn export_chart_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<ChartExportRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let format = payload.format.as_deref().unwrap_or("json");
    let filename = payload.filename.as_deref().unwrap_or("chart");

    // 生成导出文件路径
    let export_dir = std::env::temp_dir().join("chart_exports");
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| AppError::internal(format!("无法创建导出目录: {}", e)))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.{}", filename, timestamp, format);
    let file_path = export_dir.join(&filename);

    match format {
        "json" => {
            // 保存为 JSON 文件
            let json_str = serde_json::to_string_pretty(&payload.config)
                .map_err(|e| AppError::internal(format!("JSON 序列化失败: {}", e)))?;
            std::fs::write(&file_path, json_str)
                .map_err(|e| AppError::internal(format!("写入文件失败: {}", e)))?;
        }
        "svg" => {
            // ECharts 支持导出 SVG
            if let Some(svg_data) = payload.config.get("svg") {
                let svg_str = svg_data.as_str()
                    .ok_or_else(|| AppError::ValidationError("无效的 SVG 数据".to_string()))?;
                std::fs::write(&file_path, svg_str)
                    .map_err(|e| AppError::internal(format!("写入 SVG 失败: {}", e)))?;
            } else {
                // 如果没有 SVG，返回 JSON 配置
                let json_str = serde_json::to_string_pretty(&payload.config)
                    .map_err(|e| AppError::internal(format!("JSON 序列化失败: {}", e)))?;
                std::fs::write(&file_path, json_str)
                    .map_err(|e| AppError::internal(format!("写入文件失败: {}", e)))?;
            }
        }
        _ => {
            // 其他格式，返回 JSON 配置
            let json_str = serde_json::to_string_pretty(&payload.config)
                .map_err(|e| AppError::internal(format!("JSON 序列化失败: {}", e)))?;
            std::fs::write(&file_path, json_str)
                .map_err(|e| AppError::internal(format!("写入文件失败: {}", e)))?;
        }
    }

    let response = serde_json::json!({
        "code": 0,
        "message": "导出成功",
        "data": {
            "format": format,
            "filename": filename,
            "url": format!("/exports/{}", filename),
            "path": file_path.to_string_lossy()
        }
    });

    Ok(Json(response))
}

// ==================== 指标处理器 ====================

async fn list_metrics_handler(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<MetricListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let store = METRICS_STORE.read()
        .map_err(|_| AppError::internal("无法读取指标存储".to_string()))?;

    let page = params.page.max(1);
    let page_size = params.page_size.max(1).min(100);

    let mut items: Vec<&Metric> = store.values().collect();

    // 过滤
    if let Some(query) = &params.query {
        let query_lower = query.to_lowercase();
        items.retain(|m| {
            m.name.to_lowercase().contains(&query_lower)
                || m.code.to_lowercase().contains(&query_lower)
        });
    }

    // 分页
    let total = items.len() as i64;
    let start = ((page - 1) * page_size) as usize;
    let _end = (start + page_size as usize).min(items.len());
    items = items.into_iter().skip(start).take(page_size as usize).collect();

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": items,
            "total": total,
            "page": page,
            "page_size": page_size
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

    {
        let mut store = METRICS_STORE.write()
            .map_err(|_| AppError::internal("无法写入指标存储".to_string()))?;
        store.insert(metric.id, metric.clone());
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
    let store = METRICS_STORE.read()
        .map_err(|_| AppError::internal("无法读取指标存储".to_string()))?;

    let metric = store.get(&id)
        .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": metric
    });

    Ok(Json(response))
}

async fn update_metric_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMetricRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let updated_metric = {
        let mut store = METRICS_STORE.write()
            .map_err(|_| AppError::internal("无法写入指标存储".to_string()))?;
        let metric = store.get_mut(&id)
            .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?;

        if let Some(name) = payload.name {
            metric.name = name;
        }
        if let Some(description) = payload.description {
            metric.description = Some(description);
        }
        if let Some(expression) = payload.expression {
            metric.expression = expression;
        }
        if let Some(dimensions) = payload.dimensions {
            metric.dimensions = Some(serde_json::json!(dimensions));
        }
        if let Some(unit) = payload.unit {
            metric.unit = Some(unit);
        }
        if let Some(format_type) = payload.format_type {
            metric.format_type = format_type;
        }
        metric.updated_at = chrono::Utc::now();
        metric.clone()
    };

    let response = serde_json::json!({
        "code": 0,
        "message": "更新成功",
        "data": updated_metric
    });

    Ok(Json(response))
}

async fn delete_metric_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    {
        let mut store = METRICS_STORE.write()
            .map_err(|_| AppError::internal("无法写入指标存储".to_string()))?;
        if store.remove(&id).is_none() {
            return Err(AppError::NotFound("指标不存在".to_string()));
        }
    }

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn execute_metric_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 获取指标
    let metric = {
        let store = METRICS_STORE.read()
            .map_err(|_| AppError::internal("无法读取指标存储".to_string()))?;
        store.get(&id)
            .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?
            .clone()
    };

    let start = std::time::Instant::now();

    // 解析参数
    let connection_id = payload.get("connection_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::ValidationError("connection_id is required".to_string()))?;

    let dimensions: std::collections::HashMap<String, String> = payload.get("dimensions")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    // 执行指标表达式
    let config = get_connection_config(&state, connection_id).await?;
    let pool_type = state.connection_manager.get_pool(&config).await?;

    // 替换维度参数
    let mut sql = metric.expression.clone();
    for (key, val) in &dimensions {
        sql = sql.replace(&format!("{{{}}}", key), val);
    }

    let value = match pool_type {
        ConnectionPool::Postgres(pg_pool) => {
            let row: (serde_json::Value,) = sqlx::query_as(&sql)
                .fetch_one(&pg_pool)
                .await
                .map_err(|e| AppError::database(format!("指标执行失败: {}", e)))?;
            row.0
        }
        ConnectionPool::Mysql(mysql_pool) => {
            let row: (serde_json::Value,) = sqlx::query_as(&sql)
                .fetch_one(&mysql_pool)
                .await
                .map_err(|e| AppError::database(format!("指标执行失败: {}", e)))?;
            row.0
        }
    };

    let duration_ms = start.elapsed().as_millis() as i64;

    // 格式化值
    let formatted_value = match &value {
        serde_json::Value::Number(n) => {
            match metric.format_type {
                FormatType::Percent => format!("{:.2}%", n.as_f64().unwrap_or(0.0) * 100.0),
                FormatType::Currency => format!("¥{:.2}", n.as_f64().unwrap_or(0.0)),
                _ => {
                    if let Some(i) = n.as_i64() {
                        format!("{}", i)
                    } else {
                        format!("{:.2}", n.as_f64().unwrap_or(0.0))
                    }
                }
            }
        }
        _ => value.to_string(),
    };

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "metric": {
                "id": metric.id,
                "name": metric.name,
                "code": metric.code,
                "value": value,
                "formatted_value": formatted_value,
                "unit": metric.unit,
                "dimensions": dimensions
            },
            "duration_ms": duration_ms,
            "executed_at": chrono::Utc::now(),
            "error": null
        }
    });

    Ok(Json(response))
}

async fn get_metric_lineage_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 获取指标
    let metric = {
        let store = METRICS_STORE.read()
            .map_err(|_| AppError::internal("无法读取指标存储".to_string()))?;
        store.get(&id)
            .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?
            .clone()
    };

    // 解析 SQL 提取表名和列名
    let (source_tables, source_columns) = analyze_sql_lineage(&metric.expression);

    // 查找依赖的指标
    let dependent_metrics = {
        let store = METRICS_STORE.read()
            .map_err(|_| AppError::internal("无法读取指标存储".to_string()))?;
        store.values()
            .filter(|m| m.id != metric.id)
            .filter(|m| {
                // 检查其他指标是否引用了当前指标
                source_tables.iter().any(|t| m.expression.contains(t))
            })
            .map(|m| serde_json::json!({
                "id": m.id,
                "name": m.name,
                "code": m.code
            }))
            .collect::<Vec<_>>()
    };

    // 查找引用当前指标的指标
    let referenced_by = {
        let store = METRICS_STORE.read()
            .map_err(|_| AppError::internal("无法读取指标存储".to_string()))?;
        store.values()
            .filter(|m| m.id != metric.id)
            .filter(|m| m.expression.contains(&metric.code))
            .map(|m| serde_json::json!({
                "id": m.id,
                "name": m.name,
                "code": m.code
            }))
            .collect::<Vec<_>>()
    };

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "metric_id": metric.id,
            "metric_name": metric.name,
            "metric_code": metric.code,
            "source_tables": source_tables,
            "source_columns": source_columns,
            "dependent_metrics": dependent_metrics,
            "referenced_by": referenced_by
        }
    });

    Ok(Json(response))
}

/// 分析 SQL 语句提取血缘信息
fn analyze_sql_lineage(sql: &str) -> (Vec<String>, Vec<String>) {
    use regex::Regex;

    let mut tables = Vec::new();
    let mut columns = Vec::new();

    // 提取 FROM 和 JOIN 后面的表名
    let table_regex = Regex::new(r"(?i)(?:FROM|JOIN)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    for cap in table_regex.captures_iter(sql) {
        if let Some(table) = cap.get(1) {
            let table_name = table.as_str().to_string();
            if !tables.contains(&table_name) {
                tables.push(table_name);
            }
        }
    }

    // 提取 SELECT 后的列名
    let column_regex = Regex::new(r"(?i)SELECT\s+(.*?)\s+FROM").unwrap();
    if let Some(cap) = column_regex.captures(sql) {
        if let Some(cols_str) = cap.get(1) {
            // 分割可能的多个列
            let cols = cols_str.as_str().split(',');
            for col in cols {
                let col = col.trim();
                // 提取别名或完整列名
                let col_name = if col.contains(" as ") {
                    col.split(" as ").nth(1).unwrap_or(col).trim().to_string()
                } else if col.contains('.') {
                    col.split('.').last().unwrap_or(col).trim().to_string()
                } else {
                    col.to_string()
                };
                if !col_name.is_empty() && col_name != "*" {
                    columns.push(col_name);
                }
            }
        }
    }

    (tables, columns)
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
