//! JWT 工具模块
//!
//! 提供 JWT Token 生成和验证功能

use crate::config::JwtConfig;
use crate::error::{AppError, AppResult};
use crate::models::{TokenClaims, TokenType, UserRole};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

/// JWT 工具
#[derive(Clone)]
pub struct JwtUtils {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expires: u64,
    refresh_token_expires: u64,
    issuer: String,
}

impl JwtUtils {
    /// 创建新的 JWT 工具实例
    pub fn new(config: &JwtConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            access_token_expires: config.access_token_expires,
            refresh_token_expires: config.refresh_token_expires,
            issuer: config.issuer.clone(),
        }
    }

    /// 生成访问令牌
    pub fn generate_access_token(
        &self,
        user_id: &uuid::Uuid,
        username: &str,
        role: UserRole,
    ) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.access_token_expires as i64);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role: role.to_string(),
            iss: self.issuer.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: TokenType::Access,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::internal(format!("Failed to generate access token: {}", e)))
    }

    /// 生成刷新令牌
    pub fn generate_refresh_token(
        &self,
        user_id: &uuid::Uuid,
        username: &str,
        role: UserRole,
    ) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.refresh_token_expires as i64);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role: role.to_string(),
            iss: self.issuer.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: TokenType::Refresh,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::internal(format!("Failed to generate refresh token: {}", e)))
    }

    /// 验证并解码令牌
    pub fn verify_token(&self, token: &str) -> AppResult<TokenClaims> {
        let validation = Validation::default();

        decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                tracing::warn!("Token verification failed: {}", e);
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        AppError::TokenExpired
                    }
                    _ => AppError::InvalidToken("Invalid token".to_string()),
                }
            })
    }

    /// 验证访问令牌
    pub fn verify_access_token(&self, token: &str) -> AppResult<TokenClaims> {
        let claims = self.verify_token(token)?;

        if claims.token_type != TokenType::Access {
            return Err(AppError::InvalidToken(
                "Expected access token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// 验证刷新令牌
    pub fn verify_refresh_token(&self, token: &str) -> AppResult<TokenClaims> {
        let claims = self.verify_token(token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(AppError::InvalidToken(
                "Expected refresh token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// 从令牌中获取用户 ID
    pub fn get_user_id(&self, token: &str) -> AppResult<uuid::Uuid> {
        let claims = self.verify_access_token(token)?;
        uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::InvalidToken("Invalid user ID in token".to_string()))
    }

    /// 获取令牌剩余有效期（秒）
    pub fn get_token_ttl(&self, token: &str) -> AppResult<u64> {
        let claims = self.verify_token(token)?;
        let now = Utc::now().timestamp();
        let remaining = claims.exp - now;

        Ok(if remaining > 0 { remaining as u64 } else { 0 })
    }
}

impl JwtUtils {
    /// 创建用于测试的 JWT 工具（使用固定密钥）
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self::new(&JwtConfig {
            secret: "test-secret-key-for-testing-only".to_string(),
            access_token_expires: 3600,
            refresh_token_expires: 604800,
            issuer: "smartquery-test".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jwt_utils() -> JwtUtils {
        JwtUtils::new_for_test()
    }

    #[test]
    fn test_generate_and_verify_access_token() {
        let jwt = create_test_jwt_utils();
        let user_id = uuid::Uuid::new_v4();

        let token = jwt
            .generate_access_token(&user_id, "testuser", UserRole::Business)
            .unwrap();

        let claims = jwt.verify_access_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "business");
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_generate_and_verify_refresh_token() {
        let jwt = create_test_jwt_utils();
        let user_id = uuid::Uuid::new_v4();

        let token = jwt
            .generate_refresh_token(&user_id, "testuser", UserRole::Admin)
            .unwrap();

        let claims = jwt.verify_refresh_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_invalid_token() {
        let jwt = create_test_jwt_utils();

        let result = jwt.verify_access_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_type_mismatch() {
        let jwt = create_test_jwt_utils();
        let user_id = uuid::Uuid::new_v4();

        // 生成刷新令牌，但尝试作为访问令牌验证
        let refresh_token = jwt
            .generate_refresh_token(&user_id, "testuser", UserRole::Business)
            .unwrap();

        let result = jwt.verify_access_token(&refresh_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_user_id() {
        let jwt = create_test_jwt_utils();
        let user_id = uuid::Uuid::new_v4();

        let token = jwt
            .generate_access_token(&user_id, "testuser", UserRole::Business)
            .unwrap();

        let extracted_id = jwt.get_user_id(&token).unwrap();
        assert_eq!(extracted_id, user_id);
    }

    #[test]
    fn test_generate_token_pair() {
        let jwt = create_test_jwt_utils();
        let user_id = uuid::Uuid::new_v4();

        let access_token = jwt
            .generate_access_token(&user_id, "testuser", UserRole::Analyst)
            .unwrap();

        let refresh_token = jwt
            .generate_refresh_token(&user_id, "testuser", UserRole::Analyst)
            .unwrap();

        assert_ne!(access_token, refresh_token);

        // 验证两个令牌都有效
        assert!(jwt.verify_access_token(&access_token).is_ok());
        assert!(jwt.verify_refresh_token(&refresh_token).is_ok());
    }
}
