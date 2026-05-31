//! 对话模型
//!
//! 定义自然语言对话相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// 用户
    User,
    /// AI 助手
    Assistant,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 对话会话
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    /// 会话 ID
    pub id: Uuid,

    /// 用户 ID
    pub user_id: Uuid,

    /// 会话标题
    pub title: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Conversation {
    /// 创建新对话
    pub fn new(user_id: Uuid, title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            title,
            created_at: now,
            updated_at: now,
        }
    }

    /// 生成默认标题
    pub fn default_title() -> String {
        format!("新对话 {}", Utc::now().format("%Y-%m-%d %H:%M"))
    }
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    /// 消息 ID
    pub id: Uuid,

    /// 关联的会话 ID
    pub conversation_id: Uuid,

    /// 消息角色
    pub role: MessageRole,

    /// 消息内容 (用户问题/AI回答)
    pub content: String,

    /// 生成的 SQL (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_sql: Option<String>,

    /// SQL 解释 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_explanation: Option<String>,

    /// 执行结果 (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_result: Option<serde_json::Value>,

    /// 图表配置 (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_config: Option<serde_json::Value>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Message {
    /// 创建用户消息
    pub fn user_message(conversation_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            role: MessageRole::User,
            content,
            generated_sql: None,
            sql_explanation: None,
            execution_result: None,
            chart_config: None,
            created_at: Utc::now(),
        }
    }

    /// 创建 AI 消息
    pub fn assistant_message(
        conversation_id: Uuid,
        content: String,
        generated_sql: Option<String>,
        sql_explanation: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            role: MessageRole::Assistant,
            content,
            generated_sql,
            sql_explanation,
            execution_result: None,
            chart_config: None,
            created_at: Utc::now(),
        }
    }

    /// 添加执行结果
    pub fn with_execution_result(mut self, result: serde_json::Value) -> Self {
        self.execution_result = Some(result);
        self
    }

    /// 添加图表配置
    pub fn with_chart_config(mut self, config: serde_json::Value) -> Self {
        self.chart_config = Some(config);
        self
    }
}

/// 创建会话请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateConversationRequest {
    #[validate(length(min = 1, max = 200, message = "会话标题长度必须在 1-200 个字符之间"))]
    #[serde(default = "default_conversation_title")]
    pub title: String,
}

fn default_conversation_title() -> String {
    Conversation::default_title()
}

/// 更新会话请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateConversationRequest {
    #[validate(length(min = 1, max = 200, message = "会话标题长度必须在 1-200 个字符之间"))]
    pub title: String,
}

/// 发送消息请求
#[derive(Debug, Deserialize, Validate)]
pub struct SendMessageRequest {
    #[validate(length(min = 1, message = "消息内容不能为空"))]
    pub content: String,

    #[serde(default)]
    pub connection_id: Option<Uuid>,

    #[serde(default)]
    pub dialect: Option<String>,
}

/// 自然语言转 SQL 请求
#[derive(Debug, Deserialize, Validate)]
pub struct NlToSqlRequest {
    /// 连接 ID
    pub connection_id: Uuid,

    /// 用户问题
    #[validate(length(min = 1, max = 2000, message = "问题长度必须在 1-2000 个字符之间"))]
    pub question: String,

    /// 会话 ID (用于上下文)
    #[serde(default)]
    pub conversation_id: Option<Uuid>,

    /// 数据库方言
    #[serde(default = "default_dialect")]
    pub dialect: String,
}

fn default_dialect() -> String {
    "mysql".to_string()
}

/// 自然语言转 SQL 响应
#[derive(Debug, Serialize)]
pub struct NlToSqlResponse {
    /// 生成的 SQL
    pub sql: String,

    /// SQL 解释
    pub explanation: String,

    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,

    /// 预估返回行数
    pub estimated_rows: Option<i32>,

    /// 涉及的表
    pub referenced_tables: Vec<String>,
}

/// NL 执行 SQL 请求
#[derive(Debug, Deserialize)]
pub struct NlExecuteRequest {
    /// 连接 ID
    pub connection_id: Uuid,

    /// 要执行的 SQL
    pub sql: String,

    /// 图表类型 (可选)
    #[serde(default)]
    pub chart_type: Option<String>,

    /// 超时时间 (秒)
    #[serde(default)]
    pub timeout: Option<u64>,
}

/// NL 执行 SQL 响应
#[derive(Debug, Serialize)]
pub struct NlExecuteResponse {
    /// 列信息
    pub columns: Vec<crate::models::ColumnMetadata>,

    /// 行数据
    pub rows: Vec<Vec<serde_json::Value>>,

    /// 返回行数
    pub row_count: i64,

    /// 执行耗时 (毫秒)
    pub duration_ms: i64,

    /// 图表配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_config: Option<serde_json::Value>,

    /// 数据洞察
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_insight: Option<String>,
}

/// 会话列表响应
#[derive(Debug, Serialize)]
pub struct ConversationListResponse {
    pub items: Vec<ConversationItem>,
    pub total: i64,
}

/// 会话列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItem {
    pub id: Uuid,
    pub title: String,
    pub message_count: i32,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 消息列表响应
#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub items: Vec<Message>,
    pub total: i64,
}

/// 对话上下文
#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub messages: Vec<Message>,
    pub schema_info: Option<crate::models::SchemaInfo>,
}

impl ConversationContext {
    /// 构建用于 LLM 的上下文文本
    pub fn build_context_text(&self, max_messages: usize) -> String {
        let recent_messages = if self.messages.len() > max_messages {
            &self.messages[self.messages.len() - max_messages..]
        } else {
            &self.messages
        };

        let mut context = String::new();

        for msg in recent_messages {
            match msg.role {
                MessageRole::User => {
                    context.push_str(&format!("用户: {}\n", msg.content));
                }
                MessageRole::Assistant => {
                    if let Some(sql) = &msg.generated_sql {
                        context.push_str(&format!("AI: SQL: {}\n解释: {}\n",
                            sql, msg.sql_explanation.as_deref().unwrap_or("")));
                    } else {
                        context.push_str(&format!("AI: {}\n", msg.content));
                    }
                }
            }
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_creation() {
        let user_id = Uuid::new_v4();
        let conv = Conversation::new(user_id, "测试对话".to_string());

        assert_eq!(conv.user_id, user_id);
        assert_eq!(conv.title, "测试对话");
    }

    #[test]
    fn test_message_creation() {
        let conv_id = Uuid::new_v4();

        let user_msg = Message::user_message(conv_id, "帮我查一下用户数".to_string());
        assert_eq!(user_msg.role, MessageRole::User);
        assert!(user_msg.generated_sql.is_none());

        let assistant_msg = Message::assistant_message(
            conv_id,
            "已为您生成查询".to_string(),
            Some("SELECT COUNT(*) FROM users".to_string()),
            Some("统计用户总数".to_string()),
        );
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert!(assistant_msg.generated_sql.is_some());
    }

    #[test]
    fn test_conversation_context() {
        let conv_id = Uuid::new_v4();

        let messages = vec![
            Message::user_message(conv_id, "查一下用户".to_string()),
            Message::assistant_message(
                conv_id,
                "已生成".to_string(),
                Some("SELECT * FROM users".to_string()),
                None,
            ),
        ];

        let context = ConversationContext {
            conversation_id: conv_id,
            user_id: Uuid::new_v4(),
            messages,
            schema_info: None,
        };

        let text = context.build_context_text(10);
        assert!(text.contains("查一下用户"));
        assert!(text.contains("SELECT * FROM users"));
    }
}
