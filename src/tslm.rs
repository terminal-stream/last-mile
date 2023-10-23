use std::sync::Arc;

use crossbeam::channel::Sender;
use tokio::runtime::{Runtime, Builder as TokioRtBuilder};

use crate::tslm::common::error::AppError;
use crate::tslm::hub::Hub;
use crate::tslm::web::websocket::WebsocketServer;

mod common;
mod hub;
mod web;
mod endpoint;

pub struct Builder {
    // TODO: builder
}

impl Builder {
    pub fn build_and_run() -> Result<LastMileServer, AppError> {
        LastMileServer::run()
    }
}

enum ServerCommand {
    AwaitTermination
}

pub struct LastMileServer {
    runtime: Arc<Runtime>,
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
            while let Ok(cmd) = rx.recv() {
                match cmd {
                    ServerCommand::AwaitTermination => {
                        websockets.await_termination().await;
                        break;
                    }
                };
            }
        });
        Ok(LastMileServer{
            runtime,
            tx,
        })
    }

   pub fn await_termination(&self) -> Result<(), AppError> {
        Ok(self.tx.send(ServerCommand::AwaitTermination).map_err(AppError::from)?)
   }

}