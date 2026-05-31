//! 认证中间件
//!
//! 提供 JWT 认证和权限检查功能

use crate::error::{AppError, AppResult};
use crate::models::{UserRole, UserSession};
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// JWT 认证中间件
///
/// 从请求头中提取并验证 JWT Token
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::AuthenticationFailed("Missing Authorization header".to_string()))?;

    // 提取 Bearer Token
    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(AppError::AuthenticationFailed(
            "Invalid Authorization header format".to_string(),
        ));
    };

    if state.token_blacklist.contains(token).await {
        return Err(AppError::InvalidToken("Token has been revoked".to_string()));
    }

    // 创建临时 JwtUtils 用于验证
    let jwt_utils = crate::utils::JwtUtils::new(&state.config.jwt);

    // 验证 Token
    let claims = jwt_utils
        .verify_access_token(token)
        .map_err(|_| AppError::TokenExpired)?;

    // 提取用户信息
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InvalidToken("Invalid user ID in token".to_string()))?;

    let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Business);

    // 创建用户会话
    let session = UserSession {
        user_id,
        username: claims.username,
        role,
        permissions: vec![],
    };

    // 将用户会话注入请求扩展
    request.extensions_mut().insert(session);

    Ok(next.run(request).await)
}

/// 可选的认证中间件
///
/// 如果请求包含 Token 则验证，否则继续执行
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    if let Some(auth_header) = auth_header {
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..];
            let jwt_utils = crate::utils::JwtUtils::new(&state.config.jwt);

            if let Ok(claims) = jwt_utils.verify_access_token(token) {
                if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
                    if let Some(role) = UserRole::from_str(&claims.role) {
                        let session = UserSession {
                            user_id,
                            username: claims.username,
                            role,
                            permissions: vec![],
                        };
                        request.extensions_mut().insert(session);
                    }
                }
            }
        }
    }

    next.run(request).await
}

/// 角色检查中间件工厂
///
/// 创建一个检查用户角色的中间件
pub fn require_role(required_role: UserRole) -> impl Fn(State<Arc<AppState>>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>> {
    move |State(_state): State<Arc<AppState>>, request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            let session = request
                .extensions()
                .get::<UserSession>()
                .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

            // 检查角色权限
            if !has_permission(&session.role, &required_role) {
                return Err(AppError::Forbidden(format!(
                    "Requires {} role, but user has {} role",
                    required_role, session.role
                )));
            }

            Ok(next.run(request).await)
        })
    }
}

/// 检查角色权限
fn has_permission(user_role: &UserRole, required_role: &UserRole) -> bool {
    // 管理员拥有所有权限
    if user_role.is_admin() {
        return true;
    }

    // 其他角色按等级比较
    let user_level = role_level(user_role);
    let required_level = role_level(required_role);

    user_level >= required_level
}

/// 获取角色等级
fn role_level(role: &UserRole) -> u8 {
    match role {
        UserRole::Admin => 100,
        UserRole::Analyst => 50,
        UserRole::Developer => 30,
        UserRole::Business => 10,
    }
}

/// 管理员检查中间件
pub async fn admin_only(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let session = request
        .extensions()
        .get::<UserSession>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    if !session.role.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    Ok(next.run(request).await)
}

/// SQL 模式检查中间件
pub async fn require_sql_mode(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let session = request
        .extensions()
        .get::<UserSession>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    if !session.role.can_use_sql_mode() {
        return Err(AppError::Forbidden(
            "SQL mode access denied for your role".to_string(),
        ));
    }

    Ok(next.run(request).await)
}

/// 对话模式检查中间件
pub async fn require_chat_mode(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let session = request
        .extensions()
        .get::<UserSession>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    if !session.role.can_use_chat_mode() {
        return Err(AppError::Forbidden(
            "Chat mode access denied for your role".to_string(),
        ));
    }

    Ok(next.run(request).await)
}

/// 连接管理权限检查中间件
pub async fn require_connection_management(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let session = request
        .extensions()
        .get::<UserSession>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    if !session.role.can_manage_connections() {
        return Err(AppError::Forbidden(
            "Connection management access denied for your role".to_string(),
        ));
    }

    Ok(next.run(request).await)
}

/// 图表生成和导出权限检查中间件
pub async fn require_chart_generation(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let session = request
        .extensions()
        .get::<UserSession>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    if !session.role.can_generate_charts() {
        return Err(AppError::Forbidden(
            "Chart generation access denied for your role".to_string(),
        ));
    }

    Ok(next.run(request).await)
}

/// 从请求中获取当前用户会话
pub fn get_session_from_request(request: &Request) -> AppResult<UserSession> {
    request
        .extensions()
        .get::<UserSession>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))
}

/// 从请求中获取当前用户 ID
pub fn get_user_id_from_request(request: &Request) -> AppResult<uuid::Uuid> {
    get_session_from_request(request).map(|s| s.user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_level() {
        assert!(role_level(&UserRole::Admin) > role_level(&UserRole::Analyst));
        assert!(role_level(&UserRole::Analyst) > role_level(&UserRole::Developer));
        assert!(role_level(&UserRole::Developer) > role_level(&UserRole::Business));
    }

    #[test]
    fn test_has_permission() {
        // 管理员可以访问所有
        assert!(has_permission(&UserRole::Admin, &UserRole::Admin));
        assert!(has_permission(&UserRole::Admin, &UserRole::Analyst));
        assert!(has_permission(&UserRole::Admin, &UserRole::Business));

        // 其他角色只能访问同级或更低
        assert!(has_permission(&UserRole::Analyst, &UserRole::Analyst));
        assert!(has_permission(&UserRole::Analyst, &UserRole::Business));
        assert!(!has_permission(&UserRole::Analyst, &UserRole::Admin));

        assert!(has_permission(&UserRole::Business, &UserRole::Business));
        assert!(!has_permission(&UserRole::Business, &UserRole::Developer));
    }
}
