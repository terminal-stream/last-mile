use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type ChannelId = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChannelMessage {
    Text(String),
    Json(Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TerminalStreamCommand {
    CreateChannel(ChannelId),
    Subscribe(ChannelId),
    NotifyChannel(ChannelId, ChannelMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientCommand {
    /// text primitive, useful for debugging
    #[allow(dead_code)]
    Text(String),
    /// An incoming message from the given channel
    ChannelMessage(ChannelId, ChannelMessage),
    // ChannelMessageJson(ChannelId, Value),

}
