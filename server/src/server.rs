use std::error::Error;

use clap::Parser;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

use crate::settings::Settings;
use crate::tslm::server::Builder;

mod settings;
mod tslm;

#[derive(Parser, Debug)]
struct Arguments {
    /// Path to the server configuration directory.
    #[arg(short, long, default_value = "./config")]
    config_dir: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::parse();
    let settings = Settings::load(arguments.config_dir);

    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))?;

    let log_handle = log4rs::init_config(config)?;

    let server = Builder::build_and_run(settings)?;
    server.await_termination()?;

    drop(log_handle);
    Ok(())
}
