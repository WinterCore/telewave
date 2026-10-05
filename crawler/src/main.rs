use std::time::Duration;

use owo_colors::OwoColorize;
use serde_json::Value;
use tracing::error;
use tracing_subscriber::EnvFilter;

use crate::{
    auth::Auth,
    channel_manager::ChannelManager,
    config::Config,
    crawler::Crawler,
    tdjson::{ClientId, Receiver, set_log_verbosity_level}
};

mod auth;
mod config;
mod tdjson;
mod crawler;
mod channel_manager;
mod tdtypes;
mod db;
mod slug;
mod audio_downloader;

/// Blocks until TDLib produces one parseable update.
/// Timeouts are waited out; bad JSON is logged and skipped.
fn next_update(rx: &mut Receiver) -> Option<Value> {
    loop {
        match rx.receive_json(Duration::from_secs(1)) {
            Some(Ok(v)) => return Some(v),
            Some(Err(e)) => error!(
                operation = "next_update",
                error = %e,
                "Failed to decode TDLib JSON",
            ),
            None => return None,
        }
    }
}

fn log_handler_error(operation: &str, json: &Value, error: &str) {
    let entity_id = json.get("id")
        .or_else(|| json.get("chat_id"))
        .or_else(|| json.get("chat").and_then(|chat| chat.get("id")))
        .and_then(Value::as_i64);

    error!(
        operation,
        message_type = json["@type"].as_str().unwrap_or("<missing>"),
        entity_id = ?entity_id,
        request_context = %json["@extra"],
        error = %error,
        "TDLib message handler failed",
    );
}

fn log_tdlib_request_error(phase: &str, json: &Value) {
    error!(
        operation = "tdlib.request",
        phase,
        code = ?json["code"].as_i64(),
        request_context = %json["@extra"],
        error = %json["message"],
        "TDLib request failed",
    );
}

fn main() {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env();

    set_log_verbosity_level(1);
    let client = ClientId::new();

    let mut rx = Receiver::take().expect("Should acquire receiver");

    let auth = Auth::new(&client, &config);
    auth.init();

    // Phase 1: the auth handshake. Auth updates drive the state machine;
    // anything else TDLib pushes in the meantime is irrelevant until login.
    loop {
        let Some(json) = next_update(&mut rx) else { continue };

        match json["@type"].as_str() {
            Some("updateAuthorizationState") => {
                if auth.handle(&json["authorization_state"]) {
                    break;
                }
            },
            Some("error") => log_tdlib_request_error("authorization", &json),
            _ => (),
        }
    }

    let mut crawler = Crawler::new(&client, &config);
    let mut channel_manager = ChannelManager::new(&client, &config);


    // Phase 2 kickoff: read the chat list back from TDLib. It auto-loads the
    // main list after authorization, so on a warm database this returns
    // immediately with everything; getChats is a pure read-back (loadChats is
    // the side-effectful one, delivering chats via updateNewChat).

    channel_manager.sync_chats();

    // Phase 2: the main event loop.
    loop {
        let update = next_update(&mut rx);
        
        // Sync channels
        let _ = channel_manager.sync()
            .inspect_err(|err| error!(
                operation = "ChannelManager::sync",
                error = %err,
                "Channel synchronization failed",
            ));


        // Crawl
        let _ = crawler.crawl()
            .inspect_err(|err| error!(
                operation = "Crawler::crawl",
                error = %err,
                "Crawl step failed",
            ));

        let Some(json) = update else { continue };

        match json["@type"].as_str() {
            None => {
                error!(
                    operation = "main.dispatch",
                    message_type = %json["@type"],
                    request_context = %json["@extra"],
                    "TDLib message has a missing or non-string @type",
                );
                continue;
            },
            Some("updateAuthorizationState") => {
                // Auth is done; anything other than a Ready re-confirmation is trouble.
                if json["authorization_state"]["@type"] != "authorizationStateReady" {
                    println!(
                        "{}",
                        format!("Authorization state changed: {}", json["authorization_state"]).red()
                    );
                }
                continue;
            },
            Some("chats") => {
                // Chat list synced; the local database has our chats now.
                let _ = channel_manager.handle_chats(&json)
                    .inspect_err(|err| log_handler_error("ChannelManager::handle_chats", &json, err));

                continue;
            },
            Some("chat") => {
                let _ = channel_manager.handle_chat(&json)
                    .inspect_err(|err| log_handler_error("ChannelManager::handle_chat", &json, err));

                continue;
            },
            Some("updateNewChat") => {
                let Some(chat) = json.get("chat") else {
                    continue;
                };

                let _ = channel_manager.handle_chat(chat)
                    .inspect_err(|err| log_handler_error("ChannelManager::handle_chat", &json, err));

                continue;
            },
            Some("supergroup") => {
                let _ = channel_manager.handle_supergroup(&json)
                    .inspect_err(|err| log_handler_error("ChannelManager::handle_supergroup", &json, err));

                continue;
            },
            Some("error") => {
                log_tdlib_request_error("main", &json);
                continue;
            },
            Some(_) => (),
        }

        match json["@extra"]["target"].as_str() {
            None => (), // Fall through
            Some("crawler") => {
                let _ = crawler.handle_response(&json)
                    .inspect_err(|err| log_handler_error("Crawler::handle_response", &json, err));
                continue;
            },
            Some(other) => {
                println!("{}", format!("Unknown response target {other}").red());
                continue;
            },
        };

        // eprintln!("- unhandled: {} {json}", json["@type"].as_str().unwrap_or("?"));
    }
}
