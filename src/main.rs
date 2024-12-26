// src/main.rs

use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use std::env;

use sqlx::PgPool;

mod auth;
mod jwt;
mod models;
mod sse; // if you want to keep SSE
mod user_handlers;
mod websocket; // if you want to keep WebSocket

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // load .env
    env_logger::init(); // optional: logs

    // read DB URL from .env
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    // connect to Postgres
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // optional: read JWT_SECRET from env or fallback
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    HttpServer::new(move || {
        App::new()
            // store the pool in App data so routes can access
            .app_data(web::Data::new(pool.clone()))
            // also store the JWT secret if you want via app data
            .app_data(web::Data::new(jwt_secret.clone()))
            // user routes (register, login, me) from user_handlers
            .configure(user_handlers::config)
            // SSE + WebSockets
            .route("/sse", web::get().to(sse::sse_endpoint))
            .route("/ws/", web::get().to(websocket::ws_index))
    })
    // .bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
