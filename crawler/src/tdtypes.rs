use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

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

pub fn unix_to_system_time(secs: i32) -> SystemTime {
    if secs >= 0 {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs as u64)
    } else {
        SystemTime::UNIX_EPOCH - Duration::from_secs(secs.unsigned_abs() as u64)
    }
}

/// TDLib uses 0 for "never".
pub fn unix_to_system_time_opt(secs: i32) -> Option<SystemTime> {
    (secs != 0).then(|| unix_to_system_time(secs))
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
    /// Null if the supergroup or channel has no username.
    pub usernames: Option<Usernames>,
    pub date: i32,
    pub status: ChatMemberStatus,
    pub member_count: i32,
    pub is_channel: bool,
}

/// The fields needed by the crawler from TDLib's `usernames` object.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Usernames {
    /// The first entry is the primary username; empty if there is none.
    pub active_usernames: Vec<String>,
}

/// The fields needed by the crawler from TDLib's `chat` object.
///
/// TDLib includes additional fields in this object; Serde ignores those fields
/// so this remains a deliberately narrow model.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Chat {
    pub id: i64,
    pub title: String,
    #[serde(rename = "type")]
    pub kind: ChatType,
    /// Chat lists this chat belongs to; non-empty means actual membership.
    /// TDLib also sends `updateNewChat` for merely previewed chats. (`positions`
    /// is not reliable for this: a chat can have a position in a list it does
    /// not belong to.)
    pub chat_lists: Vec<ChatList>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "@type")]
pub enum ChatType {
    #[serde(rename = "chatTypeSupergroup")]
    Supergroup {
        supergroup_id: i64,
        is_channel: bool,
    },
    #[serde(other)]
    Other,
}

/// The fields needed by the crawler from TDLib's `chatListMain`/`chatListArchive`
/// objects: none. Only whether `chat.chat_lists` contains an entry matters.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ChatList {}

/// Chat ids of supergroups and channels live in their own numeric range:
/// `-1_000_000_000_000 - supergroup_id`. User ids are positive and basic-group
/// ids are negated int32s, so both stay above this threshold.
pub fn is_supergroup_chat_id(chat_id: i64) -> bool {
    chat_id < -1_000_000_000_000
}

/// The chat id a supergroup or channel is addressed by (`getChat`, `chat_id`
/// fields), from its raw `supergroup_id` (`getSupergroup`).
pub fn supergroup_to_chat_id(supergroup_id: i64) -> i64 {
    -1_000_000_000_000 - supergroup_id
}

/// The raw `supergroup_id` behind a supergroup/channel chat id.
pub fn chat_to_supergroup_id(chat_id: i64) -> i64 {
    debug_assert!(is_supergroup_chat_id(chat_id));
    -chat_id - 1_000_000_000_000
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
    use std::time::{Duration, UNIX_EPOCH};

    use super::{
        Chat, ChatMemberStatusKind, ChatType, Message, MessageContent, MessageSender,
        TextEntityType, Update, chat_to_supergroup_id, is_supergroup_chat_id,
        supergroup_to_chat_id, unix_to_system_time, unix_to_system_time_opt,
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
                "usernames": {
                    "@type": "usernames",
                    "active_usernames": ["telewave"],
                    "editable_username": "telewave"
                },
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
        assert_eq!(
            supergroup.usernames.as_ref().unwrap().active_usernames,
            vec!["telewave"]
        );
        assert_eq!(supergroup.date, 1_781_900_560);
        assert_eq!(supergroup.status.kind, ChatMemberStatusKind::Member);
        assert_eq!(supergroup.member_count, 42);
        assert!(supergroup.is_channel);
    }

    #[test]
    fn deserializes_a_supergroup_without_usernames() {
        let update: Update = serde_json::from_value(json!({
            "@type": "updateSupergroup",
            "supergroup": {
                "@type": "supergroup",
                "id": 123,
                "usernames": null,
                "date": 1_781_900_560,
                "status": {
                    "@type": "chatMemberStatusMember"
                },
                "member_count": 0,
                "is_channel": false
            }
        }))
        .expect("supergroup update should deserialize");

        let Update::Supergroup { supergroup } = update else {
            panic!("expected updateSupergroup");
        };

        assert_eq!(supergroup.usernames, None);
        assert_eq!(supergroup.member_count, 0);
    }

    #[test]
    fn deserializes_the_used_chat_fields() {
        let chat: Chat = serde_json::from_value(json!({
            "@type": "chat",
            "id": -1_003_743_724_869_i64,
            "type": {
                "@type": "chatTypeSupergroup",
                "supergroup_id": 3_743_724_869_i64,
                "is_channel": true
            },
            "title": "Telewave Radio ✦",
            "photo": null,
            "positions": [
                {
                    "@type": "chatPosition",
                    "list": { "@type": "chatListMain" },
                    "order": 100,
                    "is_pinned": false
                }
            ],
            "chat_lists": [ { "@type": "chatListMain" } ],
            "is_marked_as_unread": false
        }))
        .expect("chat should deserialize");

        assert_eq!(chat.id, -1_003_743_724_869);
        assert_eq!(chat.title, "Telewave Radio ✦");
        assert_eq!(
            chat.kind,
            ChatType::Supergroup {
                supergroup_id: 3_743_724_869,
                is_channel: true
            }
        );
        assert_eq!(chat.chat_lists.len(), 1);
    }

    #[test]
    fn deserializes_a_chat_that_is_not_a_supergroup() {
        let chat: Chat = serde_json::from_value(json!({
            "@type": "chat",
            "id": 123,
            "type": {
                "@type": "chatTypePrivate",
                "user_id": 456
            },
            "title": "Some Guy",
            "chat_lists": []
        }))
        .expect("chat should deserialize");

        assert_eq!(chat.kind, ChatType::Other);
        assert!(chat.chat_lists.is_empty());
    }

    #[test]
    fn converts_between_chat_and_supergroup_ids() {
        assert_eq!(supergroup_to_chat_id(3_743_724_869), -1_003_743_724_869);
        assert_eq!(chat_to_supergroup_id(-1_003_743_724_869), 3_743_724_869);
        assert_eq!(
            chat_to_supergroup_id(supergroup_to_chat_id(123_456_789)),
            123_456_789
        );
    }

    #[test]
    fn distinguishes_supergroup_chat_ids_from_other_chats() {
        assert!(is_supergroup_chat_id(-1_003_743_724_869));
        assert!(!is_supergroup_chat_id(-123_456)); // basic group
        assert!(!is_supergroup_chat_id(123_456)); // user
    }

    #[test]
    fn converts_unix_seconds_to_system_time() {
        assert_eq!(unix_to_system_time(0), UNIX_EPOCH);
        assert_eq!(
            unix_to_system_time(1_781_900_560),
            UNIX_EPOCH + Duration::from_secs(1_781_900_560)
        );
        assert_eq!(unix_to_system_time_opt(0), None);
        assert_eq!(
            unix_to_system_time_opt(1_781_955_266),
            Some(UNIX_EPOCH + Duration::from_secs(1_781_955_266))
        );
    }
}
