use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use tracing::error;

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
        let mut subscriptions = self.subscriptions.write()?;
        let _ = subscriptions.insert(endpoint.id, endpoint);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn unsubscribe(&self, endpoint_id: &EndpointId) -> Result<(), AppError> {
        let mut subscriptions = self.subscriptions.write()?;
        self.unsubscribe_guarded(endpoint_id, &mut subscriptions)
    }

    fn unsubscribe_guarded(
        &self,
        endpoint_id: &EndpointId,
        subscriptions: &mut RwLockWriteGuard<BTreeMap<EndpointId, Arc<Endpoint>>>,
    ) -> Result<(), AppError> {
        let _ = subscriptions.remove(endpoint_id);
        Ok(())
    }

    pub fn publish(&self, message: ChannelMessage) -> Result<(), AppError> {
        let mut prune = Vec::<EndpointId>::default();

        // Fan out message to all subscribers
        {
            let subscriptions = self.subscriptions.read()?;
            let client_cmd = ClientCommand::ChannelMessage(self.channel_id.clone(), message);

            for (id, endpoint) in subscriptions.iter() {
                // need to make a copy for each
                match endpoint.send(client_cmd.clone()) {
                    Ok(_) => {
                        // debug!("Sent msg correctly.");
                    }
                    Err(_err) => {
                        // On any send error unsubscribe the endpoint from the channel.
                        // TODO
                        //  Design Decision: Should notify hub to unregister the endpoint?
                        //  Not unregistering the endpoint from the hub for now.
                        prune.push(*id);
                    }
                }
            }
        }

        // Prune disconnected endpoints
        if !prune.is_empty() {
            let mut subs = self.subscriptions.write()?;
            prune.into_iter().for_each(|endpoint_id| {
                match self.unsubscribe_guarded(&endpoint_id, &mut subs) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "Error while unsubscribing endpoint {}, {}",
                            endpoint_id, err
                        );
                    }
                }
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tslm::directory::Directory;
    use crate::tslm::endpoint::Endpoint;
    use std::collections::HashSet;

    #[test]
    fn test_channel_creation() {
        let channel = Channel::new(String::from("test"));
        assert_eq!(channel.channel_id, "test");
    }

    #[test]
    fn test_subscribe_endpoint() {
        let directory = Arc::new(Directory::new());
        let channel = Channel::new(String::from("test_channel"));
        let permissions = HashSet::new();

        let (endpoint, _rx) = Endpoint::new(1, directory, permissions);
        let result = channel.subscribe(endpoint);

        assert!(result.is_ok());
    }

    #[test]
    fn test_publish_message() {
        let directory = Arc::new(Directory::new());
        let channel = Channel::new(String::from("test_channel"));
        let permissions = HashSet::new();

        let (endpoint, mut rx) = Endpoint::new(1, Arc::clone(&directory), permissions);
        directory.register_endpoint(Arc::clone(&endpoint)).unwrap();
        channel.subscribe(endpoint).unwrap();

        let message = ChannelMessage::Text(String::from("test message"));
        let result = channel.publish(message.clone());

        assert!(result.is_ok());

        // Check that message was received
        let received = rx.try_recv();
        assert!(received.is_ok());
    }
}
