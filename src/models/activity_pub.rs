use crate::handlers::simulator::Fleet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Actor {
    pub id: String, // Actor's unique ID (e.g., URI)
    #[serde(rename = "type")]
    pub actor_type: String, // Actor type (e.g., "Person", "Service")
    pub inbox: String, // URL of the actor's inbox
    pub outbox: String, // URL of the actor's outbox
    pub preferred_username: String, // Human-readable username
    pub name: Option<String>, // Optional display name for the actor
    pub summary: Option<String>, // Optional bio or summary
    pub public_key: Option<PublicKey>, // Optional public key for verification
}

#[derive(Serialize, Deserialize)]
pub struct PublicKey {
    pub id: String,             // Identifier for the key
    pub owner: String,          // Owner of the key (actor's ID)
    pub public_key_pem: String, // PEM-encoded public key
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    #[serde(rename = "type")]
    pub activity_type: String, // Activity type (e.g., "BattleRequest", "Message")
    pub actor: String,           // Actor who performed the activity
    pub object: String,          // Target object of the activity
    pub to: Option<Vec<String>>, // Optional recipients
    pub content: Option<String>, // Optional content for the activity
    pub fleet: Option<Fleet>,    // Optional fleet information for BattleRequest
    pub seed: Option<u64>,       // Optional seed for BattleRequest
}
