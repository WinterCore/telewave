//! A reference rewrite of the crawl loop, side-by-side with the working
//! `crawler`. Not wired into the event loop. The shape it demonstrates:
//!
//! - an **explicit state machine** (`Phase`) — "waiting for a response" is
//!   a state you can see, not an inequality between two counters;
//! - **policy / protocol / persistence separated**: `plan_tick` decides
//!   *what* should happen (pure, no I/O), `classify_page` decides *what a
//!   page means* (pure, unit-tested below), and only a handful of small
//!   functions touch the database;
//! - **typed errors**: main only ever sees things it can't act on (db
//!   failures, undecodable responses). Everything else — stray responses,
//!   TDLib rejections, stalled pagination — is handled internally, and
//!   crawl-level failures land in `channel_pages.last_crawl_error` where
//!   an operator can find them.

use std::{error::Error as StdError, fmt, time::{Duration, Instant}};

use serde::Deserialize;
use serde_json::{Value, json};
use tracing::debug;

use crate::{
    Config,
    crawler::CrawlExtra,
    db::{
        self,
        channel::{CrawlCandidate, claim_channel_due_for_crawl, finish_channel_crawl, set_channel_crawl_error},
        message::insert_messages,
    },
    tdjson::ClientId,
    tdtypes::{ExtraTarget, FoundChatMessages},
};

/// Attempts for a single page request before the channel is abandoned.
const MAX_ATTEMPTS: u8 = 12;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const PAGE_SIZE: i32 = 100;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors surfaced to `main` — the things it can't do anything about
/// except log. Everything else is either dropped with a debug log (stray
/// responses) or persisted to `last_crawl_error` (crawl failures).
#[derive(Debug)]
pub enum CrawlerError {
    /// A database call failed; carries the operation context.
    Db(db::Error),
    /// A response could not be decoded into the expected TDLib type.
    Decode {
        what: &'static str,
        message_type: String,
        source: serde_json::Error,
    },
}

impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrawlerError::Db(e) => write!(f, "{e}"),
            CrawlerError::Decode { what, message_type, source } =>
                write!(f, "decode {what} (message_type={message_type}): {source}"),
        }
    }
}

impl StdError for CrawlerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CrawlerError::Db(e) => Some(e),
            CrawlerError::Decode { source, .. } => Some(source),
        }
    }
}

impl From<db::Error> for CrawlerError {
    fn from(e: db::Error) -> Self {
        CrawlerError::Db(e)
    }
}

// ---------------------------------------------------------------------------
// Policy: when may we talk to TDLib?
// ---------------------------------------------------------------------------

/// Minimum gap between two requests. `Instant` (not `SystemTime`) because
/// timeouts must never be confused by the wall clock jumping.
struct Throttle {
    interval: Duration,
    last_sent: Option<Instant>,
}

impl Throttle {
    fn new(interval: Duration) -> Self {
        Self { interval, last_sent: None }
    }

    fn ready(&self) -> bool {
        self.last_sent.is_none_or(|at| at.elapsed() >= self.interval)
    }

    fn mark_sent(&mut self) {
        self.last_sent = Some(Instant::now());
    }
}

// ---------------------------------------------------------------------------
// Protocol: the crawl state machine
// ---------------------------------------------------------------------------

/// A page request that is out with TDLib.
struct InFlight {
    request_id: u64,
    /// The cursor the request was sent with — what the response will
    /// advance from.
    cursor: i64,
    sent_at: Instant,
}

/// Where a crawl stands. Illegal states ("response for a request we never
/// sent", "two requests in flight") are unrepresentable rather than merely
/// avoided.
enum Phase {
    /// A page request is due; the throttle decides when it goes out.
    NeedPage,
    /// Waiting on one specific request.
    AwaitingPage(InFlight),
}

struct Crawl {
    channel: CrawlCandidate,
    phase: Phase,
    /// Cursor for the next request: message id to search from (0 = newest).
    /// Crawls always start at the top; the previous checkpoint is a stop
    /// marker, not a resume point — this survives deleted messages.
    cursor: i64,
    /// Newest message seen this crawl — becomes the next checkpoint.
    newest_seen: Option<i64>,
    /// The previous crawl's checkpoint: our finish line.
    previous_checkpoint: Option<i64>,
    /// Attempts spent on the current request; reset by any good response.
    attempts: u8,
    pages: u32,
}

/// What a received page means for the crawl. Pure function of the page and
/// the crawl state — no I/O, so it is unit-testable.
enum PageVerdict {
    /// Keep paging; the cursor moved.
    Continue { next_cursor: i64 },
    /// Nothing older to fetch, or we walked back into already-crawled
    /// territory (the previous checkpoint appeared in this page).
    Finished,
    /// Non-terminal page whose cursor didn't move: the same request would
    /// repeat forever. A crawl failure — loudly, not silently absorbed.
    Stalled,
}

fn classify_page(page: &FoundChatMessages, crawl: &Crawl) -> PageVerdict {
    let reached_previous = crawl.previous_checkpoint
        .is_some_and(|cp| page.messages.iter().any(|m| m.id == cp));

    if page.next_from_message_id == 0 || reached_previous {
        PageVerdict::Finished
    } else if page.next_from_message_id == crawl.cursor {
        PageVerdict::Stalled
    } else {
        PageVerdict::Continue { next_cursor: page.next_from_message_id }
    }
}

// ---------------------------------------------------------------------------
// The crawler
// ---------------------------------------------------------------------------

pub struct Crawler<'a> {
    td: &'a ClientId,
    db: postgres::Client,
    stale_after: Duration,
    throttle: Throttle,
    /// The open crawl, if any. `None` is the idle phase.
    crawl: Option<Crawl>,
    next_request_id: u64,
    /// Flood-wait gate: no requests to TDLib before this instant.
    blocked_until: Option<Instant>,
}

/// What `on_tick` decided should happen — the entire retry/timeout policy
/// in one pure function, free of borrow entanglement with the executor.
enum TickAction {
    Claim,
    SendPage,
    Retry,
    Abandon { channel_id: i64, reason: String },
    Nothing,
}

/// What a processed response means for the crawl's persistence layer.
enum Outcome {
    Advanced,
    Finish { checkpoint: Option<i64>, pages: u32 },
    Fail { reason: String },
}

impl<'a> Crawler<'a> {
    pub fn new(td: &'a ClientId, config: &Config) -> Self {
        Self {
            db: db::connect(&config.database_url).unwrap_or_else(|e| {
                panic!("crawler_refactored: connect to PostgreSQL: {}", db::error_string(&e))
            }),
            td,
            stale_after: config.crawl_stale_after,
            throttle: Throttle::new(Duration::from_millis(60_000 / config.crawl_rate_per_min)),
            crawl: None,
            next_request_id: 0,
            blocked_until: None,
        }
    }

    // -- driving ------------------------------------------------------------

    /// Called by the event loop on every tick. Plans, then executes.
    pub fn on_tick(&mut self) -> Result<(), CrawlerError> {
        match self.plan_tick() {
            TickAction::Claim => self.claim(),
            TickAction::SendPage | TickAction::Retry => self.send_page_request(),
            TickAction::Abandon { channel_id, reason } => self.abandon(channel_id, reason),
            TickAction::Nothing => Ok(()),
        }
    }

    /// The whole tick policy: what *should* happen right now? Reads only —
    /// no borrows to fight, and testable without a database.
    fn plan_tick(&self) -> TickAction {
        let Some(crawl) = &self.crawl else {
            return TickAction::Claim;
        };

        match &crawl.phase {
            Phase::NeedPage => {
                if self.flood_blocked() || !self.throttle.ready() {
                    TickAction::Nothing
                } else {
                    TickAction::SendPage
                }
            },
            Phase::AwaitingPage(inflight) => {
                if inflight.sent_at.elapsed() < REQUEST_TIMEOUT {
                    return TickAction::Nothing;
                }

                if crawl.attempts >= MAX_ATTEMPTS {
                    TickAction::Abandon {
                        channel_id: crawl.channel.channel_id,
                        reason: format!("page request timed out {} times in a row", crawl.attempts),
                    }
                } else {
                    TickAction::Retry
                }
            },
        }
    }

    /// Called by the event loop for every response routed to the crawler
    /// (i.e. carrying `@extra.target = crawler`).
    pub fn on_response(&mut self, response: &Value) -> Result<(), CrawlerError> {
        let extra = CrawlExtra::deserialize(&response["@extra"]).map_err(|source| {
            CrawlerError::Decode {
                what: "CrawlExtra",
                message_type: response["@type"].as_str().unwrap_or("<missing>").to_owned(),
                source,
            }
        })?;

        // Correlation: accept only the response to the request we have in
        // flight, for the crawl we have open. Everything else — retry
        // duplicates, stragglers from the previous channel arriving after
        // a new claim — is expected and dropped quietly.
        if !self.is_ours_in_flight(&extra) {
            debug!(
                channel_id = extra.channel_id,
                request_id = extra.request_id,
                "Dropping stray response"
            );
            return Ok(());
        }

        // TDLib rejected the request itself (flood-wait, revoked access…).
        if response["@type"] == "error" {
            return self.on_td_error(&extra, response);
        }

        let page = FoundChatMessages::deserialize(response).map_err(|source| {
            CrawlerError::Decode {
                what: "FoundChatMessages",
                message_type: response["@type"].as_str().unwrap_or("<missing>").to_owned(),
                source,
            }
        })?;

        let outcome = {
            let crawl = self.crawl
                .as_mut()
                .expect("is_ours_in_flight guarantees an open crawl");

            // The first page's first message is the newest thing this crawl
            // has seen — the checkpoint we'll write when it ends.
            if crawl.newest_seen.is_none() {
                crawl.newest_seen = page.messages.first().map(|m| m.id);
            }

            insert_messages(&mut self.db, extra.channel_id, &page.messages)
                .map_err(|e| db::Error::new("insert crawl page", e))?;

            match classify_page(&page, crawl) {
                PageVerdict::Finished => Outcome::Finish {
                    checkpoint: crawl.newest_seen.or(crawl.previous_checkpoint),
                    pages: crawl.pages + 1,
                },
                PageVerdict::Stalled => Outcome::Fail {
                    reason: format!("pagination made no progress (cursor={})", crawl.cursor),
                },
                PageVerdict::Continue { next_cursor } => {
                    crawl.phase = Phase::NeedPage;
                    crawl.cursor = next_cursor;
                    crawl.attempts = 0;
                    crawl.pages += 1;
                    Outcome::Advanced
                },
            }
        }; // crawl borrow ends; persistence happens free of it

        match outcome {
            Outcome::Advanced => {
                debug!(channel_id = extra.channel_id, "Page stored; crawl continues");
            },
            Outcome::Finish { checkpoint, pages } => {
                debug!(
                    channel_id = extra.channel_id,
                    checkpoint = ?checkpoint,
                    pages,
                    "Crawl complete"
                );
                self.finish(extra.channel_id, checkpoint)?;
            },
            Outcome::Fail { reason } => self.abandon(extra.channel_id, reason)?,
        }

        Ok(())
    }

    // -- actions ------------------------------------------------------------

    fn claim(&mut self) -> Result<(), CrawlerError> {
        let Some(candidate) = claim_channel_due_for_crawl(&mut self.db, &self.stale_after)?
        else {
            return Ok(());
        };

        debug!(
            channel_id = candidate.channel_id,
            channel = %candidate.title,
            previous_checkpoint = ?candidate.crawl_checkpoint_message_id,
            "Claimed channel for crawling"
        );

        self.crawl = Some(Crawl {
            previous_checkpoint: candidate.crawl_checkpoint_message_id,
            cursor: 0,
            newest_seen: None,
            channel: candidate,
            phase: Phase::NeedPage,
            attempts: 0,
            pages: 0,
        });

        Ok(())
    }

    fn send_page_request(&mut self) -> Result<(), CrawlerError> {
        let crawl = self.crawl
            .as_mut()
            .expect("send_page_request requires an open crawl");

        crawl.attempts += 1;
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        debug!(
            channel_id = crawl.channel.channel_id,
            cursor = crawl.cursor,
            attempt = crawl.attempts,
            request_id,
            "Requesting message page"
        );

        self.td.send_json(&json!({
            "@type": "searchChatMessages",
            "chat_id": crawl.channel.telegram_chat_id,
            "topic_id": null,
            "query": "",
            "sender_id": null,
            "from_message_id": crawl.cursor,
            "offset": 0,
            "limit": PAGE_SIZE,
            "filter": { "@type": "searchMessagesFilterAudio" },
            "@extra": CrawlExtra {
                target: ExtraTarget::Crawler,
                request_id,
                operation: "Crawler::send_page_request".into(),
                request: "searchChatMessages".into(),
                chat_id: crawl.channel.telegram_chat_id,
                channel_id: crawl.channel.channel_id,
            },
        }));

        crawl.phase = Phase::AwaitingPage(InFlight {
            request_id,
            cursor: crawl.cursor,
            sent_at: Instant::now(),
        });
        self.throttle.mark_sent();

        Ok(())
    }

    /// A TDLib `error` response to our own request.
    fn on_td_error(&mut self, extra: &CrawlExtra, response: &Value) -> Result<(), CrawlerError> {
        let message = response["message"].as_str().unwrap_or("<no message>");
        let code = response["code"].as_i64().unwrap_or_default();

        // Flood-wait: the server told us when to come back. Wait it out
        // without burning attempt budget on doomed retries.
        if let Some(wait_secs) = flood_wait_secs(message) {
            self.blocked_until = Some(Instant::now() + Duration::from_secs(wait_secs));
            if let Some(crawl) = self.crawl.as_mut() {
                crawl.phase = Phase::NeedPage; // re-armed; attempts unchanged
            }

            debug!(
                channel_id = extra.channel_id,
                wait_secs,
                "Flood-waited; backing off"
            );
            return Ok(());
        }

        // Anything else (chat deleted, access revoked, …) is permanent for
        // this channel; record it verbatim so the reason survives.
        self.abandon(extra.channel_id, format!("TDLib error {code}: {message}"))
    }

    // -- persistence ----------------------------------------------------------

    /// Records a crawl failure and drops the crawl. The channel stays
    /// excluded from claiming until the error is cleared.
    fn abandon(&mut self, channel_id: i64, reason: String) -> Result<(), CrawlerError> {
        debug!(channel_id, reason = %reason, "Abandoning crawl");
        set_channel_crawl_error(&mut self.db, channel_id, &reason)?;
        self.crawl = None;

        Ok(())
    }

    fn finish(&mut self, channel_id: i64, checkpoint: Option<i64>) -> Result<(), CrawlerError> {
        finish_channel_crawl(&mut self.db, channel_id, checkpoint)?;
        self.crawl = None;

        Ok(())
    }

    // -- helpers ----------------------------------------------------------

    fn flood_blocked(&self) -> bool {
        self.blocked_until.is_some_and(|until| until > Instant::now())
    }

    /// True when `extra` names the request this crawler currently has in
    /// flight — the single legal response we're waiting for.
    fn is_ours_in_flight(&self, extra: &CrawlExtra) -> bool {
        self.crawl.as_ref().is_some_and(|crawl| {
            crawl.channel.channel_id == extra.channel_id
                && matches!(&crawl.phase, Phase::AwaitingPage(f) if f.request_id == extra.request_id)
        })
    }
}

/// Parses Telegram's flood-wait text: `"Too Many Requests: retry after 30"`
/// → `Some(30)`.
fn flood_wait_secs(message: &str) -> Option<u64> {
    message
        .split("retry after")
        .nth(1)?
        .trim()
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdtypes::{Message, MessageContent};

    fn msg(id: i64) -> Message {
        Message {
            id,
            chat_id: -100,
            sender_id: None,
            is_outgoing: None,
            date: 0,
            edit_date: 0,
            content: MessageContent::Other,
        }
    }

    fn page(next_from: i64, messages: Vec<Message>) -> FoundChatMessages {
        FoundChatMessages {
            total_count: messages.len() as i32,
            messages,
            next_from_message_id: next_from,
        }
    }

    fn crawl(previous_checkpoint: Option<i64>, cursor: i64) -> Crawl {
        Crawl {
            channel: CrawlCandidate {
                channel_id: 1,
                telegram_chat_id: -100,
                crawl_checkpoint_message_id: previous_checkpoint,
                title: "test".into(),
            },
            phase: Phase::NeedPage,
            cursor,
            newest_seen: None,
            previous_checkpoint,
            attempts: 0,
            pages: 0,
        }
    }

    #[test]
    fn exhausted_chat_finishes() {
        let verdict = classify_page(&page(0, vec![msg(3), msg(2)]), &crawl(None, 100));
        assert!(matches!(verdict, PageVerdict::Finished));
    }

    #[test]
    fn reaching_the_previous_checkpoint_finishes() {
        let crawl = crawl(Some(7), 100);
        let verdict = classify_page(&page(50, vec![msg(9), msg(7), msg(6)]), &crawl);
        assert!(matches!(verdict, PageVerdict::Finished));
    }

    #[test]
    fn non_advancing_cursor_stalls() {
        let verdict = classify_page(&page(100, vec![]), &crawl(None, 100));
        assert!(matches!(verdict, PageVerdict::Stalled));
    }

    #[test]
    fn advancing_cursor_continues() {
        let verdict = classify_page(&page(50, vec![msg(9), msg(8)]), &crawl(None, 100));
        assert!(matches!(
            verdict,
            PageVerdict::Continue { next_cursor: 50 }
        ));
    }

    #[test]
    fn flood_wait_message_is_parsed() {
        assert_eq!(flood_wait_secs("Too Many Requests: retry after 30"), Some(30));
        assert_eq!(flood_wait_secs("CHAT_WRITE_FORBIDDEN"), None);
    }
}
