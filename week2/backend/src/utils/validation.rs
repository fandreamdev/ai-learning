//! 输入验证工具模块
//!
//! 提供通用的输入验证函数

use crate::error::{AppError, AppResult};
use regex::Regex;

/// 验证用户名
///
/// 用户名规则：
/// - 长度 3-50 个字符
/// - 只能包含字母、数字和下划线
/// - 必须以字母开头
pub fn validate_username(username: &str) -> AppResult<()> {
    // 长度检查
    if username.len() < 3 {
        return Err(AppError::validation("用户名长度至少为 3 个字符"));
    }
    if username.len() > 50 {
        return Err(AppError::validation("用户名长度不能超过 50 个字符"));
    }

    // 格式检查
    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap();
    if !re.is_match(username) {
        return Err(AppError::validation(
            "用户名只能包含字母、数字和下划线，且必须以字母开头",
        ));
    }

    Ok(())
}

/// 验证邮箱
pub fn validate_email(email: &str) -> AppResult<()> {
    if email.is_empty() {
        return Err(AppError::validation("邮箱不能为空"));
    }

    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !re.is_match(email) {
        return Err(AppError::validation("邮箱格式不正确"));
    }

    Ok(())
}

/// 验证密码强度
pub fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::validation("密码长度至少为 8 个字符"));
    }
    if password.len() > 128 {
        return Err(AppError::validation("密码长度不能超过 128 个字符"));
    }

    let strength = crate::utils::password::PasswordUtils::check_password_strength(password);
    if !strength.meets_minimum() {
        return Err(AppError::validation(strength.description()));
    }

    Ok(())
}

/// 验证主机地址
pub fn validate_host(host: &str) -> AppResult<()> {
    if host.is_empty() {
        return Err(AppError::validation("主机地址不能为空"));
    }

    // 支持 IP 地址和域名
    // 域名: 字母、数字、点、连字符
    let domain_re = Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9\-\.]*[a-zA-Z0-9])?$").unwrap();

    if !domain_re.is_match(host) {
        return Err(AppError::validation("主机地址格式不正确"));
    }

    // 长度检查
    if host.len() > 255 {
        return Err(AppError::validation("主机地址长度不能超过 255 个字符"));
    }

    Ok(())
}

/// 验证端口号
pub fn validate_port(port: u16) -> AppResult<()> {
    if port == 0 {
        return Err(AppError::validation("端口号不能为 0"));
    }
    Ok(())
}

/// 验证数据库名称
pub fn validate_database_name(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(AppError::validation("数据库名称不能为空"));
    }

    if name.len() > 100 {
        return Err(AppError::validation("数据库名称长度不能超过 100 个字符"));
    }

    // 数据库名称通常只能包含字母、数字和下划线
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    if !re.is_match(name) {
        return Err(AppError::validation(
            "数据库名称只能包含字母、数字和下划线，且必须以字母或下划线开头",
        ));
    }

    Ok(())
}

/// 验证 SQL 查询长度
pub fn validate_sql_length(sql: &str, max_length: usize) -> AppResult<()> {
    if sql.is_empty() {
        return Err(AppError::validation("SQL 语句不能为空"));
    }

    if sql.len() > max_length {
        return Err(AppError::validation(format!(
            "SQL 语句长度不能超过 {} 个字符",
            max_length
        )));
    }

    Ok(())
}

/// 验证 JSON
pub fn validate_json(json_str: &str) -> AppResult<serde_json::Value> {
    serde_json::from_str(json_str)
        .map_err(|e| AppError::validation(format!("无效的 JSON 格式: {}", e)))
}

/// 验证 UUID
pub fn validate_uuid(uuid_str: &str) -> AppResult<uuid::Uuid> {
    uuid::Uuid::parse_str(uuid_str)
        .map_err(|_| AppError::validation("无效的 UUID 格式"))
}

/// 验证中文内容长度（按字符计算）
pub fn validate_text_length(text: &str, min: usize, max: usize) -> AppResult<()> {
    let len = text.chars().count();
    if len < min {
        return Err(AppError::validation(format!("内容长度至少为 {} 个字符", min)));
    }
    if len > max {
        return Err(AppError::validation(format!("内容长度不能超过 {} 个字符", max)));
    }
    Ok(())
}

/// 验证敏感内容（检测潜在的注入攻击）
pub fn contains_malicious_content(text: &str) -> bool {
    let malicious_patterns = [
        "<script",
        "javascript:",
        "onerror=",
        "onload=",
        "onclick=",
        "eval(",
        "document.cookie",
        "window.location",
    ];

    let lower_text = text.to_lowercase();
    malicious_patterns.iter().any(|pattern| lower_text.contains(pattern))
}

/// 验证自然语言查询长度
pub fn validate_nl_query(query: &str) -> AppResult<()> {
    let len = query.chars().count();

    if len < 1 {
        return Err(AppError::validation("查询内容不能为空"));
    }

    if len > 2000 {
        return Err(AppError::validation("查询内容不能超过 2000 个字符"));
    }

    // 检测恶意内容
    if contains_malicious_content(query) {
        return Err(AppError::validation("查询内容包含不允许的字符"));
    }

    Ok(())
}

/// 验证指标编码
pub fn validate_metric_code(code: &str) -> AppResult<()> {
    if code.is_empty() {
        return Err(AppError::validation("指标编码不能为空"));
    }

    if code.len() > 50 {
        return Err(AppError::validation("指标编码长度不能超过 50 个字符"));
    }

    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap();
    if !re.is_match(code) {
        return Err(AppError::validation(
            "指标编码只能包含字母、数字和下划线，且必须以字母开头",
        ));
    }

    Ok(())
}

/// 通用验证错误收集器
#[derive(Debug, Default)]
pub struct ValidationErrors {
    errors: Vec<String>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_result(self) -> AppResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::validation(self.errors.join("; ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("testuser").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test_user").is_ok());
        assert!(validate_username("a").is_err()); // 太短
        assert!(validate_username("123user").is_err()); // 以数字开头
        assert!(validate_username("user-name").is_err()); // 包含连字符
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name@domain.co.uk").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("test@").is_err());
    }

    #[test]
    fn test_validate_host() {
        assert!(validate_host("localhost").is_ok());
        assert!(validate_host("192.168.1.1").is_ok());
        assert!(validate_host("example.com").is_ok());
        assert!(validate_host("sub.example.com").is_ok());
        assert!(validate_host("").is_err());
    }

    #[test]
    fn test_validate_database_name() {
        assert!(validate_database_name("mydb").is_ok());
        assert!(validate_database_name("my_database").is_ok());
        assert!(validate_database_name("_private").is_ok());
        assert!(validate_database_name("123db").is_err()); // 以数字开头
        assert!(validate_database_name("my-db").is_err()); // 包含连字符
    }

    #[test]
    fn test_validate_json() {
        let result = validate_json(r#"{"key": "value"}"#);
        assert!(result.is_ok());

        let result = validate_json("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_uuid() {
        let result = validate_uuid("550e8400-e29b-41d4-a716-446655440000");
        assert!(result.is_ok());

        let result = validate_uuid("invalid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_metric_code() {
        assert!(validate_metric_code("total_users").is_ok());
        assert!(validate_metric_code("daily_active_users").is_ok());
        assert!(validate_metric_code("2metric").is_err()); // 以数字开头
        assert!(validate_metric_code("metric-name").is_err()); // 包含连字符
    }

    #[test]
    fn test_contains_malicious_content() {
        assert!(!contains_malicious_content("正常的查询内容"));
        assert!(contains_malicious_content("<script>alert(1)</script>"));
        assert!(contains_malicious_content("javascript:alert(1)"));
        assert!(contains_malicious_content("正常内容 <img onerror=alert>"));
    }
}
