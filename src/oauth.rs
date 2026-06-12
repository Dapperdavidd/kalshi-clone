use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::Claims;

#[derive(serde::Deserialize)]
pub struct GoogleLoginRequest {
    pub credential: String,
}

#[derive(serde::Deserialize)]
struct GoogleTokenInfo {
    aud: String,          
    sub: String,           
    email: String,
    #[serde(default)]
    email_verified: String, 
}

pub async fn google_login(
    body: web::Json<GoogleLoginRequest>,
    pool: web::Data<PgPool>,
    http: web::Data<reqwest::Client>,
) -> Result<HttpResponse, AppError> {
    let info = verify_google_token(&http, &body.credential).await?;

    let our_client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| AppError::Internal("GOOGLE_CLIENT_ID not set".into()))?;
    if info.aud != our_client_id {
        return Err(AppError::Unauthorized("token audience mismatch".into()));
    }
    if info.email_verified != "true" {
        return Err(AppError::Unauthorized("google email not verified".into()));
    }

    let (user_id, is_admin) = upsert_google_user(pool.get_ref(), &info).await?;
    let token = issue_jwt(user_id, is_admin)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "token": token })))
}

async fn verify_google_token(
    http: &reqwest::Client,
    credential: &str,
) -> Result<GoogleTokenInfo, AppError> {
    let resp = http
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", credential)])
        .send()
        .await
        .map_err(|e| AppError::Internal(Box::new(e)))?;

    if !resp.status().is_success() {
        return Err(AppError::Unauthorized("invalid google token".into()));
    }

    resp.json::<GoogleTokenInfo>()
        .await
        .map_err(|e| AppError::Internal(Box::new(e)))
}


async fn upsert_google_user(
    pool: &PgPool,
    info: &GoogleTokenInfo,
) -> Result<(i64, bool), AppError> {
    if let Some(row) = sqlx::query_as::<_, (i64, bool)>(
        "SELECT id, is_admin FROM users WHERE google_sub = $1",
    )
    .bind(&info.sub)
    .fetch_optional(pool)
    .await?
    {
        return Ok(row);
    }

    if let Some(row) = sqlx::query_as::<_, (i64, bool)>(
        "UPDATE users SET google_sub = $1 WHERE email = $2 RETURNING id, is_admin",
    )
    .bind(&info.sub)
    .bind(&info.email)
    .fetch_optional(pool)
    .await?
    {
        return Ok(row);
    }

    let mut tx = pool.begin().await?;
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (email, google_sub) VALUES ($1, $2) RETURNING id",
    )
    .bind(&info.email)
    .bind(&info.sub)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO balances (user_id, amount) VALUES ($1, 10000)")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok((user_id, false))
}


fn issue_jwt(user_id: i64, is_admin: bool) -> Result<String, AppError> {
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(Box::new(e)))?
        .as_secs()
        + 24 * 60 * 60;
    let claims = Claims { sub: user_id, exp: exp as usize, is_admin };
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT_SECRET not set".into()))?;
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::from)
}