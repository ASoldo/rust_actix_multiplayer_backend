use actix_web::{HttpResponse, Responder, web};
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct BattleRequestActivity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub target: String,
    pub fleet: Fleet,
    pub seed: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BattleResultActivity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub target: String,
    pub result: BattleOutcome,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct Fleet {
    ships: Option<i32>,
    fighters: Option<i32>,
    bombers: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BattleOutcome {
    winner: String,
    player_a_remaining: Fleet,
    player_b_remaining: Fleet,
}

#[derive(Deserialize)]
pub struct BattleRequest {
    player_a: String, // username of player A
    player_b: String, // username of player B
    seed: u64,        // Optional seed for reproducibility
}

#[derive(Serialize)]
pub struct BattleResponse {
    winner: String,
    player_a_remaining: Fleet,
    player_b_remaining: Fleet,
}

pub fn simulate_battle(mut player_a: Fleet, mut player_b: Fleet, seed: u64) -> BattleOutcome {
    let mut rng = Pcg64::seed_from_u64(seed);

    while (player_a.ships.unwrap_or(0) > 0
        || player_a.fighters.unwrap_or(0) > 0
        || player_a.bombers.unwrap_or(0) > 0)
        && (player_b.ships.unwrap_or(0) > 0
            || player_b.fighters.unwrap_or(0) > 0
            || player_b.bombers.unwrap_or(0) > 0)
    {
        let damage_to_b = rng.gen_range(1..10);
        let damage_to_a = rng.gen_range(1..10);

        // Apply damage to Player B
        let mut remaining_damage = damage_to_b;

        if let Some(b_ships) = player_b.ships {
            let effective_damage = b_ships.min(remaining_damage);
            player_b.ships = Some(b_ships - effective_damage);
            remaining_damage -= effective_damage;
        }

        if remaining_damage > 0 {
            if let Some(b_fighters) = player_b.fighters {
                let effective_damage = b_fighters.min(remaining_damage);
                player_b.fighters = Some(b_fighters - effective_damage);
                remaining_damage -= effective_damage;
            }
        }

        if remaining_damage > 0 {
            if let Some(b_bombers) = player_b.bombers {
                let effective_damage = b_bombers.min(remaining_damage);
                player_b.bombers = Some(b_bombers - effective_damage);
            }
        }

        // Apply damage to Player A
        let mut remaining_damage = damage_to_a;

        if let Some(a_ships) = player_a.ships {
            let effective_damage = a_ships.min(remaining_damage);
            player_a.ships = Some(a_ships - effective_damage);
            remaining_damage -= effective_damage;
        }

        if remaining_damage > 0 {
            if let Some(a_fighters) = player_a.fighters {
                let effective_damage = a_fighters.min(remaining_damage);
                player_a.fighters = Some(a_fighters - effective_damage);
                remaining_damage -= effective_damage;
            }
        }

        if remaining_damage > 0 {
            if let Some(a_bombers) = player_a.bombers {
                let effective_damage = a_bombers.min(remaining_damage);
                player_a.bombers = Some(a_bombers - effective_damage);
            }
        }
    }

    // Determine the winner by comparing total remaining units
    let total_a = player_a.ships.unwrap_or(0)
        + player_a.fighters.unwrap_or(0)
        + player_a.bombers.unwrap_or(0);
    let total_b = player_b.ships.unwrap_or(0)
        + player_b.fighters.unwrap_or(0)
        + player_b.bombers.unwrap_or(0);

    let winner = if total_a > total_b {
        "Player A".to_string()
    } else if total_b > total_a {
        "Player B".to_string()
    } else {
        "Draw".to_string() // In case of a tie
    };

    BattleOutcome {
        winner,
        player_a_remaining: player_a,
        player_b_remaining: player_b,
    }
}

pub async fn battle_handler(
    req: web::Json<BattleRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    // Fetch fleets for both players by joining `fleets` and `users` tables
    let player_a_fleet = sqlx::query_as!(
        Fleet,
        r#"
        SELECT f.ships, f.fighters, f.bombers 
        FROM fleets f
        INNER JOIN users u ON f.user_id = u.id
        WHERE u.username = $1
        "#,
        req.player_a
    )
    .fetch_one(pool.get_ref())
    .await
    .expect("Player A fleet not found");

    let player_b_fleet = sqlx::query_as!(
        Fleet,
        r#"
        SELECT f.ships, f.fighters, f.bombers 
        FROM fleets f
        INNER JOIN users u ON f.user_id = u.id
        WHERE u.username = $1
        "#,
        req.player_b
    )
    .fetch_one(pool.get_ref())
    .await
    .expect("Player B fleet not found");

    // Simulate the battle
    let outcome = simulate_battle(player_a_fleet, player_b_fleet, req.seed);

    // Update fleets for Player A
    sqlx::query!(
        r#"
        UPDATE fleets
        SET ships = $1, fighters = $2, bombers = $3
        WHERE user_id = (SELECT id FROM users WHERE username = $4)
        "#,
        outcome.player_a_remaining.ships,
        outcome.player_a_remaining.fighters,
        outcome.player_a_remaining.bombers,
        req.player_a
    )
    .execute(pool.get_ref())
    .await
    .expect("Failed to update Player A's fleet");

    // Update fleets for Player B
    sqlx::query!(
        r#"
        UPDATE fleets
        SET ships = $1, fighters = $2, bombers = $3
        WHERE user_id = (SELECT id FROM users WHERE username = $4)
        "#,
        outcome.player_b_remaining.ships,
        outcome.player_b_remaining.fighters,
        outcome.player_b_remaining.bombers,
        req.player_b
    )
    .execute(pool.get_ref())
    .await
    .expect("Failed to update Player B's fleet");

    let response = BattleResponse {
        winner: outcome.winner,
        player_a_remaining: outcome.player_a_remaining,
        player_b_remaining: outcome.player_b_remaining,
    };

    HttpResponse::Ok().json(response)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/simulate_battle", web::post().to(battle_handler));
}

pub async fn handle_battle_request(
    activity: web::Json<BattleRequestActivity>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    // Fetch actor's fleet
    let actor_fleet = sqlx::query_as!(
        Fleet,
        "SELECT ships, fighters, bombers 
         FROM fleets 
         WHERE user_id = (SELECT id FROM users WHERE username = $1)",
        activity.actor
    )
    .fetch_one(pool.get_ref())
    .await;

    if actor_fleet.is_err() {
        return Ok(HttpResponse::BadRequest().body("Actor fleet not found"));
    }
    let actor_fleet = actor_fleet.unwrap();

    // Fetch target's fleet
    let target_fleet = sqlx::query_as!(
        Fleet,
        "SELECT ships, fighters, bombers 
         FROM fleets 
         WHERE user_id = (SELECT id FROM users WHERE username = $1)",
        activity.target
    )
    .fetch_one(pool.get_ref())
    .await;

    if target_fleet.is_err() {
        return Ok(HttpResponse::BadRequest().body("Target fleet not found"));
    }
    let target_fleet = target_fleet.unwrap();

    // Simulate the battle
    let outcome = simulate_battle(actor_fleet.clone(), target_fleet.clone(), activity.seed);

    // Update actor's fleet
    sqlx::query!(
        "UPDATE fleets 
         SET ships = $1, fighters = $2, bombers = $3 
         WHERE user_id = (SELECT id FROM users WHERE username = $4)",
        outcome.player_a_remaining.ships,
        outcome.player_a_remaining.fighters,
        outcome.player_a_remaining.bombers,
        activity.actor
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Failed to update actor fleet: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to update actor fleet")
    })?;

    // Update target's fleet
    sqlx::query!(
        "UPDATE fleets 
         SET ships = $1, fighters = $2, bombers = $3 
         WHERE user_id = (SELECT id FROM users WHERE username = $4)",
        outcome.player_b_remaining.ships,
        outcome.player_b_remaining.fighters,
        outcome.player_b_remaining.bombers,
        activity.target
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Failed to update target fleet: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to update target fleet")
    })?;

    // Return battle result
    Ok(HttpResponse::Ok().json(outcome))
}

async fn _update_fleet(username: String, updated_fleet: Fleet, pool: web::Data<PgPool>) {
    sqlx::query!(
        "UPDATE fleets SET ships = $1, fighters = $2, bombers = $3 WHERE user_id = (SELECT id FROM users WHERE username = $4)",
        updated_fleet.ships,
        updated_fleet.fighters,
        updated_fleet.bombers,
        username
    )
    .execute(pool.get_ref())
    .await
    .expect("Failed to update fleet");
}

pub async fn send_battle_request(
    activity: BattleRequestActivity,
    target_inbox: &str,
) -> Result<(), reqwest::Error> {
    let client = Client::new();
    client
        .post(target_inbox)
        .header("Content-Type", "application/activity+json")
        .json(&activity)
        .send()
        .await?;
    Ok(())
}

pub async fn send_battle_request_handler(
    activity: web::Json<BattleRequestActivity>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    // Fetch the actor's fleet from the database
    let actor_fleet = sqlx::query_as!(
        Fleet,
        "SELECT ships, fighters, bombers 
         FROM fleets 
         WHERE user_id = (SELECT id FROM users WHERE username = $1)",
        activity.actor
    )
    .fetch_one(pool.get_ref())
    .await;

    if actor_fleet.is_err() {
        return HttpResponse::BadRequest().body("Actor fleet not found");
    }

    let actor_fleet = actor_fleet.unwrap();

    // Construct the activity with the fleet from the database
    let battle_request = BattleRequestActivity {
        activity_type: activity.activity_type.clone(),
        actor: activity.actor.clone(),
        target: activity.target.clone(),
        fleet: actor_fleet,
        seed: activity.seed,
    };

    // Send the battle request to the target inbox
    let target_inbox = format!(
        "http://127.0.0.1:8080/actor/{}@localhost/inbox",
        activity.target
    );

    match send_battle_request(battle_request, &target_inbox).await {
        Ok(_) => HttpResponse::Ok().body("Battle request sent successfully"),
        Err(e) => {
            eprintln!("Error sending battle request: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to send battle request")
        }
    }
}
