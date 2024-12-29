use actix_web::{HttpResponse, Responder, web};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn webfinger(
    query: web::Query<HashMap<String, String>>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    if let Some(resource) = query.get("resource") {
        if resource.starts_with("acct:") {
            let parts: Vec<&str> = resource.split('@').collect();
            if parts.len() == 2 {
                let username = parts[0].strip_prefix("acct:").unwrap_or("");
                let domain = parts[1];

                // Query the database for the user
                let user_exists =
                    sqlx::query_scalar!("SELECT 1 FROM users WHERE username = $1", username)
                        .fetch_optional(pool.get_ref())
                        .await
                        .is_ok();

                if user_exists {
                    return HttpResponse::Ok().json(json!({
                        "subject": resource,
                        "links": [
                            {
                                "rel": "self",
                                "type": "application/activity+json",
                                "href": format!("http://{}/actor/{}", domain, username)
                            }
                        ]
                    }));
                }
            }
        }
    }
    HttpResponse::NotFound().finish()
}
