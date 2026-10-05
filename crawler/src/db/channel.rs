use postgres::{GenericClient, Row, types::{FromSql, ToSql, Type}};

use std::{error::Error, time::{Duration, SystemTime}};

use super::Error as DbError;

/// Mirror of the `channelstatus` Postgres enum.
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
    /// Surrogate primary key (`GENERATED ALWAYS AS IDENTITY`) — the id every
    /// other table's `channel_id` FK points at.
    pub id: i64,
    /// TDLib chat id; negative (below -10^12) for channels/supergroups.
    /// Unique, and the upsert conflict key.
    pub telegram_chat_id: i64,
    /// Primary active `@username` of the channel; None if it has none (or
    /// the supergroup data hasn't synced yet).
    pub username: Option<String>,
    /// Channel display title (from TDLib's `chat.title`).
    pub title: String,
    /// Active = in the account's chat list, eligible for crawling.
    /// Paused = reconciliation noticed it's gone from the list.
    pub status: ChannelStatus,
    /// When the row was first inserted.
    pub created_at: SystemTime,
}

pub fn get_channel_by_telegram_chat_id(
    client: &mut impl GenericClient,
    telegram_chat_id: i64,
) -> Result<Channel, DbError> {
    let row = client.query_one(
        "SELECT * FROM channels WHERE telegram_chat_id = $1 LIMIT 1",
        &[&telegram_chat_id],
    ).map_err(|e| DbError::new(
        format!("get_channel_by_telegram_chat_id: SELECT channels (telegram_chat_id={telegram_chat_id})"),
        e,
    ))?;

    channel_from_row(&row).map_err(|e| DbError::new("get_channel_by_telegram_chat_id: decode channels row", e))
}

pub fn get_all_channels(client: &mut impl GenericClient) -> Result<Vec<Channel>, DbError> {
    let rows = client.query("SELECT * FROM channels", &[])
        .map_err(|e| DbError::new("get_all_channels: SELECT channels", e))?;

    rows.iter().map(channel_from_row).collect::<Result<_, _>>()
        .map_err(|e| DbError::new("get_all_channels: decode channels rows", e))
}

pub fn deactivate_channels(client: &mut impl GenericClient, telegram_chat_ids: &[i64]) -> Result<(), DbError> {
    if telegram_chat_ids.is_empty() {
        return Ok(())
    }

    client.execute("UPDATE channels SET status = 'paused' WHERE telegram_chat_id = ANY($1)", &[&telegram_chat_ids])
        .map_err(|e| DbError::new(
            format!("deactivate_channels: UPDATE channels (telegram_chat_ids={telegram_chat_ids:?})"),
            e,
        ))?;

    Ok(())
}

/// Partial upsert for `channels`: `None` fields mean "leave unchanged"
/// (NULL on the wire, COALESCE keeps the existing value), so the
/// getSupergroup and getChat half-writes commute.
pub struct UpdateChannel<'a> {
    /// The conflict key; always required.
    pub telegram_chat_id: &'a i64,
    /// Primary `@username` from the supergroup data, if that half arrived.
    pub username: Option<&'a String>,
    /// Title from the chat data, if that half arrived.
    pub title: Option<&'a String>,
    /// Usually `Some(Active)` — either half-write reactivates a paused channel.
    pub status: Option<&'a ChannelStatus>,
}

pub fn insert_or_update_channel(client: &mut impl GenericClient, channel: UpdateChannel) -> Result<i64, DbError> {
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
    ).map_err(|e| DbError::new(
        format!("insert_or_update_channel: upsert channels (telegram_chat_id={})", channel.telegram_chat_id),
        e,
    ))?;

    row.try_get("id").map_err(|e| DbError::new("insert_or_update_channel: decode returned id", e))
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

/// Creates the `channel_pages` row for a channel; a no-op (returning the
/// existing page id) if the page already exists. Nothing here is ever
/// updated after creation.
pub struct CreateChannelPage<'a> {
    /// `channels.id` — *not* a Telegram chat id.
    pub channel_id: &'a i64,
    /// Unique URL slug (`slugify` + random suffix). Only applied at
    /// creation; existing pages keep theirs.
    pub slug: &'a str,
    /// `display_title` override for the page; None = fall back to the
    /// channel title on the site.
    pub title: Option<&'a str>,
    /// Drafts are `false` — pages stay invisible until published manually.
    pub is_published: bool,
}

pub fn create_channel_page(client: &mut impl GenericClient, page: CreateChannelPage) -> Result<i64, DbError> {
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
    ).map_err(|e| DbError::new(
        format!("create_channel_page: upsert channel_pages (channel_id={})", page.channel_id),
        e,
    ))?;

    row.try_get("channel_id").map_err(|e| DbError::new("create_channel_page: decode returned channel_id", e))
}

pub fn set_channel_crawl_error(
    client: &mut impl GenericClient,
    channel_id: i64,
    error: &str,
) -> Result<(), DbError> {
    client.execute(
        r#"
            UPDATE channel_pages
            SET
                last_crawl_error = $1
            WHERE
                channel_id = $2
        "#,
        &[&error, &channel_id],
    ).map_err(|e| DbError::new(
        format!("set_channel_crawl_error: UPDATE channel_pages (channel_id={channel_id})"),
        e,
    ))?;

    Ok(())
}

/// Stamps a successful end-of-crawl on the page row: the completion time
/// (from which the staleness clock runs) and the message id the next crawl
/// should resume from. Also clears `last_crawl_error` — a completed crawl
/// is by definition not in the error state, and the claim filter excludes
/// channels with an error set.
///
/// `None` for the checkpoint means the channel had no messages — the next
/// crawl starts from the beginning again, same as a never-crawled channel.
pub fn finish_channel_crawl(
    client: &mut impl GenericClient,
    channel_id: i64,
    crawl_checkpoint_message_id: Option<i64>,
) -> Result<(), DbError> {
    client.execute(
        r#"
            UPDATE channel_pages
            SET
                last_crawl_completed_at = now(),
                crawl_checkpoint_message_id = $1,
                last_crawl_error = NULL
            WHERE
                channel_id = $2
        "#,
        &[&crawl_checkpoint_message_id, &channel_id],
    ).map_err(|e| DbError::new(
        format!("finish_channel_crawl: UPDATE channel_pages (channel_id={channel_id})"),
        e,
    ))?;

    Ok(())
}

/// A channel claimed for a crawl — everything the crawl loop needs to
/// start (and to stamp errors back onto the page row).
#[derive(Debug, Clone)]
pub struct CrawlCandidate {
    /// `channels.id` — for stamping `channel_pages` (claim, errors).
    pub channel_id: i64,
    /// TDLib chat id — the one `searchChatMessages` is sent to.
    pub telegram_chat_id: i64,
    /// Last message id already persisted (`channel_pages`' checkpoint);
    /// None = never crawled, start from the channel's beginning.
    pub crawl_checkpoint_message_id: Option<i64>,
    /// Channel title; for logs only.
    pub title: String,
}

pub fn claim_channel_due_for_crawl(
    client: &mut impl GenericClient,
    crawl_stale_after: &Duration
) -> Result<Option<CrawlCandidate>, DbError> {
    let mut tx = client.transaction()
        .map_err(|e| DbError::new("claim_channel_due_for_crawl: begin transaction", e))?;

    let crawl_stale_after_secs = crawl_stale_after.as_secs_f64();

    let Some(row) = tx.query_opt(
        r#"
            SELECT
                channel_id,
                telegram_chat_id,
                crawl_checkpoint_message_id,
                title
            FROM channel_pages
            INNER JOIN channels ON channels.id = channel_pages.channel_id
            WHERE 
                status = 'active'
                AND (
                    last_crawl_started_at + make_interval(secs => $1) < now()
                    OR last_crawl_started_at IS NULL
                )
                AND last_crawl_error IS NULL
            ORDER BY channel_pages.last_crawl_started_at NULLS FIRST
            LIMIT 1
            FOR UPDATE OF channel_pages SKIP LOCKED
        "#,
        &[&crawl_stale_after_secs],
    ).map_err(|e| DbError::new(
        format!("claim_channel_due_for_crawl: SELECT candidate (crawl_stale_after_secs={crawl_stale_after_secs})"),
        e,
    ))? else {
        return Ok(None)
    };

    let candidate = crawl_candidate_from_row(&row)
        .map_err(|e| DbError::new("claim_channel_due_for_crawl: decode candidate row", e))?;

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
    ).map_err(|e| DbError::new(
        format!("claim_channel_due_for_crawl: UPDATE channel_pages (channel_id={})", candidate.channel_id),
        e,
    ))?;

    tx.commit().map_err(|e| DbError::new(
        format!("claim_channel_due_for_crawl: commit transaction (channel_id={})", candidate.channel_id),
        e,
    ))?;

    Ok(Some(candidate))
}

fn crawl_candidate_from_row(row: &Row) -> Result<CrawlCandidate, postgres::Error> {
    Ok(CrawlCandidate {
        channel_id: row.try_get("channel_id")?,
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        crawl_checkpoint_message_id: row.try_get("crawl_checkpoint_message_id")?,
        title: row.try_get("title")?,
    })
}
