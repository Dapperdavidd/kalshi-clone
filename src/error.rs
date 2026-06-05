use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("internal error")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST, // 400
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED, // 401
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,    // 403
            AppError::NotFound(_) => StatusCode::NOT_FOUND,     // 404
            AppError::Conflict(_) => StatusCode::CONFLICT,      // 409
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR, // 500
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let AppError::Internal(cause) = self {
            log::error!("internal error: {cause:?}");
        }

        HttpResponse::build(self.status_code()).json(json!({ "error": self.to_string() }))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound("not found".into()),
            other => AppError::Internal(Box::new(other)),
        }
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(e: bcrypt::BcryptError) -> Self {
        AppError::Internal(Box::new(e))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized("invalid or expired token".into())
    }
}
