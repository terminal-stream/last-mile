use crate::tslm::endpoint::Endpoint;

pub(crate) struct Hub {

}

impl Hub {
    pub fn new() -> Self {
        Hub{}
    }

    pub fn register_endpoint(&self, endpoint: impl Endpoint) {

    }
}

