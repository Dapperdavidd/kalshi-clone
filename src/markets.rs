use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use std::collections::HashMap;

use crate::error::AppError;
use crate::extractors::AdminUser;
use crate::models::{
    BookLevel, CreateEventRequest, EventOption, EventRow, EventView, LevelRow, Market,
    OptionRow, OrderBookView, TradePrint,
};

pub async fn markets(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let result = sqlx::query_as::<_, Market>(
        "SELECT id, question, status, rail, event_id, option_label, last_price FROM markets",
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn markets_id(
    path: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let result = sqlx::query_as::<_, Market>(
        "SELECT id, question, status, rail, event_id, option_label, last_price \
         FROM markets WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("market {id} not found")))?;

    Ok(HttpResponse::Ok().json(result))
}

/// All events with their member options (each option is a binary market). The
/// card UI renders these directly: title + category + per-option Yes %.
pub async fn list_events(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let events = sqlx::query_as::<_, EventRow>(
        "SELECT id, title, category, status, is_new FROM events ORDER BY is_new DESC, id",
    )
    .fetch_all(pool.get_ref())
    .await?;

    let options = sqlx::query_as::<_, OptionRow>(
        "SELECT event_id, id AS market_id, option_label AS label, last_price, status \
         FROM markets WHERE event_id IS NOT NULL \
         ORDER BY event_id, last_price DESC NULLS LAST, id",
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(group_events(events, options)))
}

/// One event with all its options.
pub async fn get_event(
    path: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let event = sqlx::query_as::<_, EventRow>(
        "SELECT id, title, category, status, is_new FROM events WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("event {id} not found")))?;

    let options = sqlx::query_as::<_, OptionRow>(
        "SELECT event_id, id AS market_id, option_label AS label, last_price, status \
         FROM markets WHERE event_id = $1 ORDER BY last_price DESC NULLS LAST, id",
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await?;

    let mut views = group_events(vec![event], options);
    Ok(HttpResponse::Ok().json(views.remove(0)))
}

/// Create a new event and one binary market per option. Admin only.
pub async fn create_event(
    _admin: AdminUser,
    body: web::Json<CreateEventRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    if body.category.trim().is_empty() {
        return Err(AppError::BadRequest("category is required".into()));
    }
    if body.options.is_empty() {
        return Err(AppError::BadRequest("at least one option is required".into()));
    }

    let mut tx = pool.begin().await?;

    let event_id: i64 = sqlx::query_scalar(
        "INSERT INTO events (title, category, is_new) VALUES ($1, $2, true) RETURNING id",
    )
    .bind(body.title.trim())
    .bind(body.category.trim())
    .fetch_one(&mut *tx)
    .await?;

    for opt in &body.options {
        if opt.label.trim().is_empty() {
            return Err(AppError::BadRequest("every option needs a label".into()));
        }
        let price = opt.initial_price.unwrap_or(50);
        if !(1..=99).contains(&price) {
            return Err(AppError::BadRequest(
                "initial_price must be between 1 and 99".into(),
            ));
        }
        let question = opt
            .question
            .clone()
            .unwrap_or_else(|| format!("{} — {}", body.title.trim(), opt.label.trim()));

        sqlx::query(
            "INSERT INTO markets (question, event_id, option_label, last_price, status, rail) \
             VALUES ($1, $2, $3, $4, 'active', 'native')",
        )
        .bind(question)
        .bind(event_id)
        .bind(opt.label.trim())
        .bind(price)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "event_id": event_id,
        "options": body.options.len(),
    })))
}

/// Fold option rows into their parent events, preserving event order.
fn group_events(events: Vec<EventRow>, options: Vec<OptionRow>) -> Vec<EventView> {
    let mut views: Vec<EventView> = events
        .into_iter()
        .map(|e| EventView {
            id: e.id,
            title: e.title,
            category: e.category,
            status: e.status,
            is_new: e.is_new,
            market_count: 0,
            options: Vec::new(),
        })
        .collect();

    let index: HashMap<i64, usize> =
        views.iter().enumerate().map(|(i, v)| (v.id, i)).collect();

    for o in options {
        let Some(eid) = o.event_id else { continue };
        let Some(&i) = index.get(&eid) else { continue };
        views[i].options.push(EventOption {
            market_id: o.market_id,
            label: o.label.unwrap_or_default(),
            yes_price: o.last_price,
            status: o.status,
        });
        views[i].market_count += 1;
    }

    views
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