mod settings;
mod tslm;

use crate::settings::Settings;
use crate::tslm::server::Builder;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::load();

    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))?;

    let _log_handle = log4rs::init_config(config)?;

    let server = Builder::build_and_run(settings)?;
    server.await_termination()?;
    Ok(())
}
