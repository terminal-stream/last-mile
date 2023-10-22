mod tslm;

use std::error::Error;
use std::thread;
use std::time::Duration;

use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log::{info, LevelFilter};

use tslm::Builder;


fn main() -> Result<(), Box<dyn Error>> {

    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))?;

    let _log_handle = log4rs::init_config(config)?;

    info!("Log configuration OK.");

    let server = Builder::build()?;
    thread::sleep(Duration::from_secs(10));
    Ok(())

}
