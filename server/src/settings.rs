use std::collections::HashSet;
use std::net::IpAddr;
use std::path::PathBuf;

use config::{File, Map};
use serde::Deserialize;

use common::error::AppError;

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
    /// Optional authentication tokens. If specified, clients must provide one of these tokens
    /// in the Sec-WebSocket-Protocol header to connect.
    pub auth_tokens: Option<HashSet<String>>,
    /// Maximum message size in bytes (default: 64KB)
    pub max_message_size: Option<usize>,
    /// Maximum frame size in bytes (default: 16MB)
    pub max_frame_size: Option<usize>,
    /// Maximum number of concurrent connections (default: unlimited)
    pub max_connections: Option<usize>,
    /// Channel buffer size for backpressure control (default: unbounded, 0 = unbounded)
    pub channel_buffer_size: Option<usize>,
    /// Rate limit: messages per second per connection (default: no limit)
    pub rate_limit_per_second: Option<u32>,
}

impl ListenerConfig {
    pub fn get_max_message_size(&self) -> usize {
        self.max_message_size.unwrap_or(64 * 1024) // 64KB default
    }

    pub fn get_max_frame_size(&self) -> usize {
        self.max_frame_size.unwrap_or(16 * 1024 * 1024) // 16MB default
    }
}

const DEFAULT_TSLM_FILE_NAME: &str = "tslm";

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub listener: Map<String, ListenerConfig>,
}

impl Settings {
    pub fn load(path: PathBuf) -> Result<Self, AppError> {
        let tslm_path = path.join(DEFAULT_TSLM_FILE_NAME);
        let source = File::from(tslm_path);
        let config = config::Config::builder()
            .add_source(source)
            .build()
            .map_err(|e| AppError::InvalidConfig(format!("Failed to load configuration: {}", e)))?;

        config
            .try_deserialize()
            .map_err(|e| AppError::InvalidConfig(format!("Failed to parse configuration: {}", e)))
    }
}
