use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::channel::Sender;
use tokio::runtime::Builder as TokioRtBuilder;
use tokio::signal;
use tracing::{error, info};

use common::error::AppError;

use crate::settings::Settings;
use crate::tslm::hub::{EndpointFactorySettings, Hub};
use crate::tslm::websocket::{WebSocketServerConfig, WebsocketServer};

pub struct Builder {
    // TODO: builder
}

impl Builder {
    pub fn build_and_run(settings: Settings) -> Result<LastMileServer, AppError> {
        LastMileServer::run(settings)
    }
}

enum ServerCommand {
    AwaitTermination,
}

pub struct LastMileServer {
    tx: Sender<ServerCommand>,
    shutdown_flag: Arc<AtomicBool>,
}

impl LastMileServer {
    fn run(settings: Settings) -> Result<Self, AppError> {
        let runtime = TokioRtBuilder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(AppError::from)?;
        let runtime = Arc::new(runtime);

        let hub = Arc::new(Hub::new());
        let ws_rt = Arc::clone(&runtime);

        let mut listeners = Vec::new();
        for (name, listener_config) in settings.listener.into_iter() {
            let address = SocketAddr::new(listener_config.ip, listener_config.port);
            let listener_rt = Arc::clone(&ws_rt);

            // Extract values before creating structs to avoid partial move issues
            // Extract borrowed values first before moving any fields
            let max_message_size = listener_config.get_max_message_size();
            let max_frame_size = listener_config.get_max_frame_size();
            let channel_buffer_size = listener_config.channel_buffer_size;
            let auth_tokens = listener_config.auth_tokens.clone();
            let max_connections = listener_config.max_connections;
            let rate_limit_per_second = listener_config.rate_limit_per_second;
            // Move this last since unwrap_or_default moves the field
            let default_permissions = listener_config
                .default_endpoint_permissions
                .unwrap_or_default();

            let endpoint_factory_settings = EndpointFactorySettings {
                default_endpoint_permissions: default_permissions,
                channel_buffer_size,
            };

            let ws_config = WebSocketServerConfig {
                auth_tokens,
                max_message_size,
                max_frame_size,
                max_connections,
                rate_limit_per_second,
            };

            let websocket_listener = WebsocketServer::new(
                listener_rt,
                address,
                Arc::clone(&hub),
                endpoint_factory_settings,
                ws_config,
            );
            info!("Running websocket listener '{}' on: {}", name, address);
            listeners.push(websocket_listener);
        }

        let (tx, rx) = crossbeam::channel::unbounded::<ServerCommand>();
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let shutdown_flag_clone = Arc::clone(&shutdown_flag);

        // Spawn signal handler
        runtime.spawn(async move {
            match signal::ctrl_c().await {
                Ok(()) => {
                    info!("Received shutdown signal (Ctrl+C), initiating graceful shutdown...");
                    shutdown_flag_clone.store(true, Ordering::SeqCst);
                }
                Err(err) => {
                    error!("Error setting up signal handler: {}", err);
                }
            }
        });

        runtime.block_on(async {
            if let Ok(cmd) = rx.recv() {
                match cmd {
                    ServerCommand::AwaitTermination => {
                        info!("Shutting down WebSocket listeners...");
                        for mut listener in listeners.into_iter() {
                            listener.await_termination().await;
                        }
                        info!("All listeners terminated.");
                    }
                }
            }
        });

        Ok(LastMileServer { tx, shutdown_flag })
    }

    pub fn await_termination(&self) -> Result<(), AppError> {
        // Wait for shutdown signal
        while !self.shutdown_flag.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        self.tx
            .send(ServerCommand::AwaitTermination)
            .map_err(AppError::from)
    }

    /// Check if shutdown has been requested
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }
}
