use std::{time::Duration};

use owo_colors::OwoColorize;

use crate::{
    auth::Auth,
    config::Config,
    tdjson::{ClientId, Receiver, set_log_verbosity_level},
    crawler::Crawler
};

mod auth;
mod config;
mod tdjson;
mod crawler;
mod tdtypes;

fn main() {
    let config = Config::from_env();

    set_log_verbosity_level(1);
    let client = ClientId::new();

    let mut rx = Receiver::take().expect("Should acquire receiver");
    client.send(r#"{ "@type": "getAuthorizationState" }"#);

    let auth = Auth::new(&client, &config);

    let crawler = Crawler::new(&client);

    loop {
        let json = match rx.receive_json(Duration::from_secs(1)) {
            Some(Ok(v)) => v,
            Some(Err(e)) => {
                eprintln!("bad JSON from TDLib: {e}");
                continue;
            },
            None => continue,
        };


        let other = match json["@type"].as_str() {
            None => {
                eprintln!("bad @type from TDLib");
                continue;
            },
            Some("updateAuthorizationState") => {
                if auth.handle(&json["authorization_state"]) {
                    crawler.crawl(0, None);
                }
                continue;
            },
            Some("error") => {
                let message = &json["message"];
                println!("{}", format!("Error: {message}").red());
                continue;
            },
            Some(other) => other,
        };

        match json["@extra"]["target"].as_str() {
            None => (), // Fall through
            Some("crawler") => {
                let _ = crawler.handle_response(json).inspect_err(|x| println!("{}", format!("Crawler Error: {x}")));
                continue;
            },
            Some(other) => {
                println!("{}", format!("Unknown response target {other}").red());
                continue;
            },
        };

        eprintln!("- unhandled: {other} {json}");
    }
}
