use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;

use common::error::AppError;
use common::message::ClientCommand;

use crate::settings::Permission;
use crate::tslm::directory::Directory;
use crate::tslm::endpoint::Endpoint;

#[derive(Default)]
pub struct EndpointFactorySettings {
    pub default_endpoint_permissions: HashSet<Permission>,
}

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
        let (endpoint, rx) =
            Endpoint::new(self.endpoint_id_seq.next(), directory, allowed_commands);
        self.directory.register_endpoint(Arc::clone(&endpoint))?;
        Ok((endpoint, rx))
    }
}
