//! 自然语言转 SQL API 处理器
//!
//! 处理自然语言查询相关的 HTTP 请求

use crate::error::{AppError, AppResult};
use crate::models::{
    ConversationItem, ConversationListResponse, Message, MessageListResponse,
    NlExecuteRequest, NlExecuteResponse, NlToSqlRequest, NlToSqlResponse,
};
use crate::services::ChartGenerator;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// NL 转 SQL
pub async fn nl_to_sql(
    State(state): State<Arc<AppState>>,
    Json(request): Json<NlToSqlRequest>,
) -> AppResult<Json<NlToSqlResponse>> {
    // TODO: 调用 LLM 生成 SQL
    Ok(Json(NlToSqlResponse {
        sql: "SELECT 1".to_string(),
        explanation: "示例解释".to_string(),
        confidence: 0.9,
        estimated_rows: Some(10),
        referenced_tables: vec![],
    }))
}

/// NL 执行 SQL
pub async fn nl_execute(
    State(state): State<Arc<AppState>>,
    Json(request): Json<NlExecuteRequest>,
) -> AppResult<Json<NlExecuteResponse>> {
    // TODO: 执行生成的 SQL
    let chart_generator = ChartGenerator::new();

    Ok(Json(NlExecuteResponse {
        columns: vec![],
        rows: vec![],
        row_count: 0,
        duration_ms: 0,
        chart_config: None,
        data_insight: None,
    }))
}

/// 列出对话
pub async fn list_conversations(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<ConversationListParams>,
) -> AppResult<Json<ConversationListResponse>> {
    // TODO: 实现对话列表
    Ok(Json(ConversationListResponse {
        items: vec![],
        total: 0,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct ConversationListParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 创建对话
#[derive(Debug, serde::Deserialize)]
pub struct CreateConversationRequest {
    pub title: Option<String>,
}

pub async fn create_conversation(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<CreateConversationRequest>,
) -> AppResult<Json<ConversationItem>> {
    // TODO: 创建对话
    Ok(Json(ConversationItem {
        id: Uuid::new_v4(),
        title: request.title.unwrap_or_else(|| "新对话".to_string()),
        message_count: 0,
        last_message_at: None,
        created_at: chrono::Utc::now(),
    }))
}

/// 获取对话
pub async fn get_conversation(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ConversationItem>> {
    // TODO: 获取对话
    Err(AppError::not_found("对话"))
}

/// 更新对话
#[derive(Debug, serde::Deserialize)]
pub struct UpdateConversationRequest {
    pub title: String,
}

pub async fn update_conversation(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateConversationRequest>,
) -> AppResult<Json<ConversationItem>> {
    // TODO: 更新对话
    Ok(Json(ConversationItem {
        id,
        title: request.title,
        message_count: 0,
        last_message_at: None,
        created_at: chrono::Utc::now(),
    }))
}

/// 删除对话
pub async fn delete_conversation(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    // TODO: 删除对话
    Ok(Json(()))
}

/// 获取消息列表
pub async fn list_messages(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MessageListResponse>> {
    // TODO: 获取消息列表
    Ok(Json(MessageListResponse {
        items: vec![],
        total: 0,
    }))
}

/// 发送消息
#[derive(Debug, serde::Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

pub async fn send_message(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<SendMessageRequest>,
) -> AppResult<Json<Message>> {
    // TODO: 发送消息
    Ok(Json(Message::user_message(id, request.content)))
}
