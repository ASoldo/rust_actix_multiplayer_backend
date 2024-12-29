use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    #[serde(rename = "type")]
    pub actor_type: String,
    pub inbox: String,
    pub outbox: String,
    pub preferred_username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub object: String,
}
