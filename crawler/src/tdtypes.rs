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
    #[serde(default)]
    pub sender_id: Option<MessageSender>,
    #[serde(default)]
    pub is_outgoing: Option<bool>,
    pub date: i32,
    pub edit_date: i32,
    pub content: MessageContent,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "@type")]
pub enum MessageContent {
    #[serde(rename = "messageText")]
    Text { text: FormattedText },
    #[serde(rename = "messageAudio")]
    Audio {
        audio: Audio,
        caption: FormattedText,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "@type")]
pub enum Update {
    #[serde(rename = "updateNewMessage")]
    NewMessage { message: Message },
    #[serde(rename = "updateSupergroup")]
    Supergroup { supergroup: Supergroup },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "@type")]
pub enum TextEntityType {
    #[serde(rename = "textEntityTypeUrl")]
    Url,
    #[serde(rename = "textEntityTypeTextUrl")]
    TextUrl { url: String },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct TextEntity {
    pub offset: i32,
    pub length: i32,
    #[serde(rename = "type")]
    pub kind: TextEntityType,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "@type")]
pub enum MessageSender {
    #[serde(rename = "messageSenderUser")]
    User { user_id: i64 },
    #[serde(rename = "messageSenderChat")]
    Chat { chat_id: i64 },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ChatMemberStatus {
    #[serde(rename = "@type")]
    pub kind: ChatMemberStatusKind,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum ChatMemberStatusKind {
    #[serde(rename = "chatMemberStatusCreator")]
    Creator,
    #[serde(rename = "chatMemberStatusAdministrator")]
    Administrator,
    #[serde(rename = "chatMemberStatusMember")]
    Member,
    #[serde(rename = "chatMemberStatusRestricted")]
    Restricted,
    #[serde(rename = "chatMemberStatusLeft")]
    Left,
    #[serde(rename = "chatMemberStatusBanned")]
    Banned,
    #[serde(other)]
    Other,
}

/// The fields needed by the crawler from TDLib's `supergroup` object.
///
/// TDLib includes additional fields in this object; Serde ignores those fields
/// so this remains a deliberately narrow model.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Supergroup {
    pub id: i64,
    pub date: i32,
    pub status: ChatMemberStatus,
    pub is_channel: bool,
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
    #[serde(default)]
    pub entities: Vec<TextEntity>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ChatMemberStatusKind, Message, MessageContent, MessageSender, TextEntityType, Update,
    };

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

        let MessageContent::Audio { audio, caption } = message.content else {
            panic!("expected messageAudio");
        };
        assert_eq!(audio.file.remote.id, "remote-file-id");
        assert_eq!(audio.file.size, 7_897_093);
        assert_eq!(audio.duration, 1419);
        assert_eq!(audio.performer, "Channel author");
        assert_eq!(audio.title, "Recording title");
        assert_eq!(caption.text, "Recording caption");
    }

    #[test]
    fn deserializes_a_new_text_message_with_a_url_entity() {
        let update: Update = serde_json::from_value(json!({
            "@type": "updateNewMessage",
            "message": {
                "@type": "message",
                "id": 77,
                "chat_id": 456,
                "sender_id": {
                    "@type": "messageSenderUser",
                    "user_id": 123
                },
                "is_outgoing": false,
                "date": 1_781_900_560,
                "edit_date": 0,
                "content": {
                    "@type": "messageText",
                    "text": {
                        "@type": "formattedText",
                        "text": "https://t.me/+abc",
                        "entities": [
                            {
                                "@type": "textEntity",
                                "offset": 0,
                                "length": 17,
                                "type": {
                                    "@type": "textEntityTypeUrl"
                                }
                            }
                        ]
                    },
                    "link_preview": null,
                    "link_preview_options": null
                }
            }
        }))
        .expect("new text message should deserialize");

        let Update::NewMessage { message } = update else {
            panic!("expected updateNewMessage");
        };

        assert_eq!(
            message.sender_id,
            Some(MessageSender::User { user_id: 123 })
        );
        assert_eq!(message.is_outgoing, Some(false));

        let MessageContent::Text { text } = message.content else {
            panic!("expected messageText");
        };
        assert_eq!(text.entities[0].kind, TextEntityType::Url);
    }

    #[test]
    fn deserializes_a_channel_supergroup_update() {
        let update: Update = serde_json::from_value(json!({
            "@type": "updateSupergroup",
            "supergroup": {
                "@type": "supergroup",
                "id": 123,
                "date": 1_781_900_560,
                "status": {
                    "@type": "chatMemberStatusMember"
                },
                "member_count": 42,
                "is_channel": true
            }
        }))
        .expect("supergroup update should deserialize");

        let Update::Supergroup { supergroup } = update else {
            panic!("expected updateSupergroup");
        };

        assert_eq!(supergroup.id, 123);
        assert_eq!(supergroup.date, 1_781_900_560);
        assert_eq!(supergroup.status.kind, ChatMemberStatusKind::Member);
        assert!(supergroup.is_channel);
    }
}
