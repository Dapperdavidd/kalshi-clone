use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::models::{DbOrder, PlaceOrderRequest};
use crate::order_book::{Order, OrderBook, Side};

fn unit_collateral(side: &Side, price: i32) -> i64 {
    match side {
        Side::Buy => price as i64,
        Side::Sell => (100 - price) as i64,
    }
}

pub async fn place_order(
    user: AuthUser,
    body: web::Json<PlaceOrderRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let side = body.validate()?;
    let user_id = user.id;

    let mut tx = pool.begin().await?;

    let market_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM markets WHERE id = $1")
            .bind(body.market_id)
            .fetch_optional(&mut *tx)
            .await?;
    match market_status.as_deref() {
        None => {
            return Err(AppError::NotFound(format!(
                "market {} not found",
                body.market_id
            )));
        }
        Some("active") => {}
        Some(other) => {
            return Err(AppError::Conflict(format!(
                "market is {other}, not open for trading"
            )));
        }
    }

    let balance: i64 =
        sqlx::query_scalar("SELECT amount FROM balances WHERE user_id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::Internal("user has no balance row".into()))?;

    let required = unit_collateral(&side, body.price) * body.quantity as i64;
    if balance < required {
        return Err(AppError::Conflict(format!(
            "insufficient funds: need {required} cents, have {balance}"
        )));
    }

    sqlx::query("UPDATE balances SET amount = amount - $1 WHERE user_id = $2")
        .bind(required)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    let taker_id: i64 = sqlx::query_scalar(
        "INSERT INTO orders (user_id, market_id, side, price, quantity, remaining) \
         VALUES ($1, $2, $3, $4, $5, $5) RETURNING id",
    )
    .bind(user_id)
    .bind(body.market_id)
    .bind(&body.side)
    .bind(body.price)
    .bind(body.quantity)
    .fetch_one(&mut *tx)
    .await?;

    let mut book = load_book(&mut *tx, body.market_id, taker_id).await?;

    let taker = Order {
        id: taker_id as u64,
        side: side.clone(),
        price: body.price as u32,
        quantity: body.quantity as u32,
    };

    let fills = book.match_order(taker);
    let mut filled_total: i32 = 0;

    for fill in &fills {
        let fill_qty = fill.quantity as i32;
        let fill_price = fill.price as i32;

        sqlx::query(
            "INSERT INTO trades (market_id, maker_order_id, taker_order_id, price, quantity) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(body.market_id)
        .bind(fill.maker_id as i64)
        .bind(fill.taker_id as i64)
        .bind(fill_price)
        .bind(fill_qty)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE orders \
                SET remaining = remaining - $1, \
                    status = CASE WHEN remaining - $1 = 0 THEN 'filled' ELSE 'partially_filled' END \
                WHERE id = $2",
        )
        .bind(fill_qty)
        .bind(fill.maker_id as i64)
        .execute(&mut *tx)
        .await?;

        let maker_user_id: i64 = sqlx::query_scalar("SELECT user_id FROM orders WHERE id = $1")
            .bind(fill.maker_id as i64)
            .fetch_one(&mut *tx)
            .await?;
        let maker_delta = if side == Side::Buy {
            -fill_qty
        } else {
            fill_qty
        };
        upsert_position(&mut tx, maker_user_id, body.market_id, maker_delta).await?;

        let per_contract_refund = match side {
            Side::Buy => (body.price - fill_price) as i64,
            Side::Sell => (fill_price - body.price) as i64,
        };
        let refund = per_contract_refund * fill_qty as i64;
        if refund > 0 {
            sqlx::query("UPDATE balances SET amount = amount + $1 WHERE user_id = $2")
                .bind(refund)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        filled_total += fill_qty;
    }

    sqlx::query(
        "UPDATE orders \
            SET remaining = remaining - $1, \
                status = CASE WHEN remaining - $1 = 0 THEN 'filled' ELSE 'working' END \
            WHERE id = $2",
    )
    .bind(filled_total)
    .bind(taker_id)
    .execute(&mut *tx)
    .await?;

    let taker_delta = if side == Side::Buy {
        filled_total
    } else {
        -filled_total
    };
    upsert_position(&mut tx, user_id, body.market_id, taker_delta).await?;

    tx.commit().await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "order_id": taker_id,
        "filled": filled_total,
        "remaining": body.quantity - filled_total,
        "fills": fills.len(),
    })))
}

async fn upsert_position(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: i64,
    market_id: i64,
    delta: i32,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO positions (user_id, market_id, quantity) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, market_id) \
         DO UPDATE SET quantity = positions.quantity + EXCLUDED.quantity",
    )
    .bind(user_id)
    .bind(market_id)
    .bind(delta)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn load_book(
    tx: &mut sqlx::PgConnection,
    market_id: i64,
    exclude_order_id: i64,
) -> Result<OrderBook, AppError> {
    let rows = sqlx::query_as::<_, DbOrder>(
        "SELECT id, side, price, remaining FROM orders \
         WHERE market_id = $1 AND id <> $2 \
           AND status IN ('working','partially_filled') AND remaining > 0",
    )
    .bind(market_id)
    .bind(exclude_order_id)
    .fetch_all(&mut *tx)
    .await?;

    let mut book = OrderBook::new();
    for row in rows {
        book.add_resting(Order {
            id: row.id as u64,
            side: match row.side.as_str() {
                "buy" => Side::Buy,
                _ => Side::Sell,
            },
            price: row.price as u32,
            quantity: row.remaining as u32,
        });
    }
    Ok(book)
}
