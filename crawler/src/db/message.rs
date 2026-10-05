use postgres::{Error, GenericClient, Row};

use crate::tdtypes::{Message, MessageContent, unix_to_system_time};
use std::time::SystemTime;

pub struct ChannelPageRecording {
    /// Surrogate primary key (`GENERATED ALWAYS AS IDENTITY`).
    pub id: i64,
    /// Owning channel — `channels.id` / `channel_pages.channel_id`,
    /// *not* a Telegram chat id.
    pub channel_id: i64,
    /// Optional attribution; FK into `channel_authors`. Not populated yet —
    /// and the column is nullable while this field isn't (`Option` fix pending).
    pub author_id: i64,
    /// Optional FK into `channel_categories`. Not populated yet.
    pub category_id: Option<i64>,
    /// The message id within the Telegram channel. Half of the dedupe key
    /// (`UNIQUE (channel_id, telegram_message_id)`).
    pub telegram_message_id: i64,
    /// TDLib `remote.id` for the audio file — the handle `downloadFile` is
    /// fed. Not guaranteed stable forever by Telegram, so re-resolve via
    /// the message if a download rejects it.
    pub telegram_remote_file_id: String,
    /// Audio title from the message metadata (empty TDLib string → None).
    pub title: Option<String>,
    /// Message caption text (empty TDLib string → None).
    pub caption: Option<String>,
    /// Audio duration per TDLib.
    pub duration_seconds: Option<i32>,
    /// Size of the *stored ogg opus artifact* in bytes — not the source
    /// file's size. Stamped by the downloader; None until then.
    pub file_size_bytes: Option<i64>,
    /// Where the converted ogg opus artifact lives in storage. Set by the
    /// downloader; None until then.
    pub storage_key: Option<String>,
    /// When the message was posted to the channel (`Message.date`) — the
    /// recording's "air date" and the site's sort key.
    pub published_at: SystemTime,
    /// When the downloader finished (download + convert + rename, all of it).
    /// None = not downloaded yet. This is the downloader's whole state bit.
    pub downloaded_at: Option<SystemTime>,
    /// Failed download attempts so far; the poison-pill guard
    /// (`downloaded_at IS NULL AND download_attempts < max`).
    pub download_attempts: i32,
    /// Reason for the last failed download attempt.
    pub last_error: Option<String>,
    /// When the crawler first inserted this row.
    pub created_at: SystemTime,
}

/// Inserts one `channel_page_recordings` row per audio message, skipping
/// rows that already exist (same channel + telegram message id) — so
/// re-crawling or re-delivering a page is a no-op.
///
/// `channel_id` is the `channels.id` / `channel_pages.channel_id`, not a
/// Telegram chat id (`Message.chat_id` is *not* it).
pub fn insert_messages(
    client: &mut impl GenericClient,
    channel_id: i64,
    messages: &[Message],
) -> Result<(), Error> {
    if messages.is_empty() {
        return Ok(());
    }

    let mut tx = client.transaction()?;

    for msg in messages {
        let MessageContent::Audio { audio, caption } = &msg.content else {
            continue;
        };

        // TDLib gives "" where the schema would rather have NULL.
        let title = (!audio.title.is_empty()).then(|| audio.title.clone());
        let caption_text = (!caption.text.is_empty()).then(|| caption.text.clone());
        let published_at = unix_to_system_time(msg.date);

        tx.execute(
            r#"
                INSERT INTO channel_page_recordings (
                    channel_id, telegram_message_id, telegram_remote_file_id,
                    title, caption, duration_seconds, file_size_bytes, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (channel_id, telegram_message_id) DO NOTHING
            "#,
            &[
                &channel_id,
                &msg.id,
                &audio.file.remote.id,
                &title,
                &caption_text,
                &audio.duration,
                // Size of the *stored* artifact; stamped by the downloader.
                &None::<i64>,
                &published_at,
            ],
        )?;
    }

    tx.commit()?;

    Ok(())
}

pub fn channel_page_recording_from_row(row: &Row) -> Result<ChannelPageRecording, Error> {
    Ok(ChannelPageRecording {
        id: row.try_get("id")?,
        channel_id: row.try_get("channel_id")?,
        author_id: row.try_get("author_id")?,
        category_id: row.try_get("category_id")?,
        telegram_message_id: row.try_get("telegram_message_id")?,
        telegram_remote_file_id: row.try_get("telegram_remote_file_id")?,
        title: row.try_get("title")?,
        caption: row.try_get("caption")?,
        duration_seconds: row.try_get("duration_seconds")?,
        file_size_bytes: row.try_get("file_size_bytes")?,
        storage_key: row.try_get("storage_key")?,
        published_at: row.try_get("published_at")?,
        downloaded_at: row.try_get("downloaded_at")?,
        download_attempts: row.try_get("download_attempts")?,
        last_error: row.try_get("last_error")?,
        created_at: row.try_get("created_at")?,
    })
}
