use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use log::{debug, error};

use common::error::AppError;
use common::message::{ChannelMessage, ClientCommand};

use crate::tslm::endpoint::{Endpoint, EndpointId};

type ChannelId = common::message::ChannelId;

pub struct Channel {
    pub channel_id: ChannelId,
    subscriptions: RwLock<BTreeMap<EndpointId, Arc<Endpoint>>>,
}

impl Channel {
    pub fn new(channel_id: ChannelId) -> Self {
        Channel {
            channel_id,
            subscriptions: RwLock::new(BTreeMap::default()),
        }
    }

    pub fn subscribe(&self, endpoint: Arc<Endpoint>) -> Result<(), AppError> {
        let mut subscriptions = self.subscriptions.write().map_err(AppError::from)?;
        let _ = subscriptions.insert(endpoint.id, endpoint);
        Ok(())
    }

    pub fn unsubscribe(&self, endpoint_id: &EndpointId) -> Result<(), AppError> {
        let mut subscriptions = self.subscriptions.write().map_err(AppError::from)?;
        self.unsubscribe_guarded(endpoint_id, &mut subscriptions)
    }

    fn unsubscribe_guarded(&self, endpoint_id: &EndpointId, subscriptions: &mut RwLockWriteGuard<BTreeMap<EndpointId, Arc<Endpoint>>>) -> Result<(), AppError> {
        let _ = subscriptions.remove(endpoint_id);
        Ok(())
    }

    pub fn publish(&self, message: ChannelMessage) -> Result<(), AppError> {

        let mut prune = Vec::<EndpointId>::default();
        'fanout: {
            let subscriptions = self.subscriptions.read().map_err(AppError::from)?;
            let client_cmd = ClientCommand::ChannelMessage(self.channel_id.clone(), message);

            for (id, endpoint) in subscriptions.iter() {
                // need to make a copy for each
                match endpoint.send(client_cmd.clone()) {
                    Ok(_) => {
                        debug!("Sent msg correctly.");
                    }
                    Err(_err) => {
                        // On any send error unsubscribe the endpoint from the channel.
                        // TODO
                        //  Design Decision: Should notify hub to unregister the endpoint?
                        //  Not unregistering the endpoint from the hub for now.
                        prune.push(id.clone());
                    }
                }
            }
        }

        'prune: {
            if !prune.is_empty() {
                let mut subs = self.subscriptions.write().map_err(AppError::from)?;
                prune.into_iter().for_each(|endpoint_id| {
                    match self.unsubscribe_guarded(&endpoint_id, &mut subs) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error while unsubscribing endpoint {}, {}", endpoint_id, err);
                        }
                    }
                });
            }
        }
        Ok(())
    }
}
