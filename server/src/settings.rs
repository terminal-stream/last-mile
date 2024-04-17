use std::collections::HashSet;
use std::net::IpAddr;
use std::path::PathBuf;

use config::{File, Map};
use serde::Deserialize;

#[derive(Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub enum Permission {
    Subscribe,
    CreateChannel,
    NotifyChannel,
}

#[derive(Deserialize, Debug)]
pub struct ListenerConfig {
    pub ip: IpAddr,
    pub port: u16,
    pub default_endpoint_permissions: Option<HashSet<Permission>>,
}

const DEFAULT_TSLM_FILE_NAME: &str = "tslm";

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub listener: Map<String, ListenerConfig>,
}

impl Settings {
    pub fn load(path: PathBuf) -> Self {
        let tslm_path = path.join(DEFAULT_TSLM_FILE_NAME);
        let source = File::from(tslm_path);
        let config = config::Config::builder()
            .add_source(source)
            .build()
            .expect("Error initializing configuration");

        config
            .try_deserialize()
            .expect("Error parsing configuration.")
    }
}
