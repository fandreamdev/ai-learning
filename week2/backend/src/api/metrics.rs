//! 指标 API 处理器
//!
//! 处理指标管理相关的 HTTP 请求

use crate::error::{AppError, AppResult};
use crate::models::metric::{
    CreateMetricRequest, Metric, MetricListResponse, UpdateMetricRequest,
};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use std::sync::RwLock;

/// 指标存储（内存中，简化实现）
/// 生产环境应该使用数据库存储
lazy_static::lazy_static! {
    static ref METRICS_STORE: RwLock<HashMap<Uuid, Metric>> = RwLock::new(HashMap::new());
}

#[derive(Debug, serde::Deserialize)]
pub struct MetricListParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub query: Option<String>,
}

/// 列出指标
pub async fn list_metrics(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<MetricListParams>,
) -> AppResult<Json<MetricListResponse>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    let store = METRICS_STORE.read().map_err(|_| {
        AppError::Internal("无法读取指标存储".to_string())
    })?;

    let mut items: Vec<Metric> = store.values().cloned().collect();

    // 过滤
    if let Some(query) = &params.query {
        let query_lower = query.to_lowercase();
        items.retain(|m| {
            m.name.to_lowercase().contains(&query_lower)
                || m.code.to_lowercase().contains(&query_lower)
        });
    }

    // 分页
    let total = items.len() as i64;
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(items.len());
    items = items.into_iter().skip(start).take(page_size as usize).collect();

    Ok(Json(MetricListResponse {
        items,
        total,
        page,
        page_size,
    }))
}

/// 创建指标
pub async fn create_metric(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<CreateMetricRequest>,
) -> AppResult<Json<Metric>> {
    let mut metric = Metric::new(
        request.name,
        request.code,
        request.expression,
        None,
    );

    if let Some(desc) = request.description {
        metric = metric.with_description(desc);
    }
    if let Some(unit) = request.unit {
        metric = metric.with_unit(unit);
    }
    metric = metric.with_format_type(request.format_type);
    if let Some(dims) = request.dimensions {
        metric = metric.with_dimensions(dims);
    }

    let store = METRICS_STORE.write().map_err(|_| {
        AppError::Internal("无法写入指标存储".to_string())
    })?;

    store.insert(metric.id, metric.clone());

    Ok(Json(metric))
}

/// 获取指标
pub async fn get_metric(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Metric>> {
    let store = METRICS_STORE.read().map_err(|_| {
        AppError::Internal("无法读取指标存储".to_string())
    })?;

    store.get(&id)
        .cloned()
        .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))
        .map(Json)
}

/// 更新指标
pub async fn update_metric(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateMetricRequest>,
) -> AppResult<Json<Metric>> {
    let mut store = METRICS_STORE.write().map_err(|_| {
        AppError::Internal("无法写入指标存储".to_string())
    })?;

    let metric = store.get_mut(&id)
        .ok_or_else(|| AppError::NotFound("指标不存在".to_string()))?;

    if let Some(name) = request.name {
        metric.name = name;
    }
    if let Some(code) = request.code {
        metric.code = code;
    }
    if let Some(expression) = request.expression {
        metric.expression = expression;
    }
    if let Some(description) = request.description {
        metric.description = Some(description);
    }
    if let Some(unit) = request.unit {
        metric.unit = Some(unit);
    }
    if let Some(format_type) = request.format_type {
        metric.format_type = format_type;
    }
    if let Some(dimensions) = request.dimensions {
        metric.dimensions = dimensions;
    }

    metric.updated_at = chrono::Utc::now();

    Ok(Json(metric.clone()))
}

/// 删除指标
pub async fn delete_metric(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let mut store = METRICS_STORE.write().map_err(|_| {
        AppError::Internal("无法写入指标存储".to_string())
    })?;

    if store.remove(&id).is_none() {
        return Err(AppError::NotFound("指标不存在".to_string()));
    }

    Ok(Json(()))
}
