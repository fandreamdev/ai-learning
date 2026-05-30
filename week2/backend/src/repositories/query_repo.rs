//! 查询历史仓储
//!
//! SQL 查询历史数据的数据库操作

use crate::error::AppResult;
use crate::models::{QueryHistory, QueryStatus};
use sqlx::PgPool;
use uuid::Uuid;

/// 查询历史仓储
#[derive(Clone)]
pub struct QueryRepo {
    pool: PgPool,
}

impl QueryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建查询历史记录
    pub async fn create(&self, history: &QueryHistory) -> AppResult<QueryHistory> {
        let row = sqlx::query_as::<_, QueryHistoryRow>(
            r#"
            INSERT INTO query_history
                (id, connection_id, user_id, sql_text, status, duration_ms, row_count, error_message, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, connection_id, user_id, sql_text, status, duration_ms, row_count, error_message, created_at
            "#,
        )
        .bind(history.id)
        .bind(history.connection_id)
        .bind(history.user_id)
        .bind(&history.sql_text)
        .bind(history.status.as_str())
        .bind(history.duration_ms)
        .bind(history.row_count)
        .bind(&history.error_message)
        .bind(history.created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// 更新查询历史
    pub async fn update(&self, history: &QueryHistory) -> AppResult<QueryHistory> {
        let row = sqlx::query_as::<_, QueryHistoryRow>(
            r#"
            UPDATE query_history
            SET status = $2, duration_ms = $3, row_count = $4, error_message = $5
            WHERE id = $1
            RETURNING id, connection_id, user_id, sql_text, status, duration_ms, row_count, error_message, created_at
            "#,
        )
        .bind(history.id)
        .bind(history.status.as_str())
        .bind(history.duration_ms)
        .bind(history.row_count)
        .bind(&history.error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// 获取用户的查询历史（分页）
    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<QueryHistory>, i64)> {
        let offset = (page - 1) * page_size;

        let histories = sqlx::query_as::<_, QueryHistoryRow>(
            r#"
            SELECT id, connection_id, user_id, sql_text, status, duration_ms, row_count, error_message, created_at
            FROM query_history
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM query_history WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((histories.into_iter().map(|r| r.into()).collect(), total.0))
    }

    /// 获取连接最近的查询
    pub async fn get_recent_by_connection(
        &self,
        connection_id: Uuid,
        limit: i32,
    ) -> AppResult<Vec<QueryHistory>> {
        let histories = sqlx::query_as::<_, QueryHistoryRow>(
            r#"
            SELECT id, connection_id, user_id, sql_text, status, duration_ms, row_count, error_message, created_at
            FROM query_history
            WHERE connection_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(connection_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(histories.into_iter().map(|r| r.into()).collect())
    }
}

/// 查询历史行（数据库映射）
#[derive(Debug, sqlx::FromRow)]
struct QueryHistoryRow {
    id: Uuid,
    connection_id: Option<Uuid>,
    user_id: Uuid,
    sql_text: String,
    status: String,
    duration_ms: Option<i64>,
    row_count: Option<i64>,
    error_message: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<QueryHistoryRow> for QueryHistory {
    fn from(row: QueryHistoryRow) -> Self {
        QueryHistory {
            id: row.id,
            connection_id: row.connection_id,
            user_id: row.user_id,
            sql_text: row.sql_text,
            status: match row.status.as_str() {
                "success" => QueryStatus::Success,
                "failed" => QueryStatus::Failed,
                "cancelled" => QueryStatus::Cancelled,
                _ => QueryStatus::Failed,
            },
            duration_ms: row.duration_ms,
            row_count: row.row_count,
            error_message: row.error_message,
            created_at: row.created_at,
        }
    }
}
