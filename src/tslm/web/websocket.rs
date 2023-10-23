use std::cell::Cell;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{future, pin_mut, SinkExt, StreamExt, TryStreamExt};
use log::{debug, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tungstenite::Message;

use crate::tslm::common::error::AppError;
use crate::tslm::endpoint::{ClientCommand, Endpoint, TerminalStreamCommand};
use crate::tslm::Hub;

pub struct WebsocketEndpoint {
    addr: SocketAddr,
    ws_stream: WebSocketStream<TcpStream>,
}

impl WebsocketEndpoint {
    pub async fn handshake(stream: TcpStream, addr: SocketAddr) -> Result<Self, AppError> {
        let ws_stream = accept_async(stream).await.map_err(AppError::from)?;
        Ok(WebsocketEndpoint { ws_stream, addr })
    }
}

pub struct WebsocketServer {
    listener_handle: Cell<JoinHandle<Result<(), AppError>>>,
}

impl WebsocketServer {
    pub fn new(runtime: Arc<Runtime>, hub: Arc<Hub>) -> Self {
        let addr = "127.0.0.1:8080".to_string();
        let handler_rt = Arc::clone(&runtime);
        let listener_handle =
            Cell::new(runtime.spawn(WebsocketServer::listener(addr, hub, handler_rt)));
        WebsocketServer { listener_handle }
    }

    async fn listener(addr: String, hub: Arc<Hub>, runtime: Arc<Runtime>) -> Result<(), AppError> {
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.map_err(AppError::from)?;
        while let Ok((mut stream, addr)) = listener.accept().await {
            // Spawn asap so this does not block accepting other incoming conns.
            if let Ok((endpoint, tx)) = hub.create_endpoint() {
                runtime.spawn(WebsocketServer::connection_handler(stream, tx, endpoint));
            } // else its done
        }
        Ok(())
    }

    async fn connection_handler(
        tcp_stream: TcpStream,
        // ts_receiver: futures_channel::mpsc::UnboundedReceiver<ClientCommand>,
        ts_receiver: UnboundedReceiver<ClientCommand>,
        endpoint: Arc<Endpoint>,
    ) {
        let mut ws_stream = accept_async(tcp_stream).await.map_err(AppError::from);
        match ws_stream {
            Ok(tcp_stream) => {
                let (mut tx, rx) = tcp_stream.split();

                // decode/map into a terminal stream message.
                let handle_incoming = rx.try_for_each(|msg: Message| {
                    let ts_msg = match msg {
                        Message::Ping(_) | Message::Pong(_) => {
                            // Tungstenite takes care of pings, we just get notified, nothing to do.
                            None
                        }
                        Message::Text(txt) => {
                            // parse into ts command
                            debug!("Received ws msg {}", txt);
                            match serde_json::from_str::<TerminalStreamCommand>(txt.as_str()) {
                                Ok(ts_cmd) => {
                                    Some(ts_cmd)
                                }
                                Err(err) => {
                                    // send error msg to client?
                                    debug!("Invalid ts message {}", err);
                                    None
                                }
                            }
                        }
                        Message::Binary(_) => {
                            // We have nothing to do with binary msgs for now.
                            None
                        }
                        Message::Frame(_) => {
                            // We have nothing to do with frame msgs.
                            None
                        }
                        Message::Close(_) => {
                            // The tungstenite implementation already takes care or replying with
                            // the close protocol properly. We get notified of the close but nothing
                            // expected for us to do with the websocket.
                            // Notify the stream.
                            // Some(TsMessage::Close)
                            None
                        }
                    };

                    match ts_msg {
                        Some(msg) => {
                            match endpoint.on_command(msg) {
                                Ok(_) => {
                                    // send ack to the client?
                                    debug!("Ack.");
                                }
                                Err(err) => {
                                    // send the error to the client?
                                    debug!("Error: {}", err);
                                }
                            }

                        }
                        None => {
                            // this resulted in an impl protocol message that does not need to be sent to the endpoint
                        }
                    };

                    future::ok(())
                });

                // let handle_outgoing = ts_receiver.map(Ok).forward(tx);

                // let handle_outgoing = tx.send(Message::Text("hola".to_string()));
                let handle_outgoing = ts_receiver.map(|m| {
                   Ok(Message::Text("Hola".to_string()))
                }).forward(tx);

                /*
                // handle processing messages going to the websocket
                let handle_outgoing = ts_receiver
                    .map(|ts_msg| match ts_msg {
                        ClientCommand::Text(msg) => {
                            // debug!("sending to ws: {}", msg);
                            Ok(Message::text(msg))
                            // future::ok(Message::Text(msg))
                        }
                    })
                    .forward(tx);

                 */

                pin_mut!(handle_incoming, handle_outgoing);
                future::select(handle_incoming, handle_outgoing).await;
            }
            Err(err) => {
                warn!("Ws stream produced an error during the handshake: {}", err);
            }
        };
    }

    pub async fn await_termination(&mut self) {
        let _result = self.listener_handle.get_mut().await;
    }
}
