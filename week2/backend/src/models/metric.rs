//! 指标模型
//!
//! 定义业务指标相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 格式化类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatType {
    /// 数字
    Number,
    /// 货币
    Currency,
    /// 百分比
    Percent,
    /// 日期
    Date,
    /// 时间
    Time,
}

impl FormatType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Currency => "currency",
            Self::Percent => "percent",
            Self::Date => "date",
            Self::Time => "time",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "number" => Some(Self::Number),
            "currency" => Some(Self::Currency),
            "percent" | "percentage" => Some(Self::Percent),
            "date" => Some(Self::Date),
            "time" => Some(Self::Time),
            _ => None,
        }
    }
}

impl std::fmt::Display for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for FormatType {
    fn default() -> Self {
        Self::Number
    }
}

/// 指标实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Metric {
    /// 指标 ID
    pub id: Uuid,

    /// 指标名称
    pub name: String,

    /// 指标编码 (唯一)
    pub code: String,

    /// 指标描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 计算表达式
    pub expression: String,

    /// 维度列表 (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<serde_json::Value>,

    /// 单位
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// 格式化类型
    pub format_type: FormatType,

    /// 创建者 ID
    pub created_by: Option<Uuid>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Metric {
    /// 创建新指标
    pub fn new(
        name: String,
        code: String,
        expression: String,
        created_by: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            code,
            description: None,
            expression,
            dimensions: None,
            unit: None,
            format_type: FormatType::Number,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置单位
    pub fn with_unit(mut self, unit: String) -> Self {
        self.unit = Some(unit);
        self
    }

    /// 设置格式化类型
    pub fn with_format_type(mut self, format_type: FormatType) -> Self {
        self.format_type = format_type;
        self
    }

    /// 设置维度
    pub fn with_dimensions(mut self, dimensions: Vec<String>) -> Self {
        self.dimensions = Some(serde_json::json!(dimensions));
        self
    }
}

/// 创建指标请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMetricRequest {
    #[validate(length(min = 1, max = 100, message = "指标名称不能为空"))]
    pub name: String,

    #[validate(length(min = 1, max = 50, message = "指标编码不能为空"))]
    #[validate(regex(path = "*metric_code_regex", message = "指标编码只能包含字母、数字和下划线"))]
    pub code: String,

    #[validate(length(max = 500, message = "描述长度不能超过 500"))]
    pub description: Option<String>,

    #[validate(length(min = 1, message = "计算表达式不能为空"))]
    pub expression: String,

    #[serde(default)]
    pub dimensions: Option<Vec<String>>,

    #[validate(length(max = 20, message = "单位长度不能超过 20"))]
    pub unit: Option<String>,

    #[serde(default)]
    pub format_type: FormatType,
}

/// 自定义验证器：指标编码正则
#[allow(dead_code)]
fn metric_code_regex() -> regex::Regex {
    regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap()
}

/// 更新指标请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMetricRequest {
    #[validate(length(min = 1, max = 100, message = "指标名称不能为空"))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "描述长度不能超过 500"))]
    pub description: Option<String>,

    #[validate(length(min = 1, message = "计算表达式不能为空"))]
    pub expression: Option<String>,

    pub dimensions: Option<Vec<String>>,

    #[validate(length(max = 20, message = "单位长度不能超过 20"))]
    pub unit: Option<String>,

    pub format_type: Option<FormatType>,
}

/// 指标血缘关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricLineage {
    /// 指标 ID
    pub metric_id: Uuid,

    /// 指标名称
    pub metric_name: String,

    /// 依赖的表
    pub source_tables: Vec<String>,

    /// 依赖的列
    pub source_columns: Vec<String>,

    /// 依赖的其他指标
    pub dependent_metrics: Vec<DependentMetric>,

    /// 被依赖的指标
    pub referenced_by: Vec<DependentMetric>,
}

/// 依赖的指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependentMetric {
    /// 指标 ID
    pub id: Uuid,

    /// 指标名称
    pub name: String,

    /// 指标编码
    pub code: String,
}

/// 指标执行请求
#[derive(Debug, Deserialize)]
pub struct MetricExecuteRequest {
    /// 指标编码
    pub metric_code: String,

    /// 连接 ID
    pub connection_id: Uuid,

    /// 维度筛选条件
    #[serde(default)]
    pub filters: Option<serde_json::Value>,

    /// 超时时间 (秒)
    #[serde(default)]
    pub timeout: Option<u64>,
}

/// 指标执行响应
#[derive(Debug, Serialize)]
pub struct MetricExecuteResponse {
    /// 指标信息
    pub metric: MetricValue,

    /// 执行耗时 (毫秒)
    pub duration_ms: i64,

    /// 执行时间
    pub executed_at: DateTime<Utc>,
}

/// 指标值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub value: serde_json::Value,
    pub formatted_value: String,
    pub unit: Option<String>,
    pub dimensions: Option<serde_json::Value>,
}

/// 指标列表响应
#[derive(Debug, Serialize)]
pub struct MetricListResponse {
    pub items: Vec<Metric>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 指标维度值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDimensionValue {
    /// 维度名称
    pub dimension: String,

    /// 维度值
    pub value: String,

    /// 指标值
    pub metric_value: serde_json::Value,
}

/// 指标趋势数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTrendData {
    /// 指标信息
    pub metric: MetricValue,

    /// 时间点列表
    pub time_points: Vec<TimePoint>,

    /// 趋势描述
    pub trend_description: Option<String>,
}

/// 时间点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePoint {
    /// 时间标签
    pub label: String,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 值
    pub value: serde_json::Value,
}

/// 指标验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValidationResult {
    /// 是否有效
    pub is_valid: bool,

    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// 解析的表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referenced_tables: Vec<String>,

    /// 解析的列
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referenced_columns: Vec<String>,

    /// 语法验证
    pub syntax_valid: bool,
}

/// 指标搜索请求
#[derive(Debug, Deserialize)]
pub struct MetricSearchRequest {
    /// 搜索关键词
    pub query: Option<String>,

    /// 维度筛选
    #[serde(default)]
    pub dimensions: Option<Vec<String>>,

    /// 关联的连接 ID
    #[serde(default)]
    pub connection_id: Option<Uuid>,

    /// 页码
    #[serde(default = "default_page")]
    pub page: i32,

    /// 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new(
            "用户总数".to_string(),
            "total_users".to_string(),
            "SELECT COUNT(*) FROM users".to_string(),
            Some(Uuid::new_v4()),
        )
        .with_description("统计所有注册用户数量".to_string())
        .with_unit("人".to_string())
        .with_format_type(FormatType::Number)
        .with_dimensions(vec!["region".to_string(), "status".to_string()]);

        assert_eq!(metric.name, "用户总数");
        assert_eq!(metric.code, "total_users");
        assert_eq!(metric.format_type, FormatType::Number);
    }

    #[test]
    fn test_format_type() {
        assert_eq!(FormatType::from_str("number"), Some(FormatType::Number));
        assert_eq!(FormatType::from_str("percent"), Some(FormatType::Percent));
        assert_eq!(FormatType::from_str("invalid"), None);
    }
}
