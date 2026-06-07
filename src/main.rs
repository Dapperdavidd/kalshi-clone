use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;

mod account;
mod auth;
mod error;
mod extractors;
mod health;
mod markets;
mod models;
mod order_book;
mod orders;
mod settlement;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let data = web::Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/health", web::get().to(health::health))
            .route("/db_check", web::get().to(health::db_check))
            .route("/v1/markets", web::get().to(markets::markets))
            .route("/v1/markets/{id}", web::get().to(markets::markets_id))
            .route(
                "/v1/markets/{id}/orderbook",
                web::get().to(markets::orderbook),
            )
            .route(
                "/v1/markets/{id}/trades",
                web::get().to(markets::market_trades),
            )
            .route(
                "/v1/markets/{id}/resolve",
                web::post().to(settlement::resolve_market),
            )
            .route("/v1/auth/signup", web::post().to(auth::signup))
            .route("/v1/auth/login", web::post().to(auth::login))
            .route("/v1/me", web::get().to(auth::me))
            .route("/v1/orders", web::post().to(orders::place_order))
            .route("/v1/orders", web::get().to(orders::list_orders))
            .route("/v1/orders/{id}", web::delete().to(orders::cancel_order))
            .route("/v1/positions", web::get().to(account::positions))
            .route("/v1/balance", web::get().to(account::balance))
            .route("/v1/portfolio", web::get().to(account::portfolio))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
