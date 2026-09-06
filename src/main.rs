use std::time::Duration;

use owo_colors::OwoColorize;

use crate::{auth::Auth, config::Config, tdjson::{ClientId, Receiver, set_log_verbosity_level}};
mod auth;
mod config;
mod tdjson;

fn main() {
    let config = Config::from_env();

    set_log_verbosity_level(1);
    let client = ClientId::new();

    let mut rx = Receiver::take().expect("Should acquire receiver");
    client.send(r#"{ "@type": "getAuthorizationState" }"#);

    let auth = Auth::new(&client, &config);

    loop {
        let json = match rx.receive_json(Duration::from_secs(1)) {
            Some(Ok(v)) => v,
            Some(Err(e)) => {
                eprintln!("bad JSON from TDLib: {e}");
                continue;
            },
            None => continue,
        };


        match json["@type"].as_str() {
            None => eprintln!("bad @type from TDLib"),
            Some("updateAuthorizationState") => auth.handle(&json["authorization_state"]),
            Some("error") => {
                let message = &json["message"];
                println!("{}", format!("Error: {message}").red())
            },
            Some(other) => eprintln!("- unhandled: {other} {json}"),
        }
    }
}
