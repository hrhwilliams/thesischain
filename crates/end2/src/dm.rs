use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use tokio::sync::{
    RwLock,
    broadcast::{self, error::SendError},
};
use uuid::Uuid;

use crate::UserName;

/// Direct message

#[derive(Clone)]
pub struct DirectMessageLink {
    pub id: RoomId,
    pub user: UserName,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Eq)]
pub struct RoomId(String);

impl RoomId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct Room {
    pub history: Arc<RwLock<Vec<ChatMessage>>>,
    pub sender: broadcast::Sender<ChatMessage>,
}

impl Room {
    pub async fn history(&self) -> Vec<ChatMessage> {
        let history = self.history.read().await;
        history.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChatMessage> {
        self.sender.subscribe()
    }

    pub fn send(&self, message: ChatMessage) -> Result<usize, SendError<ChatMessage>> {
        self.sender.send(message)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub user: UserName,
    pub content: String,
}
