//! 对话仓储
//!
//! 对话和消息的数据库操作

use crate::error::AppResult;
use crate::models::{Conversation, Message, MessageRole};
use sqlx::PgPool;
use uuid::Uuid;

/// 对话仓储
#[derive(Clone)]
pub struct ConversationRepo {
    pool: PgPool,
}

impl ConversationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建对话
    pub async fn create_conversation(&self, conv: &Conversation) -> AppResult<Conversation> {
        let row = sqlx::query_as::<_, ConversationRow>(
            r#"
            INSERT INTO conversations (id, user_id, title, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, title, created_at, updated_at
            "#,
        )
        .bind(conv.id)
        .bind(conv.user_id)
        .bind(&conv.title)
        .bind(conv.created_at)
        .bind(conv.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// 获取对话
    pub async fn get_conversation(&self, id: Uuid) -> AppResult<Option<Conversation>> {
        let row = sqlx::query_as::<_, ConversationRow>(
            r#"
            SELECT id, user_id, title, created_at, updated_at
            FROM conversations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    /// 列出用户的对话
    pub async fn list_by_user(&self, user_id: Uuid) -> AppResult<Vec<Conversation>> {
        let rows = sqlx::query_as::<_, ConversationRow>(
            r#"
            SELECT id, user_id, title, created_at, updated_at
            FROM conversations
            WHERE user_id = $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// 更新对话标题
    pub async fn update_title(&self, id: Uuid, title: &str) -> AppResult<()> {
        sqlx::query("UPDATE conversations SET title = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(title)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 删除对话
    pub async fn delete_conversation(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM conversations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 创建消息
    pub async fn create_message(&self, msg: &Message) -> AppResult<Message> {
        let row = sqlx::query_as::<_, MessageRow>(
            r#"
            INSERT INTO messages
                (id, conversation_id, role, content, generated_sql, sql_explanation,
                 execution_result, chart_config, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, conversation_id, role, content, generated_sql, sql_explanation,
                      execution_result, chart_config, created_at
            "#,
        )
        .bind(msg.id)
        .bind(msg.conversation_id)
        .bind(msg.role.as_str())
        .bind(&msg.content)
        .bind(&msg.generated_sql)
        .bind(&msg.sql_explanation)
        .bind(&msg.execution_result)
        .bind(&msg.chart_config)
        .bind(msg.created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// 获取对话的消息列表
    pub async fn list_messages(&self, conversation_id: Uuid) -> AppResult<Vec<Message>> {
        let rows = sqlx::query_as::<_, MessageRow>(
            r#"
            SELECT id, conversation_id, role, content, generated_sql, sql_explanation,
                   execution_result, chart_config, created_at
            FROM messages
            WHERE conversation_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

/// 对话行
#[derive(Debug, sqlx::FromRow)]
struct ConversationRow {
    id: Uuid,
    user_id: Uuid,
    title: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ConversationRow> for Conversation {
    fn from(row: ConversationRow) -> Self {
        Conversation {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// 消息行
#[derive(Debug, sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    conversation_id: Uuid,
    role: String,
    content: String,
    generated_sql: Option<String>,
    sql_explanation: Option<String>,
    execution_result: Option<serde_json::Value>,
    chart_config: Option<serde_json::Value>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<MessageRow> for Message {
    fn from(row: MessageRow) -> Self {
        Message {
            id: row.id,
            conversation_id: row.conversation_id,
            role: match row.role.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                _ => MessageRole::User,
            },
            content: row.content,
            generated_sql: row.generated_sql,
            sql_explanation: row.sql_explanation,
            execution_result: row.execution_result,
            chart_config: row.chart_config,
            created_at: row.created_at,
        }
    }
}
