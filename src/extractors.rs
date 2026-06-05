use std::future::{Ready, ready};

use actix_web::{FromRequest, HttpRequest, dev::Payload};
use jsonwebtoken::{DecodingKey, Validation};

use crate::error::AppError;
use crate::models::Claims;

pub struct AuthUser {
    pub id: i64,
}

impl FromRequest for AuthUser {
    type Error = AppError;
    // Verifying a JWT is pure CPU work (no awaiting), so we can resolve
    // synchronously. `Ready<T>` is a future that's already complete.
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(extract_user(req))
    }
}

fn extract_user(req: &HttpRequest) -> Result<AuthUser, AppError> {
    let header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("malformed Authorization header".into()))?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("expected Bearer token".into()))?;

    let secret =
        std::env::var("JWT_SECRET").map_err(|_| AppError::Internal("JWT_SECRET not set".into()))?;

    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(AuthUser {
        id: decoded.claims.sub,
    })
}
