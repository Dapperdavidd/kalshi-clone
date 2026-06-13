#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Market {
    pub id: i64,
    pub question: String,
    pub status: String,
    pub rail: String,
    pub event_id: Option<i64>,
    pub option_label: Option<String>,
    pub last_price: Option<i32>,
}

// ───── events (group several binary markets, Kalshi-style) ─────

#[derive(sqlx::FromRow)]
pub struct EventRow {
    pub id: i64,
    pub title: String,
    pub category: String,
    pub status: String,
    pub is_new: bool,
}

#[derive(sqlx::FromRow)]
pub struct OptionRow {
    pub event_id: Option<i64>,
    pub market_id: i64,
    pub label: Option<String>,
    pub last_price: Option<i32>,
    pub status: String,
}

#[derive(serde::Serialize)]
pub struct EventOption {
    pub market_id: i64,
    pub label: String,
    pub yes_price: Option<i32>,
    pub status: String,
}

#[derive(serde::Serialize)]
pub struct EventView {
    pub id: i64,
    pub title: String,
    pub category: String,
    pub status: String,
    pub is_new: bool,
    pub market_count: i64,
    pub options: Vec<EventOption>,
}

#[derive(serde::Deserialize)]
pub struct CreateOption {
    pub label: String,
    pub question: Option<String>,
    pub initial_price: Option<i32>,
}

#[derive(serde::Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub category: String,
    pub options: Vec<CreateOption>,
}

#[derive(serde::Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub password_hash: Option<String>,
    pub is_admin: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: usize,
    #[serde(default)] // tolerate old tokens minted before is_admin existed
    pub is_admin: bool,
}

#[derive(sqlx::FromRow)]
pub struct DbOrder {
    pub id: i64,
    pub side: String,
    pub price: i32,
    pub remaining: i32,
}

#[derive(serde::Serialize)]
pub struct BookLevel {
    pub price: i32,
    pub quantity: i64,
}

#[derive(serde::Serialize)]
pub struct OrderBookView {
    pub market_id: i64,
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
}

#[derive(sqlx::FromRow)]
pub struct LevelRow {
    pub price: i32,
    pub quantity: Option<i64>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct TradePrint {
    pub price: i32,
    pub quantity: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct OrderView {
    pub id: i64,
    pub market_id: i64,
    pub side: String,
    pub price: i32,
    pub quantity: i32,
    pub remaining: i32,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
pub struct DbCancelRow {
    pub user_id: i64,
    pub market_id: i64,
    pub side: String,
    pub price: i32,
    pub remaining: i32,
    pub status: String,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct PositionView {
    pub market_id: i64,
    pub quantity: i32,   
}

#[derive(serde::Serialize)]
pub struct PortfolioPosition {
    pub market_id: i64,
    pub question: String,
    pub quantity: i32,
    pub mark_price: i32,      
    pub value_cents: i64,     
}

#[derive(serde::Serialize)]
pub struct Portfolio {
    pub balance_cents: i64,
    pub positions: Vec<PortfolioPosition>,
    pub positions_value_cents: i64,
    pub equity_cents: i64,    
}

#[derive(sqlx::FromRow)]
pub struct PortfolioRow {
    pub market_id: i64,
    pub question: String,
    pub quantity: i32,
    pub mark_price: Option<i32>,
}

#[derive(serde::Deserialize)]
pub struct PlaceOrderRequest {
    pub market_id: i64,
    pub side: String,
    pub price: i32,
    pub quantity: i32,
}

impl PlaceOrderRequest {
    pub fn validate(&self) -> Result<crate::order_book::Side, crate::error::AppError> {
        use crate::error::AppError;
        use crate::order_book::Side;

        let side = match self.side.as_str() {
            "buy" => Side::Buy,
            "sell" => Side::Sell,
            other => {
                return Err(AppError::BadRequest(format!(
                    "side must be 'buy' or 'sell', got '{other}'"
                )));
            }
        };

        if !(1..=99).contains(&self.price) {
            return Err(AppError::BadRequest(
                "price must be between 1 and 99".into(),
            ));
        }

        if !(1..=10_000).contains(&self.quantity) {
            return Err(AppError::BadRequest(
                "quantity must be between 1 and 10000".into(),
            ));
        }

        Ok(side)
    }
}
