//! 审计日志仓储

use crate::error::AppResult;
use sqlx::PgPool;
use uuid::Uuid;

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
}
