use std::fmt::Debug;

use crate::models::activity_pub::Activity;
use crate::services::simulator_service::{BattleRequestActivity, handle_battle_request};
use actix_web::{HttpResponse, Responder, web};
use reqwest::Client;
use sqlx::PgPool;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
struct SentMessage {
    recipient: String,
    content: String,
    created_at: Option<String>, // Store the formatted date as a string
    activity_type: String,
}

pub async fn inbox(
    activity: web::Json<Activity>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    match activity.activity_type.as_str() {
        "BattleRequest" => {
            // Insert BattleRequest into messages table for logging
            sqlx::query!(
                "INSERT INTO messages (sender, recipient, content, activity_type) VALUES ($1, $2, $3, $4)",
                activity.actor,
                activity.object,
                Some("Battle initiated".to_string()),
                activity.activity_type
            )
            .execute(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Error logging BattleRequest: {:?}", e);
                actix_web::error::ErrorInternalServerError("Failed to log BattleRequest")
            })?;

            if let (Some(fleet), Some(seed)) = (&activity.fleet, activity.seed) {
                let battle_request = BattleRequestActivity {
                    activity_type: activity.activity_type.clone(),
                    actor: activity.actor.clone(),
                    target: activity.object.clone(),
                    fleet: fleet.clone(),
                    seed,
                };

                handle_battle_request(web::Json(battle_request), pool).await
            } else {
                Ok(HttpResponse::BadRequest().body("Invalid BattleRequest payload"))
            }
        }
        "Message" => {
            // Insert Message into messages table
            sqlx::query!(
                "INSERT INTO messages (sender, recipient, content, activity_type) VALUES ($1, $2, $3, $4)",
                activity.actor,
                activity.object,
                activity.content,
                activity.activity_type
            )
            .execute(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Error storing activity: {:?}", e);
                actix_web::error::ErrorInternalServerError("Failed to store activity")
            })?;

            Ok(HttpResponse::Ok().body("Activity received and stored"))
        }
        _ => Ok(HttpResponse::BadRequest().body("Unsupported activity type")),
    }
}

pub async fn outbox(username: web::Path<String>, pool: web::Data<PgPool>) -> impl Responder {
    log::info!("Fetching outbox for username: {}", username);

    let rows = sqlx::query!(
        "SELECT recipient, content, created_at, activity_type FROM messages WHERE sender = $1",
        username.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            log::info!("Fetched {} messages from outbox.", rows.len());
            let messages: Vec<SentMessage> = rows
                .into_iter()
                .map(|row| SentMessage {
                    recipient: row.recipient,
                    content: row.content,
                    created_at: row.created_at.map(|dt| dt.to_string()),
                    activity_type: row.activity_type,
                })
                .collect();
            HttpResponse::Ok().json(messages)
        }
        Err(e) => {
            log::error!("Error fetching outbox: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch outbox")
        }
    }
}
