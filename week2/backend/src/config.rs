//! 配置模块
//!
//! 负责加载和管理应用配置
//!
//! 配置来源优先级（从高到低）：
//! 1. 环境变量
//! 2. .env 文件
//! 3. config.yaml

use serde::Deserialize;
use std::env;
use std::path::Path;

/// 应用配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub llm: LlmConfig,
    pub jwt: JwtConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub env: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub openai: OpenAiConfig,
    pub anthropic: AnthropicConfig,
    pub local: LocalLlmConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalLlmConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expires: u64,
    pub refresh_token_expires: u64,
    pub issuer: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub argon2: Argon2Config,
    pub cors: CorsConfig,
    pub sql: SqlSecurityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Argon2Config {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            max_age: 3600,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SqlSecurityConfig {
    pub max_query_length: usize,
    pub query_timeout: u64,
    pub allow_dangerous_functions: bool,
    pub blocked_functions: Vec<String>,
}

impl Default for SqlSecurityConfig {
    fn default() -> Self {
        Self {
            max_query_length: 65536,
            query_timeout: 30000,
            allow_dangerous_functions: false,
            blocked_functions: vec![
                "LOAD_FILE".to_string(),
                "INTO OUTFILE".to_string(),
                "BENCHMARK".to_string(),
                "SLEEP".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub include_target: bool,
    pub include_span_list: bool,
}

impl AppConfig {
    /// 从 config.yaml 加载配置
    pub fn load() -> anyhow::Result<Self> {
        Self::load_from_file("config.yaml")
    }

    /// 从指定文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();

        // 加载 .env 文件（如果存在）
        load_dotenv();

        // 解析 YAML 配置
        let config_content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.display(), e))?;

        // 替换环境变量
        let config_content = substitute_env_vars(&config_content);

        // 解析 YAML
        let config: AppConfig = serde_yaml::from_str(&config_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 检查是否为开发模式
    pub fn is_development(&self) -> bool {
        self.app.env == "development"
    }

    /// 检查是否为生产模式
    pub fn is_production(&self) -> bool {
        self.app.env == "production"
    }
}

impl AppConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.app.port == 0 {
            anyhow::bail!("App port must be greater than 0");
        }

        if self.database.url.is_empty() {
            anyhow::bail!("Database URL is required");
        }

        if self.jwt.secret.is_empty() {
            anyhow::bail!("JWT secret is required");
        }

        if self.jwt.access_token_expires == 0 {
            anyhow::bail!("JWT access token expires must be greater than 0");
        }

        Ok(())
    }
}

/// 加载 .env 文件
fn load_dotenv() {
    // 尝试加载当前目录下的 .env 文件
    if std::path::Path::new(".env").exists() {
        dotenvy::dotenv().ok();
    }

    // 也尝试从环境变量加载
    for (key, value) in env::vars() {
        if std::env::var(key.as_str()).is_err() {
            std::env::set_var(key, value);
        }
    }
}

/// 替换配置中的环境变量占位符
/// 支持格式: ${VAR_NAME} 或 ${VAR_NAME:-default_value}
fn substitute_env_vars(content: &str) -> String {
    let re = regex::Regex::new(r"\$\{([^}:]+)(?::-([^}]*))?\}").unwrap();

    re.replace_all(content, |caps: &regex::Captures| {
        let var_name = &caps[1];
        let default_value = caps.get(2).map_or("", |m| m.as_str());

        env::var(var_name).unwrap_or_else(|_| default_value.to_string())
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_env_vars() {
        env::set_var("TEST_VAR", "test_value");

        let input = "value is ${TEST_VAR} and default is ${UNSET_VAR:-fallback}";
        let output = substitute_env_vars(input);

        assert_eq!(output, "value is test_value and default is fallback");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig {
            app: AppSettings {
                name: "test".to_string(),
                host: "0.0.0.0".to_string(),
                port: 8080,
                env: "development".to_string(),
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/test".to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout: 5,
                idle_timeout: 300,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                max_connections: 5,
                pool_timeout: 5,
            },
            llm: LlmConfig {
                provider: "openai".to_string(),
                openai: OpenAiConfig {
                    api_key: "test".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                    model: "gpt-4".to_string(),
                    max_tokens: 4096,
                    temperature: 0.1,
                },
                anthropic: AnthropicConfig {
                    api_key: "".to_string(),
                    base_url: "".to_string(),
                    model: "".to_string(),
                    max_tokens: 0,
                    temperature: 0.0,
                },
                local: LocalLlmConfig {
                    base_url: "".to_string(),
                    model: "".to_string(),
                    api_key: "".to_string(),
                },
            },
            jwt: JwtConfig {
                secret: "test_secret".to_string(),
                access_token_expires: 3600,
                refresh_token_expires: 604800,
                issuer: "test".to_string(),
            },
            security: SecurityConfig {
                argon2: Argon2Config {
                    memory_cost: 19456,
                    time_cost: 2,
                    parallelism: 1,
                },
                cors: CorsConfig {
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec!["GET".to_string()],
                    allowed_headers: vec!["Content-Type".to_string()],
                    max_age: 3600,
                },
                sql: SqlSecurityConfig {
                    max_query_length: 65536,
                    query_timeout: 30000,
                    allow_dangerous_functions: false,
                    blocked_functions: vec![],
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                include_target: true,
                include_span_list: true,
            },
        };

        assert!(config.validate().is_ok());

        // 测试无效配置
        config.jwt.secret = "".to_string();
        assert!(config.validate().is_err());
    }
}
