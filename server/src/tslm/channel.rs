use std::sync::{Arc, RwLock};

use crate::tslm::endpoint::Endpoint;
use common::error::AppError;
use common::message::{ChannelMessage, ClientCommand};
use log::{debug, error};

type ChannelId = common::message::ChannelId;

pub struct Channel {
    pub channel_id: ChannelId,
    subscriptions: RwLock<Vec<Arc<Endpoint>>>,
}

impl Channel {
    pub fn new(channel_id: ChannelId) -> Self {
        Channel {
            channel_id,
            subscriptions: RwLock::new(Vec::default()),
        }
    }

    pub fn subscribe(&self, endpoint: Arc<Endpoint>) -> Result<(), AppError> {
        let mut subscriptions = self.subscriptions.write().map_err(AppError::from)?;
        subscriptions.push(endpoint);
        Ok(())
    }

    pub fn publish(&self, message: ChannelMessage) -> Result<(), AppError> {
        let subscriptions = self.subscriptions.read().map_err(AppError::from)?;

        let client_cmd = match message {
            ChannelMessage::Text(txt) => {
                ClientCommand::ChannelMessage(self.channel_id.clone(), txt)
            }
        };
        for endpoint in subscriptions.iter() {
            // need to make a copy for each
            match endpoint.send(client_cmd.clone()) {
                Ok(_) => {
                    debug!("Sent msg correctly.");
                }
                Err(err) => {
                    error!("Error sending channel message: {}", err);
                }
            }
        }
        Ok(())
    }
}
