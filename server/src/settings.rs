use std::net::IpAddr;
use std::path::PathBuf;

use config::{File, Map};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ListenerConfig {
    pub ip: IpAddr,
    pub port: u16,
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
        let settings = config
            .try_deserialize()
            .expect("Error parsing configuration.");
        settings
    }
}
