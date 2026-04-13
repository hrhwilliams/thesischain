use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{ChannelId, ChannelInfo, MessageId, OutboundChatMessage, UserId};

#[derive(Clone, Debug, Serialize)]
pub struct MessageReceipt {
    pub message_id: MessageId,
    pub channel_id: ChannelId,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize)]
pub struct NewNickname {
    pub user_id: UserId,
    pub nickname: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WsEvent {
    ChannelCreated(ChannelInfo),
    Message(OutboundChatMessage),
    MessageReceived(MessageReceipt),
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
