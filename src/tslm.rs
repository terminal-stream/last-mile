mod common;
mod hub;
mod web;
mod endpoint;

use std::sync::Arc;

use crossbeam::channel::Sender;
use tokio::runtime::Runtime;

use crate::tslm::common::error::AppError;
use crate::tslm::hub::Hub;
use crate::tslm::web::websocket::WebsocketServer;

pub struct Builder {
    // TODO: builder
}

impl Builder {
    pub fn build_and_run() -> Result<LastMileServer, AppError> {
        LastMileServer::run()
    }
}

enum Command {
    AwaitTermination
}

pub struct LastMileServer {
    runtime: Arc<Runtime>,
    tx: Sender<Command>,
}

impl LastMileServer {

    fn run() -> Result<Self, AppError> {

        let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(AppError::from)?);

        let hub = Arc::new(Hub::new());

        let ws_rt = Arc::clone(&runtime);

        let mut websockets = WebsocketServer::new(ws_rt, Arc::clone(&hub));

        let (tx, rx) = crossbeam::channel::unbounded::<Command>();

        runtime.block_on(async {
            while let Ok(cmd) = rx.recv() {
                match cmd {
                    Command::AwaitTermination => {
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
        Ok(self.tx.send(Command::AwaitTermination).map_err(AppError::from)?)
   }

}