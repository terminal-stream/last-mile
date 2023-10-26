use std::cell::Cell;
use std::sync::Arc;

use crate::tslm::endpoint::Endpoint;
use common::error::AppError;
use common::message::{ClientCommand, TerminalStreamCommand};
use futures_util::future::select;
use futures_util::stream::SplitSink;
use futures_util::{pin_mut, SinkExt, StreamExt};
use log::{debug, error, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tungstenite::Message;

use crate::tslm::hub::Hub;

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
        while let Ok((stream, _addr)) = listener.accept().await {
            // Spawn asap so this does not block accepting other incoming conns.
            if let Ok((endpoint, tx)) = hub.create_endpoint() {
                runtime.spawn(WebsocketServer::connection_handler(stream, tx, endpoint));
            } // else its done
        }
        Ok(())
    }

    async fn connection_handler(
        tcp_stream: TcpStream,
        mut ts_receiver: UnboundedReceiver<ClientCommand>,
        endpoint: Arc<Endpoint>,
    ) {
        let ws_stream = accept_async(tcp_stream).await.map_err(AppError::from);
        match ws_stream {
            Err(err) => {
                warn!("Ws stream produced an error during the handshake: {}", err);
            }
            Ok(tcp_stream) => {
                let (mut tx, mut rx) = tcp_stream.split();
                let incoming = async move {
                    // stop on none
                    while let Some(Ok(msg)) = rx.next().await {
                        Self::handle_incoming_message(&endpoint, msg);
                    }
                };

                let outgoing = async move {
                    while let Some(msg) = ts_receiver.recv().await {
                        match Self::handle_outgoing_message(&mut tx, msg).await {
                            Ok(_) => {
                                // nothing for now.
                            }
                            Err(err) => {
                                error!("Error processing outgoing message: {:?}", err);
                                break;
                            }
                        }
                    }
                };

                // either a msg incoming from tcp/websocket or from client channel going to the socket
                pin_mut!(incoming, outgoing);
                select(incoming, outgoing).await;
            }
        };
    }

    async fn handle_outgoing_message(
        tx: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
        cmd: ClientCommand,
    ) -> Result<(), AppError> {
        let json_string_command = serde_json::to_string(&cmd).map_err(AppError::from)?;
        tx.send(Message::Text(json_string_command))
            .await
            .map_err(AppError::from)
    }

    fn handle_incoming_message(endpoint: &Endpoint, msg: Message) {
        let ts_msg = match msg {
            Message::Ping(_) | Message::Pong(_) => {
                // Tungstenite takes care of pings, we just get notified, nothing to do.
                None
            }
            Message::Text(txt) => {
                // parse into ts command
                debug!("Received ws msg {}", txt);
                match serde_json::from_str::<TerminalStreamCommand>(txt.as_str()) {
                    Ok(ts_cmd) => Some(ts_cmd),
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
    }

    pub async fn await_termination(&mut self) {
        let _result = self.listener_handle.get_mut().await;
    }
}
