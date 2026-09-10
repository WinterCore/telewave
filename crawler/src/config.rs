//! Credentials, read once at startup from the environment (or `.env`).
//!
//! Loaded eagerly so a missing or malformed value fails immediately, rather than
//! three states into the authorization flow.

use std::env;

pub struct Config {
    /// Application identifier from my.telegram.org. `int32` in the TL schema.
    pub api_id: i32,
    /// Application hash from my.telegram.org.
    pub api_hash: String,
    /// Account to authenticate as, in international format.
    pub phone_number: String,
}

impl Config {
    pub fn from_env() -> Self {
        // A missing .env is fine — the variables may already be set in the
        // environment. Anything else about the file is worth knowing about.
        match dotenvy::dotenv() {
            Ok(_) => {}
            Err(e) if e.not_found() => {}
            Err(e) => panic!("Failed to read .env: {e}"),
        }

        Self {
            api_id: required("TELEWAVE_API_ID")
                .parse()
                .expect("TELEWAVE_API_ID should be a number"),
            api_hash: required("TELEWAVE_API_HASH"),
            phone_number: required("TELEWAVE_PHONE_NUMBER"),
        }
    }
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} is not set — see .env.example"))
}
