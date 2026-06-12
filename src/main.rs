use actix_cors::Cors;
use actix_web::{App, HttpServer, http, middleware::Logger, web};
use sqlx::postgres::PgPoolOptions;

mod account;
mod auth;
mod config;
mod error;
mod events;
mod extractors;
mod health;
mod markets;
mod models;
mod oauth;
mod order_book;
mod orders;
mod settlement;
mod ws;

use config::Config;
use events::Broadcaster;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = Config::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("failed to connect to Postgres");

    // Run migrations on boot so a fresh deploy provisions its own schema.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let data = web::Data::new(pool);
    let http = web::Data::new(reqwest::Client::new());
    let config_data = web::Data::new(config.clone());
    let broadcaster = web::Data::new(Broadcaster::new());
    let bind_addr = config.bind_addr.clone();

    log::info!(
        "starting server on {bind_addr}, CORS origin {}",
        config.frontend_origin
    );

    HttpServer::new(move || {
        // Build the CORS policy from config each worker.
        let cors = Cors::default()
            .allowed_origin(&config.frontend_origin)
            .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(data.clone())
            .app_data(http.clone())
            .app_data(config_data.clone())
            .app_data(broadcaster.clone())
            .wrap(cors) // CORS first
            .wrap(Logger::new("%r %s %Dms")) // then request logging
            // health
            .route("/health", web::get().to(health::health))
            .route("/db_check", web::get().to(health::db_check))
            // realtime
            .route("/ws", web::get().to(ws::ws))
            // markets
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
            // auth
            .route("/v1/auth/signup", web::post().to(auth::signup))
            .route("/v1/auth/login", web::post().to(auth::login))
            .route("/v1/auth/google", web::post().to(oauth::google_login))
            .route("/v1/me", web::get().to(auth::me))
            // orders
            .route("/v1/orders", web::post().to(orders::place_order))
            .route("/v1/orders", web::get().to(orders::list_orders))
            .route("/v1/orders/{id}", web::delete().to(orders::cancel_order))
            // account
            .route("/v1/positions", web::get().to(account::positions))
            .route("/v1/balance", web::get().to(account::balance))
            .route("/v1/portfolio", web::get().to(account::portfolio))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
