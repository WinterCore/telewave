use std::{collections::HashSet, time::{Duration, SystemTime}};

use postgres::Client;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{db, tdjson::ClientId, tdtypes::{Chat, Supergroup, SupergroupFullInfo}};
// How many times we're allowed to sync per minute
const SYNC_RATE: u8 = 3;

pub struct ChannelManager<'a> {
    db_client: Client,
    td_client: &'a ClientId,

    // Pending channels which need their info updated
    pending_channels_sync: Vec<i64>,
    last_sync_time: SystemTime,
}

impl<'a> ChannelManager<'a> {
    pub fn new(td_client: &'a ClientId) -> Self {
        Self {
            db_client: db::connect().expect("Should connect to DB"),
            td_client,

            pending_channels_sync: vec![],
            last_sync_time: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn sync_chats(&self) {
        self.td_client.send_json(&json!({
            "@type": "getChats",
            "chat_list": null,
            "limit": i32::MAX,
        }));
    }
    
    fn should_sync(&self) -> bool {
        let throttle_duration = Duration::from_millis(1 * 1000 / SYNC_RATE as u64);

        self.last_sync_time + throttle_duration < SystemTime::now()
    }

    fn sync_channel(&self) -> Result<(), String> {
        if ! self.should_sync() {
            return Ok(())
        }

        // Pick one channel
        let chat_id = self.pending_channels_sync.get(0);

        self.td_client.send_json(&json!({
            "@type": "getSupergroupFullInfo",
            "supergroup_id": chat_id,
            "@extra": {
                "chat_id": chat_id,
            },
        }));
        
        Ok(())
    }

    fn handle_supergroup(&mut self, json: &Value) -> Result<(), String> {
        let chat_id = json
            .get("@extra")
            .and_then(|x| x.get("chat_id"))
            .and_then(|x| x.as_i64())
            .ok_or("Failed to parse supergroup")?;
        let supergroup = Supergroup::deserialize(json)
            .map_err(|e| format!("Failed to parse supergroup {}", e))?;

        let username = supergroup.usernames.and_then(|x| x.active_usernames.first().cloned());
        
        db::channel::insert_or_update_channel(&mut self.db_client, db::channel::CreateChannel {
            telegram_chat_id: chat_id,
            status: db::channel::ChannelStatus::Active,
            username: username.clone(),
            // Will be updated later 
            title: username.unwrap_or("".to_owned()),
        });

        Ok(())
    }

    fn handle_chat(&mut self, json: &Value) -> Result<(), String> {
        let chat = Chat::deserialize(json)
            .map_err(|e| format!("Failed to parse chat {}", e))?;

        db::channel::insert_or_update_channel(&mut self.db_client, db::channel::CreateChannel {
            telegram_chat_id: chat_id,
            status: db::channel::ChannelStatus::Active,
            username: username.clone(),
            // Will be updated later 
            title: username.unwrap_or("".to_owned()),
        });

        Ok(())
    }
    

    pub fn handle_chats(&mut self, json: &Value) -> Result<(), String> {
        let mut tx = self.db_client.transaction().map_err(|e| format!("Failed to start db transaction: {}", e))?;

        let chat_ids = json.get("chat_ids")
            .and_then(|cids| cids.as_array())
            .map(|arr| arr
                .iter()
                .filter_map(|x| x.as_i64())
                // Filter only supergroups
                .filter(|x| *x < -1000000000000)
                .collect::<Vec<_>>()
            ).ok_or("Failed to parse chat_ids")?;

        let updated_channel_ids_set: HashSet<_> = chat_ids.iter().copied().collect();

        let existing_channels = db::channel::get_all_channels(&mut tx)
            .map_err(|e| format!("Failed to get db channels {}", e))?;

        let existing_channel_ids_set: HashSet<_> = existing_channels
            .iter()
            .map(|x| x.telegram_chat_id)
            .collect();

        let channels_to_deactivate: Vec<i64> = existing_channel_ids_set
            .difference(&updated_channel_ids_set)
            .copied()
            .collect();
        
        db::channel::deactivate_channels(&mut tx, &channels_to_deactivate)
            .map_err(|e| format!("Failed to deactivate channels {:?}: {}", channels_to_deactivate, e))?;
        
        tx.commit().map_err(|_| "Failed to update deactivated channels")?;


        self.pending_channels_sync = chat_ids;

        self.sync_channel()?;


        Ok(())
    }
}
