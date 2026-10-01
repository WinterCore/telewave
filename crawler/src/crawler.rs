use std::time::{Duration, SystemTime};

use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{debug, field::debug};

use crate::{Config, db::{self, channel::{CrawlCandidate, claim_channel_due_for_crawl}}, tdjson::ClientId, tdtypes::{Extra, ExtraTarget, Message}};

const MAX_RETRIES: u8 = 12;
const REQUEST_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Clone)]
struct ChannelCrawlState {
    nonce: u64,
    response_nonce: u64,
    channel: CrawlCandidate,
    message_id_reached: i64,
    retry_count: u8,
}

pub struct Crawler<'a> {
    client: postgres::Client,
    td_client: &'a ClientId,
    crawl_stale_after: Duration,
    last_sync_time: SystemTime,
    sync_interval: Duration,
    channel_crawl_state: Option<ChannelCrawlState>,
}

impl<'a> Crawler<'a> {
    pub fn new(td_client: &'a ClientId, config: &Config) -> Self {
        Self {
            client: db::connect(&config.database_url).unwrap_or_else(|e| {
                panic!("Crawler::new: connect to PostgreSQL: {}", db::error_string(&e))
            }),
            crawl_stale_after: config.crawl_stale_after,
            last_sync_time: SystemTime::UNIX_EPOCH,
            sync_interval: Duration::from_millis(60_000 / config.crawl_rate_per_min),
            td_client,
            channel_crawl_state: None,
        }
    }
    
    fn should_sync(&self) -> bool {
        self.last_sync_time + self.sync_interval < SystemTime::now()
    }

    fn do_crawl(&mut self) -> Result<(), String> {
        let state = self.channel_crawl_state
            .as_mut()
            .expect("crawling_channel should be available");

        let extra = Extra {
            target: ExtraTarget::Crawler,
            request_id: state.nonce,
        };

        // Still waiting for last response
        if state.response_nonce < state.nonce {

            // Check if we reached the timeout for the last request
            if self.last_sync_time + Duration::from_millis(REQUEST_TIMEOUT_MS) > SystemTime::now() {
                // Timeout not reached yet
                return Ok(())
            }


            // Timeout reached
            if state.retry_count >= MAX_RETRIES {
                debug!("Max retry limit reached when crawling channel {}", state.channel.channel_id);
                db::channel::set_channel_crawl_error(
                    &mut self.client,
                    state.channel.channel_id,
                    "Max retry limit reached",
                ).map_err(|err| format!(
                    "Crawler::do_crawl (chat_id={}, retry_count={}): {err}",
                    state.channel.telegram_chat_id,
                    state.retry_count,
                ))?;
                return Ok(())
            }

            state.retry_count += 1;
            debug!("Timeout reached when crawling channel {}: retry attempt: {}", state.channel.channel_id, state.retry_count);
        }

        debug!("Crawling channel {}: Fetching messages after {}", state.channel.channel_id, state.message_id_reached);

        self.td_client.send_json(&json!({
            "@type": "searchChatMessages",
            "chat_id": state.channel.telegram_chat_id,
            "topic_id": null,
            "query": "",
            "sender_id": null,
            "from_message_id": state.message_id_reached,
            "offset": 0,
            "limit": 100,
            "filter": { "@type": "searchMessagesFilterAudio" },
            "@extra": {
                "target": extra.target,
                "request_id": extra.request_id,
                "operation": "Crawler::do_crawl",
                "request": "searchChatMessages",
                "chat_id": state.channel.telegram_chat_id,
            },
        }));

        self.last_sync_time = SystemTime::now();
        state.nonce += 1;

        return Ok(())
    }

    pub fn crawl(&mut self) -> Result<(), String> {
        // Start crawling channel if we're not already
        if self.channel_crawl_state.is_none() {
            let candidate = claim_channel_due_for_crawl(&mut self.client, &self.crawl_stale_after)
                .map_err(|err| format!("Crawler::crawl: {err}"))?;

            let Some(candidate) = candidate else {
                return Ok(());
            };

            debug!("Started crawling channel {}: {}\n\tCheckpoint message_id_reached: {:?}", candidate.channel_id, candidate.title, candidate.crawl_checkpoint_message_id);

            let state = ChannelCrawlState {
                nonce: 0,
                response_nonce: 0,
                channel: candidate,
                message_id_reached: 0,
                retry_count: 0,
            };

            self.channel_crawl_state = Some(state);
        };

        // Throttle
        if !self.should_sync() {
            return Ok(());
        }

        // TODO: Check if crawl is stalled (i don't remember what this means)

        self.do_crawl()?;

        Ok(())
    }

    pub fn handle_response(&self, response: &Value) -> Result<(), String> {
        let message_type = response["@type"].as_str().unwrap_or("<missing>");
        let extra = Extra::deserialize(&response["@extra"]).map_err(|e| format!(
            "Crawler::handle_response: decode @extra (message_type={message_type}): {e}",
        ))?;

        if extra.target != ExtraTarget::Crawler {
            return Err(format!(
                "Crawler::handle_response: unexpected target {:?} (message_type={message_type}, request_id={})",
                extra.target,
                extra.request_id,
            ));
        }
        
        let json_messages = response.get("messages").ok_or_else(|| format!(
            "Crawler::handle_response: missing messages field (message_type={message_type}, request_id={})",
            extra.request_id,
        ))?;

        let messages = Vec::<Message>::deserialize(json_messages)
            .map_err(|e| format!(
                "Crawler::handle_response: decode messages (message_type={message_type}, request_id={}): {e}",
                extra.request_id,
            ))?;

        print!("Messages {:?}", messages.iter().len());

        
        Ok(())
    }
}
