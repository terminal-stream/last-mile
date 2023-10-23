use std::sync::{Arc, Mutex};

// use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{SinkExt, TryFutureExt};
use log::{error, info};
use serde::Deserialize;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::tslm::common::error::AppError;
use crate::tslm::hub::Directory;

pub type EndpointId = u64;
pub type ChannelId = String;

#[derive(Debug, Deserialize, Clone)]
pub enum ChannelMessage {
   // text primitive, not very useful, just for debugging
   Text(String),
}

#[derive(Debug, Deserialize)]
pub enum TerminalStreamCommand {
    CreateChannel(ChannelId),
    Subscribe(ChannelId),
    NotifyChannel(ChannelId, ChannelMessage)
}

#[derive(Debug)]
pub enum ClientCommand {
    // text primitive, not very useful, just for debugging
    Text(String)
}

pub struct Endpoint {
    pub id: EndpointId,
    tx: UnboundedSender<ClientCommand>,
    directory: Arc<Directory>,
}

impl Endpoint {

    pub fn new(id: EndpointId, directory: Arc<Directory>) -> (Arc<Self>, UnboundedReceiver<ClientCommand>) {
        // messages going out
        // let (tx, rx) = futures_channel::mpsc::unbounded();
        let (tx, rx) = unbounded_channel::<ClientCommand>();
        // let tx = Mutex::new(tx);
        let endpoint = Arc::new(Endpoint {
            id,
            tx,
            directory,
        });
        (endpoint, rx)
    }

    // the client has sent a command to process
    pub fn on_command(&self, cmd: TerminalStreamCommand) -> Result<(), AppError> {
        // debug!("cmd {:?}", ts_msg);
        match cmd {
            TerminalStreamCommand::CreateChannel(channel_id) => {
                self.directory.create_channel(channel_id)
            }
            TerminalStreamCommand::Subscribe(channel_id) => {
                self.subscribe(&channel_id)
            }
            TerminalStreamCommand::NotifyChannel(channel_id, msg) => {
                self.notify_channel(&channel_id, &msg)
            }
        }
    }

    fn notify_channel(&self, channel_id: &ChannelId, msg: &ChannelMessage) -> Result<(), AppError> {
        let channel = self.directory.find_channel(channel_id).ok_or(AppError::msg_str("Channel not found."))?;
        channel.publish(msg.clone())
    }

    fn subscribe(&self, channel_id: &ChannelId) -> Result<(), AppError> {
        let self_reference = self.directory.find_endpoint(&self.id)
            .ok_or(AppError::msg_str("Self reference not found!!"))?;
        self.directory.subscribe_to_channel(&channel_id, self_reference)
    }

    // send this command to the client
    pub fn send(&self, msg: ClientCommand) -> Result<(), AppError> {
        let mut tx = self.tx.lock().map_err(AppError::from)?;
        let a = tx.send(msg).unwrap_or_else(|a| {
            // todo unmount endpoint
            info!("Error");
        });
        info!("sent!!");
        Ok(())
    }
}

