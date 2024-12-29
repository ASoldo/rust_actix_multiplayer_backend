use dotenv::dotenv;
use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

        Config {
            database_url,
            jwt_secret,
        }
    }
}
