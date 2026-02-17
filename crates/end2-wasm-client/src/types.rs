use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChannelId(pub String);

#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceId(pub String);

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageId(pub String);

#[derive(Clone, Serialize, Deserialize)]
pub struct UserId(pub String);
