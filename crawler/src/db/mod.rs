use postgres::{Client, NoTls};

pub mod channel;

pub struct DB {
    client: Client,
}

impl DB {
    pub fn new() -> Self {
        let url = "postgres://winter@127.0.0.1:5432/telewave";
        let client = Client::connect(url, NoTls)
            .expect("Should connect to DB");

        Self { client }
    }
}
