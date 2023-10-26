use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use common::error::AppError;
use common::message::ClientCommand;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::tslm::directory::Directory;
use crate::tslm::endpoint::Endpoint;

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
    ) -> Result<(Arc<Endpoint>, UnboundedReceiver<ClientCommand>), AppError> {
        let directory = Arc::clone(&self.directory);
        let (endpoint, rx) = Endpoint::new(self.endpoint_id_seq.next(), directory);
        self.directory.register_endpoint(Arc::clone(&endpoint))?;
        Ok((endpoint, rx))
    }
}
