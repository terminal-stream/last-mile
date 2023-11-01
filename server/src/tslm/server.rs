use std::net::SocketAddr;

use std::sync::Arc;

use crate::settings::Settings;
use common::error::AppError;
use crossbeam::channel::Sender;
use tokio::runtime::Builder as TokioRtBuilder;

use crate::tslm::hub::Hub;
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
        for (_name, listener_config) in settings.listener.iter() {
            let address = SocketAddr::new(listener_config.ip, listener_config.port);
            let listener_rt = Arc::clone(&ws_rt);
            let websocket_listener = WebsocketServer::new(listener_rt, address, Arc::clone(&hub));
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
