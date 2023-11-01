use std::sync::Arc;

use crate::websocket::{Websocket, WebsocketEventHandler};
use common::error::AppError;
use common::message::{ChannelId, ChannelMessage, TerminalStreamCommand};
use log::{debug, error};
use serde_json::Value;
use tokio::runtime::Runtime;
use tungstenite::Message;

pub struct LastMileClient {
    handler: Arc<LastMileClientHandler>,
    ws: Websocket<LastMileClientHandler>,
}

impl LastMileClient {
    pub fn connect(runtime: Arc<Runtime>, url: String) -> Result<Self, AppError> {
        let handler = Arc::new(LastMileClientHandler {});

        let ws = Websocket::open(&runtime, url, Arc::clone(&handler)).map_err(AppError::from)?;

        Ok(LastMileClient { handler, ws })
    }
    fn send(&self, command: TerminalStreamCommand) -> Result<(), AppError> {
        let message = serde_json::to_string(&command).map_err(AppError::from)?;
        self.ws.send(Message::Text(message));
        Ok(())
    }

    pub fn subscribe(&self, channel_id: &ChannelId) -> Result<(), AppError> {
        let command = TerminalStreamCommand::Subscribe(channel_id.clone());
        self.send(command)
    }

    pub fn create_channel(&self, channel_id: &ChannelId) -> Result<(), AppError> {
        let command = TerminalStreamCommand::CreateChannel(channel_id.clone());
        self.send(command)
    }

    pub fn notify_channel(&self, channel_id: &ChannelId, text: String) -> Result<(), AppError> {
        let command =
            TerminalStreamCommand::NotifyChannel(channel_id.clone(), ChannelMessage::Text(text));
        self.send(command)
    }

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
