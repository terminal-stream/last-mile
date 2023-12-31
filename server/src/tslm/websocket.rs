use std::cell::Cell;
use std::net::SocketAddr;
use std::sync::Arc;

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

use common::error::AppError;
use common::message::{ClientCommand, TerminalStreamCommand};

use crate::tslm::endpoint::Endpoint;
use crate::tslm::hub::{EndpointFactorySettings, Hub};

pub struct WebsocketServer {
    listener_handle: Cell<JoinHandle<Result<(), AppError>>>,
}

impl WebsocketServer {
    pub fn new(
        runtime: Arc<Runtime>,
        addr: SocketAddr,
        hub: Arc<Hub>,
        settings: EndpointFactorySettings,
    ) -> Self {
        let handler_rt = Arc::clone(&runtime);
        let listener_handle =
            Cell::new(runtime.spawn(WebsocketServer::listener(addr, hub, handler_rt, settings)));
        WebsocketServer { listener_handle }
    }

    async fn listener(
        addr: SocketAddr,
        hub: Arc<Hub>,
        runtime: Arc<Runtime>,
        settings: EndpointFactorySettings,
    ) -> Result<(), AppError> {
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.map_err(AppError::from)?;
        while let Ok((stream, _addr)) = listener.accept().await {
            // Spawn asap so this does not block accepting other incoming conns.
            match hub.create_endpoint(&settings) {
                Ok((endpoint, tx)) => {
                    runtime.spawn(WebsocketServer::connection_handler(stream, tx, endpoint));
                }
                Err(err) => {
                    error!("Error: {}", err);
                }
            }
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
                let in_ref = Arc::clone(&endpoint);
                let incoming = async move {
                    // stop on none
                    while let Some(Ok(msg)) = rx.next().await {
                        Self::handle_incoming_message(&in_ref, msg);
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
                // either sending or receiving stopped so unregister the endpoint
                let _ = endpoint.unregister();
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
