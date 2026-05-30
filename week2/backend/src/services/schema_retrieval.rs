//! Schema 检索服务
//!
//! 提供 Schema RAG（检索增强生成）功能

use crate::error::{AppError, AppResult};
use crate::models::semantic::{SchemaColumn, SchemaRetrievalItem, SchemaRetrievalResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema 检索服务
#[derive(Clone)]
pub struct SchemaRetrievalService {
    cache: HashMap<String, SchemaCache>,
}

impl SchemaRetrievalService {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// 检索相关的 Schema 信息
    pub fn retrieve(
        &self,
        query: &str,
        schema_items: &[SchemaRetrievalItem],
    ) -> AppResult<SchemaRetrievalResponse> {
        let start = std::time::Instant::now();

        // 简单的关键词匹配（生产环境应使用向量相似度）
        let keywords = self.extract_keywords(query);
        let mut scored_items: Vec<(SchemaRetrievalItem, f32)> = schema_items
            .iter()
            .map(|item| {
                let score = self.calculate_relevance(item, &keywords);
                (item.clone(), score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // 按相关性排序
        scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 取前 5 个
        let results: Vec<SchemaRetrievalItem> = scored_items
            .into_iter()
            .take(5)
            .map(|(item, _)| item)
            .collect();

        let retrieval_time_ms = start.elapsed().as_millis() as i64;

        Ok(SchemaRetrievalResponse {
            schemas: results,
            retrieval_time_ms,
        })
    }

    /// 提取引擎关键词
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        let stop_words = vec![
            "的", "是", "在", "有", "和", "与", "或", "帮", "查", "看", "一下",
            "多少", "哪些", "什么", "如何", "怎么", "请", "我", "要", "想", "给",
        ];

        query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
            .collect::<String>()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty() && !stop_words.contains(s))
            .map(|s| s.to_lowercase())
            .collect()
    }

    /// 计算相关性分数
    fn calculate_relevance(&self, item: &SchemaRetrievalItem, keywords: &[String]) -> f32 {
        let mut score = 0.0;

        // 表名匹配
        let table_name_lower = item.table_name.to_lowercase();
        for keyword in keywords {
            if table_name_lower.contains(keyword) {
                score += 2.0; // 表名匹配权重更高
            }
        }

        // 列名匹配
        for column in &item.columns {
            let column_name_lower = column.column_name.to_lowercase();
            for keyword in keywords {
                if column_name_lower.contains(keyword) {
                    score += 1.0;
                }
                // 业务名称匹配
                if let Some(business_name) = &column.business_name {
                    if business_name.contains(keyword) {
                        score += 1.5;
                    }
                }
                // 同义词匹配
                for synonym in &column.synonyms {
                    if synonym.to_lowercase().contains(keyword) {
                        score += 1.2;
                    }
                }
            }
        }

        score
    }

    /// 构建 Schema 上下文
    pub fn build_context(&self, items: &[SchemaRetrievalItem]) -> String {
        let mut context = String::new();

        for item in items {
            context.push_str(&format!("表: {}\n", item.table_name));

            if let Some(business_name) = &item.business_name {
                context.push_str(&format!("  说明: {}\n", business_name));
            }

            context.push_str("  字段:\n");
            for column in &item.columns {
                let mut col_info = format!("    - {} ({})", column.column_name, column.data_type);

                if let Some(business_name) = &column.business_name {
                    col_info.push_str(&format!(" [{}]", business_name));
                }

                if !column.synonyms.is_empty() {
                    col_info.push_str(&format!(" 同义词: {}", column.synonyms.join(", ")));
                }

                context.push_str(&col_info);
                context.push('\n');
            }

            context.push('\n');
        }

        context
    }
}

impl Default for SchemaRetrievalService {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema 缓存
#[derive(Debug, Clone)]
struct SchemaCache {
    connection_id: String,
    tables: Vec<TableSchema>,
    cached_at: std::time::Instant,
}

/// 表 Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TableSchema {
    table_name: String,
    business_name: Option<String>,
    columns: Vec<ColumnSchema>,
}

/// 列 Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ColumnSchema {
    column_name: String,
    data_type: String,
    business_name: Option<String>,
    synonyms: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let service = SchemaRetrievalService::new();

        let keywords = service.extract_keywords("查一下用户订单数量");
        assert!(keywords.contains(&"用户".to_string()));
        assert!(keywords.contains(&"订单".to_string()));
        assert!(keywords.contains(&"数量".to_string()));
    }

    #[test]
    fn test_relevance_calculation() {
        let service = SchemaRetrievalService::new();

        let item = SchemaRetrievalItem {
            table_name: "orders".to_string(),
            columns: vec![
                SchemaColumn {
                    column_name: "user_id".to_string(),
                    data_type: "int".to_string(),
                    business_name: Some("用户ID".to_string()),
                    synonyms: vec!["用户".to_string()],
                },
                SchemaColumn {
                    column_name: "amount".to_string(),
                    data_type: "decimal".to_string(),
                    business_name: Some("金额".to_string()),
                    synonyms: vec![],
                },
            ],
            business_name: Some("订单表".to_string()),
            similarity: 0.0,
        };

        let keywords = vec!["用户".to_string(), "订单".to_string()];

        let score = service.calculate_relevance(&item, &keywords);
        assert!(score > 0.0);
    }

    #[test]
    fn test_build_context() {
        let service = SchemaRetrievalService::new();

        let items = vec![SchemaRetrievalItem {
            table_name: "users".to_string(),
            columns: vec![
                SchemaColumn {
                    column_name: "id".to_string(),
                    data_type: "int".to_string(),
                    business_name: Some("用户ID".to_string()),
                    synonyms: vec![],
                },
                SchemaColumn {
                    column_name: "name".to_string(),
                    data_type: "varchar".to_string(),
                    business_name: Some("用户名".to_string()),
                    synonyms: vec!["姓名".to_string()],
                },
            ],
            business_name: Some("用户表".to_string()),
            similarity: 0.0,
        }];

        let context = service.build_context(&items);
        assert!(context.contains("users"));
        assert!(context.contains("用户表"));
        assert!(context.contains("id"));
    }
}
