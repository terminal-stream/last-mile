mod tslm;

use std::error::Error;

use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use crate::tslm::tslm::Builder;


fn main() -> Result<(), Box<dyn Error>> {
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))?;

    let _log_handle = log4rs::init_config(config)?;

    info!("Log configuration OK.");

    let server = Builder::build_and_run()?;
    server.await_termination()?;
    Ok(())
}
