use std::sync::Arc;

use crossbeam::channel::Sender;
use tokio::runtime::Builder as TokioRtBuilder;
use crate::tslm::error::AppError;
use crate::tslm::hub::Hub;
use crate::tslm::websocket::WebsocketServer;

pub struct Builder {
    // TODO: builder
}

impl Builder {
    pub fn build_and_run() -> Result<LastMileServer, AppError> {
        LastMileServer::run()
    }
}

enum ServerCommand {
    AwaitTermination,
}

pub struct LastMileServer {
    tx: Sender<ServerCommand>,
}

impl LastMileServer {
    fn run() -> Result<Self, AppError> {
        let runtime = TokioRtBuilder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(AppError::from)?;
        let runtime = Arc::new(runtime);
        let hub = Arc::new(Hub::new());
        let ws_rt = Arc::clone(&runtime);
        let mut websockets = WebsocketServer::new(ws_rt, Arc::clone(&hub));
        let (tx, rx) = crossbeam::channel::unbounded::<ServerCommand>();
        runtime.block_on(async {
            #[allow(clippy::never_loop)]
            while let Ok(cmd) = rx.recv() {
                match cmd {
                    ServerCommand::AwaitTermination => {
                        websockets.await_termination().await;
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
