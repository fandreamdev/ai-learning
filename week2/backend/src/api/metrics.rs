//! 指标 API 处理器
//!
//! 处理指标管理相关的 HTTP 请求

use crate::error::{AppError, AppResult};
use crate::models::metric::{
    CreateMetricRequest, Metric, MetricListResponse, UpdateMetricRequest,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// 列出指标
#[derive(Debug, Deserialize)]
pub struct MetricListParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub query: Option<String>,
}

pub async fn list_metrics(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<MetricListParams>,
) -> AppResult<Json<MetricListResponse>> {
    // TODO: 实现指标列表
    Ok(Json(MetricListResponse {
        items: vec![],
        total: 0,
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
    }))
}

/// 创建指标
pub async fn create_metric(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<CreateMetricRequest>,
) -> AppResult<Json<Metric>> {
    // TODO: 创建指标
    Err(AppError::not_found("指标"))
}

/// 获取指标
pub async fn get_metric(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Metric>> {
    // TODO: 获取指标
    Err(AppError::not_found("指标"))
}

/// 更新指标
pub async fn update_metric(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateMetricRequest>,
) -> AppResult<Json<Metric>> {
    // TODO: 更新指标
    Err(AppError::not_found("指标"))
}

/// 删除指标
pub async fn delete_metric(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    // TODO: 删除指标
    Err(AppError::not_found("指标"))
}

use serde::Deserialize;
