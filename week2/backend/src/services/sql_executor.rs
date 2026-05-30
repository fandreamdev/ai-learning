//! SQL 执行器服务
//!
//! 负责 SQL 查询的执行和结果处理

use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use crate::models::{
    ColumnMetadata, DatabaseType, ExecutionPlan, QueryHistory, QueryStatus, SqlExecuteRequest,
    SqlExecuteResponse, SqlFormatRequest,
};
use chrono::Utc;
use sqlparser::ast::{Select, SetExpr, Statement};
use sqlparser::dialect::{ClickHouseDialect, GenericDialect, MySqlDialect, PostgreSqlDialect};
use sqlparser::parser::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// SQL 执行器
#[derive(Clone)]
pub struct SqlExecutor {
    config: Arc<AppConfig>,
}

impl SqlExecutor {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// 执行 SQL 查询
    pub async fn execute(
        &self,
        pool: &sqlx::PgPool,
        request: &SqlExecuteRequest,
    ) -> AppResult<(SqlExecuteResponse, QueryHistory)> {
        let start = Instant::now();
        let mut history = QueryHistory::new(Some(request.connection_id), Uuid::new_v4(), request.sql.clone());

        // 解析 SQL
        let dialect = self.get_dialect(&request.connection_id, pool).await?;
        let statements = Parser::parse_sql(&dialect, &request.sql)
            .map_err(|e| AppError::validation(format!("SQL 解析失败: {}", e)))?;

        if statements.is_empty() {
            return Err(AppError::validation("SQL 语句不能为空"));
        }

        let statement = &statements[0];

        // 执行查询并记录结果
        let result = self
            .execute_statement(pool, statement, &dialect, request.timeout)
            .await;

        let duration_ms = start.elapsed().as_millis() as i64;

        match result {
            Ok(response) => {
                history.mark_success(duration_ms, response.row_count);
                Ok((response, history))
            }
            Err(e) => {
                history.mark_failed(e.to_string());
                Err(e)
            }
        }
    }

    /// 执行单条 SQL 语句
    async fn execute_statement(
        &self,
        pool: &sqlx::PgPool,
        statement: &Statement,
        dialect: &GenericDialect,
        timeout: Option<u64>,
    ) -> AppResult<SqlExecuteResponse> {
        match statement {
            Statement::Query(query) => {
                self.execute_select(pool, query, dialect, timeout).await
            }
            _ => Err(AppError::DmlForbidden(
                "Only SELECT queries are allowed in natural language mode".to_string(),
            )),
        }
    }

    /// 执行 SELECT 查询
    async fn execute_select(
        &self,
        pool: &sqlx::PgPool,
        query: &Select,
        dialect: &GenericDialect,
        timeout: Option<u64>,
    ) -> AppResult<SqlExecuteResponse> {
        // 生成列信息
        let columns = self.extract_columns(query);

        // 构建查询
        let sql = statement_to_string(query, dialect);

        // 执行查询
        let start = Instant::now();
        let rows = sqlx::query_as::<_, (serde_json::Value,)>(&sql)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::database(format!("Query execution failed: {}", e)))?;

        let duration_ms = start.elapsed().as_millis() as i64;

        // 处理结果
        let rows: Vec<Vec<serde_json::Value>> = rows
            .into_iter()
            .map(|(row,)| {
                if let serde_json::Value::Array(arr) = row {
                    arr
                } else {
                    vec![row]
                }
            })
            .collect();

        let row_count = rows.len() as i64;

        Ok(SqlExecuteResponse {
            query_id: Uuid::new_v4(),
            columns,
            rows,
            row_count,
            duration_ms,
            execution_plan: None,
        })
    }

    /// 提取列信息
    fn extract_columns(&self, query: &Select) -> Vec<ColumnMetadata> {
        query
            .projection
            .iter()
            .enumerate()
            .map(|(i, expr)| {
                let name = expr
                    .as_ref()
                    .name()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| format!("column_{}", i + 1));
                let data_type = "unknown".to_string();

                ColumnMetadata {
                    name,
                    data_type,
                    ordinal: i as i32,
                }
            })
            .collect()
    }

    /// 获取数据库方言
    async fn get_dialect(
        &self,
        _connection_id: &Uuid,
        _pool: &sqlx::PgPool,
    ) -> AppResult<GenericDialect> {
        // TODO: 从连接配置获取实际的方言
        // 目前默认使用通用方言
        Ok(GenericDialect {})
    }

    /// 格式化 SQL
    pub fn format_sql(&self, request: &SqlFormatRequest) -> AppResult<String> {
        let dialect = match request.dialect.to_lowercase().as_str() {
            "mysql" => MySqlDialect {},
            "postgresql" | "postgres" => PostgreSqlDialect {},
            "clickhouse" => ClickHouseDialect {},
            _ => GenericDialect {},
        };

        let statements = Parser::parse_sql(&dialect, &request.sql)
            .map_err(|e| AppError::validation(format!("SQL 解析失败: {}", e)))?;

        if statements.is_empty() {
            return Err(AppError::validation("SQL 语句不能为空"));
        }

        let formatted = statement_to_string(&statements[0], &dialect);
        Ok(formatted)
    }

    /// 获取执行计划
    pub async fn explain(
        &self,
        pool: &sqlx::PgPool,
        sql: &str,
    ) -> AppResult<ExecutionPlan> {
        let explain_sql = format!("EXPLAIN {}", sql);

        let result: (String,) = sqlx::query_as(&explain_sql)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::database(format!("EXPLAIN failed: {}", e)))?;

        Ok(ExecutionPlan {
            plan_type: "unknown".to_string(),
            estimated_cost: None,
            estimated_rows: None,
            actual_rows: None,
            details: serde_json::json!({ "raw": result.0 }),
        })
    }
}

/// 将语句转换为字符串
fn statement_to_string(statement: &Statement, dialect: &GenericDialect) -> String {
    let _ = dialect;
    statement.to_string()
}

use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_executor() -> SqlExecutor {
        let config = AppConfig::load_from_file("config.yaml").unwrap_or_else(|_| {
            crate::config::AppConfig {
                app: crate::config::AppSettings {
                    name: "test".to_string(),
                    host: "0.0.0.0".to_string(),
                    port: 8080,
                    env: "development".to_string(),
                },
                database: crate::config::DatabaseConfig {
                    url: "postgres://localhost/test".to_string(),
                    max_connections: 5,
                    min_connections: 1,
                    connect_timeout: 5,
                    idle_timeout: 300,
                },
                redis: crate::config::RedisConfig {
                    url: "redis://localhost".to_string(),
                    max_connections: 5,
                    pool_timeout: 5,
                },
                llm: crate::config::LlmConfig {
                    provider: "openai".to_string(),
                    openai: crate::config::OpenAiConfig {
                        api_key: "test".to_string(),
                        base_url: "https://api.openai.com/v1".to_string(),
                        model: "gpt-4".to_string(),
                        max_tokens: 4096,
                        temperature: 0.1,
                    },
                    anthropic: crate::config::AnthropicConfig {
                        api_key: "".to_string(),
                        base_url: "".to_string(),
                        model: "".to_string(),
                        max_tokens: 0,
                        temperature: 0.0,
                    },
                    local: crate::config::LocalLlmConfig {
                        base_url: "".to_string(),
                        model: "".to_string(),
                        api_key: "".to_string(),
                    },
                },
                jwt: crate::config::JwtConfig {
                    secret: "test-secret".to_string(),
                    access_token_expires: 3600,
                    refresh_token_expires: 604800,
                    issuer: "test".to_string(),
                },
                security: crate::config::SecurityConfig {
                    argon2: crate::config::Argon2Config {
                        memory_cost: 19456,
                        time_cost: 2,
                        parallelism: 1,
                    },
                    cors: crate::config::CorsConfig {
                        allowed_origins: vec!["*".to_string()],
                        allowed_methods: vec!["GET".to_string()],
                        allowed_headers: vec!["Content-Type".to_string()],
                        max_age: 3600,
                    },
                    sql: crate::config::SqlSecurityConfig {
                        max_query_length: 65536,
                        query_timeout: 30000,
                        allow_dangerous_functions: false,
                        blocked_functions: vec![],
                    },
                },
                logging: crate::config::LoggingConfig {
                    level: "info".to_string(),
                    format: "json".to_string(),
                    include_target: true,
                    include_span_list: true,
                },
            }
        });

        SqlExecutor::new(Arc::new(config))
    }

    #[test]
    fn test_format_sql_mysql() {
        let executor = create_test_executor();
        let result = executor.format_sql(&SqlFormatRequest {
            sql: "select id,name from users where status=1".to_string(),
            dialect: "mysql".to_string(),
        });

        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("SELECT"));
        assert!(formatted.contains("id"));
        assert!(formatted.contains("name"));
    }

    #[test]
    fn test_format_sql_postgresql() {
        let executor = create_test_executor();
        let result = executor.format_sql(&SqlFormatRequest {
            sql: "select id,name from users".to_string(),
            dialect: "postgresql".to_string(),
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_sql() {
        let executor = create_test_executor();
        let result = executor.format_sql(&SqlFormatRequest {
            sql: "SELECT * FROM".to_string(),
            dialect: "mysql".to_string(),
        });

        assert!(result.is_err());
    }
}
