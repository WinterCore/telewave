use std::time::Duration;

use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{debug, error, trace};
use tracing_subscriber::EnvFilter;

use crate::{
    auth::Auth,
    channel_manager::ChannelManager,
    config::Config,
    crawler::Crawler,
    tdjson::{ClientId, Receiver, set_log_verbosity_level},
    tdtypes::Supergroup
};

mod auth;
mod config;
mod tdjson;
mod crawler;
mod channel_manager;
mod tdtypes;
mod db;

/// Blocks until TDLib produces one parseable update.
/// Timeouts are waited out; bad JSON is logged and skipped.
fn next_update(rx: &mut Receiver) -> Value {
    loop {
        match rx.receive_json(Duration::from_secs(1)) {
            Some(Ok(v)) => return v,
            Some(Err(e)) => eprintln!("bad JSON from TDLib: {e}"),
            None => continue,
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
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
        let json = next_update(&mut rx);

        match json["@type"].as_str() {
            Some("updateAuthorizationState") => {
                if auth.handle(&json["authorization_state"]) {
                    break;
                }
            },
            Some("error") => println!("{}", format!("Error: {}", json["message"]).red()),
            _ => (),
        }
    }

    let crawler = Crawler::new(&client);
    let mut channel_manager = ChannelManager::new(&client);


    // Phase 2 kickoff: read the chat list back from TDLib. It auto-loads the
    // main list after authorization, so on a warm database this returns
    // immediately with everything; getChats is a pure read-back (loadChats is
    // the side-effectful one, delivering chats via updateNewChat).

    // Phase 2: the main event loop.
    channel_manager.sync_chats();

    loop {
        let json = next_update(&mut rx);

        match json["@type"].as_str() {
            None => {
                eprintln!("bad @type from TDLib");
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
                channel_manager.handle_chats(&json)
                    .inspect_err(|x| error!(x));


                continue;
            },
            Some("error") => {
                println!("{}", format!("Error: {}", json["message"]).red());
                continue;
            },
            Some(_) => (),
        }

        match json["@extra"]["target"].as_str() {
            None => (), // Fall through
            Some("crawler") => {
                let _ = crawler.handle_response(json)
                    .inspect_err(|x| println!("{}", format!("Crawler Error: {x}")));
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
