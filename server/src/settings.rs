use config::{File, Map};
use log::debug;
use serde::Deserialize;
use std::net::IpAddr;

use common::error::AppError;

#[derive(Deserialize, Debug)]
pub struct ListenerConfig {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub listener: Map<String, ListenerConfig>,
}

impl Settings {
    pub fn load() -> Self {
        let source = File::with_name("config/tslm");
        let config = config::Config::builder()
            .add_source(source)
            .build()
            .map_err(AppError::from)
            .expect("Error initializing configuration");
        let settings = config
            .try_deserialize()
            .expect("Error parsing configuration.");
        debug!("Configuration loaded: {:?}", settings);
        settings
    }
}
