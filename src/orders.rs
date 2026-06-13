use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::events::{Broadcaster, MarketEvent};
use crate::extractors::AuthUser;
use crate::models::{DbCancelRow, DbOrder, OrderView, PlaceOrderRequest};
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
    broadcaster: web::Data<Broadcaster>,
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

    // Keep the cached Yes price fresh so market/event cards show a live %.
    if let Some(last) = fills.last() {
        sqlx::query("UPDATE markets SET last_price = $1 WHERE id = $2")
            .bind(last.price as i32)
            .bind(body.market_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    // Publish *after* commit so we never announce a trade that rolled back.
    for fill in &fills {
        broadcaster.publish(MarketEvent::Trade {
            market_id: body.market_id,
            price: fill.price as i32,
            quantity: fill.quantity as i32,
        });
    }
    // ...and a book-changed event with the new top of book.
    let (best_bid, best_ask) = top_of_book(pool.get_ref(), body.market_id).await?;
    broadcaster.publish(MarketEvent::Book {
        market_id: body.market_id,
        best_bid,
        best_ask,
    });

    Ok(HttpResponse::Created().json(serde_json::json!({
        "order_id": taker_id,
        "filled": filled_total,
        "remaining": body.quantity - filled_total,
        "fills": fills.len(),
    })))
}

/// Best bid / best ask currently resting on a market's book.
async fn top_of_book(pool: &PgPool, market_id: i64) -> Result<(Option<i32>, Option<i32>), AppError> {
    let best_bid: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(price) FROM orders WHERE market_id = $1 AND side='buy' \
         AND status IN ('working','partially_filled') AND remaining > 0",
    )
    .bind(market_id)
    .fetch_one(pool)
    .await?;
    let best_ask: Option<i32> = sqlx::query_scalar(
        "SELECT MIN(price) FROM orders WHERE market_id = $1 AND side='sell' \
         AND status IN ('working','partially_filled') AND remaining > 0",
    )
    .bind(market_id)
    .fetch_one(pool)
    .await?;
    Ok((best_bid, best_ask))
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

pub async fn list_orders(
    user: AuthUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let orders = sqlx::query_as::<_, OrderView>(
        "SELECT id, market_id, side, price, quantity, remaining, status, created_at \
         FROM orders WHERE user_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(user.id)
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(orders))
}

pub async fn cancel_order(
    user: AuthUser,
    path: web::Path<i64>,
    pool: web::Data<PgPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    let mut tx = pool.begin().await?;

    let order = sqlx::query_as::<_, DbCancelRow>(
        "SELECT user_id, market_id, side, price, remaining, status FROM orders \
         WHERE id = $1 FOR UPDATE",
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("order {order_id} not found")))?;

    if order.user_id != user.id {
        return Err(AppError::Forbidden("not your order".into()));
    }

    if !matches!(order.status.as_str(), "working" | "partially_filled") {
        return Err(AppError::Conflict(format!(
            "order is {}, cannot cancel",
            order.status
        )));
    }

    let unit = if order.side == "buy" {
        order.price as i64
    } else {
        (100 - order.price) as i64
    };
    let refund = unit * order.remaining as i64;

    sqlx::query("UPDATE balances SET amount = amount + $1 WHERE user_id = $2")
        .bind(refund)
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE orders SET status = 'cancelled', remaining = 0 WHERE id = $1")
        .bind(order_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    // A cancel changes the book — publish the new top of book after commit.
    let (best_bid, best_ask) = top_of_book(pool.get_ref(), order.market_id).await?;
    broadcaster.publish(MarketEvent::Book {
        market_id: order.market_id,
        best_bid,
        best_ask,
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({ "cancelled": order_id, "refunded": refund })))
}
