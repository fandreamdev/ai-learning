//! 连接仓储
//!
//! 数据库连接数据的数据库操作

use crate::error::AppResult;
use crate::models::{DatabaseConnection, DatabaseType};
use sqlx::PgPool;
use uuid::Uuid;

/// 连接仓储
#[derive(Clone)]
pub struct ConnectionRepo {
    pool: PgPool,
}

impl ConnectionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 查找连接
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<DatabaseConnection>> {
        let conn = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, name, db_type, host, port, database_name, username, encrypted_password,
                   is_default, created_by, created_at, updated_at
            FROM database_connections
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(conn.map(|r| r.into()))
    }

    /// 根据用户 ID 列出所有连接
    pub async fn list_by_user(&self, user_id: Uuid) -> AppResult<Vec<DatabaseConnection>> {
        let conns = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, name, db_type, host, port, database_name, username, encrypted_password,
                   is_default, created_by, created_at, updated_at
            FROM database_connections
            WHERE created_by = $1 OR created_by IS NULL
            ORDER BY is_default DESC, name ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(conns.into_iter().map(|r| r.into()).collect())
    }

    /// 获取默认连接
    pub async fn get_default(&self, user_id: Uuid) -> AppResult<Option<DatabaseConnection>> {
        let conn = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, name, db_type, host, port, database_name, username, encrypted_password,
                   is_default, created_by, created_at, updated_at
            FROM database_connections
            WHERE (created_by = $1 OR created_by IS NULL) AND is_default = TRUE
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(conn.map(|r| r.into()))
    }

    /// 创建连接
    pub async fn create(&self, conn: &DatabaseConnection) -> AppResult<DatabaseConnection> {
        let row = sqlx::query_as::<_, ConnectionRow>(
            r#"
            INSERT INTO database_connections
                (id, name, db_type, host, port, database_name, username, encrypted_password,
                 is_default, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, name, db_type, host, port, database_name, username, encrypted_password,
                      is_default, created_by, created_at, updated_at
            "#,
        )
        .bind(conn.id)
        .bind(&conn.name)
        .bind(conn.db_type.as_str())
        .bind(&conn.host)
        .bind(conn.port)
        .bind(&conn.database_name)
        .bind(&conn.username)
        .bind(&conn.encrypted_password)
        .bind(conn.is_default)
        .bind(conn.created_by)
        .bind(conn.created_at)
        .bind(conn.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// 更新连接
    pub async fn update(&self, conn: &DatabaseConnection) -> AppResult<DatabaseConnection> {
        let row = sqlx::query_as::<_, ConnectionRow>(
            r#"
            UPDATE database_connections
            SET name = $2, db_type = $3, host = $4, port = $5, database_name = $6,
                username = $7, encrypted_password = $8, is_default = $9, updated_at = $10
            WHERE id = $1
            RETURNING id, name, db_type, host, port, database_name, username, encrypted_password,
                      is_default, created_by, created_at, updated_at
            "#,
        )
        .bind(conn.id)
        .bind(&conn.name)
        .bind(conn.db_type.as_str())
        .bind(&conn.host)
        .bind(conn.port)
        .bind(&conn.database_name)
        .bind(&conn.username)
        .bind(&conn.encrypted_password)
        .bind(conn.is_default)
        .bind(conn.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// 删除连接
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM database_connections WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 设置默认连接
    pub async fn set_default(&self, id: Uuid, user_id: Uuid) -> AppResult<()> {
        // 先取消所有默认
        sqlx::query(
            "UPDATE database_connections SET is_default = FALSE WHERE created_by = $1",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // 设置新的默认
        sqlx::query("UPDATE database_connections SET is_default = TRUE WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 清除用户的所有默认连接
    pub async fn clear_default_for_user(&self, user_id: Option<Uuid>) -> AppResult<()> {
        if let Some(uid) = user_id {
            sqlx::query(
                "UPDATE database_connections SET is_default = FALSE WHERE created_by = $1",
            )
            .bind(uid)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// 分页获取连接列表
    pub async fn list_paginated(&self, page: i32, page_size: i32) -> AppResult<Vec<DatabaseConnection>> {
        let offset = (page - 1) * page_size;

        let conns = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, name, db_type, host, port, database_name, username, encrypted_password,
                   is_default, created_by, created_at, updated_at
            FROM database_connections
            ORDER BY is_default DESC, name ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(conns.into_iter().map(|r| r.into()).collect())
    }
}

/// 连接行（数据库映射）
#[derive(Debug, sqlx::FromRow)]
struct ConnectionRow {
    id: Uuid,
    name: String,
    db_type: String,
    host: String,
    port: i32,
    database_name: String,
    username: String,
    encrypted_password: String,
    is_default: bool,
    created_by: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ConnectionRow> for DatabaseConnection {
    fn from(row: ConnectionRow) -> Self {
        DatabaseConnection {
            id: row.id,
            name: row.name,
            db_type: DatabaseType::from_str(&row.db_type).unwrap_or(DatabaseType::Mysql),
            host: row.host,
            port: row.port,
            database_name: row.database_name,
            username: row.username,
            encrypted_password: row.encrypted_password,
            is_default: row.is_default,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
