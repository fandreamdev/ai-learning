//! API 路由定义
//!
//! 定义所有 API 端点

use axum::{
    extract::{Extension, Path, Query, State},
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use sqlx::{Column, Row, TypeInfo};
use std::sync::Arc;
use tower_http::services::ServeDir;
use uuid::Uuid;
use chrono::Utc;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::middleware::auth::{
    admin_only, auth_middleware, require_chart_generation, require_chat_mode,
    require_connection_management, require_sql_mode,
};
use crate::models::{
    ChangePasswordRequest, ColumnMetadata, ConnectionPublic, Conversation, CreateConnectionRequest,
    BatchSemanticRequest, CreateConversationRequest, CreateMetricRequest, CreateSemanticRequest, CreateUserRequest, DatabaseConnection,
    DatabaseType, FormatType, LoginRequest, Message, MessageRole, Metric, NlExecuteRequest, NlToSqlRequest,
    QueryHistory, QueryHistoryItem, SendMessageRequest, SqlExecuteRequest, SqlFormatRequest,
    SemanticDefinition, UpdateConnectionRequest, UpdateMetricRequest, UpdateSemanticRequest,
    UpdateUserRequest, User, UserPublic, UserRole, UserSession,
};
use crate::repositories::{AuditRepo, ConnectionRepo, ConversationRepo, MetricRepo, QueryRepo, SemanticRepo, UserRepo};
use crate::services::chart_generator::ChartGenerator;
use crate::services::connection_manager::{ConnectionConfig, ConnectionPool};
use crate::services::data_masker::DataMasker;
use crate::services::sql_analyzer::SqlAnalyzer;
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

#[derive(Debug, serde::Deserialize)]
struct QueryHistoryListRequest {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
}

#[derive(Debug, serde::Deserialize)]
struct UserListRequest {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
}

#[derive(Debug, serde::Deserialize)]
struct AuditLogListRequest {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
    query: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SemanticListRequest {
    connection_id: Option<Uuid>,
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
    query: Option<String>,
}

fn default_page() -> i32 { 1 }
fn default_page_size() -> i32 { 20 }

// ==================== 路由定义 ====================

pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let protected_routes = Router::new()
        .nest("/users", user_routes())
        .nest("/connections", connection_routes())
        .nest("/sql", sql_routes())
        .nest("/nl", nl_routes())
        .nest("/conversations", conversation_routes())
        .nest("/charts", chart_routes())
        .nest("/metrics", metric_routes())
        .nest("/audit-logs", audit_routes())
        .nest("/semantics", semantic_routes())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    let api = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_routes())
        .nest_service("/exports", ServeDir::new(std::env::temp_dir().join("chart_exports")))
        .merge(protected_routes);

    api
}

// ==================== 健康检查 ====================

async fn health_check() -> &'static str {
    "OK"
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
    let admin_routes = Router::new()
        .route("/", get(list_users_handler))
        .route("/{id}", get(get_user_handler))
        .route("/{id}", put(update_user_handler))
        .route("/{id}", delete(delete_user_handler))
        .route_layer(middleware::from_fn(admin_only));

    let password_routes = Router::new()
        .route("/{id}/password", put(change_password_handler));

    admin_routes.merge(password_routes)
}

fn connection_routes() -> Router<Arc<AppState>> {
    let read_routes = Router::new()
        .route("/", get(list_connections_handler))
        .route("/{id}", get(get_connection_handler))
        .route("/{id}/test", post(test_connection_handler))
        .route("/{id}/schema", get(get_schema_handler))
        .route_layer(middleware::from_fn(require_sql_mode));

    let write_routes = Router::new()
        .route("/", post(create_connection_handler))
        .route("/{id}", put(update_connection_handler))
        .route("/{id}", delete(delete_connection_handler))
        .route("/{id}/default", put(set_default_connection_handler))
        .route_layer(middleware::from_fn(require_connection_management));

    read_routes.merge(write_routes)
}

fn nl_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/convert", post(nl_convert_handler))
        .route("/execute", post(nl_execute_handler))
        .route_layer(middleware::from_fn(require_chat_mode))
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
        .route("/recommend", post(recommend_chart_handler))
        .route("/generate", post(generate_chart_handler))
        .route("/export", post(export_chart_handler))
        .route_layer(middleware::from_fn(require_chart_generation))
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
        .route_layer(middleware::from_fn(require_sql_mode))
}

fn sql_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/execute", post(execute_sql_handler))
        .route("/format", post(format_sql_handler))
        .route("/history", get(get_query_history_handler))
        .route("/explain", post(explain_sql_handler))
        .route("/preview", post(preview_data_handler))
        .route_layer(middleware::from_fn(require_sql_mode))
}

fn audit_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_audit_logs_handler))
        .route_layer(middleware::from_fn(admin_only))
}

fn semantic_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_semantics_handler))
        .route("/", post(create_semantic_handler))
        .route("/batch", post(batch_semantics_handler))
        .route("/stats", get(get_semantic_stats_handler))
        .route("/{id}", put(update_semantic_handler))
        .route("/{id}", delete(delete_semantic_handler))
        .route_layer(middleware::from_fn(require_sql_mode))
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

/// 创建指标仓储
fn metric_repo(state: &AppState) -> MetricRepo {
    MetricRepo::new(state.db.clone())
}

fn semantic_repo(state: &AppState) -> SemanticRepo {
    SemanticRepo::new(state.db.clone())
}

fn audit_repo(state: &AppState) -> AuditRepo {
    AuditRepo::new(state.db.clone())
}

async fn write_audit(
    state: &AppState,
    user_id: Option<Uuid>,
    action: &str,
    resource_type: Option<&str>,
    resource_id: Option<String>,
    details: serde_json::Value,
) {
    let _ = audit_repo(state)
        .create(user_id, action, resource_type, resource_id.as_deref(), details)
        .await;
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
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("注册参数校验失败: {}", e)))?;

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
    let user = User::new(payload.username, payload.email, password_hash, UserRole::Business);
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
    let jwt_utils = crate::utils::jwt::JwtUtils::new(&state.config.jwt);

    if let Some(access_token) = payload.get("access_token").and_then(|v| v.as_str()) {
        let ttl = jwt_utils.get_token_ttl(access_token)?;
        state.token_blacklist.add(access_token, ttl).await;
    }

    if let Some(refresh_token) = payload.get("refresh_token").and_then(|v| v.as_str()) {
        let ttl = jwt_utils.get_token_ttl(refresh_token)?;
        state.token_blacklist.add(refresh_token, ttl).await;
    }

    state.token_blacklist.cleanup().await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "登出成功"
    })))
}

// ==================== 用户处理器 ====================

async fn list_users_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);
    let page = params.page.max(1);
    let page_size = params.page_size.max(1).min(100);
    let (users, total) = repo.list(page, page_size).await?;

    let user_list: Vec<UserPublic> = users.iter().map(UserPublic::from).collect();

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": user_list,
            "total": total,
            "page": page,
            "page_size": page_size
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
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("用户参数校验失败: {}", e)))?;

    let repo = user_repo(&state);
    let mut user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    if let Some(username) = payload.username {
        if let Some(existing) = repo.find_by_username(&username).await? {
            if existing.id != id {
                return Err(AppError::AlreadyExists("用户名已存在".to_string()));
            }
        }
        user.username = username;
    }
    if let Some(email) = payload.email {
        if let Some(existing) = repo.find_by_email(&email).await? {
            if existing.id != id {
                return Err(AppError::AlreadyExists("邮箱已被使用".to_string()));
            }
        }
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
    write_audit(
        &state,
        Some(session.user_id),
        "user.update",
        Some("user"),
        Some(user.id.to_string()),
        serde_json::json!({"username": user.username, "email": user.email, "role": user.role.as_str()}),
    )
    .await;

    let response = serde_json::json!({
        "code": 0,
        "message": "更新成功",
        "data": UserPublic::from(&user)
    });

    Ok(Json(response))
}

async fn delete_user_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = user_repo(&state);

    if session.user_id == id {
        return Err(AppError::Forbidden("不能禁用当前登录用户".to_string()));
    }

    let mut user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    if user.role.is_admin() && user.is_active && repo.active_admin_count().await? <= 1 {
        return Err(AppError::Forbidden("不能禁用最后一个管理员".to_string()));
    }

    user.is_active = false;
    user.updated_at = Utc::now();
    let user = repo.update(&user).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "user.disable",
        Some("user"),
        Some(user.id.to_string()),
        serde_json::json!({"username": user.username}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "用户已禁用"
    })))
}

async fn change_password_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("密码参数校验失败: {}", e)))?;

    if session.user_id != id && !session.role.is_admin() {
        return Err(AppError::Forbidden("无权修改该用户密码".to_string()));
    }

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
    write_audit(
        &state,
        Some(session.user_id),
        "user.change_password",
        Some("user"),
        Some(id.to_string()),
        serde_json::json!({"self_service": session.user_id == id}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "密码修改成功"
    })))
}

// ==================== 审计日志处理器 ====================

async fn list_audit_logs_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuditLogListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page = params.page.max(1);
    let page_size = params.page_size.max(1).min(100);
    let (items, total) = audit_repo(&state)
        .list(page, page_size, params.query.as_deref())
        .await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": items,
            "total": total,
            "page": page,
            "page_size": page_size
        }
    })))
}

// ==================== 语义层处理器 ====================

async fn list_semantics_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Query(params): Query<SemanticListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(connection_id) = params.connection_id {
        let _ = get_connection_config_for_session(&state, connection_id, &session).await?;
    }

    let page = params.page.max(1);
    let page_size = params.page_size.max(1).min(100);
    let (items, total) = semantic_repo(&state)
        .list(params.connection_id, page, page_size, params.query.as_deref())
        .await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "items": items,
            "total": total,
            "page": page,
            "page_size": page_size
        }
    })))
}

async fn create_semantic_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateSemanticRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("语义定义参数校验失败: {}", e)))?;
    let _ = get_connection_config_for_session(&state, payload.connection_id, &session).await?;

    let mut semantic = SemanticDefinition::new(
        payload.connection_id,
        payload.table_name,
        payload.column_name,
        payload.business_name,
        payload.business_description,
    );
    semantic.synonyms = payload.synonyms.map(|synonyms| serde_json::json!(synonyms));

    let semantic = semantic_repo(&state).create(&semantic).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "semantic.create",
        Some("semantic"),
        Some(semantic.id.to_string()),
        serde_json::json!({"connection_id": semantic.connection_id, "name": semantic.full_name()}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "创建成功",
        "data": semantic
    })))
}

async fn update_semantic_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSemanticRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("语义定义参数校验失败: {}", e)))?;

    let repo = semantic_repo(&state);
    let mut semantic = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("语义定义不存在".to_string()))?;
    let _ = get_connection_config_for_session(&state, semantic.connection_id, &session).await?;

    if let Some(business_name) = payload.business_name {
        semantic.business_name = business_name;
    }
    if payload.business_description.is_some() {
        semantic.business_description = payload.business_description;
    }
    if let Some(synonyms) = payload.synonyms {
        semantic.synonyms = Some(serde_json::json!(synonyms));
    }
    if let Some(is_active) = payload.is_active {
        semantic.is_active = is_active;
    }

    let semantic = repo.update(&semantic).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "semantic.update",
        Some("semantic"),
        Some(semantic.id.to_string()),
        serde_json::json!({"connection_id": semantic.connection_id, "name": semantic.full_name()}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "更新成功",
        "data": semantic
    })))
}

async fn delete_semantic_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = semantic_repo(&state);
    let semantic = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("语义定义不存在".to_string()))?;
    let _ = get_connection_config_for_session(&state, semantic.connection_id, &session).await?;

    repo.delete(id).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "semantic.delete",
        Some("semantic"),
        Some(id.to_string()),
        serde_json::json!({"connection_id": semantic.connection_id, "name": semantic.full_name()}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn batch_semantics_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<BatchSemanticRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("批量语义参数校验失败: {}", e)))?;
    let _ = get_connection_config_for_session(&state, payload.connection_id, &session).await?;

    let repo = semantic_repo(&state);
    let mut items = Vec::with_capacity(payload.definitions.len());
    for definition in payload.definitions {
        let mut semantic = SemanticDefinition::new(
            payload.connection_id,
            definition.table_name,
            definition.column_name,
            definition.business_name,
            definition.business_description,
        );
        semantic.synonyms = definition.synonyms.map(|synonyms| serde_json::json!(synonyms));
        items.push(repo.create(&semantic).await?);
    }

    write_audit(
        &state,
        Some(session.user_id),
        "semantic.batch_create",
        Some("semantic"),
        Some(payload.connection_id.to_string()),
        serde_json::json!({"count": items.len()}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "批量创建成功",
        "data": {
            "items": items,
            "total": items.len()
        }
    })))
}

async fn get_semantic_stats_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = semantic_repo(&state).stats().await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": stats
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
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("连接参数校验失败: {}", e)))?;
    ensure_supported_connection_type(payload.db_type)?;

    let repo = connection_repo(&state);
    let user_id = session.user_id;

    let encrypted_password = encode_connection_password(&payload.password, &state);

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
        repo.clear_default_for_user(Some(user_id)).await?;
    }

    let conn = repo.create(&conn).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "connection.create",
        Some("connection"),
        Some(conn.id.to_string()),
        serde_json::json!({"name": conn.name, "db_type": conn.db_type.as_str()}),
    )
    .await;

    let response = serde_json::json!({
        "code": 0,
        "message": "创建成功",
        "data": ConnectionPublic::from(&conn)
    });

    Ok(Json(response))
}

async fn get_connection_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    let conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;
    ensure_connection_access(&conn, &session)?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": ConnectionPublic::from(&conn)
    });

    Ok(Json(response))
}

async fn update_connection_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConnectionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("连接参数校验失败: {}", e)))?;

    let repo = connection_repo(&state);
    let mut conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;
    ensure_connection_access(&conn, &session)?;

    if let Some(name) = payload.name {
        conn.name = name;
    }
    if let Some(db_type) = payload.db_type {
        ensure_supported_connection_type(db_type)?;
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
        conn.encrypted_password = encode_connection_password(&password, &state);
    }
    if let Some(is_default) = payload.is_default {
        if is_default {
            repo.clear_default_for_user(conn.created_by).await?;
        }
        conn.is_default = is_default;
    }
    conn.updated_at = Utc::now();

    let conn = repo.update(&conn).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "connection.update",
        Some("connection"),
        Some(conn.id.to_string()),
        serde_json::json!({"name": conn.name}),
    )
    .await;

    let response = serde_json::json!({
        "code": 0,
        "message": "更新成功",
        "data": ConnectionPublic::from(&conn)
    });

    Ok(Json(response))
}

async fn delete_connection_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    let conn = repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;
    ensure_connection_access(&conn, &session)?;

    repo.delete(id).await?;
    state.connection_manager.remove_pool(id).await;
    write_audit(
        &state,
        Some(session.user_id),
        "connection.delete",
        Some("connection"),
        Some(id.to_string()),
        serde_json::json!({"name": conn.name}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn test_connection_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = get_connection_config_for_session(&state, id, &session).await?;
    let start = std::time::Instant::now();

    let test_result = match config.db_type {
        DatabaseType::Postgresql => {
            state.connection_manager.test_connection_pg(&config).await
        }
        DatabaseType::Mysql => {
            state.connection_manager.test_connection_mysql(&config).await
        }
        DatabaseType::Sqlite => {
            state.connection_manager.test_connection_sqlite(&config).await
        }
        DatabaseType::Clickhouse => {
            state.connection_manager.test_connection_clickhouse(&config).await
        }
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
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = connection_repo(&state);
    let conn = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;
    ensure_connection_access(&conn, &session)?;

    let user_id = conn.created_by.unwrap_or(Uuid::nil());
    repo.set_default(id, user_id).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "connection.set_default",
        Some("connection"),
        Some(id.to_string()),
        serde_json::json!({}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "已设为默认连接"
    })))
}

async fn get_schema_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = get_connection_config_for_session(&state, id, &session).await?;
    let mut tables = state.connection_manager.get_schema(&config).await?;
    for table in &mut tables {
        table.columns = state
            .connection_manager
            .get_table_columns(&config, &table.table_schema, &table.table_name)
            .await
            .unwrap_or_default();
    }

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
    let config = get_connection_config_for_session(&state, payload.connection_id, &session).await?;
    let pool = state.connection_manager.get_pool(&config).await?;
    let start = std::time::Instant::now();

    let mut history = QueryHistory::new(
        Some(payload.connection_id),
        session.user_id,
        payload.sql.clone(),
    );

    let (columns, rows) = match sql_execute(&state, &pool, &payload.sql, &config.db_type).await {
        Ok(result) => result,
        Err(e) => {
            history.mark_failed(e.to_string());
            let _ = query_repo(&state).create(&history).await;
            return Err(e);
        }
    };
    let rows = mask_result_rows(&columns, rows);
    let execution_plan = if payload.explain {
        Some(build_execution_plan(&pool, &payload.sql).await?)
    } else {
        None
    };

    let duration_ms = start.elapsed().as_millis() as i64;
    let row_count = rows.len() as i64;
    history.mark_success(duration_ms, row_count);
    let history = query_repo(&state).create(&history).await.unwrap_or(history);
    write_audit(
        &state,
        Some(session.user_id),
        "sql.execute",
        Some("query_history"),
        Some(history.id.to_string()),
        serde_json::json!({"connection_id": payload.connection_id, "row_count": row_count, "duration_ms": duration_ms}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "query_id": history.id,
            "columns": columns,
            "rows": rows,
            "row_count": row_count,
            "duration_ms": duration_ms,
            "execution_plan": execution_plan
        }
    })))
}

/// 执行 SQL 查询并返回结果 (暂未使用，作为备用)
#[allow(dead_code)]
async fn sql_execute(
    state: &AppState,
    pool: &ConnectionPool,
    sql: &str,
    db_type: &DatabaseType,
) -> AppResult<(Vec<ColumnMetadata>, Vec<Vec<serde_json::Value>>)> {
    use sqlparser::dialect::{ClickHouseDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};
    use sqlparser::parser::Parser;

    validate_select_sql(state, sql)?;

    // 使用目标数据库方言再次解析 SQL
    {
        let dialect: Box<dyn sqlparser::dialect::Dialect> = match db_type {
            DatabaseType::Postgresql => Box::new(PostgreSqlDialect {}),
            DatabaseType::Mysql => Box::new(MySqlDialect {}),
            DatabaseType::Sqlite => Box::new(SQLiteDialect {}),
            DatabaseType::Clickhouse => Box::new(ClickHouseDialect {}),
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
        ConnectionPool::Sqlite(sqlite_pool) => {
            let rows = sqlx::query(sql)
                .fetch_all(sqlite_pool)
                .await
                .map_err(|e| AppError::database(format!("查询执行失败: {}", e)))?;

            Ok(extract_sqlite_rows_data(rows))
        }
        ConnectionPool::Clickhouse(config) => clickhouse_select(config, sql).await,
    }
}

fn validate_select_sql(state: &AppState, sql: &str) -> AppResult<()> {
    let analyzer = SqlAnalyzer::new(Arc::new(state.config.security.clone()));
    if analyzer.check_injection(sql) {
        return Err(AppError::DmlForbidden("检测到 SQL 注入风险".to_string()));
    }

    let analysis = analyzer.analyze(sql)?;
    if !analysis.is_safe {
        return Err(AppError::DmlForbidden(
            analysis
                .blocked_reason
                .unwrap_or_else(|| "SQL 不符合安全策略".to_string()),
        ));
    }
    if !analysis.sql_type.is_readonly() {
        return Err(AppError::DmlForbidden("只允许执行 SELECT 查询".to_string()));
    }

    Ok(())
}

fn mask_result_rows(
    columns: &[ColumnMetadata],
    rows: Vec<Vec<serde_json::Value>>,
) -> Vec<Vec<serde_json::Value>> {
    let masker = DataMasker::new();
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .enumerate()
                .map(|(index, value)| {
                    columns
                        .get(index)
                        .and_then(|column| masker.detect_field_type(&column.name))
                        .and_then(|field_type| {
                            value
                                .as_str()
                                .map(|text| serde_json::Value::String(masker.mask_value(text, &field_type)))
                        })
                        .unwrap_or(value)
                })
                .collect()
        })
        .collect()
}

fn encode_connection_password(input: &str, state: &AppState) -> String {
    let key = state.config.jwt.secret.as_bytes();
    if key.is_empty() {
        return base64_encode(input);
    }

    let nonce = Uuid::new_v4().as_bytes().to_vec();
    let encrypted = xor_with_hmac_keystream(input.as_bytes(), key, &nonce);
    let mac = connection_password_mac(key, &nonce, &encrypted);

    format!(
        "v2:{}:{}:{}",
        base64_encode_bytes(&nonce),
        base64_encode_bytes(&encrypted),
        base64_encode_bytes(&mac)
    )
}

fn decode_connection_password(encoded: &str, state: &AppState) -> String {
    if let Some(payload) = encoded.strip_prefix("v2:") {
        let key = state.config.jwt.secret.as_bytes();
        let parts: Vec<&str> = payload.split(':').collect();
        if key.is_empty() || parts.len() != 3 {
            return String::new();
        }

        let Some(nonce) = base64_decode(parts[0]) else {
            return String::new();
        };
        let Some(encrypted) = base64_decode(parts[1]) else {
            return String::new();
        };
        let Some(mac) = base64_decode(parts[2]) else {
            return String::new();
        };

        let expected_mac = connection_password_mac(key, &nonce, &encrypted);
        if expected_mac != mac {
            return String::new();
        }

        let decrypted = xor_with_hmac_keystream(&encrypted, key, &nonce);
        return String::from_utf8_lossy(&decrypted).to_string();
    }

    if let Some(ciphertext) = encoded.strip_prefix("v1:") {
        let key = state.config.jwt.secret.as_bytes();
        if !key.is_empty() {
            if let Some(bytes) = base64_decode(ciphertext) {
                let decrypted: Vec<u8> = bytes
                    .iter()
                    .enumerate()
                    .map(|(i, b)| b ^ key[i % key.len()])
                    .collect();
                return String::from_utf8_lossy(&decrypted).to_string();
            }
        }
    }

    decode_legacy_base64_password(encoded)
}

fn xor_with_hmac_keystream(input: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut output = Vec::with_capacity(input.len());
    let mut counter: u64 = 0;

    while output.len() < input.len() {
        let mut mac = Hmac::<Sha256>::new_from_slice(key)
            .expect("HMAC accepts keys of any length");
        mac.update(nonce);
        mac.update(&counter.to_be_bytes());
        let block = mac.finalize().into_bytes();

        for byte in block {
            if output.len() == input.len() {
                break;
            }
            output.push(input[output.len()] ^ byte);
        }

        counter += 1;
    }

    output
}

fn connection_password_mac(key: &[u8], nonce: &[u8], encrypted: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .expect("HMAC accepts keys of any length");
    mac.update(nonce);
    mac.update(encrypted);
    mac.finalize().into_bytes().to_vec()
}

fn decode_legacy_base64_password(encoded: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    match STANDARD.decode(encoded) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => encoded.to_string(),
    }
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(input).ok()
}

async fn format_sql_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SqlFormatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    use sqlparser::dialect::{ClickHouseDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};
    use sqlparser::parser::Parser;

    let dialect = match payload.dialect.as_str() {
        "mysql" => Box::new(MySqlDialect {}) as Box<dyn sqlparser::dialect::Dialect>,
        "postgresql" | "postgres" => Box::new(PostgreSqlDialect {}) as Box<dyn sqlparser::dialect::Dialect>,
        "sqlite" => Box::new(SQLiteDialect {}) as Box<dyn sqlparser::dialect::Dialect>,
        "clickhouse" => Box::new(ClickHouseDialect {}) as Box<dyn sqlparser::dialect::Dialect>,
        _ => {
            return Err(AppError::ValidationError(
                "Unsupported SQL dialect".to_string(),
            ));
        }
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
    Query(params): Query<QueryHistoryListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = query_repo(&state);
    let user_id = session.user_id;
    let page = params.page.max(1);
    let page_size = params.page_size.max(1).min(100);
    let (histories, total) = repo.list_by_user(user_id, page, page_size).await?;

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
            "page": page,
            "page_size": page_size
        }
    });

    Ok(Json(response))
}

async fn explain_sql_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<SqlExecuteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = get_connection_config_for_session(&state, payload.connection_id, &session).await?;
    validate_select_sql(&state, &payload.sql)?;
    let pool_type = state.connection_manager.get_pool(&config).await?;
    let result = build_execution_plan(&pool_type, &payload.sql).await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": result
    })))
}

async fn build_execution_plan(
    pool_type: &ConnectionPool,
    sql: &str,
) -> AppResult<serde_json::Value> {
    let explain_sql = match pool_type {
        ConnectionPool::Sqlite(_) => format!("EXPLAIN QUERY PLAN {}", sql),
        ConnectionPool::Clickhouse(_) => format!("EXPLAIN {}", sql),
        _ => format!("EXPLAIN {}", sql),
    };

    let start = std::time::Instant::now();

    let result = match pool_type {
        ConnectionPool::Postgres(pg_pool) => {
            let rows: Vec<(String,)> = sqlx::query_as(&explain_sql)
                .fetch_all(pg_pool)
                .await
                .map_err(|e| AppError::database(format!("EXPLAIN 失败: {}", e)))?;
            rows.into_iter().map(|row| row.0).collect::<Vec<_>>().join("\n")
        }
        ConnectionPool::Mysql(mysql_pool) => {
            let rows = sqlx::query(&explain_sql)
                .fetch_all(mysql_pool)
                .await
                .map_err(|e| AppError::database(format!("EXPLAIN 失败: {}", e)))?;
            rows.into_iter()
                .map(|row| {
                    (0..row.columns().len())
                        .map(|i| mysql_value_to_json(&row, i).to_string())
                        .collect::<Vec<_>>()
                        .join("\t")
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        ConnectionPool::Sqlite(sqlite_pool) => {
            let rows = sqlx::query(&explain_sql)
                .fetch_all(sqlite_pool)
                .await
                .map_err(|e| AppError::database(format!("EXPLAIN 失败: {}", e)))?;
            rows.into_iter()
                .map(|row| {
                    (0..row.columns().len())
                        .map(|i| sqlite_value_to_json(&row, i).to_string())
                        .collect::<Vec<_>>()
                        .join("\t")
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        ConnectionPool::Clickhouse(config) => {
            let (_columns, rows) = clickhouse_select(config, &explain_sql).await?;
            rows.into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>()
                        .join("\t")
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    };

    let duration_ms = start.elapsed().as_millis() as i64;
    let (warnings, suggestions) = analyze_execution_plan_text(&result);

    Ok(serde_json::json!({
        "plan_type": "SELECT",
        "estimated_cost": null,
        "estimated_rows": null,
        "actual_rows": null,
        "duration_ms": duration_ms,
        "warnings": warnings,
        "suggestions": suggestions,
        "details": {
            "raw": result
        }
    }))
}

fn analyze_execution_plan_text(plan: &str) -> (Vec<String>, Vec<String>) {
    let lower = plan.to_ascii_lowercase();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    if lower.contains("seq scan")
        || lower.contains("table scan")
        || lower.contains("full scan")
        || lower.contains("all\t")
        || lower.contains("\"all\"")
    {
        warnings.push("执行计划可能包含全表扫描".to_string());
        suggestions.push("检查 WHERE/JOIN 字段是否有合适索引，必要时缩小查询范围".to_string());
    }

    if lower.contains("filesort") || lower.contains("using temporary") || lower.contains("temporary") {
        warnings.push("执行计划可能使用临时表或额外排序".to_string());
        suggestions.push("检查 ORDER BY/GROUP BY 字段顺序，考虑建立联合索引".to_string());
    }

    if lower.contains("nested loop") {
        warnings.push("执行计划包含嵌套循环连接".to_string());
        suggestions.push("确认 JOIN 条件字段有索引，并关注大表连接顺序".to_string());
    }

    if lower.contains("cross join") {
        warnings.push("执行计划包含笛卡尔连接风险".to_string());
        suggestions.push("补充明确 JOIN 条件，避免结果集爆炸".to_string());
    }

    if warnings.is_empty() {
        suggestions.push("未发现明显高风险执行计划特征".to_string());
    }

    (warnings, suggestions)
}

async fn preview_data_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<PreviewDataRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let table_name = validate_table_identifier(&payload.table_name)?;
    let limit = payload.limit.clamp(1, 1000);
    let config = get_connection_config_for_session(&state, payload.connection_id, &session).await?;
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
        ConnectionPool::Sqlite(sqlite_pool) => {
            let rows = sqlx::query(&sql)
                .fetch_all(&sqlite_pool)
                .await
                .map_err(|e| AppError::database(format!("预览表数据失败: {}", e)))?;
            extract_sqlite_rows_data(rows)
        }
        ConnectionPool::Clickhouse(config) => clickhouse_select(&config, &sql).await?,
    };
    let result_rows = mask_result_rows(&columns, result_rows);

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

fn ensure_connection_access(conn: &DatabaseConnection, session: &UserSession) -> AppResult<()> {
    if session.role.is_admin() || conn.created_by.is_none() || conn.created_by == Some(session.user_id) {
        Ok(())
    } else {
        Err(AppError::Forbidden("无权访问该连接".to_string()))
    }
}

async fn get_connection_config_for_session(
    state: &AppState,
    connection_id: Uuid,
    session: &UserSession,
) -> AppResult<ConnectionConfig> {
    let repo = connection_repo(state);
    let conn = repo
        .find_by_id(connection_id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;
    ensure_connection_access(&conn, session)?;

    Ok(ConnectionConfig {
        id: conn.id,
        name: conn.name,
        db_type: conn.db_type,
        host: conn.host.clone(),
        port: conn.port,
        database_name: conn.database_name.clone(),
        username: conn.username.clone(),
        password: decode_connection_password(&conn.encrypted_password, state),
    })
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

fn ensure_supported_connection_type(db_type: DatabaseType) -> AppResult<()> {
    match db_type {
        DatabaseType::Mysql | DatabaseType::Postgresql | DatabaseType::Sqlite | DatabaseType::Clickhouse => Ok(()),
    }
}

fn ensure_dialect_matches_connection(dialect: &str, db_type: DatabaseType) -> AppResult<()> {
    let normalized = dialect.to_ascii_lowercase();
    let matches = match db_type {
        DatabaseType::Mysql => normalized == "mysql",
        DatabaseType::Postgresql => normalized == "postgresql" || normalized == "postgres",
        DatabaseType::Sqlite => normalized == "sqlite",
        DatabaseType::Clickhouse => normalized == "clickhouse",
    };

    if matches {
        Ok(())
    } else {
        Err(AppError::ValidationError(format!(
            "Dialect '{}' does not match connection type '{}'",
            dialect,
            db_type.as_str()
        )))
    }
}

// ==================== NL 处理器 ====================

async fn nl_convert_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<NlToSqlRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("自然语言查询参数校验失败: {}", e)))?;

    let llm_client = &state.llm_client;
    let config = get_connection_config_for_session(&state, payload.connection_id, &session).await?;
    ensure_dialect_matches_connection(&payload.dialect, config.db_type)?;

    // 获取 schema 上下文
    let schema_context = match state.connection_manager.get_schema(&config).await {
        Ok(tables) => {
            let mut table_info = Vec::new();
            for table in tables {
                let columns = state
                    .connection_manager
                    .get_table_columns(&config, &table.table_schema, &table.table_name)
                    .await
                    .unwrap_or_default();
                let column_info = columns
                    .iter()
                    .map(|c| {
                        let comment = c.comment.as_deref().unwrap_or("");
                        if comment.is_empty() {
                            format!("{} {}", c.name, c.data_type)
                        } else {
                            format!("{} {} ({})", c.name, c.data_type, comment)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                table_info.push(format!(
                    "{}.{} ({}) columns: [{}]",
                    table.table_schema, table.table_name, table.table_type, column_info
                ));
            }
            Some(table_info.join(", "))
        }
        Err(_) => None,
    };

    let conversation_context = if let Some(conversation_id) = payload.conversation_id {
        let repo = conversation_repo(&state);
        if let Some(conversation) = repo.get_conversation(conversation_id).await? {
            ensure_conversation_access(&conversation, &session)?;
            let messages = repo.list_messages(conversation_id).await?;
            let recent = messages
                .iter()
                .rev()
                .take(6)
                .map(|m| format!("{}: {}", m.role.as_str(), m.content))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            if recent.is_empty() {
                None
            } else {
                Some(format!("历史对话:\n{}", recent))
            }
        } else {
            return Err(AppError::NotFound("对话不存在".to_string()));
        }
    } else {
        None
    };

    let question = match conversation_context {
        Some(context) => format!(
            "目标数据库方言: {}\n{}\n当前问题: {}",
            payload.dialect, context, payload.question
        ),
        None => format!("目标数据库方言: {}\n当前问题: {}", payload.dialect, payload.question),
    };

    // 调用 LLM 进行转换
    let result = llm_client
        .convert_nl_to_sql(&question, schema_context.as_deref())
        .await?;
    validate_select_sql(&state, &result.sql)?;

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
    let config = get_connection_config_for_session(&state, payload.connection_id, &session).await?;
    let pool = state.connection_manager.get_pool(&config).await?;
    let start = std::time::Instant::now();

    let mut history = QueryHistory::new(Some(payload.connection_id), user_id, payload.sql.clone());
    let (columns, rows) = match sql_execute(&state, &pool, &payload.sql, &config.db_type).await {
        Ok(result) => result,
        Err(e) => {
            history.mark_failed(e.to_string());
            let _ = query_repo(&state).create(&history).await;
            return Err(e);
        }
    };
    let rows = mask_result_rows(&columns, rows);

    let duration_ms = start.elapsed().as_millis() as i64;
    let row_count = rows.len() as i64;
    history.mark_success(duration_ms, row_count);
    let history = query_repo(&state).create(&history).await.unwrap_or(history);
    write_audit(
        &state,
        Some(session.user_id),
        "nl.execute",
        Some("query_history"),
        Some(history.id.to_string()),
        serde_json::json!({"connection_id": payload.connection_id, "row_count": row_count, "duration_ms": duration_ms}),
    )
    .await;

    let generator = ChartGenerator::new();
    let chart_config = match payload.chart_type.as_deref() {
        Some(chart_type) => generator
            .switch_chart_type(&columns, &rows, chart_type)
            .ok()
            .map(|config| config.echarts_config),
        None => generator
            .recommend(&columns, &rows)
            .ok()
            .map(|recommendation| recommendation.chart_config.echarts_config),
    };

    let data_insight = build_data_insight(&columns, row_count, duration_ms);

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": {
            "columns": columns,
            "rows": rows,
            "row_count": row_count,
            "duration_ms": duration_ms,
            "chart_config": chart_config,
            "data_insight": data_insight
        }
    });

    Ok(Json(response))
}

fn ensure_conversation_access(conv: &Conversation, session: &UserSession) -> AppResult<()> {
    if session.role.is_admin() || conv.user_id == session.user_id {
        Ok(())
    } else {
        Err(AppError::Forbidden("无权访问该对话".to_string()))
    }
}

fn build_data_insight(
    columns: &[ColumnMetadata],
    row_count: i64,
    duration_ms: i64,
) -> Option<String> {
    if columns.is_empty() {
        return None;
    }

    let column_names = columns
        .iter()
        .take(5)
        .map(|column| column.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!(
        "本次查询返回 {} 行，耗时 {} ms，主要字段包括 {}。",
        row_count, duration_ms, column_names
    ))
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
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("对话参数校验失败: {}", e)))?;

    let repo = conversation_repo(&state);
    let user_id = session.user_id;

    let conv = Conversation::new(user_id, payload.title);
    let conv = repo.create_conversation(&conv).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "conversation.create",
        Some("conversation"),
        Some(conv.id.to_string()),
        serde_json::json!({"title": conv.title}),
    )
    .await;

    let response = serde_json::json!({
        "code": 0,
        "message": "创建成功",
        "data": conv
    });

    Ok(Json(response))
}

async fn get_conversation_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);
    let conv = repo
        .get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;
    ensure_conversation_access(&conv, &session)?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": conv
    });

    Ok(Json(response))
}

async fn delete_conversation_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);
    let conv = repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;
    ensure_conversation_access(&conv, &session)?;

    repo.delete_conversation(id).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "conversation.delete",
        Some("conversation"),
        Some(id.to_string()),
        serde_json::json!({"title": conv.title}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn list_messages_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = conversation_repo(&state);
    let conv = repo
        .get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;
    ensure_conversation_access(&conv, &session)?;

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
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("消息参数校验失败: {}", e)))?;

    let repo = conversation_repo(&state);

    // 检查对话是否存在
    let conv = repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;
    ensure_conversation_access(&conv, &session)?;

    // 创建用户消息
    let user_msg = Message::user_message(id, payload.content.clone());
    let user_msg = repo.create_message(&user_msg).await?;

    let schema_context = if let Some(connection_id) = payload.connection_id {
        let config = get_connection_config_for_session(&state, connection_id, &session).await?;
        let mut table_info = Vec::new();
        if let Ok(tables) = state.connection_manager.get_schema(&config).await {
            for table in tables {
                let columns = state
                    .connection_manager
                    .get_table_columns(&config, &table.table_schema, &table.table_name)
                    .await
                    .unwrap_or_default();
                let column_info = columns
                    .iter()
                    .map(|column| format!("{} {}", column.name, column.data_type))
                    .collect::<Vec<_>>()
                    .join(", ");
                table_info.push(format!(
                    "{}.{} columns: [{}]",
                    table.table_schema, table.table_name, column_info
                ));
            }
        }
        if table_info.is_empty() {
            None
        } else {
            Some(table_info.join("\n"))
        }
    } else {
        None
    };

    // 获取对话历史用于上下文
    let messages = repo.list_messages(id).await?;
    let history_context: String = messages
        .iter()
        .take(10)
        .map(|m| format!("{}: {}", if m.role == MessageRole::User { "用户" } else { "助手" }, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let llm_client = &state.llm_client;
    let question = format!(
        "目标数据库方言: {}\n历史对话:\n{}\n当前问题: {}",
        payload.dialect.as_deref().unwrap_or("mysql"),
        history_context,
        payload.content
    );
    let result = llm_client
        .convert_nl_to_sql(&question, schema_context.as_deref())
        .await?;
    validate_select_sql(&state, &result.sql)?;

    let assistant_msg = Message::assistant_message(
        id,
        result.explanation.clone(),
        Some(result.sql),
        Some(format!("置信度: {:.0}%", result.confidence * 100.0)),
    );
    let assistant_msg = repo.create_message(&assistant_msg).await?;

    // 更新对话时间
    let _ = repo.update_title(id, &payload.content.chars().take(40).collect::<String>()).await;

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

fn extract_sqlite_rows_data(rows: Vec<sqlx::sqlite::SqliteRow>) -> (Vec<ColumnMetadata>, Vec<Vec<serde_json::Value>>) {
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
                .map(|i| sqlite_value_to_json(row, i))
                .collect()
        })
        .collect();

    (columns, result_rows)
}

async fn clickhouse_select(
    config: &ConnectionConfig,
    sql: &str,
) -> AppResult<(Vec<ColumnMetadata>, Vec<Vec<serde_json::Value>>)> {
    let query = ensure_clickhouse_json_format(sql);
    let response = reqwest::Client::new()
        .post(config.clickhouse_http_url())
        .basic_auth(&config.username, Some(&config.password))
        .query(&[("database", config.database_name.as_str())])
        .body(query)
        .send()
        .await
        .map_err(|e| AppError::database(format!("ClickHouse 查询失败: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| AppError::database(format!("读取 ClickHouse 响应失败: {}", e)))?;

    if !status.is_success() {
        return Err(AppError::database(format!(
            "ClickHouse 返回错误 {}: {}",
            status, body
        )));
    }

    let value: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| AppError::database(format!("解析 ClickHouse JSON 失败: {}", e)))?;

    let meta = value
        .get("meta")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::database("ClickHouse JSON 缺少 meta".to_string()))?;
    let data = value
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::database("ClickHouse JSON 缺少 data".to_string()))?;

    let columns = meta
        .iter()
        .enumerate()
        .map(|(index, column)| ColumnMetadata {
            name: column
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            data_type: column
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            ordinal: index as i32,
        })
        .collect::<Vec<_>>();

    let rows = data
        .iter()
        .map(|row| {
            columns
                .iter()
                .map(|column| row.get(&column.name).cloned().unwrap_or(serde_json::Value::Null))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Ok((columns, rows))
}

fn ensure_clickhouse_json_format(sql: &str) -> String {
    let trimmed = sql.trim().trim_end_matches(';').trim();
    if regex::Regex::new(r"(?i)\bFORMAT\s+JSON\b")
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false)
    {
        trimmed.to_string()
    } else {
        format!("{} FORMAT JSON", trimmed)
    }
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

fn sqlite_value_to_json(row: &sqlx::sqlite::SqliteRow, index: usize) -> serde_json::Value {
    if let Ok(v) = row.try_get::<String, _>(index) {
        return serde_json::Value::String(v);
    }
    if let Ok(v) = row.try_get::<i64, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<f64, _>(index) {
        return serde_json::json!(v);
    }
    if let Ok(v) = row.try_get::<bool, _>(index) {
        return serde_json::json!(v);
    }
    serde_json::Value::Null
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
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
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
        "png" => {
            let png_data = payload
                .config
                .get("png")
                .or_else(|| payload.config.get("data_url"))
                .or_else(|| payload.config.get("image_data"))
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    AppError::ValidationError(
                        "PNG export requires config.png, config.data_url, or config.image_data".to_string(),
                    )
                })?;
            let png_bytes = decode_png_payload(png_data)?;
            std::fs::write(&file_path, png_bytes)
                .map_err(|e| AppError::internal(format!("写入 PNG 失败: {}", e)))?;
        }
        _ => {
            return Err(AppError::ValidationError(
                "Unsupported chart export format".to_string(),
            ));
        }
    }

    let response = serde_json::json!({
        "code": 0,
        "message": "导出成功",
        "data": {
            "format": format,
            "filename": filename,
            "url": format!("/api/v1/exports/{}", filename),
            "path": file_path.to_string_lossy()
        }
    });
    write_audit(
        &state,
        Some(session.user_id),
        "chart.export",
        Some("chart"),
        Some(filename.clone()),
        serde_json::json!({"format": format}),
    )
    .await;

    Ok(Json(response))
}

fn decode_png_payload(input: &str) -> AppResult<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let base64_part = input
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(input)
        .trim();

    let bytes = STANDARD
        .decode(base64_part)
        .map_err(|e| AppError::ValidationError(format!("无效的 PNG base64 数据: {}", e)))?;

    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if !bytes.starts_with(PNG_SIGNATURE) {
        return Err(AppError::ValidationError("PNG 数据头无效".to_string()));
    }

    Ok(bytes)
}

// ==================== 指标处理器 ====================

async fn list_metrics_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MetricListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page = params.page.max(1);
    let page_size = params.page_size.max(1).min(100);
    let (items, total) = metric_repo(&state)
        .list(page, page_size, params.query.as_deref())
        .await?;

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
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateMetricRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("指标参数校验失败: {}", e)))?;
    validate_select_sql(&state, &payload.expression)?;

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

    let metric = metric_repo(&state).create(&metric).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "metric.create",
        Some("metric"),
        Some(metric.id.to_string()),
        serde_json::json!({"code": metric.code}),
    )
    .await;

    let response = serde_json::json!({
        "code": 0,
        "message": "创建成功",
        "data": metric
    });

    Ok(Json(response))
}

async fn get_metric_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let metric = metric_repo(&state)
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?;

    let response = serde_json::json!({
        "code": 0,
        "message": "Success",
        "data": metric
    });

    Ok(Json(response))
}

async fn update_metric_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMetricRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::validation(format!("指标参数校验失败: {}", e)))?;

    let repo = metric_repo(&state);
    let mut metric = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?;

    if let Some(name) = payload.name {
        metric.name = name;
    }
    if let Some(description) = payload.description {
        metric.description = Some(description);
    }
    if let Some(expression) = payload.expression {
        validate_select_sql(&state, &expression)?;
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
    let updated_metric = repo.update(&metric).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "metric.update",
        Some("metric"),
        Some(updated_metric.id.to_string()),
        serde_json::json!({"code": updated_metric.code}),
    )
    .await;

    let response = serde_json::json!({
        "code": 0,
        "message": "更新成功",
        "data": updated_metric
    });

    Ok(Json(response))
}

async fn delete_metric_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = metric_repo(&state);
    let metric = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?;
    repo.delete(id).await?;
    write_audit(
        &state,
        Some(session.user_id),
        "metric.delete",
        Some("metric"),
        Some(id.to_string()),
        serde_json::json!({"code": metric.code}),
    )
    .await;

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

async fn execute_metric_handler(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 获取指标
    let metric = metric_repo(&state)
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?;

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
    let config = get_connection_config_for_session(&state, connection_id, &session).await?;
    let pool_type = state.connection_manager.get_pool(&config).await?;

    // 替换维度参数
    let mut sql = metric.expression.clone();
    for (key, val) in &dimensions {
        validate_metric_dimension_value(key, val)?;
        sql = sql.replace(&format!("{{{}}}", key), val);
    }
    validate_select_sql(&state, &sql)?;

    let value = match pool_type {
        ConnectionPool::Postgres(pg_pool) => {
            let row = sqlx::query(&sql)
                .fetch_one(&pg_pool)
                .await
                .map_err(|e| AppError::database(format!("指标执行失败: {}", e)))?;
            pg_value_to_json(&row, 0)
        }
        ConnectionPool::Mysql(mysql_pool) => {
            let row = sqlx::query(&sql)
                .fetch_one(&mysql_pool)
                .await
                .map_err(|e| AppError::database(format!("指标执行失败: {}", e)))?;
            mysql_value_to_json(&row, 0)
        }
        ConnectionPool::Sqlite(sqlite_pool) => {
            let row = sqlx::query(&sql)
                .fetch_one(&sqlite_pool)
                .await
                .map_err(|e| AppError::database(format!("指标执行失败: {}", e)))?;
            sqlite_value_to_json(&row, 0)
        }
        ConnectionPool::Clickhouse(config) => {
            let (_columns, rows) = clickhouse_select(&config, &sql).await?;
            rows.first()
                .and_then(|row| row.first())
                .cloned()
                .unwrap_or(serde_json::Value::Null)
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
    write_audit(
        &state,
        Some(session.user_id),
        "metric.execute",
        Some("metric"),
        Some(metric.id.to_string()),
        serde_json::json!({"code": metric.code, "duration_ms": duration_ms}),
    )
    .await;

    Ok(Json(response))
}

async fn get_metric_lineage_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 获取指标
    let repo = metric_repo(&state);
    let metric = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?;

    let analysis = SqlAnalyzer::new(Arc::new(state.config.security.clone()))
        .analyze(&metric.expression)?;
    let source_tables = analysis.referenced_tables;
    let source_columns = analysis.referenced_columns;
    let metrics = repo.list_all().await?;

    // 查找依赖的指标
    let dependent_metrics = metrics
        .iter()
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
        .collect::<Vec<_>>();

    // 查找引用当前指标的指标
    let referenced_by = metrics
        .iter()
        .filter(|m| m.id != metric.id)
        .filter(|m| m.expression.contains(&metric.code))
        .map(|m| serde_json::json!({
            "id": m.id,
            "name": m.name,
            "code": m.code
        }))
        .collect::<Vec<_>>();

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

// ==================== 工具函数 ====================

fn base64_encode(input: &str) -> String {
    base64_encode_bytes(input.as_bytes())
}

fn validate_metric_dimension_value(key: &str, value: &str) -> AppResult<()> {
    let key_re = regex::Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$")
        .map_err(|e| AppError::internal(format!("维度名校验器初始化失败: {}", e)))?;
    if !key_re.is_match(key) {
        return Err(AppError::ValidationError("Invalid metric dimension name".to_string()));
    }

    let value_re = regex::Regex::new(r"^[\p{L}\p{N}_ .:\-@/]+$")
        .map_err(|e| AppError::internal(format!("维度值校验器初始化失败: {}", e)))?;
    if !value_re.is_match(value) {
        return Err(AppError::ValidationError("Invalid metric dimension value".to_string()));
    }

    Ok(())
}

fn base64_encode_bytes(input: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(input)
}

/// 服务器启动函数
pub async fn start_server(state: Arc<crate::state::AppState>) -> anyhow::Result<()> {
    let app = routes(Arc::clone(&state)).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
