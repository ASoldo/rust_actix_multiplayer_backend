use std::fmt::Debug;

use crate::models::activity_pub::Activity;
use actix_web::{HttpResponse, Responder, web};
use reqwest::Client;
use sqlx::PgPool;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
struct SentMessage {
    recipient: String,
    content: String,
    created_at: String, // Store the formatted date as a string
    activity_type: String,
}

pub async fn inbox(activity: web::Json<Activity>, pool: web::Data<PgPool>) -> impl Responder {
    if activity.activity_type == "Message" {
        sqlx::query!(
            "INSERT INTO messages (sender, recipient, content, activity_type) VALUES ($1, $2, $3, $4)",
            activity.actor,         // sender
            activity.object,        // recipient
            activity.content,       // message content
            activity.activity_type  // type of activity
        )
        .execute(pool.get_ref())
        .await
        .expect("Failed to insert message");
    }

    println!("Received activity: {:?}", activity);
    HttpResponse::Ok().finish()
}

pub async fn outbox(username: web::Path<String>, pool: web::Data<PgPool>) -> impl Responder {
    let username = if username.contains('@') {
        username.into_inner()
    } else {
        format!("{}@localhost", username.into_inner())
    };

    println!("Fetching outbox for username: {}", username);

    let rows = sqlx::query!(
        "SELECT recipient, content, created_at, activity_type FROM messages WHERE sender = $1",
        username
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            if rows.is_empty() {
                println!("No messages found for sender: {}", username);
            }

            let sent_messages: Vec<SentMessage> = rows
                .into_iter()
                .map(|row| SentMessage {
                    recipient: row.recipient,
                    content: row.content,
                    created_at: row
                        .created_at
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    activity_type: row.activity_type,
                })
                .collect();

            println!("Outbox for {}: {:?}", username, sent_messages);

            HttpResponse::Ok().json(sent_messages)
        }
        Err(err) => {
            println!("Failed to fetch outbox messages: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn send_activity(activity: Activity, target_inbox: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    client
        .post(target_inbox)
        .header("Content-Type", "application/activity+json")
        .json(&activity)
        .send()
        .await?;
    Ok(())
}
