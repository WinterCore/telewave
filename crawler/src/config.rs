//! Credentials, read once at startup from the environment (or `.env`).
//!
//! Loaded eagerly so a missing or malformed value fails immediately, rather than
//! three states into the authorization flow.

use std::{env, time::Duration};

pub struct Config {
    /// Application identifier from my.telegram.org. `int32` in the TL schema.
    pub api_id: i32,
    /// Application hash from my.telegram.org.
    pub api_hash: String,
    /// Account to authenticate as, in international format.
    pub phone_number: String,
    /// Postgres connection string.
    pub database_url: String,
    /// How many channel metadata syncs to start per minute. One sync is a
    /// getSupergroup + getChat pair for a single channel; throttles the
    /// ChannelManager repair loop.
    pub channel_sync_rate_per_min: u64,
    /// How many searchChatMessages requests to start per minute — pagination
    /// counts too, so a long channel history is walked one throttled page at
    /// a time. Paces the Crawler's loop over channels.
    pub crawl_rate_per_min: u64,
    /// How many audio file downloads to start per minute. Downloads are the
    /// expensive half — bandwidth, TDLib prefetching, disk — and the part
    /// Telegram's flood limits watch most closely.
    pub download_rate_per_min: u64,
    /// A channel is due for a crawl when its last crawl is older than this.
    /// 0 means every channel is always due — handy while testing.
    pub crawl_stale_after: Duration,
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
            database_url: required("DATABASE_URL"),
            channel_sync_rate_per_min: rate("CHANNEL_SYNC_RATE_PER_MIN", 3),
            crawl_rate_per_min: rate("CRAWL_RATE_PER_MIN", 6),
            download_rate_per_min: rate("DOWNLOAD_RATE_PER_MIN", 2),
            crawl_stale_after: minutes("CRAWL_STALE_AFTER_MIN", 360),
        }
    }
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} is not set — see .env.example"))
}

/// Reads an optional whole-number knob, falling back to `default` when unset.
/// Malformed values are hard errors: a typo shouldn't silently become the
/// default crawl rate.
fn number(key: &str, default: u64) -> u64 {
    match env::var(key) {
        Ok(v) => v
            .parse()
            .unwrap_or_else(|_| panic!("{key} should be a whole number")),
        Err(_) => default,
    }
}

/// A knob used as "events per minute": must be at least 1, since 0 would
/// divide by zero when converted into a throttle interval.
fn rate(key: &str, default: u64) -> u64 {
    match number(key, default) {
        0 => panic!("{key} must be at least 1"),
        n => n,
    }
}

/// Reads an optional duration in whole minutes. 0 is allowed and means
/// "always due".
fn minutes(key: &str, default: u64) -> Duration {
    Duration::from_secs(number(key, default) * 60)
}
