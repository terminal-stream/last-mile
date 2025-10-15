//! TSLM client implementation.

use std::sync::Arc;

use crate::websocket::{Websocket, WebsocketEventHandler};
use common::error::AppError;
use common::message::{ChannelId, ChannelMessage, TerminalStreamCommand};
use serde_json::Value;
use tokio::runtime::Runtime;
use tracing::{debug, error};
use tungstenite::Message;

/// A client for connecting to and interacting with TSLM servers.
///
/// Provides methods for creating channels, subscribing to channels,
/// and publishing messages.
pub struct LastMileClient {
    #[allow(dead_code)]
    handler: Arc<LastMileClientHandler>,
    ws: Websocket<LastMileClientHandler>,
}

impl LastMileClient {
    /// Connect to a TSLM server at the given URL.
    ///
    /// # Arguments
    ///
    /// * `runtime` - Tokio runtime for async operations
    /// * `url` - WebSocket URL (e.g., "ws://localhost:8080")
    ///
    /// # Example
    ///
    /// ```no_run
    /// use last_mile_client::client::LastMileClient;
    /// use std::sync::Arc;
    /// use tokio::runtime::Builder;
    ///
    /// let runtime = Arc::new(Builder::new_multi_thread().enable_all().build().unwrap());
    /// let client = LastMileClient::connect(runtime, "ws://localhost:8080".to_string()).unwrap();
    /// ```
    pub fn connect(runtime: Arc<Runtime>, url: String) -> Result<Self, AppError> {
        let handler = Arc::new(LastMileClientHandler {});

        let ws = Websocket::open(&runtime, url, Arc::clone(&handler)).map_err(AppError::from)?;

        Ok(LastMileClient { handler, ws })
    }
    fn send(&self, command: TerminalStreamCommand) -> Result<(), AppError> {
        let message = serde_json::to_string(&command).map_err(AppError::from)?;
        self.ws.send(Message::Text(message.into()));
        Ok(())
    }

    /// Subscribe to a channel to receive messages.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel to subscribe to
    pub fn subscribe(&self, channel_id: &ChannelId) -> Result<(), AppError> {
        let command = TerminalStreamCommand::Subscribe(channel_id.clone());
        self.send(command)
    }

    /// Create a new channel.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID for the new channel
    pub fn create_channel(&self, channel_id: &ChannelId) -> Result<(), AppError> {
        let command = TerminalStreamCommand::CreateChannel(channel_id.clone());
        self.send(command)
    }

    /// Publish a text message to a channel.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel to publish to
    /// * `text` - The text message to publish
    pub fn notify_channel(&self, channel_id: &ChannelId, text: String) -> Result<(), AppError> {
        let command =
            TerminalStreamCommand::NotifyChannel(channel_id.clone(), ChannelMessage::Text(text));
        self.send(command)
    }

    /// Publish a JSON message to a channel.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel to publish to
    /// * `value` - The JSON value to publish
    pub fn notify_channel_json(
        &self,
        channel_id: &ChannelId,
        value: Value,
    ) -> Result<(), AppError> {
        let command =
            TerminalStreamCommand::NotifyChannel(channel_id.clone(), ChannelMessage::Json(value));
        self.send(command)
    }
}
pub struct LastMileClientHandler {}

impl WebsocketEventHandler for LastMileClientHandler {
    fn on_connect(&self) {
        debug!("TSLM client connected");
    }

    fn on_message(&self, message: Message) {
        debug!("TSLM message: {}", message);
    }

    fn on_error(&self, error: AppError) {
        error!("TSLM client error {}", error);
    }

    fn on_close(&self) {
        debug!("TSLM client closed.");
    }
}
