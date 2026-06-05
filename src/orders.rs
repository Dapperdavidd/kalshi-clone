use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::models::{DbOrder, PlaceOrderRequest};
use crate::order_book::{Order, OrderBook, Side};

pub async fn place_order(
    user: AuthUser,
    body: web::Json<PlaceOrderRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let user_id = user.id;

    let mut tx = pool.begin().await?;

    let mut book = load_book(pool.get_ref(), body.market_id).await?;

    let taker_id: i64 = sqlx::query_scalar(
        "INSERT INTO orders (user_id, market_id, side, price, quantity, remaining) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id ",
    )
    .bind(user_id)
    .bind(body.market_id)
    .bind(&body.side)
    .bind(body.price)
    .bind(body.quantity)
    .bind(body.quantity)
    .fetch_one(&mut *tx)
    .await?;

    let taker = Order {
        id: taker_id as u64,
        side: match body.side.as_str() {
            "buy" => Side::Buy,
            _ => Side::Sell,
        },
        price: body.price as u32,
        quantity: body.quantity as u32,
    };

    let fills = book.match_order(taker);
    let mut filled_total: i32 = 0;

    for fill in &fills {
        sqlx::query(
            "INSERT INTO trades (market_id, maker_order_id, taker_order_id, price, quantity) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(body.market_id)
        .bind(fill.maker_id as i64)
        .bind(fill.taker_id as i64)
        .bind(fill.price as i32)
        .bind(fill.quantity as i32)
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE orders
                        SET remaining = remaining - $1,
                        status = CASE WHEN remaining - $1 = 0 THEN 'filled' ELSE 'partially_filled' END
                        WHERE id = $2"
                    )
            .bind(fill.quantity as i32)
            .bind(fill.maker_id as i64)
            .execute(&mut *tx)
            .await?;

        let maker_user_id: i64 = sqlx::query_scalar("SELECT user_id FROM orders WHERE id = $1")
            .bind(fill.maker_id as i64)
            .fetch_one(&mut *tx)
            .await?;

        // Maker is on the opposite side of the taker.
        let maker_delta = if body.side == "buy" {
            -(fill.quantity as i32)
        } else {
            fill.quantity as i32
        };

        sqlx::query(
            "INSERT INTO positions (user_id, market_id, quantity)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, market_id)
             DO UPDATE SET quantity = positions.quantity + EXCLUDED.quantity",
        )
        .bind(maker_user_id)
        .bind(body.market_id)
        .bind(maker_delta)
        .execute(&mut *tx)
        .await?;

        let cash = fill.price as i64 * fill.quantity as i64;
        let taker_cash = if body.side == "buy" { -cash } else { cash };

        sqlx::query("UPDATE balances SET amount = amount + $1 WHERE user_id = $2")
            .bind(taker_cash)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        let maker_cash = -taker_cash;
        sqlx::query("UPDATE balances SET amount = amount + $1 WHERE user_id = $2")
            .bind(maker_cash)
            .bind(maker_user_id)
            .execute(&mut *tx)
            .await?;

        filled_total += fill.quantity as i32;
    }

    sqlx::query(
        "UPDATE orders 
                    SET remaining = remaining - $1,
                    status = CASE WHEN remaining - $1 = 0 THEN 'filled' ELSE 'partially_filled' END 
                    WHERE id = $2",
    )
    .bind(filled_total)
    .bind(taker_id)
    .execute(&mut *tx)
    .await?;

    let taker_delta = if body.side == "buy" {
        filled_total
    } else {
        -filled_total
    };

    sqlx::query(
        "INSERT INTO positions (user_id, market_id, quantity)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, market_id)
        DO UPDATE SET quantity = positions.quantity + EXCLUDED.quantity",
    )
    .bind(user_id)
    .bind(body.market_id)
    .bind(taker_delta)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Created().body(format!("order {taker_id}: {} fills", fills.len())))
}

async fn load_book(pool: &PgPool, market_id: i64) -> Result<OrderBook, AppError> {
    let result = sqlx::query_as::<_, DbOrder>("SELECT id, side, price, remaining FROM orders WHERE market_id = $1 AND status IN ('working','partially_filled') AND remaining > 0 ")
    .bind(market_id)
    .fetch_all(pool)
    .await?;

    let mut book = OrderBook::new();

    for row in result {
        let order = Order {
            id: row.id as u64,
            side: match row.side.as_str() {
                "buy" => Side::Buy,
                _ => Side::Sell,
            },
            price: row.price as u32,
            quantity: row.remaining as u32,
        };
        book.add_resting(order);
    }
    Ok(book)
}
