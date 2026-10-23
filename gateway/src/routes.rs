//! Auth routes

use axum::{
    extract::{State, Json},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{auth, AppState};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub data: Option<auth::TokenPair>,
    pub error: Option<String>,
}

fn set_refresh_cookie(headers: &mut HeaderMap, refresh_token: &str, max_age: i64) {
    let cookie = format!(
        "refresh_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/",
        refresh_token, max_age
    );
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
}



pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate username
    if payload.username.len() < 3 || payload.username.len() > 20 {
        return error_response(StatusCode::BAD_REQUEST, "Username must be 3-20 characters");
    }
    if !payload.username.chars().all(|c| c.is_alphanumeric()) {
        return error_response(StatusCode::BAD_REQUEST, "Username must be alphanumeric");
    }
    
    if payload.password.len() < 8 {
        return error_response(StatusCode::BAD_REQUEST, "Password must be at least 8 characters");
    }
    
    // Check if user exists
    if let Ok(Some(_)) = auth::get_user_by_username(&state.pool, &payload.username).await {
        return error_response(StatusCode::CONFLICT, "Username already taken");
    }
    
    // Hash password
    let password_hash = match auth::hash_password(&payload.password).await {
        Ok(h) => h,
        Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password"),
    };
    
    // Create user
    let user_id = match auth::create_user(&state.pool, &payload.username, &password_hash).await {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user"),
    };
    
    // Generate token pair
    let auth_config = auth::AuthConfig::from_settings(&state.config.jwt);
    let token_pair = match auth::generate_token_pair(
        &auth_config,
        user_id,
        &payload.username,
        1, // season_id
        "shard-1",
        None,
    ) {
        Ok(t) => t,
        Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate tokens"),
    };
    
    let mut headers = HeaderMap::new();
    set_refresh_cookie(&mut headers, &token_pair.refresh_token,
        (state.config.jwt.refresh_token_ttl_days * 24 * 3600) as i64);
    
    let response = Json(AuthResponse {
        success: true,
        data: Some(token_pair),
        error: None,
    });
    
    (headers, response).into_response()
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Get user
    let user = match auth::get_user_by_username(&state.pool, &payload.username).await {
        Ok(Some(u)) => u,
        Ok(None) => return error_response(StatusCode::UNAUTHORIZED, "Invalid credentials"),
        Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
    };
    
    if user.is_banned {
        return error_response(StatusCode::FORBIDDEN, "Account banned");
    }
    
    // Verify password
    let valid = match auth::verify_password(&payload.password, &user.password_hash).await {
        Ok(v) => v,
        Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Verification failed"),
    };
    
    if !valid {
        return error_response(StatusCode::UNAUTHORIZED, "Invalid credentials");
    }
    
    // Update last login
    let _ = auth::update_last_login(&state.pool, user.id).await;
    
    // Generate token pair
    let auth_config = auth::AuthConfig::from_settings(&state.config.jwt);
    let token_pair = match auth::generate_token_pair(
        &auth_config,
        user.id,
        &user.username,
        1,
        "shard-1",
        None,
    ) {
        Ok(t) => t,
        Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate tokens"),
    };
    
    let mut headers = HeaderMap::new();
    set_refresh_cookie(&mut headers, &token_pair.refresh_token,
        (state.config.jwt.refresh_token_ttl_days * 24 * 3600) as i64);
    
    let response = Json(AuthResponse {
        success: true,
        data: Some(token_pair),
        error: None,
    });
    
    (headers, response).into_response()
}

pub async fn refresh(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // For refresh, we'd need to extract the refresh token from cookies
    // For now, return not implemented
    error_response(StatusCode::NOT_IMPLEMENTED, "Refresh endpoint not fully implemented")
}

pub async fn logout() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let cookie = "refresh_token=; HttpOnly; Secure; SameSite=Strict; Max-Age=0; Path=/".to_string();
    headers.insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    
    let response = Json(AuthResponse {
        success: true,
        data: None,
        error: None,
    });
    
    (headers, response).into_response()
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(AuthResponse {
        success: false,
        data: None,
        error: Some(message.to_string()),
    })).into_response()
}