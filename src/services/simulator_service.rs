use actix_web::{HttpResponse, Responder, web};
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Fleet {
    ships: u32,
    fighters: u32,
    bombers: u32,
}

pub struct BattleOutcome {
    winner: String,
    player_a_remaining: Fleet,
    player_b_remaining: Fleet,
}

#[derive(Deserialize)]
pub struct BattleRequest {
    player_a: Fleet,
    player_b: Fleet,
    seed: u64, // Optional seed for reproducibility
}

#[derive(Serialize)]
pub struct BattleResponse {
    winner: String,
    player_a_remaining: Fleet,
    player_b_remaining: Fleet,
}

pub fn simulate_battle(mut player_a: Fleet, mut player_b: Fleet, seed: u64) -> BattleOutcome {
    let mut rng = Pcg64::seed_from_u64(seed);

    while (player_a.ships > 0 || player_a.fighters > 0 || player_a.bombers > 0)
        && (player_b.ships > 0 || player_b.fighters > 0 || player_b.bombers > 0)
    {
        let damage_to_b = rng.gen_range(1..10);
        let damage_to_a = rng.gen_range(1..10);

        // Apply damage to Player B
        player_b.ships = player_b.ships.saturating_sub(damage_to_b);
        if damage_to_b > player_b.ships {
            let remaining_damage = damage_to_b - player_b.ships;
            player_b.fighters = player_b.fighters.saturating_sub(remaining_damage);
        }
        if damage_to_b > player_b.ships + player_b.fighters {
            let remaining_damage = damage_to_b - player_b.ships - player_b.fighters;
            player_b.bombers = player_b.bombers.saturating_sub(remaining_damage);
        }

        // Apply damage to Player A
        player_a.ships = player_a.ships.saturating_sub(damage_to_a);
        if damage_to_a > player_a.ships {
            let remaining_damage = damage_to_a - player_a.ships;
            player_a.fighters = player_a.fighters.saturating_sub(remaining_damage);
        }
        if damage_to_a > player_a.ships + player_a.fighters {
            let remaining_damage = damage_to_a - player_a.ships - player_a.fighters;
            player_a.bombers = player_a.bombers.saturating_sub(remaining_damage);
        }
    }

    // Determine the winner by comparing total remaining units
    let total_a = player_a.ships + player_a.fighters + player_a.bombers;
    let total_b = player_b.ships + player_b.fighters + player_b.bombers;

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

pub async fn battle_handler(req: web::Json<BattleRequest>) -> impl Responder {
    let outcome = simulate_battle(req.player_a.clone(), req.player_b.clone(), req.seed);

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
