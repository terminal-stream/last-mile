use std::str::FromStr;
use std::sync::Arc;

use common::error::AppError;
use futures_util::{SinkExt, StreamExt, future, pin_mut};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::error;

pub struct Websocket<H>
where
    H: WebsocketEventHandler + Sync + Send + 'static,
{
    #[allow(dead_code)]
    handle: JoinHandle<Result<(), AppError>>,
    sender: UnboundedSender<Message>,
    #[allow(dead_code)]
    pub handler: Arc<H>,
}

pub trait WebsocketEventHandler {
    fn on_connect(&self);
    fn on_message(&self, message: Message);
    fn on_error(&self, error: AppError);
    fn on_close(&self);
}

impl<H> Websocket<H>
where
    H: WebsocketEventHandler + Sync + Send + 'static,
{
    pub fn open(runtime: &Runtime, url: String, handler: Arc<H>) -> Result<Self, AppError> {
        let uri = Uri::from_str(url.as_str()).map_err(AppError::from)?;

        let (otx, mut orx) = tokio::sync::mpsc::unbounded_channel::<Message>();

        let handler_ref = Arc::clone(&handler);
        let handle: JoinHandle<Result<(), AppError>> = runtime.spawn(async move {
            let (ws_stream, _response) = connect_async(&uri).await.map_err(AppError::from)?;

            handler_ref.on_connect();

            let (mut write, read) = ws_stream.split();

            let send_handler_ref = Arc::clone(&handler_ref);
            let send_fut = async move {
                while let Some(msg) = orx.recv().await {
                    match write.send(msg).await {
                        Ok(_) => {}
                        Err(err) => {
                            // error!("Websocket error while trying to send message.");
                            send_handler_ref.on_error(AppError::from(err));
                        }
                    };
                }
            };

            let recv_handler_ref = Arc::clone(&handler_ref);
            let recv_fut = read.for_each(
                |item: Result<Message, tokio_tungstenite::tungstenite::Error>| async {
                    match item {
                        Ok(message) => {
                            recv_handler_ref.on_message(message);
                        }
                        Err(err) => {
                            // error!("Error reading websocket message: {:?}", err);
                            recv_handler_ref.on_error(AppError::from(err));
                        }
                    }
                },
            );

            pin_mut!(recv_fut, send_fut);
            future::select(recv_fut, send_fut).await;

            handler_ref.on_close();

            Ok(())
        });

        Ok(Websocket {
            handle,
            sender: otx,
            handler,
        })
    }

    pub fn send(&self, message: Message) {
        match self.sender.send(message) {
            Ok(_) => {}
            Err(err) => {
                error!("Error!: {}", err);
            }
        }
    }
}
