use crate::models::activity_pub::Activity;
use actix_web::{HttpResponse, Responder, web};
use reqwest::Client;

pub async fn inbox(activity: web::Json<Activity>) -> impl Responder {
    println!("Received activity: {:?}", activity);
    HttpResponse::Ok().finish()
}

pub async fn outbox() -> impl Responder {
    // Return a paginated list of sent activities
    let activities = vec![Activity {
        activity_type: "Offer".to_string(),
        actor: "player@server1.game.net".to_string(),
        object: "resource_trade".to_string(),
    }];
    HttpResponse::Ok().json(activities)
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
