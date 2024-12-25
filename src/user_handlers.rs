use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use uuid::Uuid;

use crate::models::User;

// In a real application, you might store data in a DB.
// Here, we'll simulate storage with an in-memory vector or HashMap.
use std::sync::{Arc, Mutex};

type UserStorage = Arc<Mutex<Vec<User>>>;

/// Create new user
#[post("/users")]
pub async fn create_user(
    storage: web::Data<UserStorage>,
    user_data: web::Json<(String, String)>, // (username, email) as a simple example
) -> impl Responder {
    let (username, email) = user_data.into_inner();
    let mut users = storage.lock().unwrap();

    let user = User::new(username, email);
    users.push(user.clone());

    HttpResponse::Ok().json(user)
}

/// Get all users
#[get("/users")]
pub async fn get_users(storage: web::Data<UserStorage>) -> impl Responder {
    let users = storage.lock().unwrap();
    HttpResponse::Ok().json(&*users)
}

/// Get one user by ID
#[get("/users/{id}")]
pub async fn get_user(storage: web::Data<UserStorage>, path: web::Path<Uuid>) -> impl Responder {
    let user_id = path.into_inner();
    let users = storage.lock().unwrap();

    if let Some(user) = users.iter().find(|u| u.id == user_id) {
        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::NotFound().body("User not found")
    }
}

/// Update user by ID
#[put("/users/{id}")]
pub async fn update_user(
    storage: web::Data<UserStorage>,
    path: web::Path<Uuid>,
    updated_data: web::Json<(String, String)>, // (username, email)
) -> impl Responder {
    let user_id = path.into_inner();
    let (new_username, new_email) = updated_data.into_inner();
    let mut users = storage.lock().unwrap();

    if let Some(user) = users.iter_mut().find(|u| u.id == user_id) {
        user.username = new_username;
        user.email = new_email;
        return HttpResponse::Ok().json(user.clone());
    }

    HttpResponse::NotFound().body("User not found")
}

/// Delete user by ID
#[delete("/users/{id}")]
pub async fn delete_user(storage: web::Data<UserStorage>, path: web::Path<Uuid>) -> impl Responder {
    let user_id = path.into_inner();
    let mut users = storage.lock().unwrap();

    let len_before = users.len();
    users.retain(|user| user.id != user_id);

    if users.len() < len_before {
        HttpResponse::Ok().body("User deleted")
    } else {
        HttpResponse::NotFound().body("User not found")
    }
}
