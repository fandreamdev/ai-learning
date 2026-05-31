//! SQL 执行 API 处理器
//!
//! 处理 SQL 执行和格式化相关的 HTTP 请求

use crate::error::{AppError, AppResult};
use crate::models::{
    QueryFormatRequest, QueryHistoryListResponse, SqlExecuteRequest, SqlExecuteResponse,
};
use crate::services::SqlAnalyzer;
use crate::services::connection_manager::ConnectionConfig;
use crate::repositories::QueryRepo;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::sync::Arc;

/// 执行 SQL
pub async fn execute_sql(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SqlExecuteRequest>,
) -> AppResult<Json<SqlExecuteResponse>> {
    // 安全检查
    let analyzer = SqlAnalyzer::new(state.config.security.clone());
    let analysis = analyzer.analyze(&request.sql)?;

    if !analysis.is_safe {
        return Err(AppError::SqlSecurityError(
            analysis.blocked_reason.unwrap_or_else(|| "SQL 安全检查失败".to_string()),
        ));
    }

    // 获取连接信息
    let conn_repo = crate::repositories::ConnectionRepo::new(state.db.clone());
    let conn = conn_repo
        .find_by_id(request.connection_id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    // 解码密码
    let password = STANDARD.decode(&conn.encrypted_password)
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default();

    // 创建连接配置
    let config = ConnectionConfig {
        id: conn.id,
        name: conn.name,
        db_type: conn.db_type,
        host: conn.host.clone(),
        port: conn.port,
        database_name: conn.database_name.clone(),
        username: conn.username.clone(),
        password,
    };

    // 获取连接池并执行
    let pool = state.connection_manager.get_pool(&config).await?;

    // 执行 SQL
    let start = std::time::Instant::now();

    use sqlx::Row;
    let rows = sqlx::query(&request.sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::database(format!("查询执行失败: {}", e)))?;

    let duration_ms = start.elapsed().as_millis() as i64;

    // 提取列信息
    let columns: Vec<crate::models::ColumnMetadata> = rows
        .first()
        .map(|row| {
            row.columns()
                .iter()
                .enumerate()
                .map(|(i, col)| crate::models::ColumnMetadata {
                    name: col.name().to_string(),
                    data_type: "unknown".to_string(),
                    ordinal: i as i32,
                })
                .collect()
        })
        .unwrap_or_default();

    // 转换为 JSON 值
    let result_rows: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| {
            (0..row.columns().len())
                .map(|i| {
                    row.try_get::<serde_json::Value, _>(i)
                        .unwrap_or(serde_json::Value::Null)
                })
                .collect()
        })
        .collect();

    let row_count = result_rows.len() as i64;

    // 记录查询历史
    let repo = QueryRepo::new(state.db.clone());
    let mut history = crate::models::QueryHistory::new(
        Some(request.connection_id),
        uuid::Uuid::new_v4(),
        request.sql.clone(),
    );
    history.mark_success(duration_ms, row_count);
    let _ = repo.create(&history).await;

    Ok(Json(SqlExecuteResponse {
        query_id: uuid::Uuid::new_v4(),
        columns,
        rows: result_rows,
        row_count,
        duration_ms,
        execution_plan: None,
    }))
}

/// 格式化 SQL
#[derive(Debug, serde::Deserialize)]
pub struct SqlFormatRequest {
    pub sql: String,
    pub dialect: Option<String>,
}

pub async fn format_sql(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<SqlFormatRequest>,
) -> AppResult<Json<crate::models::query::SqlFormatResponse>> {
    use sqlparser::dialect::{MySqlDialect, PostgreSqlDialect, GenericDialect};
    use sqlparser::parser::Parser;

    let dialect = match request.dialect.as_deref() {
        Some("mysql") => MySqlDialect {},
        Some("postgresql") | Some("postgres") => PostgreSqlDialect {},
        _ => GenericDialect {},
    };

    let statements = Parser::parse_sql(&dialect, &request.sql)
        .map_err(|e| AppError::validation(format!("SQL 解析失败: {}", e)))?;

    let formatted = statements
        .first()
        .map(|s| s.to_string())
        .unwrap_or(request.sql);

    Ok(Json(crate::models::query::SqlFormatResponse {
        formatted_sql: formatted,
    }))
}

/// 获取查询历史
#[derive(Debug, serde::Deserialize)]
pub struct QueryHistoryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub connection_id: Option<uuid::Uuid>,
}

pub async fn get_query_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryHistoryParams>,
) -> AppResult<Json<QueryHistoryListResponse>> {
    let repo = QueryRepo::new(state.db.clone());
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    let (histories, total) = repo.list_by_user(uuid::Uuid::nil(), page, page_size).await?;

    Ok(Json(QueryHistoryListResponse {
        items: histories
            .into_iter()
            .map(|h| crate::models::QueryHistoryItem {
                id: h.id,
                connection_name: None,
                sql_text: h.sql_text,
                status: h.status,
                duration_ms: h.duration_ms,
                row_count: h.row_count,
                created_at: h.created_at,
            })
            .collect(),
        total,
        page,
        page_size,
    }))
}

/// 执行计划分析
pub async fn explain_sql(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SqlExecuteRequest>,
) -> AppResult<Json<crate::models::ExecutionPlan>> {
    // 获取连接信息
    let conn_repo = crate::repositories::ConnectionRepo::new(state.db.clone());
    let conn = conn_repo
        .find_by_id(request.connection_id)
        .await?
        .ok_or_else(|| AppError::NotFound("连接不存在".to_string()))?;

    // 解码密码
    let password = STANDARD.decode(&conn.encrypted_password)
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default();

    // 创建连接配置
    let config = ConnectionConfig {
        id: conn.id,
        name: conn.name,
        db_type: conn.db_type,
        host: conn.host.clone(),
        port: conn.port,
        database_name: conn.database_name.clone(),
        username: conn.username.clone(),
        password,
    };

    let pool = state.connection_manager.get_pool(&config).await?;

    // 执行 EXPLAIN
    let explain_sql = format!("EXPLAIN {}", request.sql);
    let result: (String,) = sqlx::query_as(&explain_sql)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::database(format!("EXPLAIN 失败: {}", e)))?;

    Ok(Json(crate::models::ExecutionPlan {
        plan_type: "SELECT".to_string(),
        estimated_cost: None,
        estimated_rows: None,
        actual_rows: None,
        details: serde_json::json!({ "raw": result.0 }),
    }))
}
