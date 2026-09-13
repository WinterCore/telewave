use postgres::{Error, Row};

use crate::DB;
use std::time::SystemTime;

#[derive(Debug, FromSql, ToSql)]
#[postgres(name = "channelstatus")]
pub enum ChannelStatus {
    #[postgres(name = "active")]
    Active,

    #[postgres(name = "paused")]
    Paused,
}

pub struct Channel {
    pub id: i64,
    pub telegram_chat_id: i64,
    pub username: Option<String>,
    pub title: String,
    pub status: ChannelStatus,
    pub created_at: SystemTime,
}

impl DB {
    pub fn get_channel_by_telegram_chat_id(&self, id: i64) -> Result<Channel, Error> {
        let rows = self.client.query("SELECT * FROM channels WHERE telegram_chat_id = $1", &[&id])?;
    }

    pub fn channel_from_row(row: Row) -> Result<Channel, Error> {
        Ok(
            Channel {
                id: row.try_get("id")?,
                username: None,
                telegram_chat_id: row.try_get("telegram_chat_id")?,
                title: row.try_get("title")?,
                status: row.try_get("status")?,
                created_at: row.try_get("created_at")?
            }
        )
    }
}
