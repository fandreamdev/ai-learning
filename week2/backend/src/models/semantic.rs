//! 语义层模型
//!
//! 定义语义定义和元数据相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 语义定义
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SemanticDefinition {
    /// 定义 ID
    pub id: Uuid,

    /// 关联的连接 ID
    pub connection_id: Uuid,

    /// 表名
    pub table_name: String,

    /// 列名 (可为空，表示表级定义)
    pub column_name: Option<String>,

    /// 业务名称
    pub business_name: String,

    /// 业务描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_description: Option<String>,

    /// 同义词数组 (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synonyms: Option<serde_json::Value>,

    /// 是否启用
    pub is_active: bool,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl SemanticDefinition {
    /// 创建新的语义定义
    pub fn new(
        connection_id: Uuid,
        table_name: String,
        column_name: Option<String>,
        business_name: String,
        business_description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            connection_id,
            table_name,
            column_name,
            business_name,
            business_description,
            synonyms: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// 获取完整名称 (表.列 或 表)
    pub fn full_name(&self) -> String {
        match &self.column_name {
            Some(col) => format!("{}.{}", self.table_name, col),
            None => self.table_name.clone(),
        }
    }
}

/// 创建语义定义请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSemanticRequest {
    #[serde(default)]
    pub connection_id: Uuid,

    #[validate(length(min = 1, max = 100, message = "表名不能为空"))]
    pub table_name: String,

    #[validate(length(max = 100, message = "列名长度不能超过 100"))]
    pub column_name: Option<String>,

    #[validate(length(min = 1, max = 100, message = "业务名称不能为空"))]
    pub business_name: String,

    #[validate(length(max = 500, message = "业务描述长度不能超过 500"))]
    pub business_description: Option<String>,

    /// 同义词数组
    #[serde(default)]
    pub synonyms: Option<Vec<String>>,
}

/// 更新语义定义请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSemanticRequest {
    #[validate(length(min = 1, max = 100, message = "业务名称不能为空"))]
    pub business_name: Option<String>,

    #[validate(length(max = 500, message = "业务描述长度不能超过 500"))]
    pub business_description: Option<String>,

    /// 同义词数组
    pub synonyms: Option<Vec<String>>,

    pub is_active: Option<bool>,
}

/// Schema 嵌入向量
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SchemaEmbedding {
    /// 记录 ID
    pub id: Uuid,

    /// 关联的连接 ID
    pub connection_id: Uuid,

    /// 表名
    pub table_name: String,

    /// 列名列表 (逗号分隔)
    pub column_names: String,

    /// 向量嵌入 (文本形式存储)
    pub embedding: Vec<f32>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl SchemaEmbedding {
    /// 创建新的 Schema 嵌入
    pub fn new(
        connection_id: Uuid,
        table_name: String,
        column_names: Vec<String>,
        embedding: Vec<f32>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            connection_id,
            table_name,
            column_names: column_names.join(", "),
            embedding,
            created_at: Utc::now(),
        }
    }
}

/// 语义搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub table_name: String,
    pub column_name: Option<String>,
    pub business_name: String,
    pub business_description: Option<String>,
    pub similarity: f32,
    pub synonyms: Vec<String>,
}

/// Schema 检索请求
#[derive(Debug, Deserialize)]
pub struct SchemaRetrievalRequest {
    /// 查询文本 (自然语言)
    pub query: String,

    /// 连接 ID
    pub connection_id: Uuid,

    /// 返回数量限制
    #[serde(default = "default_retrieval_limit")]
    pub limit: i32,
}

fn default_retrieval_limit() -> i32 {
    5
}

/// Schema 检索响应
#[derive(Debug, Serialize)]
pub struct SchemaRetrievalResponse {
    /// 检索到的 schema 信息
    pub schemas: Vec<SchemaRetrievalItem>,

    /// 检索耗时 (毫秒)
    pub retrieval_time_ms: i64,
}

/// Schema 检索项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRetrievalItem {
    /// 表名
    pub table_name: String,

    /// 列信息
    pub columns: Vec<SchemaColumn>,

    /// 业务名称
    pub business_name: Option<String>,

    /// 相关度分数
    pub similarity: f32,
}

/// Schema 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaColumn {
    /// 列名
    pub column_name: String,

    /// 数据类型
    pub data_type: String,

    /// 业务名称 (如果有)
    pub business_name: Option<String>,

    /// 同义词
    pub synonyms: Vec<String>,
}

/// 术语同义词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermSynonym {
    /// 术语
    pub term: String,

    /// 同义词列表
    pub synonyms: Vec<String>,
}

/// 语义层统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticStats {
    pub total_definitions: i64,
    pub table_level_definitions: i64,
    pub column_level_definitions: i64,
    pub connections_with_semantics: i64,
}

/// 语义列表响应
#[derive(Debug, Serialize)]
pub struct SemanticListResponse {
    pub items: Vec<SemanticDefinition>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 表字段语义批量配置
#[derive(Debug, Deserialize, Validate)]
pub struct BatchSemanticRequest {
    #[serde(default)]
    pub connection_id: Uuid,

    pub definitions: Vec<BatchSemanticItem>,
}

/// 批量语义配置项
#[derive(Debug, Deserialize, Validate)]
pub struct BatchSemanticItem {
    #[validate(length(min = 1, max = 100, message = "表名不能为空"))]
    pub table_name: String,

    #[validate(length(max = 100, message = "列名长度不能超过 100"))]
    pub column_name: Option<String>,

    #[validate(length(min = 1, max = 100, message = "业务名称不能为空"))]
    pub business_name: String,

    #[serde(default)]
    pub business_description: Option<String>,

    #[serde(default)]
    pub synonyms: Option<Vec<String>>,
}

/// 语义导出请求
#[derive(Debug, Deserialize)]
pub struct ExportSemanticRequest {
    #[serde(default)]
    pub connection_id: Option<Uuid>,

    /// 导出格式
    #[serde(default = "default_export_format")]
    pub format: String,
}

fn default_export_format() -> String {
    "json".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_definition_creation() {
        let semantic = SemanticDefinition::new(
            Uuid::new_v4(),
            "users".to_string(),
            Some("username".to_string()),
            "用户名".to_string(),
            Some("用户的登录名称".to_string()),
        );

        assert_eq!(semantic.table_name, "users");
        assert_eq!(semantic.column_name, Some("username".to_string()));
        assert_eq!(semantic.business_name, "用户名");
        assert!(semantic.is_active);
    }

    #[test]
    fn test_full_name() {
        let semantic1 = SemanticDefinition::new(
            Uuid::new_v4(),
            "users".to_string(),
            Some("username".to_string()),
            "用户名".to_string(),
            None,
        );
        assert_eq!(semantic1.full_name(), "users.username");

        let semantic2 = SemanticDefinition::new(
            Uuid::new_v4(),
            "users".to_string(),
            None,
            "用户表".to_string(),
            None,
        );
        assert_eq!(semantic2.full_name(), "users");
    }

    #[test]
    fn test_schema_embedding() {
        let embedding = SchemaEmbedding::new(
            Uuid::new_v4(),
            "users".to_string(),
            vec!["id".to_string(), "username".to_string(), "email".to_string()],
            vec![0.1, 0.2, 0.3],
        );

        assert_eq!(embedding.table_name, "users");
        assert_eq!(embedding.column_names, "id, username, email");
    }
}
