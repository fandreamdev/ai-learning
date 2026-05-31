//! 语义定义仓储

use crate::error::AppResult;
use crate::models::SemanticDefinition;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SemanticRepo {
    pool: PgPool,
}

impl SemanticRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        connection_id: Option<Uuid>,
        page: i32,
        page_size: i32,
        query: Option<&str>,
    ) -> AppResult<(Vec<SemanticDefinition>, i64)> {
        let offset = (page - 1).max(0) * page_size;
        let like_query = query.map(|q| format!("%{}%", q));

        let items = sqlx::query_as::<_, SemanticDefinition>(
            r#"
            SELECT id, connection_id, table_name, column_name, business_name,
                   business_description, synonyms, is_active, created_at, updated_at
            FROM semantic_definitions
            WHERE
                ($1::uuid IS NULL OR connection_id = $1)
                AND (
                    $2::text IS NULL
                    OR table_name ILIKE $2
                    OR column_name ILIKE $2
                    OR business_name ILIKE $2
                    OR business_description ILIKE $2
                )
            ORDER BY updated_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(connection_id)
        .bind(like_query.as_deref())
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM semantic_definitions
            WHERE
                ($1::uuid IS NULL OR connection_id = $1)
                AND (
                    $2::text IS NULL
                    OR table_name ILIKE $2
                    OR column_name ILIKE $2
                    OR business_name ILIKE $2
                    OR business_description ILIKE $2
                )
            "#,
        )
        .bind(connection_id)
        .bind(like_query.as_deref())
        .fetch_one(&self.pool)
        .await?;

        Ok((items, total))
    }

    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SemanticDefinition>> {
        let item = sqlx::query_as::<_, SemanticDefinition>(
            r#"
            SELECT id, connection_id, table_name, column_name, business_name,
                   business_description, synonyms, is_active, created_at, updated_at
            FROM semantic_definitions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn create(&self, item: &SemanticDefinition) -> AppResult<SemanticDefinition> {
        let created = sqlx::query_as::<_, SemanticDefinition>(
            r#"
            INSERT INTO semantic_definitions
                (id, connection_id, table_name, column_name, business_name,
                 business_description, synonyms, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id, connection_id, table_name, column_name, business_name,
                      business_description, synonyms, is_active, created_at, updated_at
            "#,
        )
        .bind(item.id)
        .bind(item.connection_id)
        .bind(&item.table_name)
        .bind(&item.column_name)
        .bind(&item.business_name)
        .bind(&item.business_description)
        .bind(&item.synonyms)
        .bind(item.is_active)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    pub async fn update(&self, item: &SemanticDefinition) -> AppResult<SemanticDefinition> {
        let updated = sqlx::query_as::<_, SemanticDefinition>(
            r#"
            UPDATE semantic_definitions
            SET business_name = $2,
                business_description = $3,
                synonyms = $4,
                is_active = $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, connection_id, table_name, column_name, business_name,
                      business_description, synonyms, is_active, created_at, updated_at
            "#,
        )
        .bind(item.id)
        .bind(&item.business_name)
        .bind(&item.business_description)
        .bind(&item.synonyms)
        .bind(item.is_active)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM semantic_definitions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn stats(&self) -> AppResult<serde_json::Value> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM semantic_definitions")
            .fetch_one(&self.pool)
            .await?;
        let table_level: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM semantic_definitions WHERE column_name IS NULL",
        )
        .fetch_one(&self.pool)
        .await?;
        let column_level: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM semantic_definitions WHERE column_name IS NOT NULL",
        )
        .fetch_one(&self.pool)
        .await?;
        let connections: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT connection_id) FROM semantic_definitions",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(serde_json::json!({
            "total_definitions": total,
            "table_level_definitions": table_level,
            "column_level_definitions": column_level,
            "connections_with_semantics": connections
        }))
    }
}
