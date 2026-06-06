use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::models::{Portfolio, PortfolioPosition, PortfolioRow, PositionView};

pub async fn positions(user: AuthUser, pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query_as::<_, PositionView>(
        "SELECT market_id, quantity FROM positions \
         WHERE user_id = $1 AND quantity <> 0 ORDER BY market_id",
    )
    .bind(user.id)
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(rows))
}

pub async fn balance(user: AuthUser, pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let amount: i64 = sqlx::query_scalar("SELECT amount FROM balances WHERE user_id = $1")
        .bind(user.id)
        .fetch_optional(pool.get_ref())
        .await?
        .unwrap_or(0);
    Ok(HttpResponse::Ok().json(serde_json::json!({ "balance_cents": amount })))
}

pub async fn portfolio(user: AuthUser, pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let balance: i64 = sqlx::query_scalar("SELECT amount FROM balances WHERE user_id = $1")
        .bind(user.id)
        .fetch_optional(pool.get_ref())
        .await?
        .unwrap_or(0);

    let rows = sqlx::query_as::<_, PortfolioRow>(
        "SELECT p.market_id, m.question, p.quantity, \
                (SELECT t.price FROM trades t \
                 WHERE t.market_id = p.market_id \
                 ORDER BY t.created_at DESC LIMIT 1) AS mark_price \
         FROM positions p \
         JOIN markets m ON m.id = p.market_id \
         WHERE p.user_id = $1 AND p.quantity <> 0 \
         ORDER BY p.market_id",
    )
    .bind(user.id)
    .fetch_all(pool.get_ref())
    .await?;

    let mut positions = Vec::with_capacity(rows.len());
    let mut positions_value: i64 = 0;
    for r in rows {
        let mark = r.mark_price.unwrap_or(50);
        let value = r.quantity as i64 * mark as i64;
        positions_value += value;
        positions.push(PortfolioPosition {
            market_id: r.market_id,
            question: r.question,
            quantity: r.quantity,
            mark_price: mark,
            value_cents: value,
        });
    }

    Ok(HttpResponse::Ok().json(Portfolio {
        balance_cents: balance,
        positions,
        positions_value_cents: positions_value,
        equity_cents: balance + positions_value,
    }))
}
