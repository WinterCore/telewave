use postgres::{Error, GenericClient, Row, types::ToSql};

use crate::tdtypes::{Message, MessageContent, unix_to_system_time};
use std::{any::Any, time::SystemTime};

use crate::db;

pub struct ChannelPageRecording {
    pub id: i64,
    pub channel_id: i64,
    pub author_id: i64,
    pub category_id: Option<i64>,
    pub telegram_message_id: i64,
    pub telegram_remote_file_id: String,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub duration_seconds: Option<i32>,
    pub file_size_bytes: Option<i64>,
    pub storage_key: Option<String>,
    pub published_at: SystemTime,
    pub telegram_edited_at: Option<SystemTime>,
    pub downloaded_at: Option<SystemTime>,
    pub created_at: SystemTime,
}

pub struct RecordingJob {
    pub recording_id: i64,
    pub source_storage_key: Option<String>,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub failed_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

pub fn create_recording_jobs_from_messages(
    client: &mut impl GenericClient,
    channel_id: i64,
    messages: &[Message],
) -> Result<(), Error> {
    let mut tx = client.transaction()?;

    for msg in messages {
        let MessageContent::Audio { audio, caption } = &msg.content else {
            continue;
        };

        let published_at = unix_to_system_time(msg.date); 

        let row: [&(dyn ToSql + Sync); 10] = [
            &channel_id,
            &msg.id,
            &audio.file.remote.id,
            &audio.title,
            &caption.text,
            &audio.duration,
            &None::<i64>,
            &None::<String>,
            &published_at,
            &None::<SystemTime>,
        ];


        let sql = format!("
            INSERT INTO channel_page_recordings (
                channel_id,
                telegram_message_id,
                telegram_remote_file_id,
                title,
                caption,
                duration_seconds,
                file_size_bytes,
                storage_key,
                published_at,
                downloaded_at,
            ) VALUES ({});
        ", (0..10).into_iter().map(|i| format!("${i}")).collect::<Vec<_>>().join(","));

        tx.execute(&sql, &row)?;
    }



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
        telegram_edited_at: row.try_get("telegram_edited_at")?,
        downloaded_at: row.try_get("downloaded_at")?,
        created_at: row.try_get("created_at")?,
    })
}

pub fn recording_job_from_row(row: &Row) -> Result<RecordingJob, Error> {
    Ok(RecordingJob {
        recording_id: row.try_get("recording_id")?,
        source_storage_key: row.try_get("source_storage_key")?,
        attempts: row.try_get("attempts")?,
        last_error: row.try_get("last_error")?,
        failed_at: row.try_get("failed_at")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
