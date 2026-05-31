//! 审计日志仓储

use crate::error::AppResult;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AuditLogItem {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct AuditRepo {
    pool: PgPool,
}

impl AuditRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        resource_type: Option<&str>,
        resource_id: Option<&str>,
        details: serde_json::Value,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs
                (id, user_id, action, resource_type, resource_id, details, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list(
        &self,
        page: i32,
        page_size: i32,
        query: Option<&str>,
    ) -> AppResult<(Vec<AuditLogItem>, i64)> {
        let offset = (page - 1).max(0) * page_size;
        let like_query = query.map(|q| format!("%{}%", q));

        let items = sqlx::query_as::<_, AuditLogItem>(
            r#"
            SELECT
                a.id,
                a.user_id,
                u.username,
                a.action,
                a.resource_type,
                a.resource_id::text AS resource_id,
                a.details,
                a.ip_address,
                a.created_at
            FROM audit_logs a
            LEFT JOIN users u ON u.id = a.user_id
            WHERE
                $1::text IS NULL
                OR u.username ILIKE $1
                OR a.action ILIKE $1
                OR a.resource_type ILIKE $1
                OR a.resource_id::text ILIKE $1
            ORDER BY a.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(like_query.as_deref())
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM audit_logs a
            LEFT JOIN users u ON u.id = a.user_id
            WHERE
                $1::text IS NULL
                OR u.username ILIKE $1
                OR a.action ILIKE $1
                OR a.resource_type ILIKE $1
                OR a.resource_id::text ILIKE $1
            "#,
        )
        .bind(like_query.as_deref())
        .fetch_one(&self.pool)
        .await?;

        Ok((items, total))
    }
}
