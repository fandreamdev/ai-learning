//! 应用状态模块
//!
//! 管理应用共享状态，包括数据库连接池、Redis 客户端等

use crate::config::AppConfig;
use crate::services::llm_client::LlmClient;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

/// 应用共享状态
///
/// 包含所有需要在整个应用中共享的资源
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL 连接池
    pub db: sqlx::PgPool,

    /// Redis 客户端
    pub redis: redis::aio::MultiplexedConnection,

    /// LLM 客户端
    pub llm_client: Arc<LlmClient>,

    /// 应用配置（只读）
    pub config: Arc<AppConfig>,

    /// 当前用户信息（请求级别）
    pub request_id: Arc<std::sync::atomic::AtomicU64>,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new(config: &AppConfig) -> anyhow::Result<Self> {
        tracing::info!("Initializing application state...");

        // 初始化数据库连接池
        let db = Self::create_db_pool(config).await?;

        // 初始化 Redis 连接
        let redis = Self::create_redis_client(config).await?;

        // 初始化 LLM 客户端
        let llm_client = Arc::new(LlmClient::new(config)?);

        // 创建状态
        let state = Self {
            db,
            redis,
            llm_client,
            config: Arc::new(config.clone()),
            request_id: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        };

        tracing::info!("Application state initialized successfully");

        Ok(state)
    }

    /// 创建数据库连接池
    async fn create_db_pool(config: &AppConfig) -> anyhow::Result<sqlx::PgPool> {
        tracing::info!("Creating database connection pool...");

        let pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.connect_timeout,
            ))
            .idle_timeout(std::time::Duration::from_secs(
                config.database.idle_timeout,
            ))
            .connect(&config.database.url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        tracing::info!("Database connection pool created with {} max connections",
            config.database.max_connections);

        Ok(pool)
    }

    /// 创建 Redis 客户端
    async fn create_redis_client(config: &AppConfig) -> anyhow::Result<redis::aio::MultiplexedConnection> {
        tracing::info!("Creating Redis connection...");

        let client = redis::Client::open(config.redis.url.as_str())
            .map_err(|e| anyhow::anyhow!("Failed to create Redis client: {}", e))?;

        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Redis: {}", e))?;

        tracing::info!("Redis connection established");

        Ok(connection)
    }

    /// 生成唯一的请求 ID
    pub fn next_request_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

/// 请求级别的状态提取器
pub mod axum_ext {
    use super::*;
    use axum::{
        extract::State,
        http::request::Parts,
        routing::MethodRouter,
    };
    use std::convert::Infallible;

    impl<S> tower_service::Service<S> for AppState
    where
        S: Clone,
    {
        type Response = AppState;
        type Error = Infallible;
        type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(
            &mut self,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, _state: S) -> Self::Future {
            std::future::ready(Ok(self.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_generation() {
        let state = AppState {
            db: sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect("postgres://localhost/test")
                .await
                .unwrap(),
            redis: futures::executor::block_on(async {
                let client = redis::Client::open("redis://localhost").unwrap();
                client.get_multiplexed_async_connection().await.unwrap()
            }),
            llm_client: Arc::new(LlmClient::new(&crate::config::AppConfig::load_from_file("config.yaml").unwrap()).unwrap()),
            config: Arc::new(crate::config::AppConfig::load_from_file("config.yaml").unwrap()),
            request_id: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        };

        let id1 = state.next_request_id();
        let id2 = state.next_request_id();
        let id3 = state.next_request_id();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }
}
