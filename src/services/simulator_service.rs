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
        let b_ships = player_b.ships.unwrap_or(0);
        player_b.ships = Some(b_ships.saturating_sub(damage_to_b));

        if damage_to_b > b_ships {
            let remaining_damage = damage_to_b - b_ships;
            let b_fighters = player_b.fighters.unwrap_or(0);
            player_b.fighters = Some(b_fighters.saturating_sub(remaining_damage));
        }

        if damage_to_b > b_ships + player_b.fighters.unwrap_or(0) {
            let remaining_damage = damage_to_b - b_ships - player_b.fighters.unwrap_or(0);
            let b_bombers = player_b.bombers.unwrap_or(0);
            player_b.bombers = Some(b_bombers.saturating_sub(remaining_damage));
        }

        // Apply damage to Player A
        let a_ships = player_a.ships.unwrap_or(0);
        player_a.ships = Some(a_ships.saturating_sub(damage_to_a));

        if damage_to_a > a_ships {
            let remaining_damage = damage_to_a - a_ships;
            let a_fighters = player_a.fighters.unwrap_or(0);
            player_a.fighters = Some(a_fighters.saturating_sub(remaining_damage));
        }

        if damage_to_a > a_ships + player_a.fighters.unwrap_or(0) {
            let remaining_damage = damage_to_a - a_ships - player_a.fighters.unwrap_or(0);
            let a_bombers = player_a.bombers.unwrap_or(0);
            player_a.bombers = Some(a_bombers.saturating_sub(remaining_damage));
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
    // Fetch fleets for both players

    let player_a_fleet = sqlx::query_as!(
        Fleet,
        "SELECT ships, fighters, bombers FROM fleets WHERE username = $1",
        req.player_a
    )
    .fetch_one(pool.get_ref())
    .await
    .expect("Player A not found");

    // Use `unwrap_or(0)` for calculations if needed
    let total_ships = player_a_fleet.ships.unwrap_or(0);

    let player_b_fleet = sqlx::query_as!(
        Fleet,
        "SELECT ships, fighters, bombers FROM users WHERE username = $1",
        req.player_b
    )
    .fetch_one(pool.get_ref())
    .await
    .expect("Player B not found");

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
