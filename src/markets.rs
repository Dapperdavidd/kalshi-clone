use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::{BookLevel, LevelRow, Market, OrderBookView, TradePrint};

pub async fn markets(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let result = sqlx::query_as::<_, Market>("SELECT id, question, status, rail FROM markets")
        .fetch_all(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn markets_id(
    path: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let result =
        sqlx::query_as::<_, Market>("SELECT id, question, status, rail FROM markets WHERE id = $1")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or_else(|| AppError::NotFound(format!("market {id} not found")))?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn orderbook(
    path: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let market_id = path.into_inner();

    let bid_rows = sqlx::query_as::<_, LevelRow>(
        "SELECT price, SUM(remaining)::int8 AS quantity FROM orders \
         WHERE market_id = $1 AND side = 'buy' \
           AND status IN ('working','partially_filled') AND remaining > 0 \
         GROUP BY price ORDER BY price DESC",
    )
    .bind(market_id)
    .fetch_all(pool.get_ref())
    .await?;

    let ask_rows = sqlx::query_as::<_, LevelRow>(
        "SELECT price, SUM(remaining)::int8 AS quantity FROM orders \
         WHERE market_id = $1 AND side = 'sell' \
           AND status IN ('working','partially_filled') AND remaining > 0 \
         GROUP BY price ORDER BY price ASC",
    )
    .bind(market_id)
    .fetch_all(pool.get_ref())
    .await?;

    let to_levels = |rows: Vec<LevelRow>| {
        rows.into_iter()
            .map(|r| BookLevel {
                price: r.price,
                quantity: r.quantity.unwrap_or(0),
            })
            .collect()
    };

    Ok(HttpResponse::Ok().json(OrderBookView {
        market_id,
        bids: to_levels(bid_rows),
        asks: to_levels(ask_rows),
    }))
}

pub async fn market_trades(
    path: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let market_id = path.into_inner();
    let trades = sqlx::query_as::<_, TradePrint>(
        "SELECT price, quantity, created_at FROM trades \
         WHERE market_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(market_id)
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(trades))
}