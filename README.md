# Rust Actix Multiplayer Backend

This project is a **Rust** + **Actix-Web** backend that demonstrates:

- **PostgreSQL** (via **SQLx**) for persistent user storage
- **Argon2** password hashing for secure password storage
- **JWT-based** authentication (short-lived **access tokens** + optional **refresh tokens**)
- **SSE** (Server-Sent Events) for real-time streaming
- Core functionality for multiplayer game interactions

## Prerequisites

- **Rust** (1.68+ recommended)
- **Cargo** (installed alongside Rust)
- **PostgreSQL** (running locally or a remote instance)
- **sqlx-cli** (for database migrations)
  ```bash
  cargo install sqlx-cli
  ```

## Project Setup

1. Clone the repository:

   ```sh
   git clone <your-repo-url>
   cd rust-actix-multiplayer-backend
   ```

2. Install required crates:

   ```sh
   cargo build
   ```

## Environment Variables

Create a `.env` file in the project root (same folder as `Cargo.toml`) with the following variables:

```sh
DATABASE_URL=postgres://admin:admin@localhost:5432/multiplayer_demo
JWT_SECRET=MySuperSecretKey
```

- `DATABASE_URL` points to your Postgres database.
- `JWT_SECRET` is the secret used to sign and verify JWT tokens.

## Running Migrations

1. **Initialize your database:**

   ```sh
   createdb multiplayer_demo  # or do so inside psql
   ```

2. **Crate migrations:**

   ```sh
   sqlx migrate add -r create_users_table
   ```

3. **Run migrations:**

   ```sh
   sqlx migrate run
   ```

   This applies all migrations in the `migrations/` folder to your `multiplayer_demo` database.

4. **Prepare the database for compiling queries:**

   ```sh
   DATABASE_URL=postgres://admin:admin@localhost:5432/multiplayer_demo sqlx prepare
   ```

## Building & Running

Start the Actix server locally on `127.0.0.1:8080`:

```sh
cargo run
```

You should see output like:

```sh
Finished dev [unoptimized + debuginfo] target(s) in ...
Running `target/debug/rust-actix-multiplayer-backend`
```

## Endpoints & Usage

### Register User

**POST** `/register`

**Body:**

```json
{
  "username": "...",
  "email": "...",
  "password": "..."
}
```

**Action:** Creates a new user and returns a JWT with user info.

---

### Login

**POST** `/login`

**Body:**

```json
{
  "email": "...",
  "password": "..."
}
```

**Action:** Verifies user credentials and returns access & refresh tokens.

---

### Get Current User

**GET** `/me`

**Header:**

```sh
Authorization: Bearer <access_token>
```

**Action:** Decodes the token and returns user data if valid.

---

### Refresh Token

**POST** `/refresh`

**Body:**

```json
{
  "refresh_token": "the_long_random_string"
}
```

**Action:** Returns a new short-lived `access_token`.

---

### Create Fleet

**POST** `/create_fleet`

**Body:**

```json
{
  "username": "jane@localhost",
  "ships": 50,
  "fighters": 100,
  "bombers": 30
}
```

**Action:** Creates a fleet for the specified user.

---

### Battle Simulation

**POST** `/simulate_battle`

**Body:**

```json
{
  "player_a": "rootster@localhost",
  "player_b": "john@localhost",
  "seed": 42
}
```

**Action:** Simulates a battle between two players and updates their fleets.

---

### Inbox

**POST** `/actor/{username}/inbox`

**Body (BattleRequest Example):**

```json
{
  "type": "BattleRequest",
  "actor": "rootster@localhost",
  "object": "john@localhost",
  "fleet": {
    "ships": 100,
    "fighters": 90,
    "bombers": 80
  },
  "seed": 42
}
```

**Action:** Processes a battle request or other activity.

---

### Outbox

**GET** `/actor/{username}/outbox`

**Action:** Fetches all activities sent by the user.

---

## Example Curl Commands

### Register User

```sh
curl -X POST http://127.0.0.1:8080/register \
     -H "Content-Type: application/json" \
     -d '{"username":"jane","email":"jane@example.com","password":"password123"}'
```

### Create Fleet

```sh
curl -X POST http://127.0.0.1:8080/create_fleet \
     -H "Content-Type: application/json" \
     -d '{"username":"jane@localhost","ships":50,"fighters":100,"bombers":30}'
```

### Simulate Battle

```sh
curl -X POST http://127.0.0.1:8080/simulate_battle \
     -H "Content-Type: application/json" \
     -d '{"player_a":"rootster","player_b":"jane","seed":42}'
```

### Inbox

```sh
curl -X POST http://127.0.0.1:8080/actor/jane@localhost/inbox \
     -H "Content-Type: application/json" \
     -d '{
       "type": "BattleRequest",
       "actor": "rootster@localhost",
       "object": "jane@localhost",
       "fleet": {
           "ships": 100,
           "fighters": 90,
           "bombers": 80
       },
       "seed": 42
     }'
```

### Outbox

```sh
curl -X GET http://127.0.0.1:8080/actor/jane@localhost/outbox
```
