use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use last_mile_client::client::LastMileClient;
use tokio::runtime::Builder;
use tokio::time::sleep;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let runtime = Arc::new(Builder::new_multi_thread().enable_all().build()?);
    let url = String::from("ws://localhost:8080/");

    let ws_rt = Arc::clone(&runtime);
    let _r: Result<(), Box<dyn Error>> = runtime.block_on(async move {
        let client = LastMileClient::connect(ws_rt, url)?;

        let channel_id = String::from("some_channel");
        client.create_channel(&channel_id)?;
        client.subscribe(&channel_id)?;

        for _ in 1..100 {
            sleep(Duration::from_secs(1)).await;
            client.notify_channel(&channel_id, String::from("a message"))?;
        }

        drop(client);
        Ok(())
    });

    Ok(())
}
