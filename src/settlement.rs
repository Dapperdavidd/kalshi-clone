use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::events::{Broadcaster, MarketEvent};
use crate::extractors::AdminUser;

#[derive(serde::Deserialize)]
pub struct ResolveRequest {
    pub outcome: String, // "yes" | "no"
}

#[derive(sqlx::FromRow)]
struct OpenOrderRow {
    user_id: i64,
    side: String,
    price: i32,
    remaining: i32,
}

pub async fn resolve_market(
    _admin: AdminUser, // gate: 403 for non-admins, before any DB work
    path: web::Path<i64>,
    body: web::Json<ResolveRequest>,
    pool: web::Data<PgPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, AppError> {
    let market_id = path.into_inner();

    // Validate the outcome up front — never trust the body.
    let outcome = match body.outcome.as_str() {
        "yes" | "no" => body.outcome.as_str(),
        other => {
            return Err(AppError::BadRequest(format!(
                "outcome must be 'yes' or 'no', got '{other}'"
            )));
        }
    };

    let mut tx = pool.begin().await?;

    // 1. Lock the market row and ensure it's still active. This is the
    //    double-settlement guard: a second call sees status='resolved' here.
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM markets WHERE id = $1 FOR UPDATE")
            .bind(market_id)
            .fetch_optional(&mut *tx)
            .await?;
    match status.as_deref() {
        None => return Err(AppError::NotFound(format!("market {market_id} not found"))),
        Some("active") => {}
        Some(other) => {
            return Err(AppError::Conflict(format!("market already {other}")));
        }
    }

    // 2. Refund collateral for every still-resting order, then kill it.
    let open_orders = sqlx::query_as::<_, OpenOrderRow>(
        "SELECT user_id, side, price, remaining FROM orders \
         WHERE market_id = $1 AND status IN ('working','partially_filled') AND remaining > 0 \
         FOR UPDATE",
    )
    .bind(market_id)
    .fetch_all(&mut *tx)
    .await?;

    for o in &open_orders {
        let unit = if o.side == "buy" {
            o.price as i64
        } else {
            (100 - o.price) as i64
        };
        let refund = unit * o.remaining as i64;
        sqlx::query("UPDATE balances SET amount = amount + $1 WHERE user_id = $2")
            .bind(refund)
            .bind(o.user_id)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query(
        "UPDATE orders SET status = 'cancelled', remaining = 0 \
         WHERE market_id = $1 AND status IN ('working','partially_filled')",
    )
    .bind(market_id)
    .execute(&mut *tx)
    .await?;

    // 3. Pay out positions. One set-based UPDATE handles every winner.
    //    YES: winners are longs  -> pay 100 * GREATEST(quantity, 0)
    //    NO : winners are shorts -> pay 100 * GREATEST(-quantity, 0)
    //    sqlx 0.9 only accepts &'static str, so we use two literal queries
    //    rather than building the SQL dynamically — same result, no injection
    //    surface to audit. market_id is a bound parameter either way.
    if outcome == "yes" {
        sqlx::query(
            "UPDATE balances b \
             SET amount = amount + (100 * GREATEST(p.quantity, 0))::bigint \
             FROM positions p \
             WHERE p.user_id = b.user_id AND p.market_id = $1 AND p.quantity <> 0",
        )
        .bind(market_id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            "UPDATE balances b \
             SET amount = amount + (100 * GREATEST(-p.quantity, 0))::bigint \
             FROM positions p \
             WHERE p.user_id = b.user_id AND p.market_id = $1 AND p.quantity <> 0",
        )
        .bind(market_id)
        .execute(&mut *tx)
        .await?;
    }

    // 4. Close the market.
    sqlx::query(
        "UPDATE markets SET status = 'resolved', outcome = $1, resolved_at = now() \
         WHERE id = $2",
    )
    .bind(outcome)
    .bind(market_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Announce resolution after commit — only durable facts reach clients.
    broadcaster.publish(MarketEvent::Resolved {
        market_id,
        outcome: outcome.to_string(),
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "market_id": market_id,
        "outcome": outcome,
        "open_orders_cancelled": open_orders.len(),
    })))
}
