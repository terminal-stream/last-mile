use std::error::Error;

use clap::Parser;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use last_mile_server::settings::Settings;
use last_mile_server::tslm::server::Builder;

#[derive(Parser, Debug)]
struct Arguments {
    /// Path to the server configuration directory.
    #[arg(short, long, default_value = "./config")]
    config_dir: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::parse();
    let settings = Settings::load(arguments.config_dir)?;

    // Initialize tracing subscriber with default INFO level or from RUST_LOG env
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting TSLM server...");
    let server = Builder::build_and_run(settings)?;

    tracing::info!("Server running. Press Ctrl+C to shutdown gracefully.");
    server.await_termination()?;

    tracing::info!("Server shutdown complete.");
    Ok(())
}
