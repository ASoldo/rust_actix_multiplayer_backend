use actix_web::{HttpResponse, Responder, web};
use serde_json::json;
use std::collections::HashMap;

pub async fn webfinger(query: web::Query<HashMap<String, String>>) -> impl Responder {
    if let Some(resource) = query.get("resource") {
        // Ensure the resource is in the expected format (e.g., "acct:username@domain")
        if resource.starts_with("acct:") {
            let parts: Vec<&str> = resource.split('@').collect();
            if parts.len() == 2 {
                let username = parts[0].strip_prefix("acct:").unwrap_or("");
                let domain = parts[1];
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
    HttpResponse::NotFound().finish()
}
