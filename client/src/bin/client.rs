use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
use last_mile_client::client::LastMileClient;
use tokio::runtime::Builder;
use tokio::time::sleep;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "tslm-client")]
#[command(about = "TSLM Client - Connect to TSLM WebSocket servers", long_about = None)]
struct Cli {
    /// WebSocket server URL
    #[arg(short, long, default_value = "ws://localhost:8080")]
    url: String,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Subscribe to a channel and listen for messages
    Subscribe {
        /// Channel ID to subscribe to
        channel: String,
        /// Duration to listen in seconds (0 = forever)
        #[arg(short, long, default_value = "0")]
        duration: u64,
    },
    /// Create a channel
    CreateChannel {
        /// Channel ID to create
        channel: String,
    },
    /// Publish a message to a channel
    Publish {
        /// Channel ID to publish to
        channel: String,
        /// Message to publish
        message: String,
        /// Number of times to publish (for testing)
        #[arg(short, long, default_value = "1")]
        count: usize,
        /// Interval between messages in milliseconds
        #[arg(short, long, default_value = "1000")]
        interval: u64,
    },
    /// Run interactive test scenario
    Test {
        /// Channel ID to use for testing
        #[arg(short, long, default_value = "test-channel")]
        channel: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let runtime = Arc::new(Builder::new_multi_thread().enable_all().build()?);

    match cli.command {
        Commands::Subscribe { channel, duration } => {
            let url = cli.url.clone();
            let client_rt = Arc::clone(&runtime);

            runtime.block_on(async move {
                println!("Connecting to {}...", url);
                let client = LastMileClient::connect(client_rt, url)?;
                println!("✓ Connected");

                println!("Subscribing to channel '{}'...", channel);
                client.subscribe(&channel)?;
                println!("✓ Subscribed. Listening for messages...");

                if duration > 0 {
                    println!("Will listen for {} seconds", duration);
                    sleep(Duration::from_secs(duration)).await;
                } else {
                    println!("Listening forever (Ctrl+C to stop)...");
                    loop {
                        sleep(Duration::from_secs(3600)).await;
                    }
                }

                Ok::<(), Box<dyn Error>>(())
            })?;
        }

        Commands::CreateChannel { channel } => {
            let url = cli.url.clone();
            let client_rt = Arc::clone(&runtime);

            runtime.block_on(async move {
                println!("Connecting to {}...", url);
                let client = LastMileClient::connect(client_rt, url)?;
                println!("✓ Connected");

                println!("Creating channel '{}'...", channel);
                client.create_channel(&channel)?;
                println!("✓ Channel created");

                Ok::<(), Box<dyn Error>>(())
            })?;
        }

        Commands::Publish {
            channel,
            message,
            count,
            interval,
        } => {
            let url = cli.url.clone();
            let client_rt = Arc::clone(&runtime);

            runtime.block_on(async move {
                println!("Connecting to {}...", url);
                let client = LastMileClient::connect(client_rt, url)?;
                println!("✓ Connected");

                println!("Publishing {} message(s) to '{}'...", count, channel);
                for i in 1..=count {
                    let msg = if count > 1 {
                        format!("{} ({})", message, i)
                    } else {
                        message.clone()
                    };

                    client.notify_channel(&channel, msg.clone())?;
                    println!("  → Published: {}", msg);

                    if i < count {
                        sleep(Duration::from_millis(interval)).await;
                    }
                }

                println!("✓ Done");
                sleep(Duration::from_millis(500)).await; // Allow messages to flush

                Ok::<(), Box<dyn Error>>(())
            })?;
        }

        Commands::Test { channel } => {
            println!("Running test scenario with channel '{}'...\n", channel);

            let url = cli.url.clone();
            let client_rt = Arc::clone(&runtime);

            runtime.block_on(async move {
                println!("1. Connecting to {}...", url);
                let client = LastMileClient::connect(client_rt, url)?;
                println!("✓ Connected\n");

                println!("2. Creating channel '{}'...", channel);
                client.create_channel(&channel)?;
                println!("✓ Channel created\n");

                println!("3. Subscribing to channel...");
                client.subscribe(&channel)?;
                println!("✓ Subscribed\n");

                println!("4. Publishing 3 test messages...");
                for i in 1..=3 {
                    let msg = format!("Test message #{}", i);
                    client.notify_channel(&channel, msg.clone())?;
                    println!("  → Published: {}", msg);
                    sleep(Duration::from_millis(500)).await;
                }

                println!("\n✓ Test completed successfully!");
                println!("Note: Messages are received asynchronously in the background.\n");

                sleep(Duration::from_secs(1)).await;
                Ok::<(), Box<dyn Error>>(())
            })?;
        }
    }

    Ok(())
}
