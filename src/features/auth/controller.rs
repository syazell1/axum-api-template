use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{AppendHeaders, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::TypedHeader;
use cookie::{time::Duration, Cookie};
use reqwest::header::SET_COOKIE;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    configurations::JwtSettings,
    errors::AppError,
    utils::jwt::{decode_jwt, generate_jwt},
};

use super::{
    models::{LoginFormData, RegisterFormData},
    repository::{
        add_refresh_token_by_user_id, create_user, delete_all_refresh_token_by_user_id,
        delete_refresh_token_by_token, get_user_by_id, get_user_tokens_by_token,
        validate_credentials,
    },
};

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login_user))
        .route("/register", post(register_user))
        .route("/refresh", get(refresh_user_token))
        .route("/logout", post(logout_user))
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    id: Uuid,
    access_token: String,
}

#[tracing::instrument(name = "Logging User In", skip(app_state, cookie, input))]
async fn login_user(
    State(app_state): State<Arc<AppState>>,
    TypedHeader(cookie): TypedHeader<headers::Cookie>,
    Json(input): Json<LoginFormData>,
) -> Result<Response, AppError> {
    let input = input.try_into()?;

    let id = validate_credentials(&input, &app_state.pool, &app_state.pwd_hasher).await?;

    let (at, rt) = generate_auth_tokens(id, &app_state.jwt_settings)?;

    let a = if let Some(data) = cookie.get("rt") {
        let token_data = get_user_tokens_by_token(data, &app_state.pool).await?;

        if token_data.is_none() {
            delete_all_refresh_token_by_user_id(id, &app_state.pool).await?;
        }

        data
    } else {
        ""
    };
    delete_refresh_token_by_token(a, &app_state.pool).await?;

    add_refresh_token_by_user_id(rt.value(), id, &app_state.pool).await?;

    Ok((
        StatusCode::OK,
        AppendHeaders([(SET_COOKIE, rt.to_string())]),
        Json(AuthResponse {
            id,
            access_token: at,
        }),
    )
        .into_response())
}

#[tracing::instrument(name = "Registering User", skip(app_state, input))]
async fn register_user(
    State(app_state): State<Arc<AppState>>,
    Json(input): Json<RegisterFormData>,
) -> Result<Response, AppError> {
    let input = input.try_into()?;

    let id = create_user(&input, &app_state.pool, &app_state.pwd_hasher).await?;

    let (at, rt) = generate_auth_tokens(id, &app_state.jwt_settings)?;

    Ok((
        StatusCode::OK,
        AppendHeaders([(SET_COOKIE, rt.to_string())]),
        Json(AuthResponse {
            id,
            access_token: at,
        }),
    )
        .into_response())
}

#[tracing::instrument(name = "Refreshing User token", skip(app_state, cookie))]
async fn refresh_user_token(
    State(app_state): State<Arc<AppState>>,
    TypedHeader(cookie): TypedHeader<headers::Cookie>,
) -> Result<Response, AppError> {
    let rt = match cookie.get("rt") {
        Some(data) => data,
        None => {
            return Err(AppError::UnauthorizedError(
                "Refresh token was not found.".into(),
            ))
        }
    };

    let user_token_data = match get_user_tokens_by_token(rt, &app_state.pool).await? {
        Some(data) => data,
        None => {
            let token_data = decode_jwt(rt, &app_state.jwt_settings, true)
                .map_err(|e| AppError::UnauthorizedError(e.to_string()))?;

            let user = get_user_by_id(token_data.claims.id, &app_state.pool).await?;

            delete_all_refresh_token_by_user_id(user.id, &app_state.pool).await?;
            return Err(AppError::UnauthorizedError(
                "Refresh token reuse found.".into(),
            ));
        }
    };

    let token_data = match decode_jwt(rt, &app_state.jwt_settings, true) {
        Ok(data) => data,
        Err(e) => {
            if *e.kind() == jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                delete_refresh_token_by_token(rt, &app_state.pool).await?;
            }
            return Err(AppError::UnauthorizedError(e.to_string()));
        }
    };

    if user_token_data.user_id != token_data.claims.id {
        return Err(AppError::UnauthorizedError("Invalid jwt token".into()));
    }

    let user = get_user_by_id(token_data.claims.id, &app_state.pool).await?;

    delete_refresh_token_by_token(rt, &app_state.pool).await?;

    let (at, rt) = generate_auth_tokens(user.id, &app_state.jwt_settings)?;

    add_refresh_token_by_user_id(rt.value(), token_data.claims.id, &app_state.pool).await?;

    Ok((
        StatusCode::OK,
        AppendHeaders([(SET_COOKIE, rt.to_string())]),
        Json(AuthResponse {
            id: user.id,
            access_token: at,
        }),
    )
        .into_response())
}

#[tracing::instrument(name = "Logging User out", skip(app_state, cookie))]
async fn logout_user(
    State(app_state): State<Arc<AppState>>,
    TypedHeader(cookie): TypedHeader<headers::Cookie>,
) -> Result<Response, AppError> {
    let empty_rt = cookie::Cookie::build(("rt", ""))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .max_age(Duration::ZERO)
        .build();

    let rt = match cookie.get("rt") {
        Some(data) => data,
        None => {
            return Ok((
                StatusCode::OK,
                AppendHeaders([(SET_COOKIE, empty_rt.to_string())]),
            )
                .into_response())
        }
    };

    if get_user_tokens_by_token(rt, &app_state.pool)
        .await?
        .is_none()
    {
        return Ok((
            StatusCode::OK,
            AppendHeaders([(SET_COOKIE, empty_rt.to_string())]),
        )
            .into_response());
    }

    delete_refresh_token_by_token(rt, &app_state.pool).await?;

    Ok((
        StatusCode::OK,
        AppendHeaders([(SET_COOKIE, empty_rt.to_string())]),
    )
        .into_response())
}

pub fn generate_auth_tokens(
    user_id: Uuid,
    jwt_settings: &JwtSettings,
) -> Result<(String, Cookie), AppError> {
    let at = generate_jwt(user_id, jwt_settings, false)
        .map_err(|e| AppError::UnexpectedError(e.to_string()))?;
    let rt = generate_jwt(user_id, jwt_settings, true)
        .map_err(|e| AppError::UnexpectedError(e.to_string()))?;

    let cookie = cookie::CookieBuilder::new("rt", rt)
        .http_only(true)
        .max_age(Duration::days(7))
        .path("/")
        .same_site(cookie::SameSite::Lax)
        .secure(true)
        .build();

    Ok((at, cookie))
}
