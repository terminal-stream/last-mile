use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use last_mile_client::client::LastMileClient;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use tokio::runtime::Builder;
use tokio::time::sleep;

pub fn main() -> Result<(), Box<dyn Error>> {
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))?;

    let _log_handle = log4rs::init_config(config)?;

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
