//! 用户仓储
//!
//! 用户数据的数据库操作

use crate::error::AppResult;
use crate::models::{User, UserRole};
use sqlx::PgPool;
use uuid::Uuid;

/// 用户仓储
#[derive(Clone)]
pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 查找用户
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|r| r.into()))
    }

    /// 根据用户名查找用户
    pub async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|r| r.into()))
    }

    /// 根据邮箱查找用户
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|r| r.into()))
    }

    /// 创建用户
    pub async fn create(&self, user: &User) -> AppResult<User> {
        let role_str = user.role.as_str();

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, username, email, password_hash, role, is_active, created_at, updated_at
            "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(role_str)
        .bind(user.is_active)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// 更新用户
    pub async fn update(&self, user: &User) -> AppResult<User> {
        let role_str = user.role.as_str();

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users
            SET username = $2, email = $3, role = $4, is_active = $5, updated_at = $6
            WHERE id = $1
            RETURNING id, username, email, password_hash, role, is_active, created_at, updated_at
            "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(role_str)
        .bind(user.is_active)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// 删除用户
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn active_admin_count(&self) -> AppResult<i64> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE role = 'admin' AND is_active = TRUE",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(total.0)
    }

    /// 更新密码
    pub async fn update_password(&self, user_id: Uuid, password_hash: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(password_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 列出所有用户（分页）
    pub async fn list(&self, page: i32, page_size: i32) -> AppResult<(Vec<User>, i64)> {
        let offset = (page - 1) * page_size;

        let users = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        Ok((users.into_iter().map(|r| r.into()).collect(), total.0))
    }
}

/// 用户行（数据库映射）
#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    role: String,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            role: UserRole::from_str(&row.role).unwrap_or(UserRole::Business),
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
