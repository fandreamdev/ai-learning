//! 自然语言转 SQL API 处理器
//!
//! 处理自然语言查询和对话相关的 HTTP 请求

use crate::error::{AppError, AppResult};
use crate::models::{
    Conversation, ConversationItem, ConversationListResponse, Message, MessageListResponse,
    NlExecuteRequest, NlExecuteResponse, NlToSqlRequest, NlToSqlResponse,
};
use crate::repositories::ConversationRepo;
use crate::services::{ChartGenerator, LlmClient};
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
    let llm_client = &state.llm_client;

    // 获取 Schema 信息用于上下文
    let schema_context = if let Some(connection_id) = request.connection_id {
        let conn_config = state.connection_manager.get_cached_config(connection_id).await;
        if let Some(config) = conn_config {
            // 获取表结构信息
            match state.connection_manager.get_schema(&config).await {
                Ok(tables) => {
                    let table_names: Vec<String> = tables.iter().map(|t| t.table_name.clone()).collect();
                    Some(table_names.join(", "))
                }
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    let result = llm_client
        .convert_nl_to_sql(&request.question, schema_context.as_deref())
        .await?;

    Ok(Json(NlToSqlResponse {
        sql: result.sql,
        explanation: result.explanation,
        confidence: result.confidence,
        estimated_rows: result.estimated_rows,
        referenced_tables: result.referenced_tables,
    }))
}

/// NL 执行 SQL
pub async fn nl_execute(
    State(state): State<Arc<AppState>>,
    Json(request): Json<NlExecuteRequest>,
) -> AppResult<Json<NlExecuteResponse>> {
    let llm_client = &state.llm_client;
    let chart_generator = ChartGenerator::new();

    // 获取 Schema 上下文
    let schema_context = if let Some(connection_id) = request.connection_id {
        let conn_config = state.connection_manager.get_cached_config(connection_id).await;
        if let Some(config) = conn_config {
            match state.connection_manager.get_schema(&config).await {
                Ok(tables) => {
                    let table_names: Vec<String> = tables.iter().map(|t| t.table_name.clone()).collect();
                    Some(table_names.join(", "))
                }
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    // 调用 LLM 转换并执行
    let result = llm_client
        .convert_nl_to_sql(&request.question, schema_context.as_deref())
        .await?;

    let columns = vec![];
    let rows = vec![];
    let row_count = 0i64;
    let chart_config = chart_generator.suggest_chart(&columns, &rows).ok();

    Ok(Json(NlExecuteResponse {
        columns,
        rows,
        row_count,
        duration_ms: 0,
        chart_config,
        data_insight: Some(result.explanation),
    }))
}

/// 列出对话
pub async fn list_conversations(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConversationListParams>,
) -> AppResult<Json<ConversationListResponse>> {
    let repo = ConversationRepo::new(state.db.clone());

    // 从请求头获取用户 ID (简化处理)
    let conversations = repo.list_by_user(Uuid::nil()).await?;

    let items: Vec<ConversationItem> = conversations
        .into_iter()
        .map(|c| ConversationItem {
            id: c.id,
            title: c.title,
            message_count: 0, // 需要单独查询
            last_message_at: Some(c.updated_at),
            created_at: c.created_at,
        })
        .collect();

    Ok(Json(ConversationListResponse {
        items,
        total: items.len() as i64,
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
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateConversationRequest>,
) -> AppResult<Json<ConversationItem>> {
    let repo = ConversationRepo::new(state.db.clone());

    let title = request.title.unwrap_or_else(|| "新对话".to_string());
    let conv = Conversation::new(Uuid::nil(), title.clone());
    let conv = repo.create_conversation(&conv).await?;

    Ok(Json(ConversationItem {
        id: conv.id,
        title: conv.title,
        message_count: 0,
        last_message_at: Some(conv.updated_at),
        created_at: conv.created_at,
    }))
}

/// 获取对话
pub async fn get_conversation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ConversationItem>> {
    let repo = ConversationRepo::new(state.db.clone());

    let conv = repo
        .get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    let messages = repo.list_messages(id).await?;

    Ok(Json(ConversationItem {
        id: conv.id,
        title: conv.title,
        message_count: messages.len() as i64,
        last_message_at: messages.last().map(|m| m.created_at),
        created_at: conv.created_at,
    }))
}

/// 更新对话
#[derive(Debug, serde::Deserialize)]
pub struct UpdateConversationRequest {
    pub title: String,
}

pub async fn update_conversation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateConversationRequest>,
) -> AppResult<Json<ConversationItem>> {
    let repo = ConversationRepo::new(state.db.clone());

    repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    repo.update_title(id, &request.title).await?;

    let conv = repo.get_conversation(id).await?.unwrap();

    Ok(Json(ConversationItem {
        id: conv.id,
        title: conv.title,
        message_count: 0,
        last_message_at: Some(conv.updated_at),
        created_at: conv.created_at,
    }))
}

/// 删除对话
pub async fn delete_conversation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let repo = ConversationRepo::new(state.db.clone());

    repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    repo.delete_conversation(id).await?;

    Ok(Json(()))
}

/// 获取消息列表
pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MessageListResponse>> {
    let repo = ConversationRepo::new(state.db.clone());

    // 检查对话是否存在
    repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    let messages = repo.list_messages(id).await?;

    Ok(Json(MessageListResponse {
        items: messages,
        total: messages.len() as i64,
    }))
}

/// 发送消息
#[derive(Debug, serde::Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<SendMessageRequest>,
) -> AppResult<Json<Message>> {
    let repo = ConversationRepo::new(state.db.clone());
    let llm_client = &state.llm_client;

    // 检查对话是否存在
    repo.get_conversation(id)
        .await?
        .ok_or_else(|| AppError::NotFound("对话不存在".to_string()))?;

    // 创建用户消息
    let user_msg = Message::user_message(id, request.content.clone());
    let user_msg = repo.create_message(&user_msg).await?;

    // 获取 Schema 上下文
    let schema_context = None; // 可以从请求中获取 connection_id

    // 调用 LLM 生成回复
    let llm_result = llm_client.convert_nl_to_sql(&request.content, schema_context).await;

    let (assistant_content, generated_sql, sql_explanation) = match llm_result {
        Ok(result) => (
            result.explanation,
            Some(result.sql),
            Some(format!("置信度: {:.0}%", result.confidence * 100.0)),
        ),
        Err(_) => (
            "抱歉，我无法理解您的问题。请尝试重新描述。".to_string(),
            None,
            None,
        ),
    };

    // 创建助手消息
    let assistant_msg = Message::assistant_message(id, assistant_content, generated_sql, sql_explanation);
    let assistant_msg = repo.create_message(&assistant_msg).await?;

    // 更新对话标题（如果是第一条消息）
    let messages = repo.list_messages(id).await?;
    if messages.len() == 2 {
        // 只有刚创建的 user 和 assistant 消息
        let new_title = if request.content.len() > 20 {
            format!("{}...", &request.content[..20])
        } else {
            request.content
        };
        repo.update_title(id, &new_title).await?;
    }

    Ok(Json(assistant_msg))
}
