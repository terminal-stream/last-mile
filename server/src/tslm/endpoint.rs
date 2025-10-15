use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::warn;

use common::error::AppError;
use common::message::{ChannelId, ChannelMessage, ClientCommand, TerminalStreamCommand};

use crate::settings::Permission;
use crate::tslm::directory::Directory;

pub type EndpointId = u64;

pub struct Endpoint {
    pub id: EndpointId,
    directory: Arc<Directory>,
    tx: UnboundedSender<ClientCommand>,
    allowed_commands: HashSet<Permission>,
}

impl Endpoint {
    pub fn new(
        id: EndpointId,
        directory: Arc<Directory>,
        allowed_commands: HashSet<Permission>,
    ) -> (Arc<Endpoint>, UnboundedReceiver<ClientCommand>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let endpoint = Arc::new(Endpoint {
            id,
            tx,
            directory,
            allowed_commands,
        });
        (endpoint, rx)
    }

    /// Create an endpoint with a bounded channel for backpressure control
    pub fn new_bounded(
        id: EndpointId,
        directory: Arc<Directory>,
        allowed_commands: HashSet<Permission>,
        buffer_size: usize,
    ) -> (Arc<Endpoint>, tokio::sync::mpsc::Receiver<ClientCommand>) {
        let (tx_bounded, rx) = tokio::sync::mpsc::channel(buffer_size);

        // We still use an unbounded sender internally but with async send
        let (tx_internal, mut rx_internal) = tokio::sync::mpsc::unbounded_channel();

        // Spawn a task to forward messages with backpressure
        tokio::spawn(async move {
            while let Some(msg) = rx_internal.recv().await {
                if tx_bounded.send(msg).await.is_err() {
                    break;
                }
            }
        });

        let endpoint = Arc::new(Endpoint {
            id,
            tx: tx_internal,
            directory,
            allowed_commands,
        });
        (endpoint, rx)
    }

    pub fn on_command(&self, cmd: TerminalStreamCommand) -> Result<(), AppError> {
        let result = match cmd {
            TerminalStreamCommand::CreateChannel(ref channel_id) => {
                if self.allowed_commands.contains(&Permission::CreateChannel) {
                    self.directory.create_channel(channel_id.clone())
                } else {
                    warn!(
                        "Endpoint {} attempted to create a channel without permissions.",
                        self.id
                    );
                    Err(AppError::PermissionDenied("CreateChannel".to_string()))
                }
            }
            TerminalStreamCommand::Subscribe(ref channel_id) => {
                if self.allowed_commands.contains(&Permission::Subscribe) {
                    self.subscribe(channel_id)
                } else {
                    warn!(
                        "Endpoint {} attempted to subscribe to a channel without permissions.",
                        self.id
                    );
                    Err(AppError::PermissionDenied("Subscribe".to_string()))
                }
            }
            TerminalStreamCommand::NotifyChannel(ref channel_id, ref msg) => {
                if self.allowed_commands.contains(&Permission::NotifyChannel) {
                    self.notify_channel(channel_id, msg)
                } else {
                    warn!(
                        "Endpoint {} attempted to notify a channel without permissions.",
                        self.id
                    );
                    Err(AppError::PermissionDenied("NotifyChannel".to_string()))
                }
            }
        };

        // Send response back to client
        match result {
            Ok(_) => {
                let _ = self.send(ClientCommand::Success(format!(
                    "Command executed: {:?}",
                    cmd
                )));
            }
            Err(ref err) => {
                let _ = self.send(ClientCommand::Error(err.to_string()));
            }
        }

        result
    }

    fn notify_channel(&self, channel_id: &ChannelId, msg: &ChannelMessage) -> Result<(), AppError> {
        let channel = self
            .directory
            .find_channel(channel_id)
            .ok_or_else(|| AppError::ChannelNotFound(channel_id.clone()))?;
        channel.publish(msg.clone())
    }

    fn subscribe(&self, channel_id: &ChannelId) -> Result<(), AppError> {
        let self_reference = self
            .directory
            .find_endpoint(&self.id)
            .ok_or_else(|| AppError::EndpointNotFound(self.id.to_string()))?;
        self.directory
            .subscribe_to_channel(channel_id, self_reference)
    }

    // send this command to the client
    pub fn send(&self, msg: ClientCommand) -> Result<(), AppError> {
        self.tx
            .send(msg)
            .map_err(|e| AppError::ChannelSend(e.to_string()))
    }

    pub fn unregister(&self) -> Result<(), AppError> {
        self.directory.unregister_endpoint(&self.id)
    }
}
