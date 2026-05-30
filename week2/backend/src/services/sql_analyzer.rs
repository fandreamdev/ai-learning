//! SQL AST 安全分析器
//!
//! 使用 sqlparser 分析 SQL 语句的安全性

use crate::config::SecurityConfig;
use crate::error::{AppError, AppResult};
use crate::models::{DangerousFunctionCheck, SqlAnalysisResult, SqlType};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{
    AlterOperation, AlterTableOperation, Statement, Visitor,
};
use sqlparser::dialect::{GenericDialect, MySqlDialect, PostgreSqlDialect};
use sqlparser::parser::Parser;
use std::collections::HashSet;
use std::sync::Arc;

/// SQL 安全分析器
#[derive(Clone)]
pub struct SqlAnalyzer {
    config: Arc<SecurityConfig>,
}

impl SqlAnalyzer {
    pub fn new(config: Arc<SecurityConfig>) -> Self {
        Self { config }
    }

    /// 分析 SQL 语句的安全性
    pub fn analyze(&self, sql: &str) -> AppResult<SqlAnalysisResult> {
        // 检查长度
        if sql.len() > self.config.sql.max_query_length {
            return Err(AppError::validation(format!(
                "SQL 语句长度不能超过 {} 个字符",
                self.config.sql.max_query_length
            )));
        }

        // 解析 SQL
        let dialect = GenericDialect {};
        let statements = Parser::parse_sql(&dialect, sql)
            .map_err(|e| AppError::validation(format!("SQL 解析失败: {}", e)))?;

        if statements.is_empty() {
            return Err(AppError::validation("SQL 语句不能为空"));
        }

        let mut warnings = Vec::new();
        let mut referenced_tables = Vec::new();
        let mut referenced_columns = Vec::new();
        let mut blocked_reason = None;
        let mut sql_type = SqlType::Other;

        for statement in &statements {
            // 分析 SQL 类型
            sql_type = self.classify_statement(statement);

            // 检查是否包含禁止的操作
            if let Some(reason) = self.check_forbidden_operations(statement) {
                blocked_reason = Some(reason);
                break;
            }

            // 检查危险函数
            let func_checks = self.check_dangerous_functions(sql);
            for check in func_checks {
                if check.detected {
                    warnings.push(format!("检测到危险函数: {}", check.function_name));
                }
            }

            // 提取引用的表和列
            let (tables, columns) = self.extract_references(statement);
            referenced_tables.extend(tables);
            referenced_columns.extend(columns);
        }

        // 去重
        referenced_tables.sort();
        referenced_tables.dedup();
        referenced_columns.sort();
        referenced_columns.dedup();

        Ok(SqlAnalysisResult {
            is_safe: blocked_reason.is_none(),
            blocked_reason,
            warnings,
            sql_type,
            referenced_tables,
            referenced_columns,
        })
    }

    /// 分类 SQL 语句类型
    fn classify_statement(&self, statement: &Statement) -> SqlType {
        match statement {
            Statement::Query(_) => SqlType::Select,
            Statement::Insert(_) => SqlType::Insert,
            Statement::Update(_) => SqlType::Update,
            Statement::Delete(_) => SqlType::Delete,
            Statement::CreateTable(_)
            | Statement::Drop(_)
            | Statement::AlterTable(_)
            | Statement::Truncate(_) => SqlType::Ddl,
            _ => SqlType::Other,
        }
    }

    /// 检查禁止的操作
    fn check_forbidden_operations(&self, statement: &Statement) -> Option<String> {
        // DDL 操作
        match statement {
            Statement::Insert(_) => {
                if !self.config.sql.allow_dangerous_functions {
                    return Some("禁止执行 INSERT 操作".to_string());
                }
            }
            Statement::Update(_) => {
                if !self.config.sql.allow_dangerous_functions {
                    return Some("禁止执行 UPDATE 操作".to_string());
                }
            }
            Statement::Delete(_) => {
                if !self.config.sql.allow_dangerous_functions {
                    return Some("禁止执行 DELETE 操作".to_string());
                }
            }
            Statement::Drop(_) => {
                return Some("禁止执行 DROP 操作".to_string());
            }
            Statement::Truncate(_) => {
                return Some("禁止执行 TRUNCATE 操作".to_string());
            }
            Statement::AlterTable(alter) => {
                return Some(format!("禁止执行 ALTER TABLE 操作"));
            }
            Statement::CreateTable(create) => {
                if create.or_replace.is_some() {
                    return Some("禁止使用 OR REPLACE".to_string());
                }
            }
            _ => {}
        }

        // 检查危险函数
        let func_checks = self.check_dangerous_functions(&statement.to_string());
        for check in func_checks {
            if check.detected {
                return Some(format!("检测到危险函数: {}", check.function_name));
            }
        }

        None
    }

    /// 检查危险函数
    pub fn check_dangerous_functions(&self, sql: &str) -> Vec<DangerousFunctionCheck> {
        if self.config.sql.allow_dangerous_functions {
            return vec![];
        }

        let mut results = Vec::new();

        for func_name in &self.config.sql.blocked_functions {
            let pattern = Regex::new(&format!(r"(?i)\b{}\s*\(", func_name)).unwrap();

            let mut check = DangerousFunctionCheck {
                function_name: func_name.clone(),
                detected: false,
                position: None,
            };

            if let Some(m) = pattern.find(sql) {
                check.detected = true;
                check.position = Some((m.start(), m.end()));
            }

            results.push(check);
        }

        results
    }

    /// 提取引用的表和列
    fn extract_references(&self, statement: &Statement) -> (Vec<String>, Vec<String>) {
        let mut tables = Vec::new();
        let mut columns = Vec::new();

        struct TableColumnVisitor {
            tables: Vec<String>,
            columns: Vec<String>,
        }

        impl Visitor for TableColumnVisitor {
            type Error = std::convert::Infallible;

            fn visit_table(&mut self, table: &sqlparser::ast::Table) -> Result<(), Self::Error> {
                if let Some(table_name) = &table.name {
                    self.tables.push(table_name.to_string());
                }
                Ok(())
            }

            fn visit_column(&mut self, column: &sqlparser::ast::Column) -> Result<(), Self::Error> {
                self.columns.push(column.to_string());
                Ok(())
            }
        }

        let mut visitor = TableColumnVisitor {
            tables: vec![],
            columns: vec![],
        };

        visitor.visit_statement(statement).ok();

        (visitor.tables, visitor.columns)
    }

    /// 快速检查（只检查类型，不提取详细信息）
    pub fn quick_check(&self, sql: &str) -> AppResult<SqlType> {
        let dialect = GenericDialect {};
        let statements = Parser::parse_sql(&dialect, sql)
            .map_err(|e| AppError::validation(format!("SQL 解析失败: {}", e)))?;

        if statements.is_empty() {
            return Err(AppError::validation("SQL 语句不能为空"));
        }

        let statement = &statements[0];

        let sql_type = match statement {
            Statement::Query(_) => SqlType::Select,
            Statement::Insert(_) => SqlType::Insert,
            Statement::Update(_) => SqlType::Update,
            Statement::Delete(_) => SqlType::Delete,
            Statement::CreateTable(_)
            | Statement::Drop(_)
            | Statement::AlterTable(_)
            | Statement::Truncate(_) => SqlType::Ddl,
            _ => SqlType::Other,
        };

        // 检查是否允许
        if !sql_type.is_readonly() && !self.config.sql.allow_dangerous_functions {
            return Err(AppError::DmlForbidden(format!(
                "Only SELECT queries are allowed, but found {:?}",
                sql_type
            )));
        }

        Ok(sql_type)
    }

    /// 检查 SQL 注入特征
    pub fn check_injection(&self, sql: &str) -> bool {
        let injection_patterns = [
            r"(?i)union\s+select",
            r"(?i)into\s+outfile",
            r"(?i)load\s+data",
            r"(?i)--\s*$",
            r"(?i);\s*drop",
            r"(?i);\s*delete",
            r"(?i);\s*insert",
            r"(?i);\s*update",
            r"(?i)exec\s*\(",
            r"(?i)execute\s*\(",
            r"'\s*or\s+'1'\s*=\s*'1",
            r"'\s*or\s+1\s*=\s*1",
        ];

        let lower_sql = sql.to_lowercase();

        for pattern in injection_patterns {
            if Regex::new(pattern).unwrap().is_match(&lower_sql) {
                return true;
            }
        }

        false
    }
}

/// SQL 安全检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheckResult {
    pub is_safe: bool,
    pub violations: Vec<SecurityViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    pub violation_type: String,
    pub description: String,
    pub severity: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_analyzer() -> SqlAnalyzer {
        SqlAnalyzer::new(Arc::new(SecurityConfig {
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
                blocked_functions: vec![
                    "LOAD_FILE".to_string(),
                    "INTO OUTFILE".to_string(),
                    "BENCHMARK".to_string(),
                    "SLEEP".to_string(),
                ],
            },
        }))
    }

    #[test]
    fn test_safe_select() {
        let analyzer = create_test_analyzer();
        let result = analyzer.analyze("SELECT id, name FROM users WHERE status = 1");

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.is_safe);
        assert_eq!(analysis.sql_type, SqlType::Select);
    }

    #[test]
    fn test_block_insert() {
        let analyzer = create_test_analyzer();
        let result = analyzer.analyze("INSERT INTO users (name) VALUES ('test')");

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(!analysis.is_safe);
        assert!(analysis.blocked_reason.is_some());
    }

    #[test]
    fn test_block_delete() {
        let analyzer = create_test_analyzer();
        let result = analyzer.analyze("DELETE FROM users WHERE id = 1");

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(!analysis.is_safe);
    }

    #[test]
    fn test_block_drop() {
        let analyzer = create_test_analyzer();
        let result = analyzer.analyze("DROP TABLE users");

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(!analysis.is_safe);
    }

    #[test]
    fn test_dangerous_function() {
        let analyzer = create_test_analyzer();
        let result = analyzer.analyze("SELECT SLEEP(5)");

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(!analysis.is_safe);
        assert!(analysis.warnings.contains(&"检测到危险函数: SLEEP".to_string()));
    }

    #[test]
    fn test_injection_detection() {
        let analyzer = create_test_analyzer();

        assert!(analyzer.check_injection("' OR '1'='1"));
        assert!(analyzer.check_injection("; DROP TABLE users;--"));
        assert!(analyzer.check_injection("UNION SELECT * FROM passwords"));

        assert!(!analyzer.check_injection("SELECT * FROM users WHERE id = 1"));
        assert!(!analyzer.check_injection("SELECT name FROM users WHERE status = 1"));
    }

    #[test]
    fn test_extract_references() {
        let analyzer = create_test_analyzer();
        let result = analyzer.analyze("SELECT u.id, u.name, o.order_no FROM users u JOIN orders o ON u.id = o.user_id");

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.referenced_tables.contains(&"users".to_string()));
        assert!(analysis.referenced_tables.contains(&"orders".to_string()));
    }
}
