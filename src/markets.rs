use actix_web::{Responder, web};
use sqlx::PgPool;

use crate::models::Market;

pub async fn markets(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Market>("SELECT id, question, status, rail FROM markets")
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    web::Json(result)
}

pub async fn markets_id(path: web::Path<i64>, pool: web::Data<PgPool>) -> impl Responder {
    let result =
        sqlx::query_as::<_, Market>("SELECT id, question, status, rail FROM markets WHERE id = $1")
            .bind(path.into_inner())
            .fetch_one(pool.get_ref())
            .await
            .unwrap();

    web::Json(result)
}
