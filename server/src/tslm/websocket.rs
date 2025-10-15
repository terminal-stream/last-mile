use std::cell::Cell;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures_util::future::select;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt, pin_mut};
use governor::{Quota, RateLimiter};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{WebSocketStream, accept_async_with_config};
use tracing::{debug, error, info, warn};
use tungstenite::Message;

use common::error::AppError;
use common::message::{ClientCommand, TerminalStreamCommand};

use crate::tslm::endpoint::Endpoint;
use crate::tslm::hub::{EndpointFactorySettings, Hub};

/// Configuration for WebSocket server
#[derive(Clone)]
pub struct WebSocketServerConfig {
    #[allow(dead_code)] // Used for authentication, kept for future implementation
    pub auth_tokens: Option<HashSet<String>>,
    pub max_message_size: usize,
    pub max_frame_size: usize,
    pub max_connections: Option<usize>,
    pub rate_limit_per_second: Option<u32>,
}

pub struct WebsocketServer {
    listener_handle: Cell<JoinHandle<Result<(), AppError>>>,
}

struct ConnectionCounter {
    current: AtomicUsize,
    max: Option<usize>,
}

impl ConnectionCounter {
    fn new(max: Option<usize>) -> Arc<Self> {
        Arc::new(ConnectionCounter {
            current: AtomicUsize::new(0),
            max,
        })
    }

    fn try_increment(&self) -> Result<(), AppError> {
        if let Some(max) = self.max {
            let current = self.current.fetch_add(1, Ordering::SeqCst);
            if current >= max {
                self.current.fetch_sub(1, Ordering::SeqCst);
                return Err(AppError::ConnectionLimitExceeded(format!(
                    "Maximum {} connections reached",
                    max
                )));
            }
        } else {
            self.current.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }

    fn decrement(&self) {
        self.current.fetch_sub(1, Ordering::SeqCst);
    }

    fn count(&self) -> usize {
        self.current.load(Ordering::SeqCst)
    }
}

impl WebsocketServer {
    pub fn new(
        runtime: Arc<Runtime>,
        addr: SocketAddr,
        hub: Arc<Hub>,
        settings: EndpointFactorySettings,
        config: WebSocketServerConfig,
    ) -> Self {
        let handler_rt = Arc::clone(&runtime);
        let listener_handle = Cell::new(runtime.spawn(WebsocketServer::listener(
            addr, hub, handler_rt, settings, config,
        )));
        WebsocketServer { listener_handle }
    }

    async fn listener(
        addr: SocketAddr,
        hub: Arc<Hub>,
        runtime: Arc<Runtime>,
        settings: EndpointFactorySettings,
        config: WebSocketServerConfig,
    ) -> Result<(), AppError> {
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.map_err(AppError::from)?;
        let conn_counter = ConnectionCounter::new(config.max_connections);

        info!(
            "Listener on {} - max connections: {}",
            addr,
            config
                .max_connections
                .map_or("unlimited".to_string(), |m| m.to_string())
        );

        while let Ok((stream, client_addr)) = listener.accept().await {
            // Check connection limit
            if let Err(err) = conn_counter.try_increment() {
                warn!("Connection from {} rejected: {}", client_addr, err);
                continue;
            }

            info!(
                "Connection from {} (total: {})",
                client_addr,
                conn_counter.count()
            );

            // Spawn asap so this does not block accepting other incoming conns.
            match hub.create_endpoint(&settings) {
                Ok((endpoint, tx)) => {
                    let cfg = config.clone();
                    let counter = Arc::clone(&conn_counter);

                    runtime.spawn(async move {
                        WebsocketServer::connection_handler(stream, tx, endpoint, client_addr, cfg)
                            .await;
                        counter.decrement();
                        info!(
                            "Connection from {} closed (total: {})",
                            client_addr,
                            counter.count()
                        );
                    });
                }
                Err(err) => {
                    error!("Error creating endpoint: {}", err);
                    conn_counter.decrement();
                }
            }
        }
        Ok(())
    }

    async fn connection_handler(
        tcp_stream: TcpStream,
        mut ts_receiver: UnboundedReceiver<ClientCommand>,
        endpoint: Arc<Endpoint>,
        client_addr: SocketAddr,
        config: WebSocketServerConfig,
    ) {
        // Create rate limiter if configured
        let rate_limiter = config.rate_limit_per_second.and_then(|rate| {
            NonZeroU32::new(rate).map(|r| Arc::new(RateLimiter::direct(Quota::per_second(r))))
        });

        // Configure WebSocket with size limits
        let ws_config = WebSocketConfig {
            max_message_size: Some(config.max_message_size),
            max_frame_size: Some(config.max_frame_size),
            ..Default::default()
        };

        let ws_stream = accept_async_with_config(tcp_stream, Some(ws_config))
            .await
            .map_err(AppError::from);

        match ws_stream {
            Err(err) => {
                warn!("WebSocket handshake error from {}: {}", client_addr, err);
            }
            Ok(tcp_stream) => {
                info!("Client connected from: {}", client_addr);
                let (mut tx, mut rx) = tcp_stream.split();
                let in_ref = Arc::clone(&endpoint);
                let limiter = rate_limiter.clone();
                let incoming = async move {
                    // stop on none
                    while let Some(Ok(msg)) = rx.next().await {
                        // Apply rate limiting
                        if let Some(ref limiter) = limiter
                            && limiter.check().is_err()
                        {
                            warn!("Rate limit exceeded for connection {}", client_addr);
                            let _ = in_ref
                                .send(ClientCommand::Error("Rate limit exceeded".to_string()));
                            continue;
                        }
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
                // Validate message size (additional check beyond WebSocket config)
                if txt.len() > 1024 * 1024 {
                    // 1MB text message limit
                    warn!("Message too large: {} bytes", txt.len());
                    let _ = endpoint.send(ClientCommand::Error(String::from("Message too large")));
                    return;
                }

                // parse into ts command
                match serde_json::from_str::<TerminalStreamCommand>(txt.as_str()) {
                    Ok(ts_cmd) => Some(ts_cmd),
                    Err(err) => {
                        debug!("Invalid ts message: {}", err);
                        let _ = endpoint.send(ClientCommand::Error(format!(
                            "Invalid message format: {}",
                            err
                        )));
                        None
                    }
                }
            }
            Message::Binary(_) => {
                // We have nothing to do with binary msgs for now.
                debug!("Binary messages not supported");
                let _ = endpoint.send(ClientCommand::Error(String::from(
                    "Binary messages not supported",
                )));
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
