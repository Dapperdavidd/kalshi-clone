use actix_web::{HttpRequest, HttpResponse, Responder, web};
use jsonwebtoken::{DecodingKey, Validation};
use sqlx::PgPool;

use crate::models::{Claims, LoginRequest, SignupRequest, User};

/// Pull the JWT off the `Authorization: Bearer <token>` header, verify it,
/// and return the authenticated user's id. Shared by every protected handler.
pub fn authenticate(req: &HttpRequest) -> i64 {
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

    decoded.claims.sub
}

pub async fn signup(body: web::Json<SignupRequest>, pool: web::Data<PgPool>) -> impl Responder {
    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).unwrap();
    let _ = sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
        .bind(&body.email)
        .bind(&hash)
        .execute(pool.get_ref())
        .await
        .unwrap();

    HttpResponse::Created().body("user created")
}

pub async fn login(body: web::Json<LoginRequest>, pool: web::Data<PgPool>) -> impl Responder {
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

pub async fn me(req: HttpRequest) -> impl Responder {
    let user_id = authenticate(&req);
    HttpResponse::Ok().body(format!("you are user {}", user_id))
}
