//! 查询模型
//!
//! 定义 SQL 查询和执行结果相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 查询执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryStatus {
    /// 执行成功
    Success,
    /// 执行失败
    Failed,
    /// 已取消
    Cancelled,
}

impl QueryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for QueryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// SQL 查询历史记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueryHistory {
    /// 记录 ID
    pub id: Uuid,

    /// 关联的连接 ID
    pub connection_id: Option<Uuid>,

    /// 用户 ID
    pub user_id: Uuid,

    /// SQL 文本
    pub sql_text: String,

    /// 执行状态
    pub status: QueryStatus,

    /// 执行耗时 (毫秒)
    pub duration_ms: Option<i64>,

    /// 返回行数
    pub row_count: Option<i64>,

    /// 错误信息
    pub error_message: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl QueryHistory {
    /// 创建新的查询历史记录
    pub fn new(
        connection_id: Option<Uuid>,
        user_id: Uuid,
        sql_text: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            connection_id,
            user_id,
            sql_text,
            status: QueryStatus::Failed,
            duration_ms: None,
            row_count: None,
            error_message: None,
            created_at: Utc::now(),
        }
    }

    /// 标记为成功
    pub fn mark_success(&mut self, duration_ms: i64, row_count: i64) {
        self.status = QueryStatus::Success;
        self.duration_ms = Some(duration_ms);
        self.row_count = Some(row_count);
    }

    /// 标记为失败
    pub fn mark_failed(&mut self, error_message: String) {
        self.status = QueryStatus::Failed;
        self.error_message = Some(error_message);
    }
}

/// SQL 执行请求
#[derive(Debug, Deserialize)]
pub struct SqlExecuteRequest {
    /// 连接 ID
    pub connection_id: Uuid,

    /// SQL 语句
    pub sql: String,

    /// 超时时间 (秒)
    #[serde(default)]
    pub timeout: Option<u64>,

    /// 是否返回执行计划
    #[serde(default)]
    pub explain: bool,
}

/// SQL 格式化请求
#[derive(Debug, Deserialize)]
pub struct SqlFormatRequest {
    pub sql: String,

    /// 方言类型
    #[serde(default = "default_dialect")]
    pub dialect: String,
}

fn default_dialect() -> String {
    "mysql".to_string()
}

/// SQL 执行响应
#[derive(Debug, Serialize)]
pub struct SqlExecuteResponse {
    /// 查询 ID
    pub query_id: Uuid,

    /// 列信息
    pub columns: Vec<ColumnMetadata>,

    /// 行数据
    pub rows: Vec<Vec<serde_json::Value>>,

    /// 返回行数
    pub row_count: i64,

    /// 执行耗时 (毫秒)
    pub duration_ms: i64,

    /// 执行计划 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_plan: Option<ExecutionPlan>,
}

/// 列元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    /// 列名
    pub name: String,

    /// 数据类型
    pub data_type: String,

    /// 显示顺序
    pub ordinal: i32,
}

/// 执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// 计划类型
    pub plan_type: String,

    /// 预估成本
    pub estimated_cost: Option<f64>,

    /// 预估行数
    pub estimated_rows: Option<i64>,

    /// 实际行数
    pub actual_rows: Option<i64>,

    /// 计划详情 (JSON)
    pub details: serde_json::Value,
}

/// 查询历史列表响应
#[derive(Debug, Serialize)]
pub struct QueryHistoryListResponse {
    pub items: Vec<QueryHistoryItem>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 查询历史项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistoryItem {
    pub id: Uuid,
    pub connection_name: Option<String>,
    pub sql_text: String,
    pub status: QueryStatus,
    pub duration_ms: Option<i64>,
    pub row_count: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// SQL 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlAnalysisResult {
    /// 是否安全
    pub is_safe: bool,

    /// 阻止原因 (如果被阻止)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,

    /// 警告信息
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,

    /// SQL 类型
    pub sql_type: SqlType,

    /// 涉及的表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referenced_tables: Vec<String>,

    /// 涉及的列
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referenced_columns: Vec<String>,
}

/// SQL 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SqlType {
    /// SELECT 查询
    Select,
    /// INSERT
    Insert,
    /// UPDATE
    Update,
    /// DELETE
    Delete,
    /// DDL (CREATE, DROP, ALTER, TRUNCATE)
    Ddl,
    /// 其他
    Other,
}

impl SqlType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Ddl => "ddl",
            Self::Other => "other",
        }
    }

    /// 是否是只读操作
    pub fn is_readonly(&self) -> bool {
        matches!(self, Self::Select)
    }
}

/// 危险函数检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousFunctionCheck {
    /// 函数名
    pub function_name: String,

    /// 是否检测到
    pub detected: bool,

    /// 位置
    pub position: Option<(usize, usize)>,
}

/// 数据预览请求
#[derive(Debug, Deserialize)]
pub struct DataPreviewRequest {
    /// 连接 ID
    pub connection_id: Uuid,

    /// 表名
    pub table_name: String,

    /// 限制行数
    #[serde(default = "default_preview_limit")]
    pub limit: i32,
}

fn default_preview_limit() -> i32 {
    100
}

/// 数据预览响应
#[derive(Debug, Serialize)]
pub struct DataPreviewResponse {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: i32,
    pub table_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_history_creation() {
        let user_id = Uuid::new_v4();
        let conn_id = Uuid::new_v4();

        let mut history = QueryHistory::new(
            Some(conn_id),
            user_id,
            "SELECT * FROM users".to_string(),
        );

        assert_eq!(history.status, QueryStatus::Failed);
        assert!(history.duration_ms.is_none());

        history.mark_success(50, 10);

        assert_eq!(history.status, QueryStatus::Success);
        assert_eq!(history.duration_ms, Some(50));
        assert_eq!(history.row_count, Some(10));
    }

    #[test]
    fn test_sql_type() {
        assert!(SqlType::Select.is_readonly());
        assert!(!SqlType::Update.is_readonly());
        assert!(!SqlType::Delete.is_readonly());
    }
}
