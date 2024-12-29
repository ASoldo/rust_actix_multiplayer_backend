use actix_web::{HttpResponse, Responder, web};
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct Fleet {
    ships: Option<i32>,
    fighters: Option<i32>,
    bombers: Option<i32>,
}

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
