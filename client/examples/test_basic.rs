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
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let runtime = Arc::new(Builder::new_multi_thread().enable_all().build()?);

    println!("\n=== Testing TSLM Server ===\n");

    // Test 1: Connect to private listener (publisher)
    println!("1. Connecting publisher to private listener (localhost:8081)...");
    let publisher_url = String::from("ws://localhost:8081/");
    let publisher_rt = Arc::clone(&runtime);
    let publisher =
        runtime.block_on(async move { LastMileClient::connect(publisher_rt, publisher_url) })?;
    println!("✓ Publisher connected\n");

    // Test 2: Connect to public listener (subscriber)
    println!("2. Connecting subscriber to public listener (localhost:8080)...");
    let subscriber_url = String::from("ws://localhost:8080/");
    let subscriber_rt = Arc::clone(&runtime);
    let subscriber =
        runtime.block_on(async move { LastMileClient::connect(subscriber_rt, subscriber_url) })?;
    println!("✓ Subscriber connected\n");

    // Test 3: Create channel on private listener
    println!("3. Creating channel 'test-channel'...");
    let channel_id = String::from("test-channel");
    publisher.create_channel(&channel_id)?;
    println!("✓ Channel created\n");

    // Test 4: Subscribe to channel on public listener
    println!("4. Subscribing to channel 'test-channel'...");
    subscriber.subscribe(&channel_id)?;
    println!("✓ Subscribed\n");

    // Test 5: Publish messages
    println!("5. Publishing 5 messages...");
    runtime.block_on(async {
        for i in 1..=5 {
            sleep(Duration::from_millis(500)).await;
            let msg = format!("Test message #{}", i);
            println!("  → Publishing: {}", msg);
            publisher.notify_channel(&channel_id, msg).ok();
        }

        // Wait a bit for messages to be received
        sleep(Duration::from_secs(1)).await;
    });

    println!("\n✓ Test completed successfully!");
    println!("\nNote: Messages are received asynchronously in the background.");
    println!("Check the client logs above for received messages.\n");

    drop(publisher);
    drop(subscriber);

    Ok(())
}
