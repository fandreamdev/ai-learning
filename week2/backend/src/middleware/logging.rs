//! 日志中间件
//!
//! 提供请求日志和追踪功能

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tracing::{info, warn, Span, Level};

/// 请求日志中间件
///
/// 记录请求的详细信息，包括方法、路径、状态码、耗时等
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let request_id = generate_request_id();

    // 添加请求 ID 到请求扩展
    let mut request = request;
    request.extensions_mut().insert(RequestId(request_id.clone()));

    // 记录请求开始
    let span = tracing::info_span!(
        "http_request",
        method = %method,
        path = %path,
        request_id = %request_id
    );

    let response = span.in_scope(|| async move {
        let response = next.run(request).await;
        response
    }.in_current_span());

    let duration = start.elapsed();

    // 记录响应信息
    let status = response.status();
    let status_code = status.as_u16();

    if status.is_success() || status.is_redirection() {
        info!(
            method = %method,
            path = %path,
            status = status_code,
            duration_ms = duration.as_millis() as u64,
            request_id = %request_id,
            "Request completed"
        );
    } else {
        warn!(
            method = %method,
            path = %path,
            status = status_code,
            duration_ms = duration.as_millis() as u64,
            request_id = %request_id,
            "Request failed"
        );
    }

    response
}

/// 请求 ID 生成器
fn generate_request_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()[..8].to_string()
}

/// 请求 ID 类型
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 请求日志数据
#[derive(Debug, Clone)]
pub struct RequestLog {
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub query_string: Option<String>,
    pub headers: HashMap<String, String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub started_at: Instant,
}

impl RequestLog {
    pub fn from_request(request: &Request) -> Self {
        let mut headers = HashMap::new();
        for (key, value) in request.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        let client_ip = headers
            .get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .cloned();

        let user_agent = headers.get("user-agent").cloned();

        Self {
            request_id: generate_request_id(),
            method: request.method().to_string(),
            path: request.uri().path().to_string(),
            query_string: request.uri().query().map(String::from),
            headers,
            client_ip,
            user_agent,
            started_at: Instant::now(),
        }
    }

    /// 计算请求耗时
    pub fn duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// 格式化日志输出
    pub fn format(&self) -> String {
        format!(
            "[{}] {} {} {} {:?}",
            self.request_id, self.method, self.path,
            self.query_string.as_deref().unwrap_or(""),
            self.duration()
        )
    }
}

/// 慢请求警告
///
/// 记录超过指定阈值的慢请求
pub async fn slow_request_logger(
    threshold_ms: u64,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let response = next.run(request).await;
    let duration = start.elapsed();

    if duration.as_millis() as u64 > threshold_ms {
        warn!(
            method = %method,
            path = %path,
            duration_ms = duration.as_millis() as u64,
            threshold_ms = threshold_ms,
            "Slow request detected"
        );
    }

    response
}

/// CORS 预检请求日志
pub async fn cors_preflight_logger(request: Request, next: Next) -> Response {
    if request.method() == axum::http::Method::OPTIONS {
        info!(
            path = request.uri().path(),
            "CORS preflight request"
        );
    }
    next.run(request).await
}

/// 添加追踪头
pub async fn tracing_headers_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|id| id.0.clone())
        .unwrap_or_else(generate_request_id);

    let response = next.run(request).await;

    let (mut parts, body) = response.into_parts();

    // 添加追踪头
    parts.headers.insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id).unwrap_or_default(),
    );

    parts.headers.insert(
        HeaderName::from_static("x-response-time"),
        HeaderValue::from_static(""),
    );

    Response::from_parts(parts, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();

        assert_eq!(id1.len(), 8);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_request_log_format() {
        let log = RequestLog {
            request_id: "12345678".to_string(),
            method: "GET".to_string(),
            path: "/api/v1/users".to_string(),
            query_string: Some("page=1&size=10".to_string()),
            headers: HashMap::new(),
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: Some("Test/1.0".to_string()),
            started_at: Instant::now(),
        };

        let formatted = log.format();
        assert!(formatted.contains("12345678"));
        assert!(formatted.contains("GET"));
        assert!(formatted.contains("/api/v1/users"));
    }
}
