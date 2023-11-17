use std::net::SocketAddr;
use std::sync::Arc;

use crossbeam::channel::Sender;
use log::info;
use tokio::runtime::Builder as TokioRtBuilder;

use common::error::AppError;

use crate::settings::Settings;
use crate::tslm::hub::{EndpointFactorySettings, Hub};
use crate::tslm::websocket::WebsocketServer;

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

            // TODO: move to builder when more options are introduced.
            let endpoint_factory_settings = match listener_config.default_endpoint_permissions {
                None => EndpointFactorySettings::default(),
                Some(permissions) => EndpointFactorySettings {
                    default_endpoint_permissions: permissions,
                },
            };
            let websocket_listener = WebsocketServer::new(
                listener_rt,
                address,
                Arc::clone(&hub),
                endpoint_factory_settings,
            );
            info!("Running websocket listener '{}' on: {}", name, address);
            listeners.push(websocket_listener);
        }

        let (tx, rx) = crossbeam::channel::unbounded::<ServerCommand>();

        runtime.block_on(async {
            while let Ok(cmd) = rx.recv() {
                match cmd {
                    ServerCommand::AwaitTermination => {
                        for mut listener in listeners.into_iter() {
                            listener.await_termination().await;
                        }
                        break;
                    }
                };
            }
        });

        Ok(LastMileServer { tx })
    }

    pub fn await_termination(&self) -> Result<(), AppError> {
        self.tx
            .send(ServerCommand::AwaitTermination)
            .map_err(AppError::from)
    }
}
