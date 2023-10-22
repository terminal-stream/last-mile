use std::sync::Arc;

use crossbeam::channel::Sender;
use tokio::runtime::Runtime;

use crate::tslm::common::error::AppError;
use crate::tslm::web::websocket::WebsocketServer;

mod common;
mod web;

pub struct Builder {

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

        let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build()?);

        let ws_rt = Arc::clone(&runtime);
        let mut websockets = WebsocketServer::new(ws_rt);

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
        Ok(self.tx.send(Command::AwaitTermination).map_err(|_err| AppError::msg_str("Could not send command."))?)
   }

}