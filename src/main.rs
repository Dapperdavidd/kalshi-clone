use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use jsonwebtoken::{DecodingKey, Validation};
use sqlx::{PgPool, postgres::PgPoolOptions};

use order_book::{Order, OrderBook, Side};
mod order_book;

#[derive(serde::Serialize, sqlx::FromRow)]
struct Market {
    id: i64,
    question: String,
    status: String,
    rail: String,
}

#[derive(serde::Deserialize)]
struct SignupRequest {
    email: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(sqlx::FromRow)]
struct User {
    id: i64,
    password_hash: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: i64,
    exp: usize,
}

#[derive(serde::Deserialize)]
struct PlaceOrderRequest {
    market_id: i64,
    side: String,
    price: i32,
    quantity: i32,
}

#[derive(sqlx::FromRow)]
struct DbOrder {
    id: i64,
    side: String,
    price: i32,
    remaining: i32,
}

#[actix_web::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let data = web::Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/health", web::get().to(health))
            .route("/db_check", web::get().to(db_check))
            .route("/v1/markets", web::get().to(markets))
            .route("/v1/markets/{id}", web::get().to(markets_id))
            .route("/v1/auth/signup", web::post().to(signup))
            .route("/v1/auth/login", web::post().to(login))
            .route("/v1/me", web::get().to(me))
            .route("/v1/orders", web::post().to(place_order))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .await
    .unwrap();
}

async fn health() -> impl Responder {
    HttpResponse::Ok().body("hello")
}

async fn db_check(pool: web::Data<PgPool>) -> impl Responder {
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(pool.get_ref())
        .await
        .unwrap();

    HttpResponse::Ok().body(result.to_string())
}

async fn markets(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Market>("SELECT id, question, status, rail FROM markets")
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    web::Json(result)
}

async fn markets_id(path: web::Path<i64>, pool: web::Data<PgPool>) -> impl Responder {
    let result =
        sqlx::query_as::<_, Market>("SELECT id, question, status, rail FROM markets WHERE id = $1")
            .bind(path.into_inner())
            .fetch_one(pool.get_ref())
            .await
            .unwrap();

    web::Json(result)
}

async fn signup(body: web::Json<SignupRequest>, pool: web::Data<PgPool>) -> impl Responder {
    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).unwrap();
    let _ = sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
        .bind(&body.email)
        .bind(&hash)
        .execute(pool.get_ref())
        .await
        .unwrap();

    HttpResponse::Created().body("user created")
}

async fn login(body: web::Json<LoginRequest>, pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, User>("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap();

    match result {
        Some(user) => {
            if bcrypt::verify(&body.password, &user.password_hash).unwrap() {
                let exp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 24 * 60 * 60;
                let claims = Claims {
                    sub: user.id,
                    exp: exp.try_into().unwrap(),
                };
                let secret = std::env::var("JWT_SECRET").unwrap();
                let token = jsonwebtoken::encode(
                    &jsonwebtoken::Header::default(),
                    &claims,
                    &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
                )
                .unwrap();
                HttpResponse::Ok().body(token)
            } else {
                HttpResponse::Unauthorized().body("invalid login details")
            }
        }
        None => HttpResponse::Unauthorized().body("invalid login details"),
    }
}

async fn me(req: HttpRequest) -> impl Responder {
    let token = req
        .headers()
        .get("Authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix("Bearer ")
        .unwrap();

    let secret = std::env::var("JWT_SECRET").unwrap();

    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .unwrap();

    HttpResponse::Ok().body(format!("you are user {}", decoded.claims.sub))
}

async fn place_order(
    req: HttpRequest,
    body: web::Json<PlaceOrderRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let token = req
        .headers()
        .get("Authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix("Bearer ")
        .unwrap();

    let secret = std::env::var("JWT_SECRET").unwrap();

    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .unwrap();

    let mut tx = pool.begin().await.unwrap();

    let mut book = load_book(pool.get_ref(), body.market_id).await;

    let taker_id: i64 = sqlx::query_scalar(
        "INSERT INTO orders (user_id, market_id, side, price, quantity, remaining) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id ",
    )
    .bind(decoded.claims.sub)
    .bind(body.market_id)
    .bind(&body.side)
    .bind(body.price)
    .bind(body.quantity)
    .bind(body.quantity)
    .fetch_one(&mut *tx)
    .await
    .unwrap();

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
        .await
        .unwrap();

        sqlx::query("UPDATE orders
                        SET remaining = remaining - $1,
                        status = CASE WHEN remaining - $1 = 0 THEN 'filled' ELSE 'partially_filled' END
                        WHERE id = $2"
                    )
            .bind(fill.quantity as i32)
            .bind(fill.maker_id as i64)
            .execute(&mut *tx)
            .await
            .unwrap();

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
    .await
    .unwrap();

    tx.commit().await.unwrap();

    HttpResponse::Created().body(format!("order {taker_id}: {} fills", fills.len()))
}

async fn load_book(pool: &PgPool, market_id: i64) -> OrderBook {
    let result = sqlx::query_as::<_, DbOrder>("SELECT id, side, price, remaining FROM orders WHERE market_id = $1 AND status IN ('working','partially_filled') AND remaining > 0 ")
    .bind(market_id)
    .fetch_all(pool)
    .await
    .unwrap();

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
    book
}
