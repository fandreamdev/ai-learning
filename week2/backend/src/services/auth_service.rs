//! 认证服务
//!
//! 提供用户认证相关功能

use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use crate::models::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, User, UserPublic,
    UserRole, UserSession,
};
use crate::repositories::UserRepo;
use crate::utils::{JwtUtils, PasswordUtils};
use sqlx::PgPool;
use std::sync::Arc;

/// 认证服务
#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepo,
    jwt_utils: JwtUtils,
    password_utils: PasswordUtils,
}

impl AuthService {
    pub fn new(config: Arc<AppConfig>, pool: PgPool) -> Self {
        Self {
            user_repo: UserRepo::new(pool),
            jwt_utils: JwtUtils::new(&config.jwt),
            password_utils: PasswordUtils::new(&config.security),
        }
    }

    /// 用户注册
    pub async fn register(&self, request: CreateUserRequest) -> AppResult<UserPublic> {
        // 检查用户名是否存在
        if self
            .user_repo
            .find_by_username(&request.username)
            .await?
            .is_some()
        {
            return Err(AppError::AlreadyExists("用户名已存在".to_string()));
        }

        // 检查邮箱是否存在
        if self
            .user_repo
            .find_by_email(&request.email)
            .await?
            .is_some()
        {
            return Err(AppError::AlreadyExists("邮箱已被使用".to_string()));
        }

        // 哈希密码
        let password_hash = self.password_utils.hash_password(&request.password)?;

        // 创建用户
        let user = User::new(request.username, request.email, password_hash, request.role);

        let user = self.user_repo.create(&user).await?;

        Ok(UserPublic::from(&user))
    }

    /// 用户登录
    pub async fn login(&self, request: LoginRequest) -> AppResult<LoginResponse> {
        // 查找用户
        let user = self
            .user_repo
            .find_by_username(&request.username)
            .await?
            .ok_or_else(|| AppError::AuthenticationFailed("用户名或密码错误".to_string()))?;

        // 验证密码
        if !self
            .password_utils
            .verify_password(&request.password, &user.password_hash)
        {
            return Err(AppError::AuthenticationFailed(
                "用户名或密码错误".to_string(),
            ));
        }

        // 检查用户是否激活
        if !user.is_active {
            return Err(AppError::Forbidden("账号已被禁用".to_string()));
        }

        // 生成 Token
        let access_token =
            self.jwt_utils
                .generate_access_token(&user.id, &user.username, user.role)?;
        let refresh_token =
            self.jwt_utils
                .generate_refresh_token(&user.id, &user.username, user.role)?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            expires_in: 3600, // 1小时
            token_type: "Bearer".to_string(),
            user: UserPublic::from(&user),
        })
    }

    /// 刷新 Token
    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<LoginResponse> {
        // 验证刷新令牌
        let claims = self.jwt_utils.verify_refresh_token(refresh_token)?;

        // 获取用户
        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::InvalidToken("Invalid user ID".to_string()))?;

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        // 检查用户是否激活
        if !user.is_active {
            return Err(AppError::Forbidden("账号已被禁用".to_string()));
        }

        // 生成新 Token
        let new_access_token =
            self.jwt_utils
                .generate_access_token(&user.id, &user.username, user.role)?;
        let new_refresh_token =
            self.jwt_utils
                .generate_refresh_token(&user.id, &user.username, user.role)?;

        Ok(LoginResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            user: UserPublic::from(&user),
        })
    }

    /// 验证 Token 并获取用户会话
    pub fn verify_token(&self, token: &str) -> AppResult<UserSession> {
        let claims = self.jwt_utils.verify_access_token(token)?;

        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::InvalidToken("Invalid user ID".to_string()))?;

        let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Business);

        Ok(UserSession {
            user_id,
            username: claims.username,
            role,
            permissions: vec![],
        })
    }

    /// 修改密码
    pub async fn change_password(
        &self,
        user_id: uuid::Uuid,
        request: ChangePasswordRequest,
    ) -> AppResult<()> {
        // 获取用户
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        // 验证旧密码
        if !self
            .password_utils
            .verify_password(&request.old_password, &user.password_hash)
        {
            return Err(AppError::validation("旧密码不正确".to_string()));
        }

        // 哈希新密码
        let new_hash = self.password_utils.hash_password(&request.new_password)?;

        // 更新密码
        self.user_repo.update_password(user_id, &new_hash).await?;

        Ok(())
    }
}

/// 用户仓储
pub mod repositories {
    use crate::error::AppResult;
    use crate::models::User;
    use sqlx::PgPool;
    use uuid::Uuid;

    pub struct UserRepository {
        pool: PgPool,
    }

    impl UserRepository {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }

        pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
            let user = sqlx::query_as::<_, User>(
                r#"
                SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
                FROM users
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(user)
        }

        pub async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
            let user = sqlx::query_as::<_, User>(
                r#"
                SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
                FROM users
                WHERE username = $1
                "#,
            )
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

            Ok(user)
        }

        pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
            let user = sqlx::query_as::<_, User>(
                r#"
                SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
                FROM users
                WHERE email = $1
                "#,
            )
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

            Ok(user)
        }

        pub async fn create(&self, user: User) -> AppResult<User> {
            let role_str = user.role.as_str();

            let created_user = sqlx::query_as::<_, User>(
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

            Ok(created_user)
        }

        pub async fn update_password(&self, user_id: Uuid, new_hash: &str) -> AppResult<()> {
            sqlx::query(
                r#"
                UPDATE users
                SET password_hash = $1, updated_at = NOW()
                WHERE id = $2
                "#,
            )
            .bind(new_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength_check() {
        assert!(!PasswordUtils::check_password_strength("123").meets_minimum());
        assert!(PasswordUtils::check_password_strength("Password123!").meets_minimum());
    }
}
