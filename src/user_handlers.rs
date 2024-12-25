// user_handlers.rs (example)
use crate::auth::{hash_password, verify_password};
use crate::jwt::generate_jwt;
use crate::jwt::decode_jwt;
use crate::models::User;
use actix_web::{Error, HttpRequest, HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use actix_web::web::ServiceConfig;

#[derive(serde::Deserialize)]
pub struct RegisterDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

// Example: Insert user
pub async fn register_user(
    pool: web::Data<PgPool>,
    form: web::Json<RegisterDto>,
) -> Result<HttpResponse, Error> {
    let hashed = hash_password(&form.password);

    // Use explicit casts for returning columns:
    // e.g.  id as "id: Uuid", created_at as "created_at: chrono::DateTime<Utc>"
    let inserted_user = sqlx::query_as!(
        User,
        r#"
    INSERT INTO users (id, username, email, password)
    VALUES ($1, $2, $3, $4)
    RETURNING
      id         as "id: Uuid",
      username   as "username!",
      email      as "email!",
      password   as "password!",
      created_at as "created_at!: chrono::DateTime<Utc>"
    "#,
        Uuid::new_v4(),
        form.username,
        form.email,
        hashed
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?; // Generate JWT or do other logic...
    let token = generate_jwt(&inserted_user.id.to_string(), "secret");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": {
            "id": inserted_user.id,
            "username": inserted_user.username,
            "email": inserted_user.email,
            "created_at": inserted_user.created_at,
        },
        "token": token
    })))
}

#[derive(serde::Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

pub async fn login_user(
    pool: web::Data<PgPool>,
    form: web::Json<LoginDto>,
) -> Result<HttpResponse, Error> {
    // 1) Query user by email
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
          id         as "id: Uuid",
          username   as "username!",
          email      as "email!",
          password   as "password!",
          created_at as "created_at!: chrono::DateTime<Utc>"
        FROM users
        WHERE email = $1
        "#,
        form.email
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid credentials"))?;

    // 2) Verify password
    if !verify_password(&user.password, &form.password) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid credentials"));
    }

    // 3) Generate JWT
    let token = generate_jwt(&user.id.to_string(), "secret");

    // 4) Return user + token
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": {
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "created_at": user.created_at
        },
        "token": token
    })))
}

pub async fn get_me(
    pool: web::Data<PgPool>,
    req: HttpRequest, // we read the Authorization header from here
) -> Result<HttpResponse, Error> {
    // 1) Grab Authorization header
    let auth_header = req.headers().get("Authorization");
    if auth_header.is_none() {
        return Err(actix_web::error::ErrorUnauthorized("Missing token"));
    }

    let header_str = auth_header
        .unwrap()
        .to_str()
        .map_err(|_| actix_web::error::ErrorUnauthorized("Bad auth header"))?;
    if !header_str.starts_with("Bearer ") {
        return Err(actix_web::error::ErrorUnauthorized("Invalid token format"));
    }

    let token = &header_str[7..];

    // 2) Decode the JWT
    let secret = "secret"; // or from .env
    let claims = decode_jwt(token, secret)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token"))?;

    // 3) Extract user ID from claims
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid user ID"))?;

    // 4) Query DB for user
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
          id         as "id: Uuid",
          username   as "username!",
          email      as "email!",
          password   as "password!",
          created_at as "created_at!: chrono::DateTime<Utc>"
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorUnauthorized("User not found"))?;

    // 5) Return user (omitting password)
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "created_at": user.created_at
    })))
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.route("/register", web::post().to(register_user));
    cfg.route("/login", web::post().to(login_user));
    cfg.route("/me", web::get().to(get_me));
}
