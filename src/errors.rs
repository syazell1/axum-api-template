use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use validator::ValidationErrors;

#[derive(thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFoundError(String),
    #[error("{0}")]
    UnauthorizedError(String),
    #[error("{0}")]
    UnexpectedError(String),
    #[error("{0}")]
    ValidationError(#[from] ValidationErrors),
    #[error("{0}")]
    DbError(#[from] sqlx::Error)
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Serialize)]
pub struct AppErrorDetails {
    error_code : u16,
    error_type : String,
    title : String,
    details : String
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFoundError(_) => (StatusCode::NOT_FOUND).into_response(),
            AppError::UnauthorizedError(e) => (
                StatusCode::UNAUTHORIZED,
                Json(AppErrorDetails {
                    error_code : StatusCode::UNAUTHORIZED.as_u16(),
                    error_type: "UnauthorizedError".into(),
                    title : "Unauthorized".into(),
                    details : e.to_string()
                })
            ).into_response(),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST).into_response(),
            AppError::UnexpectedError(e) => (
                StatusCode::BAD_REQUEST,
                Json(AppErrorDetails {
                    error_code : StatusCode::BAD_REQUEST.as_u16(),
                    error_type: "UnexpectedError".into(),
                    title : "Unexpected Error".into(),
                    details : e.to_string()
                })
            ).into_response(),
            AppError::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}