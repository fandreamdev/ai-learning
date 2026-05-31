//! 应用状态模块
//!
//! 管理应用共享状态，包括数据库连接池、Redis 客户端等

use crate::config::AppConfig;
use crate::services::connection_manager::ConnectionManager;
use crate::services::llm_client::LlmClient;
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Token 黑名单
#[derive(Clone, Default)]
pub struct TokenBlacklist {
    /// 存储已撤销的 token (使用内存存储，Redis 作为备份)
    tokens: Arc<RwLock<HashMap<String, i64>>>,
}

impl TokenBlacklist {
    /// 添加 token 到黑名单
    pub async fn add(&self, token: &str, ttl_seconds: u64) {
        if token.is_empty() || ttl_seconds == 0 {
            return;
        }

        let expires_at = Utc::now().timestamp() + ttl_seconds as i64;
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.to_string(), expires_at);
    }

    /// 检查 token 是否在黑名单中
    pub async fn contains(&self, token: &str) -> bool {
        let now = Utc::now().timestamp();
        let mut tokens = self.tokens.write().await;
        tokens.retain(|_, expires_at| *expires_at > now);
        tokens.contains_key(token)
    }

    /// 清理过期的 token (可以定期调用)
    pub async fn cleanup(&self) {
        let now = Utc::now().timestamp();
        let mut tokens = self.tokens.write().await;
        tokens.retain(|_, expires_at| *expires_at > now);
    }
}

/// 应用共享状态
///
/// 包含所有需要在整个应用中共享的资源
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL 连接池 (元数据存储)
    pub db: sqlx::PgPool,

    /// Redis 客户端
    pub redis: redis::aio::MultiplexedConnection,

    /// LLM 客户端
    pub llm_client: Arc<LlmClient>,

    /// 应用配置（只读）
    pub config: Arc<AppConfig>,

    /// 当前用户信息（请求级别）
    pub request_id: Arc<std::sync::atomic::AtomicU64>,

    /// 目标数据库连接管理器
    pub connection_manager: ConnectionManager,

    /// Token 黑名单
    pub token_blacklist: TokenBlacklist,
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

        // 初始化连接管理器
        let connection_manager = ConnectionManager::new();

        // 创建状态
        let state = Self {
            db,
            redis,
            llm_client,
            config: Arc::new(config.clone()),
            request_id: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            connection_manager,
            token_blacklist: TokenBlacklist::default(),
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
