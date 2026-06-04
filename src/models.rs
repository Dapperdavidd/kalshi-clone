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

#[derive(serde::Deserialize)]
pub struct PlaceOrderRequest {
    pub market_id: i64,
    pub side: String,
    pub price: i32,
    pub quantity: i32,
}

#[derive(sqlx::FromRow)]
pub struct DbOrder {
    pub id: i64,
    pub side: String,
    pub price: i32,
    pub remaining: i32,
}
