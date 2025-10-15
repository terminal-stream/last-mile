//! # TSLM Client
//!
//! A WebSocket client library for connecting to TSLM servers.
//!
//! ## Example
//!
//! ```no_run
//! use last_mile_client::client::LastMileClient;
//! use std::sync::Arc;
//! use tokio::runtime::Builder;
//!
//! let runtime = Arc::new(Builder::new_multi_thread().enable_all().build().unwrap());
//! let client = LastMileClient::connect(runtime, "ws://localhost:8080".to_string()).unwrap();
//!
//! let channel_id = "my_channel".to_string();
//! client.create_channel(&channel_id).unwrap();
//! client.subscribe(&channel_id).unwrap();
//! client.notify_channel(&channel_id, "Hello, world!".to_string()).unwrap();
//! ```

pub mod client;
mod websocket;
