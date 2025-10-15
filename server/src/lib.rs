//! # TSLM - Terminal Stream Last Mile
//!
//! A WebSocket gateway server that enables secure pub/sub messaging between
//! internal networks and public clients.
//!
//! ## Architecture
//!
//! The server uses a hub-and-spoke model:
//! - **Hub**: Central coordinator that manages endpoints
//! - **Directory**: Registry of channels and endpoints
//! - **Channel**: Pub/sub message distribution
//! - **Endpoint**: Represents a WebSocket connection with permissions
//!
//! ## Example
//!
//! ```no_run
//! use last_mile_server::settings::Settings;
//! use last_mile_server::tslm::server::Builder;
//! use std::path::PathBuf;
//!
//! let settings = Settings::load(PathBuf::from("./config")).unwrap();
//! let server = Builder::build_and_run(settings).unwrap();
//! server.await_termination().unwrap();
//! ```

pub mod settings;
pub mod tslm;
