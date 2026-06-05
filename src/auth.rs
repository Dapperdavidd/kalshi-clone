use actix_web::{HttpRequest, HttpResponse, Responder, web};
use jsonwebtoken::{DecodingKey, Validation};

use sqlx::PgPool;

use crate::error::AppError;
use crate::models::{Claims, LoginRequest, SignupRequest, User};

/// Pull the JWT off the `Authorization: Bearer <token>` header, verify it,
/// and return the authenticated user's id. Shared by every protected handler.
pub fn authenticate(req: &HttpRequest) -> Result<i64, AppError> {
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

    Ok(decoded.claims.sub)
}

pub async fn signup(
    body: web::Json<SignupRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).unwrap();
    let _ = sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
        .bind(&body.email)
        .bind(&hash)
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::Created().body("user created"))
}

pub async fn login(
    body: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid login details".into()))?;

    if !bcrypt::verify(&body.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("invalid login details".into()));
    }

    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(Box::new(e)))?
        .as_secs()
        + 24 * 60 * 60;

    let claims = Claims {
        sub: user.id,
        exp: exp as usize,
    };
    let secret =
        std::env::var("JWT_SECRET").map_err(|_| AppError::Internal("JWT_SECRET not set".into()))?;
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(HttpResponse::Ok().body(token))
}

pub async fn me(req: HttpRequest) -> Result<HttpResponse, AppError> {
    let user_id = authenticate(&req)?;
    Ok(HttpResponse::Ok().body(format!("you are user {user_id}")))
}
