use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;

mod auth;
mod health;
mod markets;
mod models;
mod order_book;
mod orders;

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
            .route("/health", web::get().to(health::health))
            .route("/db_check", web::get().to(health::db_check))
            .route("/v1/markets", web::get().to(markets::markets))
            .route("/v1/markets/{id}", web::get().to(markets::markets_id))
            .route("/v1/auth/signup", web::post().to(auth::signup))
            .route("/v1/auth/login", web::post().to(auth::login))
            .route("/v1/me", web::get().to(auth::me))
            .route("/v1/orders", web::post().to(orders::place_order))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .await
    .unwrap();
}
