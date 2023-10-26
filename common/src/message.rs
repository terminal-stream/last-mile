use serde::{Deserialize, Serialize};

pub type ChannelId = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChannelMessage {
    Text(String),
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
    ChannelMessage(ChannelId, String),
}
