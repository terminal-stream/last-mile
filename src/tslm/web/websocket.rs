use std::cell::Cell;
use std::net::SocketAddr;
use std::sync::Arc;
use log::{error, info};

use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, WebSocketStream};

use crate::tslm::common::error::AppError;
use crate::tslm::{Endpoint, Hub};

pub struct WebsocketEndpoint {
    addr: SocketAddr,
    ws_stream: WebSocketStream<TcpStream>,
}

impl WebsocketEndpoint {
    pub async fn handshake(stream: TcpStream, addr: SocketAddr) -> Result<Self, AppError> {
       let ws_stream  = accept_async(stream).await.map_err(AppError::from)?;
       Ok(WebsocketEndpoint {
           ws_stream,
           addr,
       })
    }
}

impl Endpoint for WebsocketEndpoint {

}

pub struct WebsocketServer {
    listener_handle: Cell<JoinHandle<Result<(), AppError>>>,
}

impl WebsocketServer {


    pub fn new(runtime: Arc<Runtime>, hub: Arc<Hub>) -> Self {

        let addr = "127.0.0.1:8080".to_string();

        let listener_handle = Cell::new(runtime.spawn(WebsocketServer::listener(addr, hub)));

        WebsocketServer {
            listener_handle,
        }
    }

    async fn listener(addr: String, hub: Arc<Hub>) -> Result<(), AppError> {

        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.map_err(AppError::from)?;
        while let Ok((stream, addr)) = listener.accept().await {

            let ws_endpoint = WebsocketEndpoint::handshake(stream, addr).await;

            match ws_endpoint {
                Ok(endpoint) => {
                    hub.register_endpoint(endpoint);
                }
                Err(error) => {
                    info!("Error during websocket handshake: {}", error);
                }
            }

            // tokio::spawn(handle_connection(state.clone(), stream, addr));
            println!("received a connection");
        }
        Ok(())

    }

    pub async fn await_termination(&mut self) {
        let _result = self.listener_handle.get_mut().await;
    }
}