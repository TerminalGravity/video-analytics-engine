use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::{AuthResponse, Claims, CreateUserInput, LoginInput, User, UserRole, UserSession},
    AppState,
};

pub async fn register(
    State((state, _)): State<(AppState, crate::graphql::Schema)>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<serde_json::Value>> {
    // Validate input
    input.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&input.email)
    .fetch_optional(state.db.pool())
    .await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("User already exists".to_string()));
    }

    // Hash password
    let password_hash = hash(&input.password, state.config.auth.bcrypt_cost)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

    // Create user
    let user_id = Uuid::new_v4();
    let role = input.role.unwrap_or(UserRole::User);

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(&input.email)
    .bind(&password_hash)
    .bind(role)
    .fetch_one(state.db.pool())
    .await?;

    tracing::info!("User registered: {}", user.email);

    Ok(Json(json!({
        "message": "User registered successfully",
        "user": {
            "id": user.id,
            "email": user.email,
            "role": user.role,
            "created_at": user.created_at
        }
    })))
}

pub async fn login(
    State((state, _)): State<(AppState, crate::graphql::Schema)>,
    Json(input): Json<LoginInput>,
) -> Result<Json<AuthResponse>> {
    // Validate input
    input.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Find user
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&input.email)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| AppError::Authentication("Invalid credentials".to_string()))?;

    // Verify password
    let is_valid = verify(&input.password, &user.password_hash)
        .map_err(|e| AppError::Internal(format!("Password verification failed: {}", e)))?;

    if !is_valid {
        return Err(AppError::Authentication("Invalid credentials".to_string()));
    }

    // Generate tokens
    let (access_token, expires_at) = generate_access_token(&user, &state.config.jwt_secret)?;
    let refresh_token = generate_refresh_token(&user, &state)?;

    // Store refresh token in database
    let session_id = Uuid::new_v4();
    let refresh_token_hash = hash(&refresh_token, DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash refresh token: {}", e)))?;

    sqlx::query(
        r#"
        INSERT INTO user_sessions (id, user_id, token_hash, expires_at, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
    )
    .bind(session_id)
    .bind(user.id)
    .bind(&refresh_token_hash)
    .bind(Utc::now() + Duration::days(state.config.auth.refresh_token_expiry_days))
    .execute(state.db.pool())
    .await?;

    tracing::info!("User logged in: {}", user.email);

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user,
        expires_at,
    }))
}

pub async fn refresh_token(
    State((state, _)): State<(AppState, crate::graphql::Schema)>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>> {
    // Extract refresh token from Authorization header
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| AppError::Authentication("Missing authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Authentication("Invalid authorization header".to_string()))?;

    let refresh_token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Authentication("Invalid authorization format".to_string()))?;

    // Find valid session
    let sessions = sqlx::query_as::<_, UserSession>(
        "SELECT * FROM user_sessions WHERE expires_at > NOW()"
    )
    .fetch_all(state.db.pool())
    .await?;

    let mut valid_session = None;
    for session in sessions {
        if verify(refresh_token, &session.token_hash).unwrap_or(false) {
            valid_session = Some(session);
            break;
        }
    }

    let session = valid_session
        .ok_or_else(|| AppError::Authentication("Invalid refresh token".to_string()))?;

    // Get user
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(session.user_id)
    .fetch_one(state.db.pool())
    .await?;

    // Generate new access token
    let (access_token, expires_at) = generate_access_token(&user, &state.config.jwt_secret)?;
    let new_refresh_token = generate_refresh_token(&user, &state)?;

    // Update session with new refresh token
    let new_refresh_token_hash = hash(&new_refresh_token, DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash refresh token: {}", e)))?;

    sqlx::query(
        "UPDATE user_sessions SET token_hash = $1, expires_at = $2 WHERE id = $3"
    )
    .bind(&new_refresh_token_hash)
    .bind(Utc::now() + Duration::days(state.config.auth.refresh_token_expiry_days))
    .bind(session.id)
    .execute(state.db.pool())
    .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        user,
        expires_at,
    }))
}

fn generate_access_token(user: &User, secret: &str) -> Result<(String, chrono::DateTime<Utc>)> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(24); // 24 hours

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role,
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok((token, expires_at))
}

fn generate_refresh_token(user: &User, state: &AppState) -> Result<String> {
    let now = Utc::now();
    let expires_at = now + Duration::days(state.config.auth.refresh_token_expiry_days);

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role,
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
    )?;

    Ok(token)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

pub async fn get_user_from_token(
    token: &str,
    secret: &str,
    db: &crate::database::Database,
) -> Result<User> {
    let claims = verify_token(token, secret)?;
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| AppError::Authentication("User not found".to_string()))?;

    Ok(user)
} 