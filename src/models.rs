// src/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password: String::new(),
            created_at: Utc::now(),
        }
    }
}
