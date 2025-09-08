use bevygap_shared::nats::*;
use tracing_subscriber::{layer::*, util::*};

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    // Start logging to console
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    println!("🧪 Testing NATS retry mechanism with async_nats retry_on_initial_connect");
    
    // Test 1: Invalid host to demonstrate retry behavior
    println!("\n=== Test 1: Invalid host (should retry and fail) ===");
    std::env::set_var("NATS_HOST", "invalid-nats-host.example.com:4222");
    std::env::set_var("NATS_USER", "test");
    std::env::set_var("NATS_PASSWORD", "test");
    std::env::set_var("NATS_INSECURE", "1"); // Skip TLS for testing
    
    match BevygapNats::new_and_connect("bevygap_nats_retry_test").await {
        Ok(_) => println!("✅ Unexpected success - connected to invalid host!"),
        Err(e) => println!("❌ Expected failure: {}", e),
    }
    
    // Test 2: localhost (should work if a NATS server is running, otherwise demonstrate retry)
    println!("\n=== Test 2: localhost connection ===");
    std::env::set_var("NATS_HOST", "localhost:4222");
    
    match BevygapNats::new_and_connect("bevygap_nats_retry_test").await {
        Ok(_) => println!("✅ Successfully connected to localhost!"),
        Err(e) => println!("❌ Failed to connect to localhost: {}", e),
    }
    
    // Test 3: Test host variant fallback (IPv6/IPv4)
    println!("\n=== Test 3: Host variant fallback ===");
    std::env::set_var("NATS_HOST", "127.0.0.1:4222");
    match BevygapNats::new_and_connect("bevygap_nats_retry_test").await {
        Ok(_) => println!("✅ Connected with host fallback!"),
        Err(e) => println!("❌ Failed with host fallback: {}", e),
    }

    println!("\n🏁 async_nats retry mechanism test completed!");
}