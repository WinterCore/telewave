use serde::Deserialize;
use serde_json::{Value, json};

use crate::{tdjson::ClientId, tdtypes::{Extra, ExtraTarget, Message}};

pub struct Crawler<'a> {
    client: &'a ClientId,
    state: u64,
}

impl<'a> Crawler<'a> {
    pub fn new(client: &'a ClientId) -> Self {
        Self {
            client,
            state: 0,
        }
    }
    
    pub fn crawl(&self, channel_id: i64, after_message_id: Option<i64>) {
        let from_message_id = after_message_id.unwrap_or(0);
        let extra = Extra {
            target: ExtraTarget::Crawler,
            request_id: self.state,
        };

        self.client.send_json(&json!({
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
    }

    pub fn handle_response(&self, response: Value) -> Result<(), String> {
        let extra = Extra::deserialize(&response["@extra"]).map_err(|_| "Unknown target")?;

        if extra.target != ExtraTarget::Crawler || extra.request_id != self.state {
            return Err("[Crawler]: Invalid response".to_owned());
        }
        
        let json_messages = response.get("messages").ok_or(".messages is missing")?;

        let messages = Vec::<Message>::deserialize(json_messages).map_err(|_| "Failed to parse .messages");

        Ok(())
    }
}
