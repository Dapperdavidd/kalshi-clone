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

/// Like AuthUser, but extraction fails with 403 unless the token's is_admin
/// claim is true. Use it on privileged handlers (e.g. resolving a market).
pub struct AdminUser {
    pub id: i64,
}

impl FromRequest for AdminUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(extract_admin(req))
    }
}

/// Shared JWT decoding: pull the Bearer token, verify the signature/expiry,
/// and return the claims. Both extractors build on this.
fn decode_claims(req: &HttpRequest) -> Result<Claims, AppError> {
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

    Ok(decoded.claims)
}

fn extract_user(req: &HttpRequest) -> Result<AuthUser, AppError> {
    Ok(AuthUser {
        id: decode_claims(req)?.sub,
    })
}

fn extract_admin(req: &HttpRequest) -> Result<AdminUser, AppError> {
    let claims = decode_claims(req)?;
    if !claims.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    Ok(AdminUser { id: claims.sub })
}
