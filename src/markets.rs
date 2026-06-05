use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::Market;

pub async fn markets(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let result = sqlx::query_as::<_, Market>("SELECT id, question, status, rail FROM markets")
        .fetch_all(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn markets_id(
    path: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let result =
        sqlx::query_as::<_, Market>("SELECT id, question, status, rail FROM markets WHERE id = $1")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or_else(|| AppError::NotFound(format!("market {id} not found")))?;

    Ok(HttpResponse::Ok().json(result))
}
