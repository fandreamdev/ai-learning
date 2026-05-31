//! 数据库连接管理器
//!
//! 管理到目标数据库的连接池

use crate::error::{AppError, AppResult};
use crate::models::DatabaseType;
use serde::{Deserialize, Serialize};
use sqlx::{
    mysql::{MySqlPool, MySqlPoolOptions},
    postgres::{PgPool, PgPoolOptions},
    sqlite::{SqlitePool, SqlitePoolOptions},
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// 数据库连接类型
#[derive(Clone)]
pub enum ConnectionPool {
    Postgres(PgPool),
    Mysql(MySqlPool),
    Sqlite(SqlitePool),
    Clickhouse(ConnectionConfig),
}

/// 连接管理器
#[derive(Clone)]
pub struct ConnectionManager {
    /// PostgreSQL 连接池缓存
    pg_pools: Arc<tokio::sync::RwLock<HashMap<Uuid, PgPool>>>,
    /// MySQL 连接池缓存
    mysql_pools: Arc<tokio::sync::RwLock<HashMap<Uuid, MySqlPool>>>,
    sqlite_pools: Arc<tokio::sync::RwLock<HashMap<Uuid, SqlitePool>>>,
    /// 目标数据库连接配置 (不包含密码)
    configs: Arc<tokio::sync::RwLock<HashMap<Uuid, ConnectionConfig>>>,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new() -> Self {
        Self {
            pg_pools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            mysql_pools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            sqlite_pools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            configs: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 获取 PostgreSQL 连接池
    pub async fn get_pg_pool(&self, config: &ConnectionConfig) -> AppResult<PgPool> {
        // 先检查缓存
        {
            let pools = self.pg_pools.read().await;
            if let Some(pool) = pools.get(&config.id) {
                return Ok(pool.clone());
            }
        }

        if config.db_type != DatabaseType::Postgresql {
            return Err(AppError::validation("不是 PostgreSQL 连接".to_string()));
        }
        let url = config.postgres_url()?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&url)
            .await
            .map_err(|e| AppError::database(format!("PostgreSQL 连接失败: {}", e)))?;

        // 缓存连接池
        {
            let mut pools = self.pg_pools.write().await;
            pools.insert(config.id, pool.clone());
        }

        Ok(pool)
    }

    /// 获取 MySQL 连接池
    pub async fn get_mysql_pool(&self, config: &ConnectionConfig) -> AppResult<MySqlPool> {
        // 先检查缓存
        {
            let pools = self.mysql_pools.read().await;
            if let Some(pool) = pools.get(&config.id) {
                return Ok(pool.clone());
            }
        }

        if config.db_type != DatabaseType::Mysql {
            return Err(AppError::validation("不是 MySQL 连接".to_string()));
        }
        let url = config.mysql_url()?;
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&url)
            .await
            .map_err(|e| AppError::database(format!("MySQL 连接失败: {}", e)))?;

        // 缓存连接池
        {
            let mut pools = self.mysql_pools.write().await;
            pools.insert(config.id, pool.clone());
        }

        Ok(pool)
    }

    /// 获取指定类型的连接池
    pub async fn get_pool(&self, config: &ConnectionConfig) -> AppResult<ConnectionPool> {
        match config.db_type {
            DatabaseType::Postgresql => {
                let pool = self.get_pg_pool(config).await?;
                Ok(ConnectionPool::Postgres(pool))
            }
            DatabaseType::Mysql => {
                let pool = self.get_mysql_pool(config).await?;
                Ok(ConnectionPool::Mysql(pool))
            }
            DatabaseType::Sqlite => {
                let pool = self.get_sqlite_pool(config).await?;
                Ok(ConnectionPool::Sqlite(pool))
            }
            DatabaseType::Clickhouse => Ok(ConnectionPool::Clickhouse(config.clone())),
        }
    }

    pub async fn get_sqlite_pool(&self, config: &ConnectionConfig) -> AppResult<SqlitePool> {
        {
            let pools = self.sqlite_pools.read().await;
            if let Some(pool) = pools.get(&config.id) {
                return Ok(pool.clone());
            }
        }

        if config.db_type != DatabaseType::Sqlite {
            return Err(AppError::validation("不是 SQLite 连接".to_string()));
        }
        let url = config.sqlite_url()?;
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&url)
            .await
            .map_err(|e| AppError::database(format!("SQLite 连接失败: {}", e)))?;

        {
            let mut pools = self.sqlite_pools.write().await;
            pools.insert(config.id, pool.clone());
        }

        Ok(pool)
    }

    /// 测试数据库连接 (PostgreSQL)
    pub async fn test_connection_pg(&self, config: &ConnectionConfig) -> AppResult<String> {
        let pool = self.get_pg_pool(config).await?;
        let version: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::database(format!("查询版本失败: {}", e)))?;
        Ok(version.0)
    }

    /// 测试数据库连接 (MySQL)
    pub async fn test_connection_mysql(&self, config: &ConnectionConfig) -> AppResult<String> {
        let pool = self.get_mysql_pool(config).await?;
        let version: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::database(format!("查询版本失败: {}", e)))?;
        Ok(version.0)
    }

    pub async fn test_connection_sqlite(&self, config: &ConnectionConfig) -> AppResult<String> {
        let pool = self.get_sqlite_pool(config).await?;
        let version: (String,) = sqlx::query_as("SELECT sqlite_version()")
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::database(format!("查询版本失败: {}", e)))?;
        Ok(format!("SQLite {}", version.0))
    }

    pub async fn test_connection_clickhouse(&self, config: &ConnectionConfig) -> AppResult<String> {
        let value = self
            .clickhouse_query_json(config, "SELECT version() AS version FORMAT JSON")
            .await?;
        let version = value
            .get("data")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.first())
            .and_then(|row| row.get("version"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::database("ClickHouse version response is invalid".to_string()))?;
        Ok(format!("ClickHouse {}", version))
    }

    /// 获取数据库 Schema 信息
    pub async fn get_schema(&self, config: &ConnectionConfig) -> AppResult<Vec<TableInfo>> {
        match config.db_type {
            DatabaseType::Postgresql => {
                let pool = self.get_pg_pool(config).await?;
                let tables: Vec<TableInfoRow> = sqlx::query_as(
                    r#"
                    SELECT
                        t.table_name,
                        t.table_schema,
                        t.table_type,
                        obj_description((t.table_schema || '.' || t.table_name)::regclass, 'pg_class') as comment
                    FROM information_schema.tables t
                    WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')
                    ORDER BY t.table_name
                    "#,
                )
                .fetch_all(&pool)
                .await?;

                Ok(tables.into_iter().map(|r| TableInfo {
                    table_name: r.table_name,
                    table_schema: r.table_schema,
                    table_type: r.table_type,
                    comment: r.comment,
                    row_count: None,
                    columns: vec![],
                }).collect())
            }
            DatabaseType::Mysql => {
                let pool = self.get_mysql_pool(config).await?;
                let tables: Vec<TableInfoRow> = sqlx::query_as(
                    r#"
                    SELECT
                        t.TABLE_NAME as table_name,
                        t.TABLE_SCHEMA as table_schema,
                        t.TABLE_TYPE as table_type,
                        t.TABLE_COMMENT as comment
                    FROM information_schema.TABLES t
                    WHERE t.TABLE_SCHEMA = ?
                    AND t.TABLE_TYPE IN ('BASE TABLE', 'VIEW')
                    ORDER BY t.TABLE_NAME
                    "#,
                )
                .bind(&config.database_name)
                .fetch_all(&pool)
                .await?;

                Ok(tables.into_iter().map(|r| TableInfo {
                    table_name: r.table_name,
                    table_schema: r.table_schema,
                    table_type: r.table_type,
                    comment: r.comment,
                    row_count: None,
                    columns: vec![],
                }).collect())
            }
            DatabaseType::Sqlite => {
                let pool = self.get_sqlite_pool(config).await?;
                let tables: Vec<TableInfoRow> = sqlx::query_as(
                    r#"
                    SELECT
                        name as table_name,
                        'main' as table_schema,
                        type as table_type,
                        NULL as comment
                    FROM sqlite_master
                    WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'
                    ORDER BY name
                    "#,
                )
                .fetch_all(&pool)
                .await?;

                Ok(tables.into_iter().map(|r| TableInfo {
                    table_name: r.table_name,
                    table_schema: r.table_schema,
                    table_type: r.table_type,
                    comment: r.comment,
                    row_count: None,
                    columns: vec![],
                }).collect())
            }
            DatabaseType::Clickhouse => {
                let sql = format!(
                    "SELECT name AS table_name, database AS table_schema, engine AS table_type, comment FROM system.tables WHERE database = '{}' ORDER BY name FORMAT JSON",
                    escape_clickhouse_string(&config.database_name)
                );
                let value = self.clickhouse_query_json(config, &sql).await?;
                let rows = value
                    .get("data")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| AppError::database("ClickHouse schema response is invalid".to_string()))?;

                Ok(rows
                    .iter()
                    .map(|row| TableInfo {
                        table_name: json_string(row, "table_name"),
                        table_schema: json_string(row, "table_schema"),
                        table_type: json_string(row, "table_type"),
                        comment: json_optional_string(row, "comment"),
                        row_count: None,
                        columns: vec![],
                    })
                    .collect())
            }
        }
    }

    /// 获取表的列信息
    pub async fn get_table_columns(
        &self,
        config: &ConnectionConfig,
        table_schema: &str,
        table_name: &str,
    ) -> AppResult<Vec<ColumnInfo>> {
        match config.db_type {
            DatabaseType::Postgresql => {
                let pool = self.get_pg_pool(config).await?;
                let columns: Vec<ColumnInfoRow> = sqlx::query_as(
                    r#"
                    SELECT
                        c.column_name,
                        c.data_type,
                        c.is_nullable,
                        c.column_default,
                        c.character_maximum_length,
                        c.numeric_precision,
                        c.numeric_scale,
                        col_description((c.table_schema || '.' || c.table_name)::regclass, c.ordinal_position) as comment
                    FROM information_schema.columns c
                    WHERE c.table_name = $1 AND c.table_schema = $2
                    ORDER BY c.ordinal_position
                    "#,
                )
                .bind(table_name)
                .bind(table_schema)
                .fetch_all(&pool)
                .await?;

                Ok(columns.into_iter().map(|r| ColumnInfo {
                    name: r.column_name,
                    data_type: r.data_type,
                    nullable: r.is_nullable == "YES",
                    default_value: r.column_default,
                    comment: r.comment,
                }).collect())
            }
            DatabaseType::Mysql => {
                let pool = self.get_mysql_pool(config).await?;
                let columns: Vec<ColumnInfoRow> = sqlx::query_as(
                    r#"
                    SELECT
                        c.COLUMN_NAME as column_name,
                        c.DATA_TYPE as data_type,
                        c.IS_NULLABLE as is_nullable,
                        c.COLUMN_DEFAULT as column_default,
                        c.CHARACTER_MAXIMUM_LENGTH as character_maximum_length,
                        c.NUMERIC_PRECISION as numeric_precision,
                        c.NUMERIC_SCALE as numeric_scale,
                        c.COLUMN_COMMENT as comment
                    FROM information_schema.COLUMNS c
                    WHERE c.TABLE_NAME = ? AND c.TABLE_SCHEMA = ?
                    ORDER BY c.ORDINAL_POSITION
                    "#,
                )
                .bind(table_name)
                .bind(table_schema)
                .fetch_all(&pool)
                .await?;

                Ok(columns.into_iter().map(|r| ColumnInfo {
                    name: r.column_name,
                    data_type: r.data_type,
                    nullable: r.is_nullable == "YES",
                    default_value: r.column_default,
                    comment: r.comment,
                }).collect())
            }
            DatabaseType::Sqlite => {
                let pool = self.get_sqlite_pool(config).await?;
                let quoted_table = table_name.replace('\'', "''");
                let query = format!("PRAGMA table_info('{}')", quoted_table);
                let columns: Vec<SqliteColumnInfoRow> = sqlx::query_as(&query)
                    .fetch_all(&pool)
                    .await?;

                Ok(columns.into_iter().map(|r| ColumnInfo {
                    name: r.name,
                    data_type: r.data_type,
                    nullable: r.notnull == 0,
                    default_value: r.dflt_value,
                    comment: None,
                }).collect())
            }
            DatabaseType::Clickhouse => {
                let sql = format!(
                    "SELECT name, type AS data_type, default_expression AS default_value, comment FROM system.columns WHERE database = '{}' AND table = '{}' ORDER BY position FORMAT JSON",
                    escape_clickhouse_string(table_schema),
                    escape_clickhouse_string(table_name)
                );
                let value = self.clickhouse_query_json(config, &sql).await?;
                let rows = value
                    .get("data")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| AppError::database("ClickHouse columns response is invalid".to_string()))?;

                Ok(rows
                    .iter()
                    .map(|row| ColumnInfo {
                        name: json_string(row, "name"),
                        data_type: json_string(row, "data_type"),
                        nullable: json_string(row, "data_type").starts_with("Nullable("),
                        default_value: json_optional_string(row, "default_value"),
                        comment: json_optional_string(row, "comment"),
                    })
                    .collect())
            }
        }
    }

    async fn clickhouse_query_json(
        &self,
        config: &ConnectionConfig,
        sql: &str,
    ) -> AppResult<serde_json::Value> {
        let response = reqwest::Client::new()
            .post(config.clickhouse_http_url())
            .basic_auth(&config.username, Some(&config.password))
            .query(&[("database", config.database_name.as_str())])
            .body(sql.to_string())
            .send()
            .await
            .map_err(|e| AppError::database(format!("ClickHouse 请求失败: {}", e)))?;

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

        serde_json::from_str(&body)
            .map_err(|e| AppError::database(format!("解析 ClickHouse JSON 失败: {}", e)))
    }

    /// 缓存连接配置
    pub async fn cache_config(&self, id: Uuid, config: ConnectionConfig) {
        let mut configs = self.configs.write().await;
        configs.insert(id, config);
    }

    /// 获取缓存的配置
    pub async fn get_cached_config(&self, id: Uuid) -> Option<ConnectionConfig> {
        let configs = self.configs.read().await;
        configs.get(&id).cloned()
    }

    /// 移除缓存的配置
    pub async fn remove_cached_config(&self, id: Uuid) {
        let mut configs = self.configs.write().await;
        configs.remove(&id);
    }

    pub async fn remove_pool(&self, id: Uuid) {
        self.pg_pools.write().await.remove(&id);
        self.mysql_pools.write().await.remove(&id);
        self.sqlite_pools.write().await.remove(&id);
        self.remove_cached_config(id).await;
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 连接配置
#[derive(Clone, Debug)]
pub struct ConnectionConfig {
    pub id: Uuid,
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub password: String,
}

impl ConnectionConfig {
    /// 生成 PostgreSQL 连接 URL
    pub fn postgres_url(&self) -> AppResult<String> {
        if self.db_type != DatabaseType::Postgresql {
            return Err(AppError::validation("不是 PostgreSQL 连接".to_string()));
        }
        Ok(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        ))
    }

    /// 生成 MySQL 连接 URL
    pub fn mysql_url(&self) -> AppResult<String> {
        if self.db_type != DatabaseType::Mysql {
            return Err(AppError::validation("不是 MySQL 连接".to_string()));
        }
        Ok(format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        ))
    }

    pub fn sqlite_url(&self) -> AppResult<String> {
        if self.db_type != DatabaseType::Sqlite {
            return Err(AppError::validation("不是 SQLite 连接".to_string()));
        }
        Ok(format!("sqlite://{}", self.database_name))
    }

    pub fn clickhouse_http_url(&self) -> String {
        format!("http://{}:{}/", self.host, self.port)
    }
}

fn escape_clickhouse_string(input: &str) -> String {
    input.replace('\\', "\\\\").replace('\'', "\\'")
}

fn json_string(row: &serde_json::Value, key: &str) -> String {
    row.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn json_optional_string(row: &serde_json::Value, key: &str) -> Option<String> {
    row.get(key)
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
}

/// 表信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub table_schema: String,
    pub table_type: String,
    pub comment: Option<String>,
    pub row_count: Option<i64>,
    pub columns: Vec<ColumnInfo>,
}

/// 列信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
}

/// 表信息行（PostgreSQL）
#[derive(Debug, sqlx::FromRow)]
struct TableInfoRow {
    table_name: String,
    table_schema: String,
    table_type: String,
    comment: Option<String>,
}

/// 列信息行
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct ColumnInfoRow {
    column_name: String,
    data_type: String,
    is_nullable: String,
    column_default: Option<String>,
    character_maximum_length: Option<i64>,
    numeric_precision: Option<i32>,
    numeric_scale: Option<i32>,
    comment: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct SqliteColumnInfoRow {
    #[allow(dead_code)]
    cid: i64,
    name: String,
    #[sqlx(rename = "type")]
    data_type: String,
    notnull: i64,
    dflt_value: Option<String>,
    #[allow(dead_code)]
    pk: i64,
}
