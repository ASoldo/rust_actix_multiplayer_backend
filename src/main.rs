use actix_web::{App, HttpServer, web};
use std::sync::{Arc, Mutex};

mod models; // your model
mod sse; // your SSE code
mod user_handlers; // your user routes
mod websocket; // your websocket code

use models::User;
use sse::*;
use user_handlers::*;
use websocket::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let user_storage = Arc::new(Mutex::new(Vec::<User>::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(user_storage.clone()))
            .service(create_user)
            .service(get_users)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .route("/sse", web::get().to(sse_endpoint))
            .route("/ws/", web::get().to(ws_index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
