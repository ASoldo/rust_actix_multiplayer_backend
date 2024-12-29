use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct FleetRequest {
    username: String,
    ships: i32,
    fighters: i32,
    bombers: i32,
}

pub async fn create_fleet(pool: web::Data<PgPool>, req: web::Json<FleetRequest>) -> impl Responder {
    let user = sqlx::query!("SELECT id FROM users WHERE username = $1", req.username)
        .fetch_one(pool.get_ref())
        .await;

    match user {
        Ok(user) => {
            let result = sqlx::query!(
                "INSERT INTO fleets (user_id, ships, fighters, bombers) VALUES ($1, $2, $3, $4)",
                user.id,
                req.ships,
                req.fighters,
                req.bombers
            )
            .execute(pool.get_ref())
            .await;

            match result {
                Ok(_) => HttpResponse::Ok().body("Fleet created successfully"),
                Err(e) => {
                    eprintln!("Error creating fleet: {:?}", e);
                    HttpResponse::InternalServerError().body("Error creating fleet")
                }
            }
        }
        Err(_) => HttpResponse::NotFound().body("User not found"),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/create_fleet", web::post().to(create_fleet));
}
