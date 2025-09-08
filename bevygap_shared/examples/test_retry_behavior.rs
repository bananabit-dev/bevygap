// Test to compare retry behaviors
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("=== Testing RETRY_ON_INITIAL_CONNECT behavior ===");
    
    // Test 1: Without retry_on_initial_connect - should fail fast
    println!("\n1. Testing WITHOUT retry_on_initial_connect:");
    let start = Instant::now();
    
    let opts1 = async_nats::ConnectOptions::new()
        .name("test_no_retry")
        .user_and_password("test".to_string(), "test".to_string())
        .max_reconnects(10)
        .require_tls(false);
        // Note: NO .retry_on_initial_connect() call
    
    let result1 = opts1.connect("nonexistent-host.invalid:4222").await;
    let duration1 = start.elapsed();
    
    println!("  Duration: {:?}", duration1);
    let result1_success = result1.is_ok();
    match result1 {
        Ok(_) => println!("  Result: ✅ Connected (unexpected!)"),
        Err(ref e) => println!("  Result: ❌ Failed: {}", e),
    }
    
    // Test 2: With retry_on_initial_connect - should succeed immediately but retry in background
    println!("\n2. Testing WITH retry_on_initial_connect:");
    let start = Instant::now();
    
    let opts2 = async_nats::ConnectOptions::new()
        .name("test_with_retry")
        .user_and_password("test".to_string(), "test".to_string())
        .max_reconnects(10)
        .require_tls(false)
        .retry_on_initial_connect(); // This should enable background retries
    
    let result2 = opts2.connect("nonexistent-host.invalid:4222").await;
    let duration2 = start.elapsed();
    
    println!("  Duration: {:?}", duration2);
    let result2_success = result2.is_ok();
    match result2 {
        Ok(_) => println!("  Result: ✅ Connected (client will retry in background)"),
        Err(ref e) => println!("  Result: ❌ Failed: {}", e),
    }
    
    println!("\n=== Analysis ===");
    if duration1.as_millis() < 1000 && duration2.as_millis() < 1000 {
        println!("Both connections were fast (< 1s)");
        if !result1_success && result2_success {
            println!("✅ retry_on_initial_connect() is working correctly!");
            println!("   - Without it: connect() fails immediately");
            println!("   - With it: connect() succeeds, retries happen in background");
        } else {
            println!("⚠️ Unexpected behavior - both had same result");
        }
    } else {
        println!("⚠️ One or both connections took longer than expected");
    }
}