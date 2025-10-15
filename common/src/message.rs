//! Message types for TSLM protocol communication.
//!
//! This module defines the messages exchanged between clients and the server.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Unique identifier for a channel.
pub type ChannelId = String;

/// Messages published to channels.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChannelMessage {
    /// Plain text message
    Text(String),
    /// JSON structured message
    Json(Value),
}

/// Commands sent from clients to the server.
#[derive(Debug, Serialize, Deserialize)]
pub enum TerminalStreamCommand {
    /// Create a new channel with the given ID
    CreateChannel(ChannelId),
    /// Subscribe to receive messages from a channel
    Subscribe(ChannelId),
    /// Publish a message to a channel
    NotifyChannel(ChannelId, ChannelMessage),
}

/// Messages sent from the server to clients.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientCommand {
    /// Text primitive, useful for debugging
    #[allow(dead_code)]
    Text(String),
    /// An incoming message from the given channel
    ChannelMessage(ChannelId, ChannelMessage),
    /// Error response when a command fails
    Error(String),
    /// Success acknowledgment for a command
    Success(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_terminal_stream_command() {
        let cmd = TerminalStreamCommand::CreateChannel(String::from("test"));
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("CreateChannel"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_deserialize_terminal_stream_command() {
        let json = r#"{"CreateChannel":"test_channel"}"#;
        let cmd: TerminalStreamCommand = serde_json::from_str(json).unwrap();

        match cmd {
            TerminalStreamCommand::CreateChannel(id) => assert_eq!(id, "test_channel"),
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_serialize_client_command() {
        let cmd = ClientCommand::Error(String::from("test error"));
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("Error"));
    }

    #[test]
    fn test_channel_message_text() {
        let msg = ChannelMessage::Text(String::from("hello"));
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ChannelMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            ChannelMessage::Text(text) => assert_eq!(text, "hello"),
            _ => panic!("Wrong message type"),
        }
    }
}
