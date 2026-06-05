use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;

use crate::error::AppError;

pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("hello")
}

pub async fn db_check(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().body(result.to_string()))
}
