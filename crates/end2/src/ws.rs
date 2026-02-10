use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{ChannelInfo, OutboundChatMessage};

#[derive(Clone, Debug, Serialize)]
pub struct MessageId {
    pub message_id: Uuid,
    pub channel_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize)]
pub struct NewNickname {
    pub user_id: Uuid,
    pub nickname: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WsEvent {
    ChannelCreated(ChannelInfo),
    Message(OutboundChatMessage),
    MessageReceived(MessageId),
    NicknameChanged(NewNickname),
}

/// Wraps a `WsEvent` with a monotonic counter for replay detection.
#[derive(Clone, Debug, Serialize)]
pub struct CountedEvent {
    pub counter: u64,
    #[serde(flatten)]
    pub event: WsEvent,
}

/// Client replay request: resend all events after the given counter.
#[derive(Debug, Deserialize)]
pub struct ReplayRequest {
    pub replay: i64,
}
