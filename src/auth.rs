use std::io::{self, Write};

use serde::Deserialize;
use serde_json::{Value, json};
use owo_colors::OwoColorize;

use crate::config::Config;
use crate::tdjson::ClientId;

#[derive(Deserialize)]
#[serde(tag = "@type")]
enum AuthenticationCodeType {
    #[serde(rename = "authenticationCodeTypeTelegramMessage")]
    AuthenticationCodeTypeTelegramMessage { length: u8 }
}

#[derive(Deserialize)]
#[serde(tag = "@type", rename = "setAuthenticationPhoneNumber")]
struct AuthenticationCodeInfo {
    phone_number: String,
    #[serde(rename = "type")]
    auth_type: AuthenticationCodeType,
}

#[derive(Deserialize)]
#[serde(tag = "@type")]
enum AuthorizationState {
    #[serde(rename = "authorizationStateWaitTdlibParameters")]
    WaitTdlibParameters,

    #[serde(rename = "authorizationStateWaitPhoneNumber")]
    WaitPhoneNumber,

    #[serde(rename = "authorizationStateWaitCode")]
    WaitCode { code_info: AuthenticationCodeInfo },

    #[serde(rename = "authorizationStateReady")]
    Ready,

    #[serde(other)]
    Other,
}

pub struct Auth<'a> {
    client: &'a ClientId,
    config: &'a Config,
}

impl<'a> Auth<'a> {
    pub fn new(client: &'a ClientId, config: &'a Config) -> Self {
        Self { client, config }
    }

    fn send_tdlib_params(&self) {
        self.client.send_json(
            &json!({
                "@type": "setTdlibParameters",
                "use_test_dc": false,
                "database_directory": "./.data/tdlib/telewave",
                "files_directory": "./.data/files/telewave",
                "database_encryption_key": "",
                "use_file_database": true,
                "use_chat_info_database": true,
                "use_message_database": true,
                "use_secret_chats": false,
                "api_id": self.config.api_id,
                "api_hash": self.config.api_hash,
                "system_language_code": "en",
                "device_model": "telewave",
                "system_version": "",
                "application_version": "0.1.0"
            })
        );
        println!("{}", "Sent tdlib params".green())
    }

    fn send_phone_number(&self) {
        self.client.send_json(
            &json!({
                "@type": "setAuthenticationPhoneNumber",
                "phone_number": self.config.phone_number,
                "settings": null,
            })
        );
        println!("{}", "Sent phone number".green())
    }

    fn handle_auth_code(&self, info: AuthenticationCodeInfo) {
        let code_length = match info.auth_type {
            AuthenticationCodeType::AuthenticationCodeTypeTelegramMessage { length } => length,
        };
        
        loop {
            print!("Enter the {code_length} digit code you received: ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            io::stdin().read_line(&mut line).expect("Should read code");
            let code = line.trim();

            if code.len() != code_length as usize {
                println!("{}", "Invalid code length!".yellow());
                continue;
            }


            self.client.send_json(
                &json!({
                    "@type": "checkAuthenticationCode",
                    "code": code,
                })
            );
            println!("{}", "Sent auth code".green());
            break;
        }
    }

    pub fn handle(&self, json: &Value) {
        let state = AuthorizationState::deserialize(json);

        let state = match state {
            Err(e) => {
                eprintln!("Unsupported updateAuthorizationState {e}\n{json}");
                return;
            },
            Ok(state) => state,
        };

        match state {
            AuthorizationState::WaitTdlibParameters => self.send_tdlib_params(),
            AuthorizationState::WaitPhoneNumber => self.send_phone_number(),
            AuthorizationState::WaitCode { code_info } => self.handle_auth_code(code_info),
            AuthorizationState::Ready => {
                println!("{}", "Auth completed".green())
            },
            AuthorizationState::Other => {
                eprintln!("Unhandled updateAuthorizationState {json}");
            },
        }
    }
}
