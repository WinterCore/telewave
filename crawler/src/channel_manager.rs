use std::{collections::HashSet, time::{Duration, SystemTime}};

use postgres::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::debug;

use crate::{Config, db, slug::slugify, tdjson::ClientId, tdtypes::{Chat, ChatType, Supergroup, chat_to_supergroup_id, is_supergroup_chat_id, supergroup_to_chat_id}};

pub struct ChannelManager<'a> {
    db_client: Client,
    td_client: &'a ClientId,

    // Pending channels which need their info updated
    pending_chat_ids_sync: Vec<i64>,
    last_sync_time: SystemTime,
    // Minimum gap between syncs, precomputed from the rate config.
    sync_interval: Duration,
}

impl<'a> ChannelManager<'a> {
    pub fn new(td_client: &'a ClientId, config: &Config) -> Self {
        Self {
            db_client: db::connect(&config.database_url).unwrap_or_else(|e| {
                panic!("ChannelManager::new: connect to PostgreSQL: {}", db::error_string(&e))
            }),
            td_client,

            pending_chat_ids_sync: vec![],
            last_sync_time: SystemTime::UNIX_EPOCH,
            sync_interval: Duration::from_millis(60_000 / config.channel_sync_rate_per_min),
        }
    }

    pub fn sync_chats(&self) {
        self.td_client.send_json(&json!({
            "@type": "getChats",
            "chat_list": null,
            "limit": i32::MAX,
            "@extra": {
                "operation": "ChannelManager::sync_chats",
                "request": "getChats",
            },
        }));
    }
    
    fn should_sync(&self) -> bool {
        self.last_sync_time + self.sync_interval < SystemTime::now()
    }

    pub fn sync(&mut self) -> Result<(), String> {
        if ! self.should_sync() {
            return Ok(())
        }

        if self.pending_chat_ids_sync.is_empty() {
            return Ok(())
        }

        let chat_id = self.pending_chat_ids_sync.remove(0);
        self.last_sync_time = SystemTime::now();

        let supergroup_id = chat_to_supergroup_id(chat_id);

        debug!("Sent supergroup & chat metadata request for chat {}", chat_id);
        self.td_client.send_json(&json!({
            "@type": "getSupergroup",
            "supergroup_id": supergroup_id,
            "@extra": {
                "operation": "ChannelManager::sync",
                "request": "getSupergroup",
                "chat_id": chat_id,
                "supergroup_id": supergroup_id,
            },
        }));

        self.td_client.send_json(&json!({
            "@type": "getChat",
            "chat_id": chat_id,
            "@extra": {
                "operation": "ChannelManager::sync",
                "request": "getChat",
                "chat_id": chat_id,
            },
        }));
        
        Ok(())
    }

    pub fn handle_supergroup(&mut self, json: &Value) -> Result<(), String> {
        let supergroup: Supergroup = decode_tdlib(json, "ChannelManager::handle_supergroup")?;

        // Not a supergroup
        if !supergroup.is_channel {
            return Ok(())
        }

        let username = supergroup.usernames
            .as_ref()
            .and_then(|x| x.active_usernames.first().cloned());
        let chat_id = supergroup_to_chat_id(supergroup.id);
        
        debug!("Supergroup data (chat_id={}) {:?}", chat_id, supergroup);

        db::channel::insert_or_update_channel(
            &mut self.db_client,
            db::channel::UpdateChannel {
                telegram_chat_id: &chat_id,
                status: Some(&db::channel::ChannelStatus::Active),
                username: username.as_ref(),
                // Updated by handle_chat
                title: None,
            })
            .map_err(|e| format!("ChannelManager::handle_supergroup (chat_id={chat_id}): {e}"))?;

        Ok(())
    }

    pub fn handle_chat(&mut self, json: &Value) -> Result<(), String> {
        let chat: Chat = decode_tdlib(json, "ChannelManager::handle_chat")?;

        // Not a supergroup channel
        if !matches!(chat.kind, ChatType::Supergroup { is_channel: true, .. }) {
            return Ok(())
        }

        debug!("Chat data (chat_id={}) {:?}", chat.id, chat);
        
        db::with_tx(&mut self.db_client, |tx| {
            let channel_id = db::channel::insert_or_update_channel(
                tx,
                db::channel::UpdateChannel {
                    telegram_chat_id: &chat.id,
                    status: Some(&db::channel::ChannelStatus::Active),
                    username: None,
                    // Will be updated later 
                    title: Some(&chat.title),
                }
            )?;

            db::channel::create_channel_page(
                tx,
                db::channel::CreateChannelPage {
                    channel_id: &channel_id,
                    title: Some(&chat.title),
                    is_published: true,
                    slug: &slugify(&chat.title),
                }
            )
        }).map_err(|e| format!("ChannelManager::handle_chat (chat_id={}): {e}", chat.id))?;

        Ok(())
    }


    pub fn handle_chats(&mut self, json: &Value) -> Result<(), String> {
        let mut tx = self.db_client.transaction()
            .map_err(|e| format!("ChannelManager::handle_chats: begin transaction: {}", db::error_string(&e)))?;

        let chat_ids = json.get("chat_ids")
            .and_then(|cids| cids.as_array())
            .map(|arr| arr
                .iter()
                .filter_map(|x| x.as_i64())
                // Filter only supergroups
                .filter(|x| is_supergroup_chat_id(*x))
                .collect::<Vec<_>>()
            ).ok_or_else(|| format!(
                "ChannelManager::handle_chats: decode chat_ids (message_type={}): expected an array",
                json["@type"].as_str().unwrap_or("<missing>"),
            ))?;

        debug!("ChannelManager::handle_chats: Chat list updated {:?}", chat_ids);

        let updated_channel_ids_set: HashSet<_> = chat_ids.iter().copied().collect();

        let existing_channels = db::channel::get_all_channels(&mut tx)
            .map_err(|e| format!("ChannelManager::handle_chats: {e}"))?;

        let existing_channel_ids_set: HashSet<_> = existing_channels
            .iter()
            .map(|x| x.telegram_chat_id)
            .collect();

        let channels_to_deactivate: Vec<i64> = existing_channel_ids_set
            .difference(&updated_channel_ids_set)
            .copied()
            .collect();
        
        db::channel::deactivate_channels(&mut tx, &channels_to_deactivate)
            .map_err(|e| format!("ChannelManager::handle_chats: {e}"))?;

        tx.commit().map_err(|e| format!("ChannelManager::handle_chats: commit transaction: {}", db::error_string(&e)))?;


        debug!("Pending channel metadata update {:?}", chat_ids);
        self.pending_chat_ids_sync = chat_ids;

        self.sync()?;


        Ok(())
    }
}

fn decode_tdlib<'de, T: Deserialize<'de>>(json: &'de Value, operation: &str) -> Result<T, String> {
    T::deserialize(json).map_err(|e| format!(
        "{operation}: decode TDLib message (message_type={}): {e}\n--------------------------------\n{}",
        json["@type"].as_str().unwrap_or("<missing>"),
        json
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_parse_error_identifies_handler_message_type_and_cause() {
        let update = json!({"@type": "updateNewChat", "chat": {"id": 42}});
        let error = decode_tdlib::<Chat>(&update, "ChannelManager::handle_chat").unwrap_err();

        assert!(error.contains("ChannelManager::handle_chat"));
        assert!(error.contains("message_type=updateNewChat"));
        assert!(error.contains("missing field `id`"));
    }
}
