//! SmartQuery AI - 程序入口
//!
//! 负责：
//! 1. 初始化日志
//! 2. 加载配置
//! 3. 创建应用状态
//! 4. 启动 HTTP 服务器

use smartquery_backend::{config::AppConfig, state::AppState};
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
    let app = build_app(state.clone(), &config);

    // ==================== 启动服务器 ====================
    let addr: SocketAddr = format!("{}:{}", config.app.host, config.app.port)
        .parse()
        .expect("Invalid address");

    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// 构建 Axum 应用
fn build_app(state: AppState, config: &AppConfig) -> axum::Router {
    use smartquery_backend::api::routes;

    let mut app = axum::Router::new()
        .nest("/api/v1", routes())
        .with_state(state)
        .layer(
            TraceLayer::newForHttp()
                .make_span_with(tower_http::trace::DefaultMakeSpan)
                .on_response(tower_http::trace::DefaultOnResponse),
        )
        .layer(CompressionLayer::new());

    // 添加 CORS 中间件
    if config.app.env == "development" {
        use axum::http::Method;
        use tower_http::cors::{Any, CorsLayer};

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        app = app.layer(cors);
    }

    app
}

/// 初始化追踪日志系统
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true))
        .init();
}

/// 优雅关闭信号处理
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, initiating graceful shutdown...");
}
