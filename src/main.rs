mod auth;
mod config;
mod handlers;
mod models;

use actix_web::{App, HttpServer, web};
use config::Config;
use handlers::activity_pub::{inbox, outbox};
use handlers::webfinger::webfinger;
use sqlx::PgPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // optional: logs

    let config = Config::from_env();
    let pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres");
    let jwt_secret = config.jwt_secret.clone();
    HttpServer::new(move || {
        App::new()
            // store the pool in App data so routes can access
            .app_data(web::Data::new(pool.clone()))
            // also store the JWT secret if you want via app data
            .app_data(web::Data::new(jwt_secret.clone()))
            // user routes (register, login, me) from user_handlers
            .configure(handlers::user::config)
            .configure(handlers::simulator::config)
            .configure(handlers::fleet::config)
            // SSE + WebSockets
            .route("/sse", web::get().to(handlers::sse::sse_endpoint))
            .route("/ws/", web::get().to(handlers::websocket::ws_index))
            // .route("/actor/{username}/inbox", web::post().to(inbox))
            // .route("/actor/{username}/outbox", web::get().to(outbox))
            .route("/actor/{username}/inbox", web::post().to(inbox))
            .route("/actor/{username}/outbox", web::get().to(outbox))
            .route("/.well-known/webfinger", web::get().to(webfinger))
            .route(
                "/battle-request/",
                web::post().to(handlers::simulator::send_battle_request_handler),
            )
    })
    // .bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
