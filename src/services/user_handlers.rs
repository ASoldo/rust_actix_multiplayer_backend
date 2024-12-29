use crate::auth;
use crate::models::user::User;
use actix_web::{Error, HttpRequest, HttpResponse, web};
use chrono::{Duration, Utc};
use rand::RngCore; // for generating random tokens
use rand::rngs::OsRng;
use sqlx::PgPool;
use uuid::Uuid;

use actix_web::web::ServiceConfig;

#[derive(serde::Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

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
    let hashed = auth::password::hash_password(&form.password);

    // Append domain to the username
    let username_with_domain = format!("{}@localhost", form.username);

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
        username_with_domain,
        form.email,
        hashed
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?; // Generate JWT or do other logic...

    let token = auth::jwt::generate_jwt(&inserted_user.id.to_string(), "secret");

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
    jwt_secret: web::Data<String>, // from .env
) -> Result<HttpResponse, Error> {
    // 1) Find user by email
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

    // 2) Check password
    if !auth::password::verify_password(&user.password, &form.password) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid credentials"));
    }

    // 3) Generate short-lived access token
    let access_token = auth::jwt::generate_jwt(&user.id.to_string(), &jwt_secret);

    // 4) Generate refresh token
    let refresh_str = generate_refresh_token();
    // store in DB with an expiry, e.g. 14 days from now
    let refresh_exp = Utc::now() + Duration::days(14);
    let rt_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        rt_id,
        user.id,
        refresh_str,
        refresh_exp
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 5) Return JSON with both tokens
    let resp = serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_str,
        "user": {
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "created_at": user.created_at
        }
    });

    Ok(HttpResponse::Ok().json(resp))
}

pub async fn get_me(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    jwt_secret: web::Data<String>, // <-- retrieve from App data
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
    // Instead of a hard-coded "secret", use the real secret from .env
    let claims = auth::jwt::decode_jwt(token, &jwt_secret)
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
    cfg.route("/refresh", web::post().to(refresh_token));
}

// For the refresh token
pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes) // a hex string like "aabbcc..."
}

#[derive(serde::Deserialize)]
pub struct RefreshDto {
    pub refresh_token: String,
}

pub async fn refresh_token(
    pool: web::Data<PgPool>,
    form: web::Json<RefreshDto>,
    jwt_secret: web::Data<String>,
) -> Result<HttpResponse, Error> {
    // 1) find refresh token row
    let row = sqlx::query!(
        r#"
        SELECT rt.user_id, rt.expires_at, u.username, u.email, u.password, u.created_at
        FROM refresh_tokens rt
        JOIN users u ON rt.user_id = u.id
        WHERE rt.token = $1
        "#,
        form.refresh_token
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid refresh token"))?;

    // 2) check expiry
    let now = Utc::now();
    if row.expires_at < now {
        return Err(actix_web::error::ErrorUnauthorized("Refresh token expired"));
    }

    // 3) generate a new short-lived access token
    let new_access = auth::jwt::generate_jwt(&row.user_id.to_string(), &jwt_secret);

    // 4) optional: rotate refresh token or keep the same. For simplicity, let's keep it the same.

    // 5) return the new access token & existing refresh token
    let resp = serde_json::json!({
        "access_token": new_access,
        "refresh_token": form.refresh_token, // or a new one if rotating
        "user": {
            "id": row.user_id,
            "username": row.username,
            "email": row.email,
            "created_at": row.created_at
        }
    });

    Ok(HttpResponse::Ok().json(resp))
}
