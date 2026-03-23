use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::infrastructure::auth::{hash_password, verify_password, create_token, LoginRequest, RegisterRequest, AuthResponse};
use crate::presentation::routes::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let pool = &state.pool;
    let auth_config = &state.auth_config;
    
    // Check if user already exists
    let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(pool)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Database error: {}", e) }),
        ))?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse { error: "Username already exists".to_string() }),
        ));
    }

    // Hash password
    let password_hash = hash_password(&req.password).map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: format!("Failed to hash password: {}", e) }),
    ))?;

    // Create user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, username, password_hash, email) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(&req.username)
        .bind(&password_hash)
        .bind(&req.email)
        .execute(pool)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to create user: {}", e) }),
        ))?;

    // Create token
    let token = create_token(&user_id.to_string(), &req.username, &auth_config.jwt_secret).map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: format!("Failed to create token: {}", e) }),
    ))?;

    Ok(Json(AuthResponse {
        token,
        username: req.username,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let pool = &state.pool;
    let auth_config = &state.auth_config;
    
    // Find user
    #[derive(sqlx::FromRow)]
    struct UserRow {
        id: Uuid,
        username: String,
        password_hash: String,
    }
    
    let user: UserRow = sqlx::query_as("SELECT id, username, password_hash FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(pool)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Database error: {}", e) }),
        ))?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "Invalid credentials".to_string() }),
        ))?;

    // Verify password
    let valid = verify_password(&req.password, &user.password_hash).map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: format!("Failed to verify password: {}", e) }),
    ))?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "Invalid credentials".to_string() }),
        ));
    }

    // Create token
    let token = create_token(&user.id.to_string(), &user.username, &auth_config.jwt_secret).map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: format!("Failed to create token: {}", e) }),
    ))?;

    Ok(Json(AuthResponse {
        token,
        username: user.username,
    }))
}
