use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExtraTarget {
    Crawler,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Extra {
    pub target: ExtraTarget,
    pub request_id: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub date: i32,
    pub edit_date: i32,
    pub content: MessageContent,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "@type")]
pub enum MessageContent {
    #[serde(rename = "messageAudio")]
    Audio {
        audio: Audio,
        caption: FormattedText,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Audio {
    #[serde(rename = "audio")]
    pub file: TdFile,
    pub duration: i32,
    pub performer: String,
    pub title: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct TdFile {
    pub remote: RemoteFile,
    pub size: i64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RemoteFile {
    pub id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct FormattedText {
    pub text: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Message, MessageContent};

    #[test]
    fn deserializes_the_used_audio_message_fields() {
        let message: Message = serde_json::from_value(json!({
            "@type": "message",
            "id": 102_760_448,
            "chat_id": -1_003_743_724_869_i64,
            "date": 1_781_900_560,
            "edit_date": 1_781_955_266,
            "is_channel_post": true,
            "content": {
                "@type": "messageAudio",
                "audio": {
                    "@type": "audio",
                    "audio": {
                        "@type": "file",
                        "id": 1311,
                        "size": 7_897_093,
                        "remote": {
                            "@type": "remoteFile",
                            "id": "remote-file-id",
                            "unique_id": "ignored"
                        }
                    },
                    "duration": 1419,
                    "file_name": "ignored.ogg",
                    "mime_type": "audio/ogg",
                    "performer": "Channel author",
                    "title": "Recording title"
                },
                "caption": {
                    "@type": "formattedText",
                    "entities": [],
                    "text": "Recording caption"
                }
            }
        }))
        .expect("audio message should deserialize");

        assert_eq!(message.id, 102_760_448);
        assert_eq!(message.chat_id, -1_003_743_724_869);
        assert_eq!(message.date, 1_781_900_560);
        assert_eq!(message.edit_date, 1_781_955_266);

        let MessageContent::Audio { audio, caption } = message.content;
        assert_eq!(audio.file.remote.id, "remote-file-id");
        assert_eq!(audio.file.size, 7_897_093);
        assert_eq!(audio.duration, 1419);
        assert_eq!(audio.performer, "Channel author");
        assert_eq!(audio.title, "Recording title");
        assert_eq!(caption.text, "Recording caption");
    }
}
