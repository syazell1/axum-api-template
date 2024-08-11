use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::TypedHeader;
use chrono::{Duration, Utc};
use headers::{authorization::Bearer, Authorization};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, configurations::JwtSettings, errors::AppError};

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub id: Uuid,
}

pub fn generate_jwt(
    user_id: Uuid,
    jwt_settings: &JwtSettings,
    is_refresh_token: bool,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        iss: jwt_settings.issuer.to_string(),
        aud: jwt_settings.audience.to_string(),
        id: user_id,
        exp: if is_refresh_token {
            (Utc::now() + Duration::try_weeks(7).unwrap()).timestamp() as usize
        } else {
            (Utc::now() + Duration::try_minutes(12).unwrap()).timestamp() as usize
        },
    };

    let secret_key = if is_refresh_token {
        jwt_settings.refresh_token_secret.expose_secret()
    } else {
        jwt_settings.access_token_secret.expose_secret()
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes()),
    )
}

pub fn decode_jwt(
    token: &str,
    jwt_settings: &JwtSettings,
    is_refresh_token: bool,
) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_audience(&[jwt_settings.audience.to_string()]);
    validation.set_issuer(&[jwt_settings.issuer.to_string()]);

    let secret_key = if is_refresh_token {
        jwt_settings.refresh_token_secret.expose_secret()
    } else {
        jwt_settings.access_token_secret.expose_secret()
    };

    decode(
        token,
        &DecodingKey::from_secret(secret_key.as_bytes()),
        &validation,
    )
}

pub struct AuthUser {
    pub id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::UnauthorizedError("Bearer token was not found.".into()))?;

        let app_state = Arc::from_ref(state);

        let token_data = decode_jwt(bearer.token(), &app_state.jwt_settings, false)
            .map_err(|e| AppError::UnauthorizedError(e.to_string()))?;

        Ok(Self {
            id: token_data.claims.id,
        })
    }
}
