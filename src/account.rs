use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::models::PositionView;

pub async fn positions(
    user: AuthUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query_as::<_, PositionView>(
        "SELECT market_id, quantity FROM positions \
         WHERE user_id = $1 AND quantity <> 0 ORDER BY market_id",
    )
    .bind(user.id)
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(rows))
}

pub async fn balance(
    user: AuthUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let amount: i64 = sqlx::query_scalar("SELECT amount FROM balances WHERE user_id = $1")
        .bind(user.id)
        .fetch_optional(pool.get_ref())
        .await?
        .unwrap_or(0);
    Ok(HttpResponse::Ok().json(serde_json::json!({ "balance_cents": amount })))
}