use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use log::{debug, error};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::tslm::endpoint::{ChannelMessage, ClientCommand, Endpoint, EndpointId};
use crate::tslm::error::AppError;

pub type ChannelId = String;

pub struct Sequence {
    gen: AtomicU64,
}

impl Sequence {
    pub fn new() -> Self {
        Sequence {
            gen: AtomicU64::default(),
        }
    }
    pub fn next(&self) -> u64 {
        self.gen.fetch_add(1, Ordering::SeqCst)
    }
}

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

pub struct Directory {
    channels_by_id: RwLock<HashMap<ChannelId, Arc<Channel>>>,
    endpoints_by_id: RwLock<HashMap<EndpointId, Arc<Endpoint>>>,
}

impl Directory {
    pub fn new() -> Self {
        Directory {
            channels_by_id: RwLock::new(HashMap::default()),
            endpoints_by_id: RwLock::new(HashMap::default()),
        }
    }

    pub fn register_endpoint(&self, endpoint: Arc<Endpoint>) -> Result<(), AppError> {
        let mut endpoints = self.endpoints_by_id.write().map_err(AppError::from)?;
        // would be weird if this was registered twice
        if endpoints.contains_key(&endpoint.id) {
            return Err(AppError::msg_str(
                "Endpoint already registered under given id.",
            ));
        }
        let _ = endpoints.insert(endpoint.id, endpoint);
        Ok(())
    }

    pub fn find_endpoint(&self, endpoint_id: &EndpointId) -> Option<Arc<Endpoint>> {
        let endpoints = self.endpoints_by_id.read().map_err(AppError::from).ok()?;
        endpoints.get(endpoint_id).map(Arc::clone)
    }

    /// Create a channel.
    /// Does not subscribe, only creates.
    /// Endpoints do not need to be subscribed to publish messages to the channel.
    pub fn create_channel(&self, channel_id: ChannelId) -> Result<(), AppError> {
        let mut channels = self.channels_by_id.write().map_err(AppError::from)?;
        if channels.contains_key(&channel_id) {
            return Err(AppError::msg_str("Channel id is already registered."));
        }
        let channel = Arc::new(Channel::new(channel_id));
        channels.insert(channel.channel_id.clone(), channel);
        Ok(())
    }

    pub fn find_channel(&self, channel_id: &ChannelId) -> Option<Arc<Channel>> {
        let channels = self.channels_by_id.read().map_err(AppError::from).ok()?;
        channels.get(channel_id).map(Arc::clone)
    }

    /// Subscribe the endpoint to the given channel id.
    pub fn subscribe_to_channel(
        &self,
        channel_id: &ChannelId,
        endpoint: Arc<Endpoint>,
    ) -> Result<(), AppError> {
        let channel = self
            .find_channel(channel_id)
            .ok_or(AppError::msg_str("No such channel"))?;
        channel.subscribe(endpoint)
    }
}

pub struct Hub {
    endpoint_id_seq: Sequence,
    directory: Arc<Directory>,
}

impl Hub {
    pub fn new() -> Self {
        let directory = Arc::new(Directory::new());
        Hub {
            endpoint_id_seq: Sequence::new(),
            directory,
        }
    }

    pub fn create_endpoint(
        &self,
    ) -> Result<(Arc<Endpoint>, UnboundedReceiver<ClientCommand>), AppError> {
        let directory = Arc::clone(&self.directory);
        let (endpoint, rx) = Endpoint::new(self.endpoint_id_seq.next(), directory);
        self.directory.register_endpoint(Arc::clone(&endpoint))?;
        Ok((endpoint, rx))
    }
}
