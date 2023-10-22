use std::cell::Cell;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

async fn listener(addr: String) {

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    while let Ok((stream, addr)) = listener.accept().await {
        // tokio::spawn(handle_connection(state.clone(), stream, addr));
        println!("received a connection");
    }

    ()
}

pub struct WebsocketServer {
    listener_handle: Cell<JoinHandle<()>>,
}
impl WebsocketServer {


    pub fn new(runtime: Arc<Runtime>) -> Self {

        let addr = "127.0.0.1:8080".to_string();

        let listener_handle = Cell::new(runtime.spawn(listener(addr)));

        WebsocketServer {
            listener_handle
        }
    }

    pub async fn await_termination(&mut self) {
        let _result = self.listener_handle.get_mut().await;
    }
}