use std::sync::Arc;
use futures_channel::mpsc::UnboundedReceiver;
use crate::tslm::endpoint::{ClientCommand, Endpoint};

pub struct Directory {

}

impl Directory {
    pub fn new() -> Self {
        Directory{}
    }
}

pub struct Hub {
    directory: Arc<Directory>
}

impl Hub {

    pub fn new() -> Self {

        let directory = Arc::new(Directory::new());
        Hub{
            directory
        }
    }

    pub fn create_endpoint(&self) -> (Arc<Endpoint>, UnboundedReceiver<ClientCommand>) {
        let directory = Arc::clone(&self.directory);
        let (endpoint, tx) = Endpoint::new(directory);
        (Arc::new(endpoint), tx)
    }
}

