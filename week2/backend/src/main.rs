//! SmartQuery AI - 程序入口
//!
//! 负责：
//! 1. 初始化日志
//! 2. 加载配置
//! 3. 创建应用状态
//! 4. 启动 HTTP 服务器

use smartquery_backend::{config::AppConfig, state::AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ==================== 初始化日志 ====================
    init_tracing();

    info!("Starting SmartQuery AI Backend v{}", env!("CARGO_PKG_VERSION"));

    // ==================== 加载配置 ====================
    let config = AppConfig::load()?;
    info!("Configuration loaded successfully");
    info!("Running in {} mode", config.app.env);

    // ==================== 创建应用状态 ====================
    let state = AppState::new(&config).await?;
    info!("Application state initialized");

    // ==================== 构建路由 ====================
    let app = build_app(state, &config);

    // ==================== 启动服务器 ====================
    let addr: SocketAddr = format!("{}:{}", config.app.host, config.app.port)
        .parse()
        .expect("Invalid address");

    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// 构建 Axum 应用
fn build_app(state: AppState, config: &AppConfig) -> axum::Router {
    use smartquery_backend::api::routes;

    let shared_state = Arc::new(state);
    let app = axum::Router::new()
        .nest("/api/v1", routes(Arc::clone(&shared_state)))
        .with_state(shared_state)
        .layer(CompressionLayer::new());

    // 添加 CORS 中间件
    if config.app.env == "development" {
        use tower_http::cors::{Any, CorsLayer};

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        app.layer(cors)
    } else {
        app
    }
}

/// 初始化追踪日志系统
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}
