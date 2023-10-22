
mod common;
mod web;

use std::sync::Arc;
use tokio::runtime::Runtime;
use crate::tslm::common::error::AppError;
use crate::tslm::web::websocket::WebsocketServer;


pub struct Builder {

}
impl Builder {
    pub fn build() -> Result<LastMileServer, AppError> {
        LastMileServer::new()
    }
}
pub struct LastMileServer {
    runtime: Arc<Runtime>,
}

impl LastMileServer {

    fn new() -> Result<Self, AppError> {

        let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build()?);

        let ws_rt = Arc::clone(&runtime);
        let result = runtime.block_on(async {
            let mut websockets = WebsocketServer::new(ws_rt);
            websockets.await_termination().await;
            ()
        });

        Ok(LastMileServer{
            runtime
        })
    }

}