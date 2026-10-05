use std::{borrow::Cow, time::{Duration, SystemTime}};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::debug;

use crate::{Config, db::{self, channel::{CrawlCandidate, claim_channel_due_for_crawl}}, tdjson::ClientId, tdtypes::{ExtraTarget, FoundChatMessages}};

const MAX_RETRIES: u8 = 12;
const REQUEST_TIMEOUT_MS: u64 = 30_000;

/// The `@extra` payload stamped on a searchChatMessages request, so the
/// response can be routed back to the crawler and tied to its crawl state.
#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlExtra {
    pub target: ExtraTarget,
    pub request_id: u64,
    pub operation: Cow<'static, str>,
    pub request: Cow<'static, str>,
    pub chat_id: i64,
    pub channel_id: i64,
}

#[derive(Debug, Clone)]
struct ChannelCrawlState {
    nonce: u64,
    response_nonce: u64,
    channel: CrawlCandidate,

    /// First message id in the current crawl which will be used for 
    /// checkpoint_message_id once we're done
    most_recent_message_id: Option<i64>,

    /// Used for pagination
    message_id_reached: i64,

    /// Where the previous crawl started, this is our finish line
    checkpoint_message_id: Option<i64>,
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

        let extra = CrawlExtra {
            target: ExtraTarget::Crawler,
            request_id: state.nonce,
            operation: "Crawler::do_crawl".into(),
            request: "searchChatMessages".into(),
            chat_id: state.channel.telegram_chat_id,
            channel_id: state.channel.channel_id,
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
                db::channel::set_channel_crawl_error(&mut self.client, state.channel.channel_id, "Max retry limit reached")
                    .map_err(|err| format!(
                        "Crawler::do_crawl (chat_id={}, retry_count={}): {err}",
                        state.channel.telegram_chat_id,
                        state.retry_count,
                    ))?;

                self.channel_crawl_state = None;

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
            "@extra": extra,
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

            let checkpoint_message_id = candidate.crawl_checkpoint_message_id;

            let state = ChannelCrawlState {
                nonce: 0,
                response_nonce: 0,
                channel: candidate,
                message_id_reached: 0,
                checkpoint_message_id,
                most_recent_message_id: None,
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

    pub fn handle_response(&mut self, response: &Value) -> Result<(), String> {
        let message_type = response["@type"].as_str().unwrap_or("<missing>");
        let extra = CrawlExtra::deserialize(&response["@extra"]).map_err(|e| format!(
            "Crawler::handle_response: decode @extra (message_type={message_type}): {e}",
        ))?;

        if extra.target != ExtraTarget::Crawler {
            return Err(format!(
                "Crawler::handle_response: unexpected target {:?} (message_type={}, request_id={})",
                extra.target,
                message_type,
                extra.request_id,
            ));
        }

        let Some(state) = self.channel_crawl_state.as_mut() else {
            return Err("Crawler::handle_response: Not crawling any channels atm".to_owned())
        };

        // Correlation: only accept the response to this crawl's latest
        // request. Strays (duplicate responses after a retry, stragglers
        // from the previous channel arriving after a new claim) would
        // otherwise overwrite the pagination cursor with an older page's.
        if extra.channel_id != state.channel.channel_id || state.nonce != extra.request_id + 1 {
            debug!(
                response_channel_id = extra.channel_id,
                response_request_id = extra.request_id,
                state_channel_id = state.channel.channel_id,
                state_latest_request_id = state.nonce,
                "Dropping stray searchChatMessages response"
            );

            return Ok(());
        }

        let page = FoundChatMessages::deserialize(response).map_err(|e| format!(
            "Crawler::handle_response: decode foundChatMessages (channel_id={}, request_id={}): {e}",
            extra.channel_id,
            extra.request_id,
        ))?;

        // Surpassed checkpoint from the previous crawl
        let reached_end = state.checkpoint_message_id
            .is_some_and(|cid| page.messages.iter().any(|m| m.id == cid));
        
        if state.most_recent_message_id.is_none(){
            state.most_recent_message_id = page.messages.first().map(|m| m.id);
        }

        db::message::insert_messages(&mut self.client, extra.channel_id, &page.messages)
            .map_err(|e| format!(
                "Crawler::handle_response: insert {} messages (channel_id={}): {}",
                page.messages.len(),
                extra.channel_id,
                db::error_string(&e),
            ))?;

        debug!("Crawler found {} messages for channel_id: {}", page.messages.len(), extra.channel_id);

        // Finished crawl
        if page.next_from_message_id == 0 || reached_end {
            let checkpoint = state.most_recent_message_id.or(state.checkpoint_message_id);

            debug!(
                channel_id = extra.channel_id,
                channel = %state.channel.title,
                reason = if page.next_from_message_id == 0 { "exhausted" } else { "reached previous checkpoint" },
                started_at_checkpoint = ?state.checkpoint_message_id,
                finished_at_checkpoint = ?checkpoint,
                pages = state.response_nonce + 1,
                requests_sent = state.nonce,
                retries_since_last_response = state.retry_count,
                last_page_size = page.messages.len(),
                next_from = page.next_from_message_id,
                state = ?state,
                "Finished crawling channel"
            );

            db::channel::finish_channel_crawl(&mut self.client, extra.channel_id, checkpoint)
                .map_err(|e| format!(
                    "Crawler::handle_response: finish crawl (channel_id={}, checkpoint={checkpoint:?}): {e}",
                    extra.channel_id,
                ))?;

            self.channel_crawl_state = None;

            return Ok(())
        }

        // A non-terminal page whose cursor didn't move would replay the
        // same request forever — TDLib did something we didn't model.
        if page.next_from_message_id == state.message_id_reached {
            let cursor = state.message_id_reached;

            db::channel::set_channel_crawl_error(
                &mut self.client,
                extra.channel_id,
                &format!("Pagination made no progress (cursor={cursor})"),
            )
            .map_err(|e| format!(
                "Crawler::handle_response: set no-progress error (channel_id={}, cursor={cursor}): {e}",
                extra.channel_id,
            ))?;

            debug!(channel_id = extra.channel_id, cursor, "Crawl aborted: pagination made no progress");
            self.channel_crawl_state = None;

            return Ok(());
        }

        state.response_nonce += 1;
        state.message_id_reached = page.next_from_message_id;
        state.retry_count = 0;

        
        Ok(())
    }
}
