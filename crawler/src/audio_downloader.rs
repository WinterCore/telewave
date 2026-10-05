use std::time::Duration;

use postgres::Client;

use crate::{config::Config, db, tdjson::{self, ClientId}};


pub struct AudioDownloader<'a> {
    db_client: Client,
    td_client: &'a ClientId,

    file_download_interval: Duration,
}

impl<'a> AudioDownloader<'a> {
    pub fn new(td_client: &'a ClientId, config: &Config) -> Self {
        Self {
            db_client: db::connect(&config.database_url).unwrap_or_else(|e| {
                panic!("ChannelManager::new: connect to PostgreSQL: {}", db::error_string(&e))
            }),
            td_client,

            file_download_interval: Duration::from_millis(60_000 / config.download_rate_per_min),
        }
    }
    
}
