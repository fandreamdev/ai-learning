//! Schema 检索服务
//!
//! 提供 Schema RAG（检索增强生成）功能
//! 支持基于关键词的检索和 pgvector 向量检索

use crate::error::{AppError, AppResult};
use crate::models::semantic::{SchemaColumn, SchemaRetrievalItem, SchemaRetrievalResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

/// Schema 检索服务
#[derive(Clone)]
pub struct SchemaRetrievalService {
    pool: Option<PgPool>,
    cache: Arc<std::sync::RwLock<HashMap<String, SchemaCache>>>,
}

impl SchemaRetrievalService {
    /// 创建新的 Schema 检索服务
    pub fn new() -> Self {
        Self {
            pool: None,
            cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 使用数据库连接池创建服务
    pub fn with_pool(pool: PgPool) -> Self {
        Self {
            pool: Some(pool),
            cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 检索相关的 Schema 信息
    pub fn retrieve(
        &self,
        query: &str,
        schema_items: &[SchemaRetrievalItem],
    ) -> AppResult<SchemaRetrievalResponse> {
        let start = std::time::Instant::now();

        // 简单的关键词匹配
        let keywords = self.extract_keywords(query);
        let mut scored_items: Vec<(SchemaRetrievalItem, f32)> = schema_items
            .iter()
            .map(|item| {
                let score = self.calculate_relevance(item, &keywords);
                (item.clone(), score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // 按相关性排序
        scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 取前 5 个
        let results: Vec<SchemaRetrievalItem> = scored_items
            .into_iter()
            .take(5)
            .map(|(item, _)| item)
            .collect();

        let retrieval_time_ms = start.elapsed().as_millis() as i64;

        Ok(SchemaRetrievalResponse {
            schemas: results,
            retrieval_time_ms,
        })
    }

    /// 检索相关的 Schema 信息（使用 pgvector）
    pub async fn retrieve_with_vector(
        &self,
        query: &str,
        connection_id: &str,
    ) -> AppResult<SchemaRetrievalResponse> {
        let start = std::time::Instant::now();

        let pool = self.pool.as_ref()
            .ok_or_else(|| AppError::InternalError("数据库连接池未初始化".to_string()))?;

        // 检查 pgvector 是否可用
        let vector_available = self.check_pgvector(pool).await.unwrap_or(false);

        let schemas = if vector_available {
            // 使用 pgvector 进行向量检索
            self.vector_search(pool, query, connection_id).await?
        } else {
            // 回退到关键词检索
            self.keyword_search(pool, query, connection_id).await?
        };

        let retrieval_time_ms = start.elapsed().as_millis() as i64;

        Ok(SchemaRetrievalResponse {
            schemas,
            retrieval_time_ms,
        })
    }

    /// 检查 pgvector 扩展是否可用
    async fn check_pgvector(&self, pool: &PgPool) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::database(format!("检查 pgvector 失败: {}", e)))?;

        Ok(result.0)
    }

    /// 向量检索
    async fn vector_search(
        &self,
        pool: &PgPool,
        query: &str,
        connection_id: &str,
    ) -> AppResult<Vec<SchemaRetrievalItem>> {
        // 检查表是否存在
        let table_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'schema_embeddings')"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::database(format!("检查表失败: {}", e)))?;

        if !table_exists.0 {
            return Ok(vec![]);
        }

        // 执行向量相似度搜索
        let rows: Vec<SchemaEmbeddingRow> = sqlx::query_as(
            r#"
            SELECT id, table_name, column_name, description, embedding <=> get_embedding($1) as distance
            FROM schema_embeddings
            WHERE connection_id = $2
            ORDER BY embedding <=> get_embedding($1)
            LIMIT 10
            "#
        )
        .bind(query)
        .bind(connection_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("向量检索失败: {}", e)))?;

        // 按表名分组
        let mut table_map: HashMap<String, SchemaRetrievalItem> = HashMap::new();
        for row in rows {
            table_map.entry(row.table_name.clone()).or_insert_with(|| {
                SchemaRetrievalItem {
                    table_name: row.table_name.clone(),
                    columns: vec![],
                    business_name: None,
                    similarity: 1.0 - row.distance.min(1.0),
                }
            });

            if let Some(table) = table_map.get_mut(&row.table_name) {
                if !row.column_name.is_empty() {
                    table.columns.push(SchemaColumn {
                        column_name: row.column_name.clone(),
                        data_type: "unknown".to_string(),
                        business_name: row.description,
                        synonyms: vec![],
                    });
                }
            }
        }

        Ok(table_map.into_values().collect())
    }

    /// 关键词检索
    async fn keyword_search(
        &self,
        pool: &PgPool,
        query: &str,
        connection_id: &str,
    ) -> AppResult<Vec<SchemaRetrievalItem>> {
        // 检查表是否存在
        let table_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'schema_embeddings')"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::database(format!("检查表失败: {}", e)))?;

        if !table_exists.0 {
            return Ok(vec![]);
        }

        let keywords = self.extract_keywords(query);
        let search_pattern = format!("%{}%", keywords.join("%"));

        let rows: Vec<SchemaEmbeddingRow> = sqlx::query_as(
            r#"
            SELECT id, table_name, column_name, description, 0.0 as distance
            FROM schema_embeddings
            WHERE connection_id = $1
            AND (table_name ILIKE $2 OR column_name ILIKE $2 OR description ILIKE $2)
            LIMIT 20
            "#
        )
        .bind(connection_id)
        .bind(&search_pattern)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("关键词检索失败: {}", e)))?;

        // 按表名分组
        let mut table_map: HashMap<String, SchemaRetrievalItem> = HashMap::new();
        for row in rows {
            table_map.entry(row.table_name.clone()).or_insert_with(|| {
                SchemaRetrievalItem {
                    table_name: row.table_name.clone(),
                    columns: vec![],
                    business_name: None,
                    similarity: 1.0,
                }
            });

            if let Some(table) = table_map.get_mut(&row.table_name) {
                if !row.column_name.is_empty() {
                    table.columns.push(SchemaColumn {
                        column_name: row.column_name.clone(),
                        data_type: "unknown".to_string(),
                        business_name: row.description,
                        synonyms: vec![],
                    });
                }
            }
        }

        Ok(table_map.into_values().collect())
    }

    /// 存储 Schema 嵌入向量
    pub async fn store_embeddings(
        &self,
        pool: &PgPool,
        connection_id: &str,
        items: &[SchemaRetrievalItem],
    ) -> AppResult<()> {
        // 确保表存在
        self.ensure_table(pool).await?;

        for item in items {
            // 存储表级别的嵌入
            let table_text = format!("表: {}", item.table_name);
            let _ = sqlx::query(
                r#"
                INSERT INTO schema_embeddings (connection_id, table_name, column_name, description, embedding)
                VALUES ($1, $2, '', '', get_embedding($3))
                ON CONFLICT (connection_id, table_name, column_name)
                DO UPDATE SET description = EXCLUDED.description, embedding = EXCLUDED.embedding
                "#
            )
            .bind(connection_id)
            .bind(&item.table_name)
            .bind(&table_text)
            .execute(pool)
            .await;

            // 存储列级别的嵌入
            for col in &item.columns {
                let col_text = format!("表 {} 列: {} ({})", item.table_name, col.column_name, col.business_name.as_deref().unwrap_or(""));
                let _ = sqlx::query(
                    r#"
                    INSERT INTO schema_embeddings (connection_id, table_name, column_name, description, embedding)
                    VALUES ($1, $2, $3, $4, get_embedding($5))
                    ON CONFLICT (connection_id, table_name, column_name)
                    DO UPDATE SET description = EXCLUDED.description, embedding = EXCLUDED.embedding
                    "#
                )
                .bind(connection_id)
                .bind(&item.table_name)
                .bind(&col.column_name)
                .bind(&col.business_name)
                .bind(&col_text)
                .execute(pool)
                .await;
            }
        }

        Ok(())
    }

    /// 确保嵌入表存在
    async fn ensure_table(&self, pool: &PgPool) -> AppResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_embeddings (
                id SERIAL PRIMARY KEY,
                connection_id VARCHAR(255) NOT NULL,
                table_name VARCHAR(255) NOT NULL,
                column_name VARCHAR(255) NOT NULL DEFAULT '',
                description TEXT,
                embedding VECTOR(1536),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(connection_id, table_name, column_name)
            );

            CREATE INDEX IF NOT EXISTS idx_schema_embeddings_connection
            ON schema_embeddings(connection_id);

            CREATE INDEX IF NOT EXISTS idx_schema_embeddings_vector
            ON schema_embeddings USING ivfflat (embedding vector_cosine_ops)
            WITH (lists = 100);
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("创建嵌入表失败: {}", e)))?;

        // 检查 get_embedding 函数是否存在
        let fn_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM pg_proc WHERE proname = 'get_embedding')"
        )
        .fetch_one(pool)
        .await
        .unwrap_or((false,));

        if !fn_exists.0 {
            // 创建 get_embedding 函数（使用 OpenAI API）
            sqlx::query(
                r#"
                CREATE OR REPLACE FUNCTION get_embedding(text TEXT)
                RETURNS VECTOR(1536) AS $$
                BEGIN
                    -- 这是一个占位函数
                    -- 实际生产环境需要集成 OpenAI embedding API
                    -- 或者使用其他 embedding 服务
                    RETURN NULL::VECTOR;
                END;
                $$ LANGUAGE plpgsql;
                "#
            )
            .execute(pool)
            .await
            .map_err(|e| AppError::database(format!("创建 embedding 函数失败: {}", e)))?;
        }

        Ok(())
    }

    /// 提取引擎关键词
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        let stop_words = vec![
            "的", "是", "在", "有", "和", "与", "或", "帮", "查", "看", "一下",
            "多少", "哪些", "什么", "如何", "怎么", "请", "我", "要", "想", "给",
            "the", "a", "an", "is", "are", "and", "or", "to", "of", "in",
        ];

        query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
            .collect::<String>()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty() && !stop_words.contains(s))
            .map(|s| s.to_lowercase())
            .collect()
    }

    /// 计算相关性分数
    fn calculate_relevance(&self, item: &SchemaRetrievalItem, keywords: &[String]) -> f32 {
        let mut score = 0.0;

        // 表名匹配
        let table_name_lower = item.table_name.to_lowercase();
        for keyword in keywords {
            if table_name_lower.contains(keyword) {
                score += 2.0;
            }
        }

        // 列名匹配
        for column in &item.columns {
            let column_name_lower = column.column_name.to_lowercase();
            for keyword in keywords {
                if column_name_lower.contains(keyword) {
                    score += 1.0;
                }
                if let Some(business_name) = &column.business_name {
                    if business_name.contains(keyword) {
                        score += 1.5;
                    }
                }
                for synonym in &column.synonyms {
                    if synonym.to_lowercase().contains(keyword) {
                        score += 1.2;
                    }
                }
            }
        }

        score
    }

    /// 构建 Schema 上下文
    pub fn build_context(&self, items: &[SchemaRetrievalItem]) -> String {
        let mut context = String::new();

        for item in items {
            context.push_str(&format!("表: {}\n", item.table_name));

            if let Some(business_name) = &item.business_name {
                context.push_str(&format!("  说明: {}\n", business_name));
            }

            context.push_str("  字段:\n");
            for column in &item.columns {
                let mut col_info = format!("    - {} ({})", column.column_name, column.data_type);

                if let Some(business_name) = &column.business_name {
                    col_info.push_str(&format!(" [{}]", business_name));
                }

                if !column.synonyms.is_empty() {
                    col_info.push_str(&format!(" 同义词: {}", column.synonyms.join(", ")));
                }

                context.push_str(&col_info);
                context.push('\n');
            }

            context.push('\n');
        }

        context
    }

    /// 从数据库获取表结构信息
    pub async fn fetch_schema_from_db(
        &self,
        pool: &PgPool,
        connection_id: &str,
    ) -> AppResult<Vec<SchemaRetrievalItem>> {
        let rows: Vec<TableSchemaRow> = sqlx::query_as(
            r#"
            SELECT
                t.table_name,
                t.table_schema,
                obj_description((t.table_schema || '.' || t.table_name)::regclass, 'pg_class') as description
            FROM information_schema.tables t
            WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')
            AND t.table_type = 'BASE TABLE'
            ORDER BY t.table_name
            LIMIT 50
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("获取表结构失败: {}", e)))?;

        let mut items = Vec::new();

        for row in rows {
            let columns = self.fetch_columns(pool, &row.table_schema, &row.table_name).await?;

            items.push(SchemaRetrievalItem {
                table_name: format!("{}.{}", row.table_schema, row.table_name),
                columns,
                business_name: row.description,
                similarity: 0.0,
            });
        }

        Ok(items)
    }

    /// 获取表的列信息
    async fn fetch_columns(
        &self,
        pool: &PgPool,
        schema: &str,
        table: &str,
    ) -> AppResult<Vec<SchemaColumn>> {
        let rows: Vec<ColumnSchemaRow> = sqlx::query_as(
            r#"
            SELECT
                c.column_name,
                c.data_type,
                col_description((c.table_schema || '.' || c.table_name)::regclass, c.ordinal_position) as description
            FROM information_schema.columns c
            WHERE c.table_schema = $1 AND c.table_name = $2
            ORDER BY c.ordinal_position
            "#
        )
        .bind(schema)
        .bind(table)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("获取列信息失败: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| SchemaColumn {
                column_name: r.column_name,
                data_type: r.data_type,
                business_name: r.description,
                synonyms: vec![],
            })
            .collect())
    }
}

impl Default for SchemaRetrievalService {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema 缓存
#[derive(Debug, Clone)]
struct SchemaCache {
    connection_id: String,
    tables: Vec<TableSchema>,
    cached_at: std::time::Instant,
}

/// 表 Schema
#[derive(Debug, Clone)]
struct TableSchema {
    table_name: String,
    business_name: Option<String>,
    columns: Vec<ColumnSchema>,
}

/// 列 Schema
#[derive(Debug, Clone)]
struct ColumnSchema {
    column_name: String,
    data_type: String,
    business_name: Option<String>,
    synonyms: Vec<String>,
}

/// Schema 嵌入行
#[derive(Debug, Clone, sqlx::FromRow)]
struct SchemaEmbeddingRow {
    id: i32,
    table_name: String,
    column_name: String,
    description: Option<String>,
    distance: f32,
}

/// 表结构行
#[derive(Debug, Clone, sqlx::FromRow)]
struct TableSchemaRow {
    table_name: String,
    table_schema: String,
    description: Option<String>,
}

/// 列结构行
#[derive(Debug, Clone, sqlx::FromRow)]
struct ColumnSchemaRow {
    column_name: String,
    data_type: String,
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let service = SchemaRetrievalService::new();

        let keywords = service.extract_keywords("查一下用户订单数量");
        assert!(keywords.contains(&"用户".to_string()));
        assert!(keywords.contains(&"订单".to_string()));
        assert!(keywords.contains(&"数量".to_string()));
    }

    #[test]
    fn test_relevance_calculation() {
        let service = SchemaRetrievalService::new();

        let item = SchemaRetrievalItem {
            table_name: "orders".to_string(),
            columns: vec![
                SchemaColumn {
                    column_name: "user_id".to_string(),
                    data_type: "int".to_string(),
                    business_name: Some("用户ID".to_string()),
                    synonyms: vec!["用户".to_string()],
                },
                SchemaColumn {
                    column_name: "amount".to_string(),
                    data_type: "decimal".to_string(),
                    business_name: Some("金额".to_string()),
                    synonyms: vec![],
                },
            ],
            business_name: Some("订单表".to_string()),
            similarity: 0.0,
        };

        let keywords = vec!["用户".to_string(), "订单".to_string()];

        let score = service.calculate_relevance(&item, &keywords);
        assert!(score > 0.0);
    }

    #[test]
    fn test_build_context() {
        let service = SchemaRetrievalService::new();

        let items = vec![SchemaRetrievalItem {
            table_name: "users".to_string(),
            columns: vec![
                SchemaColumn {
                    column_name: "id".to_string(),
                    data_type: "int".to_string(),
                    business_name: Some("用户ID".to_string()),
                    synonyms: vec![],
                },
                SchemaColumn {
                    column_name: "name".to_string(),
                    data_type: "varchar".to_string(),
                    business_name: Some("用户名".to_string()),
                    synonyms: vec!["姓名".to_string()],
                },
            ],
            business_name: Some("用户表".to_string()),
            similarity: 0.0,
        }];

        let context = service.build_context(&items);
        assert!(context.contains("users"));
        assert!(context.contains("用户表"));
        assert!(context.contains("id"));
    }
}
