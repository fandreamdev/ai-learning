//! 指标仓储
//!
//! 指标数据的数据库操作

use crate::error::AppResult;
use crate::models::{FormatType, Metric};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct MetricRepo {
    pool: PgPool,
}

impl MetricRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        page: i32,
        page_size: i32,
        query: Option<&str>,
    ) -> AppResult<(Vec<Metric>, i64)> {
        let page = page.max(1);
        let page_size = page_size.max(1).min(100);
        let offset = (page - 1) * page_size;
        let pattern = query.map(|q| format!("%{}%", q));

        let items = sqlx::query_as::<_, MetricRow>(
            r#"
            SELECT id, name, code, description, expression, dimensions, unit,
                   format_type, created_by, created_at, updated_at
            FROM metrics
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR code ILIKE $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(pattern.as_deref())
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM metrics
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR code ILIKE $1)
            "#,
        )
        .bind(pattern.as_deref())
        .fetch_one(&self.pool)
        .await?;

        Ok((items.into_iter().map(Into::into).collect(), total.0))
    }

    pub async fn create(&self, metric: &Metric) -> AppResult<Metric> {
        let row = sqlx::query_as::<_, MetricRow>(
            r#"
            INSERT INTO metrics
                (id, name, code, description, expression, dimensions, unit,
                 format_type, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, name, code, description, expression, dimensions, unit,
                      format_type, created_by, created_at, updated_at
            "#,
        )
        .bind(metric.id)
        .bind(&metric.name)
        .bind(&metric.code)
        .bind(&metric.description)
        .bind(&metric.expression)
        .bind(&metric.dimensions)
        .bind(&metric.unit)
        .bind(metric.format_type.as_str())
        .bind(metric.created_by)
        .bind(metric.created_at)
        .bind(metric.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Metric>> {
        let row = sqlx::query_as::<_, MetricRow>(
            r#"
            SELECT id, name, code, description, expression, dimensions, unit,
                   format_type, created_by, created_at, updated_at
            FROM metrics
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    pub async fn update(&self, metric: &Metric) -> AppResult<Metric> {
        let row = sqlx::query_as::<_, MetricRow>(
            r#"
            UPDATE metrics
            SET name = $2, description = $3, expression = $4, dimensions = $5,
                unit = $6, format_type = $7, updated_at = $8
            WHERE id = $1
            RETURNING id, name, code, description, expression, dimensions, unit,
                      format_type, created_by, created_at, updated_at
            "#,
        )
        .bind(metric.id)
        .bind(&metric.name)
        .bind(&metric.description)
        .bind(&metric.expression)
        .bind(&metric.dimensions)
        .bind(&metric.unit)
        .bind(metric.format_type.as_str())
        .bind(metric.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM metrics WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_all(&self) -> AppResult<Vec<Metric>> {
        let rows = sqlx::query_as::<_, MetricRow>(
            r#"
            SELECT id, name, code, description, expression, dimensions, unit,
                   format_type, created_by, created_at, updated_at
            FROM metrics
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MetricRow {
    id: Uuid,
    name: String,
    code: String,
    description: Option<String>,
    expression: String,
    dimensions: Option<serde_json::Value>,
    unit: Option<String>,
    format_type: String,
    created_by: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<MetricRow> for Metric {
    fn from(row: MetricRow) -> Self {
        Metric {
            id: row.id,
            name: row.name,
            code: row.code,
            description: row.description,
            expression: row.expression,
            dimensions: row.dimensions,
            unit: row.unit,
            format_type: FormatType::from_str(&row.format_type).unwrap_or_default(),
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
