use std::sync::Arc;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::SinkExt;
use log::info;
use crate::tslm::hub::Directory;

pub type EndpointId = u64;

#[derive(Debug)]
pub enum TerminalStreamCommand {
    Text(String)
}

#[derive(Debug)]
pub enum ClientCommand {
    Text(String)
}

pub struct Endpoint {
    id: EndpointId,
    tx: UnboundedSender<ClientCommand>,
    directory: Arc<Directory>,
}

impl Endpoint {

    pub fn new(directory: Arc<Directory>) -> (Self, UnboundedReceiver<ClientCommand>) {
        // messages going out
        let (tx, rx) = futures_channel::mpsc::unbounded();

        (Endpoint {
            id: 1,
            directory,
            tx,
        }, rx)
    }

    // the client has sent a command to process
    pub fn on_command(&self, ts_msg: TerminalStreamCommand) {
        info!("cmd {:?}", ts_msg);

        // subscribe to channel

        // publish message to channel

    }

    // send this command to the client
    pub fn send(&mut self, msg: ClientCommand) {
        &self.tx.send(msg);
    }
}

