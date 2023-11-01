use std::sync::Arc;

use common::error::AppError;
use common::message::{ChannelId, ChannelMessage, ClientCommand, TerminalStreamCommand};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::tslm::directory::Directory;

pub type EndpointId = u64;

pub struct Endpoint {
    pub id: EndpointId,
    directory: Arc<Directory>,
    tx: UnboundedSender<ClientCommand>,
}

impl Endpoint {
    pub fn new(
        id: EndpointId,
        directory: Arc<Directory>,
    ) -> (Arc<Endpoint>, UnboundedReceiver<ClientCommand>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let endpoint = Arc::new(Endpoint { id, tx, directory });
        (endpoint, rx)
    }

    pub fn on_command(&self, cmd: TerminalStreamCommand) -> Result<(), AppError> {
        match cmd {
            TerminalStreamCommand::CreateChannel(channel_id) => {
                self.directory.create_channel(channel_id)?
            }
            TerminalStreamCommand::Subscribe(channel_id) => self.subscribe(&channel_id)?,
            TerminalStreamCommand::NotifyChannel(channel_id, msg) => {
                self.notify_channel(&channel_id, &msg)?
            }
        }
        Ok(())
    }

    fn notify_channel(&self, channel_id: &ChannelId, msg: &ChannelMessage) -> Result<(), AppError> {
        let channel = self
            .directory
            .find_channel(channel_id)
            .ok_or(AppError::msg_str("Channel not found."))?;
        channel.publish(msg.clone())
    }

    fn subscribe(&self, channel_id: &ChannelId) -> Result<(), AppError> {
        let self_reference = self
            .directory
            .find_endpoint(&self.id)
            .ok_or(AppError::msg_str("Self reference not found!!"))?;
        self.directory
            .subscribe_to_channel(channel_id, self_reference)
    }

    // send this command to the client
    pub fn send(&self, msg: ClientCommand) -> Result<(), AppError> {
        self.tx.send(msg).map_err(AppError::from)
    }

    pub fn unregister(&self) -> Result<(), AppError> {
        self.directory.unregister_endpoint(&self.id)
    }
}
