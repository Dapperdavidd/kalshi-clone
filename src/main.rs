use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use jsonwebtoken::{DecodingKey, Validation};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::collections::{BTreeMap, VecDeque};

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

#[derive(Debug, Clone, PartialEq)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    side: Side,
    price: u32,
    quantity: u32,
}

#[derive(Debug, Clone)]
struct OrderBook {
    bids: BTreeMap<u32, VecDeque<Order>>,
    asks: BTreeMap<u32, VecDeque<Order>>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn add_resting(&mut self, order: Order) {
        match order.side {
            Side::Buy => {
                self.bids
                    .entry(order.price) // look up this price in the bids map
                    .or_default() // get the queue there, or make an empty one
                    .push_back(order); // add the order to the back of that queue
            }
            Side::Sell => {
                self.asks
                    .entry(order.price) // look up this price in the bids map
                    .or_default() // get the queue there, or make an empty one
                    .push_back(order); // add the order to the back of that queue
            }
        }
    }

    fn match_order(&mut self, mut taker: Order) -> Vec<Fill> {
        let mut fills = Vec::new();

        match taker.side {
            // A BUY matches against the ASKS, cheapest first.
            Side::Buy => {
                while taker.quantity > 0 {
                    // Find the lowest ask price. If there are no asks, stop.
                    let best_price = match self.asks.keys().next() {
                        Some(&p) => p,
                        None => break,
                    };
                    // If the cheapest ask costs more than we're willing to pay, stop.
                    if best_price > taker.price {
                        break;
                    }

                    let level = self.asks.get_mut(&best_price).unwrap(); // the queue at that price
                    let maker = level.front_mut().unwrap(); // the oldest order there

                    // Trade as much as both sides allow.
                    let fill_qty = taker.quantity.min(maker.quantity);
                    fills.push(Fill {
                        price: best_price, // fill at the MAKER's price (§7.4)
                        quantity: fill_qty,
                        maker_id: maker.id,
                        taker_id: taker.id,
                    });
                    maker.quantity -= fill_qty;
                    taker.quantity -= fill_qty;

                    // If the maker is fully filled, remove it; if the level is now empty, drop it.
                    if maker.quantity == 0 {
                        level.pop_front();
                        if level.is_empty() {
                            self.asks.remove(&best_price);
                        }
                    }
                }
            }

            // A SELL matches against the BIDS, highest first.
            Side::Sell => {
                while taker.quantity > 0 {
                    // Find the lowest ask price. If there are no asks, stop.
                    let best_price = match self.bids.keys().next_back() {
                        Some(&p) => p,
                        None => break,
                    };
                    // If the cheapest ask costs more than we're willing to pay, stop.
                    if best_price < taker.price {
                        break;
                    }

                    let level = self.bids.get_mut(&best_price).unwrap(); // the queue at that price
                    let maker = level.front_mut().unwrap(); // the oldest order there

                    // Trade as much as both sides allow.
                    let fill_qty = taker.quantity.min(maker.quantity);
                    fills.push(Fill {
                        price: best_price, // fill at the MAKER's price (§7.4)
                        quantity: fill_qty,
                        maker_id: maker.id,
                        taker_id: taker.id,
                    });
                    maker.quantity -= fill_qty;
                    taker.quantity -= fill_qty;

                    // If the maker is fully filled, remove it; if the level is now empty, drop it.
                    if maker.quantity == 0 {
                        level.pop_front();
                        if level.is_empty() {
                            self.bids.remove(&best_price);
                        }
                    }
                }
            }
        }

        // Whatever didn't fill rests on the book (limit-order behavior).
        if taker.quantity > 0 {
            self.add_resting(taker);
        }

        fills
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Fill {
    price: u32,
    quantity: u32,
    maker_id: u64, // the resting order that was sitting on the book
    taker_id: u64, // the incoming order that matched against it
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
