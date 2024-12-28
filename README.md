# Rust Actix Multiplayer Backend

This project is a **Rust** + **Actix-Web** backend that demonstrates:

- **PostgreSQL** (via **SQLx**) for persistent user storage
- **Argon2** password hashing for secure password storage
- **JWT-based** authentication (short-lived **access tokens** + optional **refresh tokens**)
- **SSE** (Server-Sent Events) for real-time streaming
- Basic routes: `/register`, `/login`, `/me`, `/refresh`, `/sse`

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Project Setup](#project-setup)
3. [Environment Variables](#environment-variables)
4. [Running Migrations](#running-migrations)
5. [Building & Running](#building--running)
6. [Endpoints & Usage](#endpoints--usage)
   - [Register](#register)
   - [Login](#login)
   - [Get Current User (`/me`)](#get-current-user-me)
   - [Refresh Token](#refresh-token)
   - [SSE](#sse)
7. [Example Curl Commands](#example-curl-commands)
8. [Tips & Next Steps](#tips--next-steps)

---

## 1. Prerequisites

- **Rust** (1.68+ recommended)
- **Cargo** (installed alongside Rust)
- **PostgreSQL** (running locally or a remote instance)
- **sqlx-cli** (for database migrations)
  ```bash
  cargo install sqlx-cli
  ```

## 2. Project Setup

Clone the repository:

```sh
git clone <your-repo-url>
cd rust-actix-multiplayer-backend
```

Install required crates automatically when building:

```sh
cargo build
```

## 3. Environment Variables

Create a `.env` file in the project root (same folder as `Cargo.toml`) with the following variables:

```sh
DATABASE_URL=postgres://admin:admin@localhost:5432/multiplayer_demo
JWT_SECRET=MySuperSecretKey
```

- `DATABASE_URL` points to your Postgres database.
- `JWT_SECRET` is the secret used to sign and verify JWT tokens.

**Note:** You can change these values as needed (e.g. different DB user/password, random JWT secret, etc.).

## 4. Running Migrations

We use SQLx migrations to create and alter tables (users, refresh_tokens, etc.).

### Initialize your database (if not done yet):

```sh
createdb multiplayer_demo  # or do so inside psql
```

### Create a new migration like this:

```sh
sqlx migrate add -r create_users_table
```

### Run migrations:

```sh
sqlx migrate run
```

This applies all migrations in the `migrations/` folder to your `multiplayer_demo` database.

Prepare your database for compiling queries:

```sh
DATABASE_URL=postgres://admin:admin@localhost:5432/multiplayer_demo sqlx prepare
```

## 5. Building & Running

Start the Actix server locally on `127.0.0.1:8080`:

```sh
cargo run
```

You should see output like:

```sh
Finished dev [unoptimized + debuginfo] target(s) in ...
Running `target/debug/rust-actix-multiplayer-backend`
```

## 6. Endpoints & Usage

### 6.1. Register

**POST** `/register`

**Body:**

```json
{
  "username": "...",
  "email": "...",
  "password": "..."
}
```

**Action:** Creates a new user (with Argon2-hashed password), returns a short-lived JWT + user info.

---

### 6.2. Login

**POST** `/login`

**Body:**

```json
{
  "email": "...",
  "password": "..."
}
```

**Action:** Verifies user credentials, returns a new short-lived access token + a refresh token + user info.

---

### 6.3. Get Current User (`/me`)

**GET** `/me`

**Header:**

```sh
Authorization: Bearer <access_token>
```

**Action:** Decodes the token, verifies the user from the database, returns user data if valid. Otherwise, 401 Unauthorized.

---

### 6.4. Refresh Token

**POST** `/refresh` (if you implemented this)

**Body:**

```json
{
  "refresh_token": "the_long_random_string"
}
```

**Action:** Checks the `refresh_tokens` table for this token, ensures not expired, returns new short-lived `access_token`.

---

### 6.5. SSE

**GET** `/sse`

**Action:** Provides a continuous server-sent events stream (infinite counting or your custom logic).

---

## 7. Example Curl Commands

### Register:

```sh
curl -X POST http://127.0.0.1:8080/register \
     -H "Content-Type: application/json" \
     -d '{"username":"rootster","email":"rootster@example.com","password":"ToorToor"}'
```

Returns:

```json
{"token":"<JWT>", "user": {...}}
```

---

### Login:

```sh
curl -X POST http://127.0.0.1:8080/login \
     -H "Content-Type: application/json" \
     -d '{"email":"rootster@example.com","password":"ToorToor"}'
```

Returns:

```json
{"access_token":"<short-lived-JWT>","refresh_token":"<long-string>", "user": {...}}
```

---

### Get Current User:

```sh
curl http://127.0.0.1:8080/me \
     -H "Authorization: Bearer <ACCESS_TOKEN_FROM_LOGIN>"
```

Returns user info if valid token, otherwise Invalid token.

---

### Refresh Token (if used):

```sh
curl -X POST http://127.0.0.1:8080/refresh \
     -H "Content-Type: application/json" \
     -d '{"refresh_token":"<LONG_RANDOM_STRING_FROM_LOGIN>"}'
```

Returns new `access_token` and optionally a new `refresh_token`.

---

### SSE:

```sh
curl -N http://127.0.0.1:8080/sse
```

Streams events every couple seconds (e.g. `data: 0`, `data: 1`, etc.).
