//! LLM 客户端服务
//!
//! 提供与 LLM 服务的交互功能

use crate::config::{AppConfig, LlmConfig};
use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// NL 转 SQL 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlToSqlResult {
    pub sql: String,
    pub explanation: String,
    pub confidence: f32,
    pub estimated_rows: Option<i32>,
    pub referenced_tables: Vec<String>,
}

/// LLM 客户端
#[derive(Clone)]
pub struct LlmClient {
    config: Arc<LlmConfig>,
    http_client: Client,
}

impl LlmClient {
    /// 创建新的 LLM 客户端
    pub fn new(config: &AppConfig) -> AppResult<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // 增加超时时间
            .build()
            .map_err(|e| AppError::internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config: Arc::new(config.llm.clone()),
            http_client,
        })
    }

    /// NL 转 SQL（主要接口）
    pub async fn convert_nl_to_sql(
        &self,
        question: &str,
        schema_context: Option<&str>,
    ) -> AppResult<NlToSqlResult> {
        let prompt = self.build_nl_to_sql_prompt(question, schema_context);
        let model = &self.config.openai.model;

        match self.call_llm(&prompt, model).await {
            Ok(response) => {
                self.parse_nl_to_sql_response(&response)
            }
            Err(e) => {
                // 如果 API Key 未配置，返回友好的错误消息
                if e.to_string().contains("API key not configured") {
                    Err(AppError::LlmUnavailable(
                        "LLM API Key 未配置，请检查 .env 文件中的 OPENAI_API_KEY".to_string()
                    ))
                } else {
                    Err(e)
                }
            }
        }
    }

    /// 生成 SQL（自然语言转 SQL）
    pub async fn generate_sql(&self, schema: &str, question: &str) -> AppResult<SqlGenerationResponse> {
        let prompt = self.build_nl_to_sql_prompt(question, Some(schema));
        let model = &self.config.openai.model;

        let response = self.call_llm(&prompt, model).await?;
        self.parse_sql_response(&response)
    }

    /// 生成数据洞察
    pub async fn generate_insight(&self, data_summary: &str) -> AppResult<String> {
        let prompt = format!(
            r#"作为数据分析师，请根据以下数据摘要生成简洁的数据洞察（1-2句话）：

数据摘要：
{}

要求：
1. 用中文回答
2. 突出关键发现和趋势
3. 语言简洁专业
4. 不要使用 Markdown 格式"#,
            data_summary
        );

        let response = self.call_llm(&prompt, "gpt-4o-mini").await?;
        Ok(response)
    }

    /// 解释 SQL
    pub async fn explain_sql(&self, sql: &str, schema: &str) -> AppResult<String> {
        let prompt = format!(
            r#"作为 SQL 专家，请解释以下 SQL 查询的作用：

SQL：
```sql
{}
```

Schema 信息：
{}

要求：
1. 用中文回答
2. 解释查询的目的和逻辑
3. 简要说明涉及的表和字段
4. 不要使用 Markdown 格式"#,
            sql, schema
        );

        let response = self.call_llm(&prompt, "gpt-4o-mini").await?;
        Ok(response)
    }

    /// 调用 LLM (公开接口)
    pub async fn call_llm(&self, prompt: &str, model: &str) -> AppResult<String> {
        let api_key = &self.config.openai.api_key;
        let base_url = &self.config.openai.base_url;

        if api_key.is_empty() {
            return Err(AppError::LlmUnavailable("API key not configured".to_string()));
        }

        let request_body = LlmRequest {
            model: model.to_string(),
            messages: vec![LlmMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.config.openai.temperature,
            max_tokens: self.config.openai.max_tokens,
        };

        let response = self
            .http_client
            .post(format!("{}/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::LlmError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::LlmError(format!(
                "API returned error {}: {}",
                status, body
            )));
        }

        let llm_response: LlmResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmParseError(format!("Failed to parse response: {}", e)))?;

        let content = llm_response
            .choices
            .first()
            .map(|c| c.message.content.as_str())
            .ok_or_else(|| AppError::LlmParseError("No content in response".to_string()))?
            .to_string();

        Ok(content)
    }

    /// 构建 NL 转 SQL 的提示词
    fn build_nl_to_sql_prompt(&self, question: &str, schema_context: Option<&str>) -> String {
        let schema_info = schema_context
            .map(|s| {
                format!(
                    r#"## 数据库 Schema 信息
{}

"#,
                    s
                )
            })
            .unwrap_or_else(|| String::new());

        format!(
            r#"你是一个专业的 SQL 专家。请根据以下 Schema 信息，将用户问题转换为 SQL 查询。

{schema_info}
## 要求
1. 只生成 SELECT 查询，不要生成 INSERT/UPDATE/DELETE
2. 考虑性能，必要时添加 LIMIT
3. 如果问题不明确，进行合理假设
4. 只返回 SQL，不要其他解释
5. 确保 SQL 语法正确

## 用户问题
{}

请按以下 JSON 格式返回：
{{
  "sql": "生成的 SQL",
  "explanation": "查询逻辑的中文解释",
  "confidence": 0.0-1.0 的置信度
}}"#,
            question
        )
    }

    /// 解析 NL 转 SQL 响应
    fn parse_nl_to_sql_response(&self, response: &str) -> AppResult<NlToSqlResult> {
        let json_str = self.extract_json(response)?;

        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| AppError::LlmParseError(format!("Failed to parse JSON: {}", e)))?;

        let sql = parsed["sql"]
            .as_str()
            .ok_or_else(|| AppError::LlmParseError("Missing 'sql' field".to_string()))?
            .to_string();

        let explanation = parsed["explanation"]
            .as_str()
            .unwrap_or("无法生成解释")
            .to_string();

        let confidence = parsed["confidence"]
            .as_f64()
            .unwrap_or(0.5) as f32;

        // 提取引用的表
        let referenced_tables: Vec<String> = parsed["referenced_tables"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // 估算行数
        let estimated_rows = parsed["estimated_rows"]
            .as_i64()
            .map(|v| v as i32);

        Ok(NlToSqlResult {
            sql,
            explanation,
            confidence,
            estimated_rows,
            referenced_tables,
        })
    }

    /// 解析 SQL 生成响应
    fn parse_sql_response(&self, response: &str) -> AppResult<SqlGenerationResponse> {
        // 尝试提取 JSON
        let json_str = self.extract_json(response)?;

        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| AppError::LlmParseError(format!("Failed to parse JSON: {}", e)))?;

        let sql = parsed["sql"]
            .as_str()
            .ok_or_else(|| AppError::LlmParseError("Missing 'sql' field".to_string()))?
            .to_string();

        let explanation = parsed["explanation"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let confidence = parsed["confidence"]
            .as_f64()
            .unwrap_or(0.5) as f32;

        Ok(SqlGenerationResponse {
            sql,
            explanation,
            confidence,
            estimated_rows: None,
            referenced_tables: vec![],
        })
    }

    /// 从文本中提取 JSON
    fn extract_json(&self, text: &str) -> AppResult<String> {
        // 尝试直接解析
        if serde_json::from_str::<serde_json::Value>(text).is_ok() {
            return Ok(text.to_string());
        }

        // 尝试提取 ```json ... ``` 块
        let re = regex::Regex::new(r"```json\s*([\s\S]*?)\s*```").unwrap();
        if let Some(caps) = re.captures(text) {
            if let Some(json_str) = caps.get(1) {
                return Ok(json_str.as_str().trim().to_string());
            }
        }

        // 尝试提取 { ... }
        let re = regex::Regex::new(r"\{[\s\S]*\}").unwrap();
        if let Some(m) = re.find(text) {
            return Ok(m.as_str().to_string());
        }

        Err(AppError::LlmParseError("Failed to extract JSON from response".to_string()))
    }
}

/// SQL 生成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlGenerationResponse {
    pub sql: String,
    pub explanation: String,
    pub confidence: f32,
    pub estimated_rows: Option<i32>,
    pub referenced_tables: Vec<String>,
}

/// LLM 请求
#[derive(Debug, Serialize)]
struct LlmRequest {
    model: String,
    messages: Vec<LlmMessage>,
    temperature: f32,
    max_tokens: u32,
}

/// LLM 消息
#[derive(Debug, Serialize)]
struct LlmMessage {
    role: String,
    content: String,
}

/// LLM 响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LlmResponse {
    id: String,
    choices: Vec<LlmChoice>,
    usage: LlmUsage,
}

/// LLM 选择
#[derive(Debug, Deserialize)]
struct LlmChoice {
    message: LlmResponseMessage,
}

/// LLM 响应消息
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LlmResponseMessage {
    role: String,
    content: String,
}

/// LLM 使用量
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LlmUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client() -> LlmClient {
        let config = AppConfig {
            app: crate::config::AppSettings {
                name: "test".to_string(),
                host: "0.0.0.0".to_string(),
                port: 8080,
                env: "development".to_string(),
            },
            database: crate::config::DatabaseConfig {
                url: "postgres://localhost/test".to_string(),
                max_connections: 5,
                min_connections: 1,
                connect_timeout: 5,
                idle_timeout: 300,
            },
            redis: crate::config::RedisConfig {
                url: "redis://localhost".to_string(),
                max_connections: 5,
                pool_timeout: 5,
            },
            llm: crate::config::LlmConfig {
                provider: "openai".to_string(),
                openai: crate::config::OpenAiConfig {
                    api_key: "test-key".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                    model: "gpt-4o".to_string(),
                    max_tokens: 4096,
                    temperature: 0.1,
                },
                anthropic: crate::config::AnthropicConfig {
                    api_key: "".to_string(),
                    base_url: "".to_string(),
                    model: "".to_string(),
                    max_tokens: 0,
                    temperature: 0.0,
                },
                local: crate::config::LocalLlmConfig {
                    base_url: "".to_string(),
                    model: "".to_string(),
                    api_key: "".to_string(),
                },
            },
            jwt: crate::config::JwtConfig {
                secret: "test-secret".to_string(),
                access_token_expires: 3600,
                refresh_token_expires: 604800,
                issuer: "test".to_string(),
            },
            security: crate::config::SecurityConfig {
                argon2: crate::config::Argon2Config {
                    memory_cost: 19456,
                    time_cost: 2,
                    parallelism: 1,
                },
                cors: crate::config::CorsConfig {
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec!["GET".to_string()],
                    allowed_headers: vec!["Content-Type".to_string()],
                    max_age: 3600,
                },
                sql: crate::config::SqlSecurityConfig {
                    max_query_length: 65536,
                    query_timeout: 30000,
                    allow_dangerous_functions: false,
                    blocked_functions: vec![],
                },
            },
            logging: crate::config::LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                include_target: true,
                include_span_list: true,
            },
        };

        LlmClient::new(&config).unwrap()
    }

    #[test]
    fn test_extract_json() {
        let client = create_test_client();

        // 测试直接 JSON
        let json = r#"{"sql": "SELECT * FROM users", "explanation": "test", "confidence": 0.9}"#;
        let result = client.extract_json(json);
        assert!(result.is_ok());

        // 测试 markdown 代码块
        let markdown = r#"```json
{"sql": "SELECT * FROM users", "explanation": "test", "confidence": 0.9}
```"#;
        let result = client.extract_json(markdown);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sql_response() {
        let client = create_test_client();

        let json = r#"{"sql": "SELECT * FROM users", "explanation": "查询所有用户", "confidence": 0.95}"#;
        let result = client.parse_sql_response(json);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.sql, "SELECT * FROM users");
        assert_eq!(response.explanation, "查询所有用户");
        assert!((response.confidence - 0.95).abs() < 0.01);
    }
}
