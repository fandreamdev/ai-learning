//! 数据库连接模型
//!
//! 定义数据库连接相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 支持的数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    /// MySQL
    Mysql,
    /// PostgreSQL
    Postgresql,
    /// ClickHouse
    Clickhouse,
    /// SQLite
    Sqlite,
}

impl DatabaseType {
    /// 从字符串转换为数据库类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mysql" => Some(Self::Mysql),
            "postgresql" | "postgres" => Some(Self::Postgresql),
            "clickhouse" => Some(Self::Clickhouse),
            "sqlite" => Some(Self::Sqlite),
            _ => None,
        }
    }

    /// 转换为数据库存储字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mysql => "mysql",
            Self::Postgresql => "postgresql",
            Self::Clickhouse => "clickhouse",
            Self::Sqlite => "sqlite",
        }
    }

    /// 获取默认端口
    pub fn default_port(&self) -> u16 {
        match self {
            Self::Mysql => 3306,
            Self::Postgresql => 5432,
            Self::Clickhouse => 8123,
            Self::Sqlite => 0,
        }
    }

    /// 获取 SQL 方言名称 (用于 sqlparser)
    pub fn sql_dialect(&self) -> &'static str {
        match self {
            Self::Mysql => "MySqlDialect",
            Self::Postgresql => "PostgreSqlDialect",
            Self::Clickhouse => "ClickHouseDialect",
            Self::Sqlite => "SQLiteDialect",
        }
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for DatabaseType {
    fn default() -> Self {
        Self::Mysql
    }
}

/// 数据库连接实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DatabaseConnection {
    /// 连接 ID (UUID)
    pub id: Uuid,

    /// 连接名称
    pub name: String,

    /// 数据库类型
    pub db_type: DatabaseType,

    /// 主机地址
    pub host: String,

    /// 端口
    pub port: i32,

    /// 数据库名称
    pub database_name: String,

    /// 用户名
    pub username: String,

    /// 加密后的密码
    #[serde(skip_serializing)]
    pub encrypted_password: String,

    /// 是否为默认连接
    pub is_default: bool,

    /// 创建者 ID
    pub created_by: Option<Uuid>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl DatabaseConnection {
    /// 创建新的数据库连接
    pub fn new(
        name: String,
        db_type: DatabaseType,
        host: String,
        port: i32,
        database_name: String,
        username: String,
        encrypted_password: String,
        created_by: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            db_type,
            host,
            port,
            database_name,
            username,
            encrypted_password,
            is_default: false,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// 获取连接字符串（不含密码）
    pub fn connection_string(&self) -> String {
        match self.db_type {
            DatabaseType::Mysql => format!(
                "mysql://{}@{}:{}/{}",
                self.username, self.host, self.port, self.database_name
            ),
            DatabaseType::Postgresql => format!(
                "postgresql://{}@{}:{}/{}",
                self.username, self.host, self.port, self.database_name
            ),
            DatabaseType::Clickhouse => format!(
                "clickhouse://{}@{}:{}/{}",
                self.username, self.host, self.port, self.database_name
            ),
            DatabaseType::Sqlite => self.database_name.clone(),
        }
    }
}

/// 连接公开信息（不包含密码）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPublic {
    pub id: Uuid,
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&DatabaseConnection> for ConnectionPublic {
    fn from(conn: &DatabaseConnection) -> Self {
        Self {
            id: conn.id,
            name: conn.name.clone(),
            db_type: conn.db_type,
            host: conn.host.clone(),
            port: conn.port,
            database_name: conn.database_name.clone(),
            username: conn.username.clone(),
            is_default: conn.is_default,
            created_at: conn.created_at,
        }
    }
}

/// 创建连接请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateConnectionRequest {
    #[validate(length(min = 1, max = 100, message = "连接名称长度必须在 1-100 个字符之间"))]
    pub name: String,

    pub db_type: DatabaseType,

    #[validate(length(min = 1, max = 255, message = "主机地址不能为空"))]
    pub host: String,

    #[validate(range(min = 1, max = 65535, message = "端口必须在 1-65535 之间"))]
    pub port: i32,

    #[validate(length(min = 1, max = 100, message = "数据库名称不能为空"))]
    pub database_name: String,

    #[validate(length(min = 1, max = 100, message = "用户名不能为空"))]
    pub username: String,

    #[validate(length(min = 1, max = 255, message = "密码不能为空"))]
    pub password: String,

    #[serde(default)]
    pub is_default: bool,
}

/// 更新连接请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateConnectionRequest {
    #[validate(length(min = 1, max = 100, message = "连接名称长度必须在 1-100 个字符之间"))]
    pub name: Option<String>,

    pub db_type: Option<DatabaseType>,

    #[validate(length(min = 1, max = 255, message = "主机地址不能为空"))]
    pub host: Option<String>,

    #[validate(range(min = 1, max = 65535, message = "端口必须在 1-65535 之间"))]
    pub port: Option<i32>,

    #[validate(length(min = 1, max = 100, message = "数据库名称不能为空"))]
    pub database_name: Option<String>,

    #[validate(length(min = 1, max = 100, message = "用户名不能为空"))]
    pub username: Option<String>,

    pub password: Option<String>,

    pub is_default: Option<bool>,
}

/// 连接测试请求
#[derive(Debug, Deserialize)]
pub struct TestConnectionRequest {
    pub db_type: DatabaseType,
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub password: String,
}

/// 连接测试响应
#[derive(Debug, Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
    pub server_version: Option<String>,
    pub latency_ms: Option<i64>,
}

/// 表结构信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub table_schema: Option<String>,
    pub table_type: String,
    pub row_count: Option<i64>,
}

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub index_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
}

/// 数据库 schema 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub connection_id: Uuid,
    pub tables: Vec<TableInfo>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_default_port() {
        assert_eq!(DatabaseType::Mysql.default_port(), 3306);
        assert_eq!(DatabaseType::Postgresql.default_port(), 5432);
        assert_eq!(DatabaseType::Clickhouse.default_port(), 8123);
    }

    #[test]
    fn test_connection_creation() {
        let conn = DatabaseConnection::new(
            "Test MySQL".to_string(),
            DatabaseType::Mysql,
            "localhost".to_string(),
            3306,
            "testdb".to_string(),
            "root".to_string(),
            "encrypted_pass".to_string(),
            None,
        );

        assert_eq!(conn.name, "Test MySQL");
        assert_eq!(conn.db_type, DatabaseType::Mysql);
        assert!(!conn.is_default);
    }
}
