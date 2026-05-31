//! 用户模型
//!
//! 定义用户和角色相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, TypeInfo};
use uuid::Uuid;
use validator::Validate;

/// 用户角色枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// 管理员 - 全部权限
    Admin,
    /// 数据分析师 - 连接只读、图表生成、导出
    Analyst,
    /// 开发人员 - SQL模式、连接只读
    Developer,
    /// 业务人员 - 对话模式、预定义报表
    Business,
}

impl UserRole {
    /// 从字符串转换为角色枚举
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Self::Admin),
            "analyst" => Some(Self::Analyst),
            "developer" => Some(Self::Developer),
            "business" => Some(Self::Business),
            _ => None,
        }
    }

    /// 转换为数据库存储字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Analyst => "analyst",
            Self::Developer => "developer",
            Self::Business => "business",
        }
    }

    /// 检查是否具有管理员权限
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }

    /// 检查是否可以使用 SQL 模式
    pub fn can_use_sql_mode(&self) -> bool {
        matches!(self, Self::Admin | Self::Analyst | Self::Developer)
    }

    /// 检查是否可以使用对话模式
    pub fn can_use_chat_mode(&self) -> bool {
        matches!(self, Self::Admin | Self::Analyst | Self::Business)
    }

    /// 检查是否可以管理连接
    pub fn can_manage_connections(&self) -> bool {
        matches!(self, Self::Admin | Self::Developer)
    }

    /// 检查是否可以生成图表
    pub fn can_generate_charts(&self) -> bool {
        matches!(self, Self::Admin | Self::Analyst)
    }

    /// 检查是否可以导出数据
    pub fn can_export_data(&self) -> bool {
        matches!(self, Self::Admin | Self::Analyst)
    }

    /// 检查是否可以管理用户
    pub fn can_manage_users(&self) -> bool {
        matches!(self, Self::Admin)
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Business
    }
}

// sqlx 支持
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for UserRole {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s: &'r str = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        UserRole::from_str(s)
            .ok_or_else(|| format!("Invalid role: {}", s).into())
    }
}

impl sqlx::Type<sqlx::Postgres> for UserRole {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        let name = ty.name();
        name == "VARCHAR" || name == "TEXT"
    }
}

/// 用户实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    /// 用户 ID (UUID)
    pub id: Uuid,

    /// 用户名 (唯一)
    pub username: String,

    /// 邮箱 (唯一)
    pub email: String,

    /// 密码哈希
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// 用户角色
    pub role: UserRole,

    /// 是否激活
    pub is_active: bool,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// 创建新用户
    pub fn new(username: String, email: String, password_hash: String, role: UserRole) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            role,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 用户公开信息（不包含敏感字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&User> for UserPublic {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        Self::from(&user)
    }
}

/// 用户创建请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50, message = "用户名长度必须在 3-50 个字符之间"))]
    pub username: String,

    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    #[validate(length(min = 8, max = 128, message = "密码长度必须在 8-128 个字符之间"))]
    pub password: String,

    #[serde(default)]
    pub role: UserRole,
}

/// 用户更新请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 50, message = "用户名长度必须在 3-50 个字符之间"))]
    pub username: Option<String>,

    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,

    pub role: Option<UserRole>,

    pub is_active: Option<bool>,
}

/// 密码修改请求
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 8, max = 128, message = "密码长度必须在 8-128 个字符之间"))]
    pub old_password: String,

    #[validate(length(min = 8, max = 128, message = "新密码长度必须在 8-128 个字符之间"))]
    pub new_password: String,
}

/// 登录请求
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "用户名不能为空"))]
    pub username: String,

    #[validate(length(min = 1, message = "密码不能为空"))]
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub user: UserPublic,
}

/// 认证令牌 Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    /// 用户 ID
    pub sub: String,

    /// 用户名
    pub username: String,

    /// 角色
    pub role: String,

    /// 签发者
    pub iss: String,

    /// 过期时间 (Unix 时间戳)
    pub exp: i64,

    /// 签发时间 (Unix 时间戳)
    pub iat: i64,

    /// Token 类型
    pub token_type: TokenType,
}

/// Token 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// 访问令牌
    Access,
    /// 刷新令牌
    Refresh,
}

/// 用户会话信息
#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
}

impl From<&User> for UserSession {
    fn from(user: &User) -> Self {
        Self {
            user_id: user.id,
            username: user.username.clone(),
            role: user.role,
            permissions: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_permissions() {
        assert!(UserRole::Admin.is_admin());
        assert!(!UserRole::Analyst.is_admin());

        assert!(UserRole::Admin.can_use_sql_mode());
        assert!(UserRole::Developer.can_use_sql_mode());
        assert!(!UserRole::Business.can_use_sql_mode());

        assert!(UserRole::Admin.can_use_chat_mode());
        assert!(UserRole::Business.can_use_chat_mode());
        assert!(!UserRole::Developer.can_use_chat_mode());
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            UserRole::Business,
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
    }
}
