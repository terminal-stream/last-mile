use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::mpsc::UnboundedReceiver;

use common::error::AppError;
use common::message::ClientCommand;

use crate::settings::Permission;
use crate::tslm::directory::Directory;
use crate::tslm::endpoint::Endpoint;

#[derive(Default)]
pub struct EndpointFactorySettings {
    pub default_endpoint_permissions: HashSet<Permission>,
    pub channel_buffer_size: Option<usize>,
}

pub struct Sequence {
    counter: AtomicU64,
}

impl Sequence {
    pub fn new() -> Self {
        Sequence {
            counter: AtomicU64::default(),
        }
    }
    pub fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
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
        endpoint_factory_settings: &EndpointFactorySettings,
    ) -> Result<(Arc<Endpoint>, UnboundedReceiver<ClientCommand>), AppError> {
        let directory = Arc::clone(&self.directory);
        let allowed_commands = endpoint_factory_settings
            .default_endpoint_permissions
            .clone();
        let endpoint_id = self.endpoint_id_seq.next();

        let (endpoint, rx) = if let Some(buffer_size) =
            endpoint_factory_settings.channel_buffer_size
        {
            if buffer_size > 0 {
                // Bounded channel - but we need to convert the receiver type
                let (ep, bounded_rx) =
                    Endpoint::new_bounded(endpoint_id, directory, allowed_commands, buffer_size);

                // Convert bounded receiver to unbounded using a forwarding task
                let (tx, unbounded_rx) = tokio::sync::mpsc::unbounded_channel();
                tokio::spawn(async move {
                    let mut bounded = bounded_rx;
                    while let Some(msg) = bounded.recv().await {
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                });
                (ep, unbounded_rx)
            } else {
                Endpoint::new(endpoint_id, directory, allowed_commands)
            }
        } else {
            Endpoint::new(endpoint_id, directory, allowed_commands)
        };

        self.directory.register_endpoint(Arc::clone(&endpoint))?;
        Ok((endpoint, rx))
    }
}
