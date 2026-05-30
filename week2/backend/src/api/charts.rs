//! 图表 API 处理器
//!
//! 处理图表相关的 HTTP 请求

use crate::error::AppResult;
use crate::models::query::ColumnMetadata;
use crate::services::chart_generator::{ChartConfig, ChartGenerator, ChartRecommendation};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 推荐图表
#[derive(Debug, Deserialize)]
pub struct ChartRecommendRequest {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

pub async fn recommend_chart(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ChartRecommendRequest>,
) -> AppResult<Json<ChartRecommendation>> {
    let generator = ChartGenerator::new();
    let recommendation = generator.recommend(&request.columns, &request.rows)?;
    Ok(Json(recommendation))
}

/// 生成图表配置
#[derive(Debug, Deserialize)]
pub struct ChartGenerateRequest {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub chart_type: String,
}

pub async fn generate_chart(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ChartGenerateRequest>,
) -> AppResult<Json<ChartConfig>> {
    let generator = ChartGenerator::new();
    let config = generator.switch_chart_type(
        &request.columns,
        &request.rows,
        &request.chart_type,
    )?;
    Ok(Json(config))
}
