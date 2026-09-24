use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{Config, db, tdjson::ClientId, tdtypes::{Extra, ExtraTarget, Message}};

pub struct Crawler<'a> {
    client: postgres::Client,
    td_client: &'a ClientId,
    crawl_stale_after: Duration,

    state: u64,
}

impl<'a> Crawler<'a> {
    pub fn new(td_client: &'a ClientId, config: &Config) -> Self {
        Self {
            client: db::connect(&config.database_url).expect("Should connect to DB"),
            crawl_stale_after: config.crawl_stale_after,
            td_client,
            state: 0,
        }
    }
    
    pub fn crawl(&self) -> Result<(), String> {
        let from_message_id = after_message_id.unwrap_or(0);
        let extra = Extra {
            target: ExtraTarget::Crawler,
            request_id: self.state,
        };

        self.td_client.send_json(&json!({
            "@type": "searchChatMessages",
            "chat_id": channel_id,
            "topic_id": null,
            "query": "",
            "sender_id": null,
            "from_message_id": from_message_id,
            "offset": 0,
            "limit": 100,
            "filter": {
                "@type": "searchMessagesFilterAudio"
            },
            "@extra": extra,
        }));

        Ok(())
    }

    pub fn handle_response(&self, response: Value) -> Result<(), String> {
        let extra = Extra::deserialize(&response["@extra"]).map_err(|_| "Unknown target")?;

        if extra.target != ExtraTarget::Crawler || extra.request_id != self.state {
            return Err("[Crawler]: Invalid response".to_owned());
        }
        
        let json_messages = response.get("messages").ok_or(".messages is missing")?;

        let messages = Vec::<Message>::deserialize(json_messages).map_err(|_| "Failed to parse .messages");

        println!("-----------------------------------------");
        println!("Read {:?}", messages);

        Ok(())
    }
}
