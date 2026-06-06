#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Market {
    pub id: i64,
    pub question: String,
    pub status: String,
    pub rail: String,
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
    pub password_hash: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: usize,
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
    pub side: String,
    pub price: i32,
    pub remaining: i32,
    pub status: String,
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
