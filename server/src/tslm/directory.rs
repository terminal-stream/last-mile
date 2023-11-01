use crate::tslm::channel::Channel;
use common::error::AppError;
use common::message::ChannelId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::tslm::endpoint::{Endpoint, EndpointId};

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

    pub fn unregister_endpoint(&self, endpoint_id: &EndpointId) -> Result<(), AppError> {
        let mut endpoints = self.endpoints_by_id.write().map_err(AppError::from)?;
        let _ = endpoints.remove(endpoint_id);
        // TODO
        //  Design Decision: unregistering endpoints do not tell channels to unsubscribe the endpoint.
        //  The way it is designed now channels will fail to send a message and de-subscribe the
        //  endpoints on their own.
        //  Also means that a channel can detect a disconnection earlier than the hub.
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
