use actix_web::{HttpResponse, web};
use jsonwebtoken;

use sqlx::PgPool;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::models::{Claims, LoginRequest, SignupRequest, User};

const STARTING_BALANCE_CENTS: i64 = 10_000;

pub async fn signup(
    body: web::Json<SignupRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    if !body.email.contains('@') || body.email.len() > 254 {
        return Err(AppError::BadRequest("invalid email".into()));
    }
    if body.password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }

    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)?;

    let mut tx = pool.begin().await?;

    let user_id: i64 =
        sqlx::query_scalar("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id")
            .bind(&body.email)
            .bind(&hash)
            .fetch_one(&mut *tx)
            .await
            .map_err(signup_db_error)?;

    sqlx::query("INSERT INTO balances (user_id, amount) VALUES ($1, $2)")
        .bind(user_id)
        .bind(STARTING_BALANCE_CENTS)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Created().json(serde_json::json!({ "id": user_id })))
}

fn signup_db_error(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &e {
        // 23505 = unique_violation in Postgres.
        if db.code().as_deref() == Some("23505") {
            return AppError::Conflict("email already registered".into());
        }
    }
    AppError::from(e) // fall back to our generic conversion (-> 500)
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

pub async fn me(user: AuthUser) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().body(format!("you are user {}", user.id)))
}
