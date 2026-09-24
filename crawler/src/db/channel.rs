use postgres::{GenericClient, Row, types::{FromSql, ToSql, Type}};

use std::{error::Error, time::{Duration, SystemTime}};

#[derive(Debug)]
pub enum ChannelStatus {
    Active,
    Paused,
}

impl ToSql for ChannelStatus {
    fn to_sql(&self, _: &Type, out: &mut postgres::types::private::BytesMut) -> Result<postgres::types::IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized {
        let value = match self {
            ChannelStatus::Active => "active",
            ChannelStatus::Paused => "paused",
        };

        out.extend_from_slice(value.as_bytes());

        Ok(postgres::types::IsNull::No)
    }

    fn accepts(ty: &Type) -> bool
    where
        Self: Sized {
        ty.name() == "channelstatus"
    }

    
    postgres::types::to_sql_checked!();
}

impl<'a> FromSql<'a> for ChannelStatus {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        match std::str::from_utf8(raw)? {
            "active" => Ok(ChannelStatus::Active),
            "paused" => Ok(ChannelStatus::Paused),
            _ => Err("unknown variant".into()),
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty.name() == "channelstatus"
    }
}

pub struct Channel {
    pub id: i64,
    pub telegram_chat_id: i64,
    pub username: Option<String>,
    pub title: String,
    pub status: ChannelStatus,
    pub created_at: SystemTime,
}

pub fn get_channel_by_telegram_chat_id(
    client: &mut impl GenericClient,
    telegram_chat_id: i64,
) -> Result<Channel, postgres::Error> {
    let row = client.query_one(
        "SELECT * FROM channels WHERE telegram_chat_id = $1 LIMIT 1",
        &[&telegram_chat_id],
    )?;

    channel_from_row(&row)
}

pub fn get_all_channels(client: &mut impl GenericClient) -> Result<Vec<Channel>, postgres::Error> {
    let rows = client.query("SELECT * FROM channels", &[])?;

    rows.iter().map(channel_from_row).collect()
}

pub fn deactivate_channels(client: &mut impl GenericClient, telegram_chat_ids: &[i64]) -> Result<(), postgres::Error> {
    if telegram_chat_ids.is_empty() {
        return Ok(())
    }

    client.execute("UPDATE channels SET status = 'paused' WHERE telegram_chat_id in ({})", &[&telegram_chat_ids])?;

    Ok(())
}

pub struct UpdateChannel<'a> {
    /// The conflict key; always required.
    pub telegram_chat_id: &'a i64,
    /// The rest are partial-update columns: None means "leave unchanged"
    /// (NULL on the wire, COALESCE keeps the existing value).
    pub username: Option<&'a String>,
    pub title: Option<&'a String>,
    pub status: Option<&'a ChannelStatus>,
}

pub fn insert_or_update_channel(client: &mut impl GenericClient, channel: UpdateChannel) -> Result<i64, postgres::Error> {
    let row = client.query_one(
        r#"
            INSERT INTO channels (telegram_chat_id, username, title, status)
            VALUES ($1, $2, COALESCE($3, ''), COALESCE($4::channelstatus, 'active'))
            ON CONFLICT (telegram_chat_id) DO UPDATE SET
                username = COALESCE(EXCLUDED.username, channels.username),
                title = COALESCE(EXCLUDED.title, channels.title),
                status = COALESCE(EXCLUDED.status, channels.status)
            RETURNING id
        "#,
        &[
            &channel.telegram_chat_id,
            &channel.username,
            &channel.title,
            &channel.status,
        ]
    )?;

    row.try_get("id")
}

pub fn channel_from_row(row: &Row) -> Result<Channel, postgres::Error> {
    Ok(
        Channel {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            telegram_chat_id: row.try_get("telegram_chat_id")?,
            title: row.try_get("title")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?
        }
    )
}

pub struct CreateChannelPage<'a> {
    pub channel_id: &'a i64,
    pub slug: &'a str,
    pub title: Option<&'a str>,
    pub is_published: bool,
}

pub fn create_channel_page(client: &mut impl GenericClient, page: CreateChannelPage) -> Result<i64, postgres::Error> {
    let row = client.query_one(
        r#"
            INSERT INTO channel_pages (channel_id, slug, display_title, is_published)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (channel_id) DO UPDATE SET channel_id = channel_pages.channel_id
            RETURNING channel_id
        "#,
        &[
            &page.channel_id,
            &page.slug,
            &page.title,
            &page.is_published,
        ]
    )?;

    row.try_get("channel_id")
}

pub struct CrawlCandidate {
    pub channel_id: i64,
    pub telegram_chat_id: i64,
    pub last_scanned_message_id: Option<i64>,
    pub title: String,
}

pub fn claim_channel_due_for_crawl(
    client: &mut impl GenericClient,
    crawl_stale_after: &Duration
) -> Result<Option<CrawlCandidate>, postgres::Error> {
    let mut tx = client.transaction()?;

    let Some(row) = tx.query_opt(
        r#"
            SELECT
                channel_id,
                telegram_chat_id,
                last_scanned_message_id,
                title
            FROM channel_pages
            INNER JOIN channels ON channels.id = channel_pages.channel_id
            WHERE 
                status = 'active'
                AND (
                    last_crawl_started_at + $1::interval < now()
                    OR last_crawl_started_at IS NULL
                )
            ORDER BY channel_pages.last_crawl_started_at NULLS FIRST
            LIMIT 1
            FOR UPDATE OF channel_pages SKIP LOCKED
        "#,
        &[&format!("{} seconds", crawl_stale_after.as_secs())],
    )? else { return Ok(None) };
    
    let candidate = CrawlCandidate {
        channel_id: row.try_get("channel_id")?,
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        last_scanned_message_id: row.try_get("last_scanned_message_id")?,
        title: row.try_get("title")?,
    };
    
    tx.execute(
        r#"
            UPDATE channel_pages
            SET
                last_crawl_started_at = now(),
                last_crawl_completed_at = NULL,
                last_crawl_error = NULL
            WHERE
                channel_id = $1
        "#,
        &[&candidate.channel_id],
    )?;

    tx.commit()?;

    Ok(Some(candidate))
}
