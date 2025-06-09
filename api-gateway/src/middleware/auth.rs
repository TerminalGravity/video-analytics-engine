use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::{
    auth::{get_user_from_token, verify_token},
    error::AppError,
    models::{Claims, User, UserRole},
    AppState,
};

#[derive(Clone)]
pub struct AuthContext {
    pub user: User,
    pub claims: Claims,
}

pub async fn auth_middleware(
    State((state, _)): State<(AppState, crate::graphql::Schema)>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let headers = request.headers();
    
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| AppError::Authentication("Missing authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Authentication("Invalid authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Authentication("Invalid authorization format".to_string()))?;

    // Verify token and get user
    let user = get_user_from_token(token, &state.config.jwt_secret, &state.db).await?;
    let claims = verify_token(token, &state.config.jwt_secret)?;

    // Add auth context to request extensions
    let auth_context = AuthContext { user, claims };
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

pub async fn admin_auth_middleware(
    State((state, _)): State<(AppState, crate::graphql::Schema)>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // First run the regular auth middleware logic
    let headers = request.headers();
    
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| AppError::Authentication("Missing authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Authentication("Invalid authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Authentication("Invalid authorization format".to_string()))?;

    let user = get_user_from_token(token, &state.config.jwt_secret, &state.db).await?;
    let claims = verify_token(token, &state.config.jwt_secret)?;

    // Check if user is admin
    if user.role != UserRole::Admin {
        return Err(AppError::Authorization("Admin access required".to_string()));
    }

    // Add auth context to request extensions
    let auth_context = AuthContext { user, claims };
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

pub fn get_auth_context(request: &Request) -> Option<&AuthContext> {
    request.extensions().get::<AuthContext>()
}

pub fn require_auth_context(request: &Request) -> Result<&AuthContext, AppError> {
    get_auth_context(request)
        .ok_or_else(|| AppError::Authentication("Authentication required".to_string()))
} 