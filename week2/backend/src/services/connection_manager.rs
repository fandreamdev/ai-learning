//! 数据库连接管理器
//!
//! 管理到目标数据库的连接池

use crate::error::{AppError, AppResult};
use crate::models::DatabaseType;
use sqlx::{
    mysql::{MySqlPool, MySqlPoolOptions},
    postgres::{PgPool, PgPoolOptions},
    Pool,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 数据库连接类型
#[derive(Clone)]
pub enum DbPool {
    Postgres(PgPool),
    Mysql(MySqlPool),
}

/// 连接管理器
#[derive(Clone)]
pub struct ConnectionManager {
    /// 连接池缓存
    pools: Arc<RwLock<HashMap<Uuid, DbPool>>>,
    /// 目标数据库连接配置 (不包含密码)
    configs: Arc<RwLock<HashMap<Uuid, ConnectionConfig>>>,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取或创建连接池
    pub async fn get_pool(&self, config: &ConnectionConfig) -> AppResult<Pool<sqlx::any::Any>> {
        match config.db_type {
            DatabaseType::Postgresql => {
                let url = config.postgres_url()?;
                let pool = PgPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(std::time::Duration::from_secs(30))
                    .connect(&url)
                    .await
                    .map_err(|e| AppError::database(format!("PostgreSQL 连接失败: {}", e)))?;
                Ok(pool.into())
            }
            DatabaseType::Mysql => {
                let url = config.mysql_url()?;
                let pool = MySqlPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(std::time::Duration::from_secs(30))
                    .connect(&url)
                    .await
                    .map_err(|e| AppError::database(format!("MySQL 连接失败: {}", e)))?;
                Ok(pool.into())
            }
            _ => Err(AppError::validation(format!(
                "不支持的数据库类型: {:?}",
                config.db_type
            ))),
        }
    }

    /// 测试数据库连接
    pub async fn test_connection(&self, config: &ConnectionConfig) -> AppResult<ConnectionTestResult> {
        let pool = self.get_pool(config).await;

        match pool {
            Ok(pool) => {
                // 执行简单查询测试连接
                let result: Result<(i64,), _> = sqlx::query_as("SELECT 1")
                    .fetch_one(&pool)
                    .await;

                match result {
                    Ok(_) => Ok(ConnectionTestResult {
                        success: true,
                        message: "连接成功".to_string(),
                        server_version: None,
                        latency_ms: 0,
                    }),
                    Err(e) => Ok(ConnectionTestResult {
                        success: false,
                        message: format!("查询失败: {}", e),
                        server_version: None,
                        latency_ms: 0,
                    }),
                }
            }
            Err(e) => Ok(ConnectionTestResult {
                success: false,
                message: format!("连接失败: {}", e),
                server_version: None,
                latency_ms: 0,
            }),
        }
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
}

/// 连接测试结果
#[derive(Debug, serde::Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub server_version: Option<String>,
    pub latency_ms: u64,
}
