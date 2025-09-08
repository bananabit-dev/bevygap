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

    println!("🔐 Bevygap NATS Root CA Certificate Loading Example");
    println!("===================================================");
    println!();
    
    // Check if required environment variables are set
    let missing_vars = check_required_env_vars();
    if !missing_vars.is_empty() {
        print_usage(&missing_vars);
        return;
    }
    
    // Check CA configuration
    let ca_config = check_ca_configuration();
    match ca_config {
        CaConfig::SystemTrustStore => {
            println!("📋 CA Configuration: Using system trust store (suitable for LetsEncrypt/trusted CAs)");
        }
        CaConfig::FilePath(path) => {
            println!("📋 CA Configuration: Using CA certificate file: {}", path);
            if !std::path::Path::new(&path).exists() {
                println!("❌ ERROR: CA certificate file does not exist: {}", path);
                println!("   Please ensure the file exists and is readable.");
                return;
            }
        }
        CaConfig::Contents(len) => {
            println!("📋 CA Configuration: Using CA certificate contents ({} bytes)", len);
        }
        CaConfig::Insecure => {
            println!("⚠️  CA Configuration: TLS verification DISABLED (insecure mode)");
            println!("   This is not recommended for production use!");
        }
    }
    
    println!();
    println!("🔌 Attempting to connect to NATS server...");
    
    // Attempt the connection
    match BevygapNats::new_and_connect("bevygap_ca_example").await {
        Ok(_nats) => {
            println!("✅ SUCCESS: Connected to NATS server with TLS verification!");
            println!("   Your CA certificate configuration is working correctly.");
        }
        Err(e) => {
            println!("❌ FAILED: Could not connect to NATS server");
            println!("   Error: {}", e);
            println!();
            print_troubleshooting_guide(&e);
        }
    }
}

#[derive(Debug)]
enum CaConfig {
    SystemTrustStore,
    FilePath(String),
    Contents(usize),
    Insecure,
}

fn check_required_env_vars() -> Vec<String> {
    let required_vars = ["NATS_HOST", "NATS_USER", "NATS_PASSWORD"];
    let mut missing = Vec::new();
    
    for var in &required_vars {
        if std::env::var(var).is_err() {
            missing.push(var.to_string());
        }
    }
    
    missing
}

fn check_ca_configuration() -> CaConfig {
    if std::env::var("NATS_INSECURE").is_ok() {
        return CaConfig::Insecure;
    }
    
    if let Ok(ca_path) = std::env::var("NATS_CA") {
        return CaConfig::FilePath(ca_path);
    }
    
    if let Ok(ca_contents) = std::env::var("NATS_CA_CONTENTS") {
        return CaConfig::Contents(ca_contents.len());
    }
    
    CaConfig::SystemTrustStore
}

fn print_usage(missing_vars: &[String]) {
    println!("❌ Missing required environment variables: {}", missing_vars.join(", "));
    println!();
    println!("Required environment variables:");
    println!("  NATS_HOST=<server:port>     # e.g., nats.example.com:4222 or 192.168.1.100:4222");
    println!("  NATS_USER=<username>        # e.g., matchmaker");
    println!("  NATS_PASSWORD=<password>    # e.g., your_secure_password");
    println!();
    println!("Optional environment variables for CA certificate configuration:");
    println!();
    println!("For self-signed certificates (choose ONE option):");
    println!("  NATS_CA=/path/to/rootCA.pem              # Path to CA certificate file");
    println!("  NATS_CA_CONTENTS=\"$(cat rootCA.pem)\"      # CA certificate contents");
    println!();
    println!("For trusted certificates (LetsEncrypt, etc.):");
    println!("  # No additional variables needed - system trust store will be used");
    println!();
    println!("For development/testing only:");
    println!("  NATS_INSECURE=1                          # Disable TLS verification (NOT RECOMMENDED)");
    println!();
    println!("Example usage:");
    println!("  export NATS_HOST=\"nats.example.com:4222\"");
    println!("  export NATS_USER=\"matchmaker\"");
    println!("  export NATS_PASSWORD=\"your_password\"");
    println!("  export NATS_CA=\"/path/to/your/rootCA.pem\"");
    println!("  cargo run --example nats_ca_example");
}

fn print_troubleshooting_guide(error: &dyn std::fmt::Display) {
    let error_msg = format!("{}", error);
    
    println!("🔧 Troubleshooting Guide:");
    println!();
    
    if error_msg.contains("certificate") || error_msg.contains("tls") || error_msg.contains("handshake") {
        println!("This appears to be a TLS certificate verification error.");
        println!();
        println!("Common causes and solutions:");
        println!("1. Self-signed certificate without CA configuration:");
        println!("   → Set NATS_CA=/path/to/rootCA.pem (the CA that signed your server cert)");
        println!("   → Or set NATS_CA_CONTENTS with the certificate contents");
        println!();
        println!("2. Wrong CA certificate:");
        println!("   → Ensure you're using the CA that signed your NATS server certificate");
        println!("   → For mkcert: Use the rootCA.pem from $(mkcert -CAROOT)");
        println!();
        println!("3. File permission or path issues:");
        println!("   → Check that the CA file exists and is readable");
        println!("   → Verify the file path is correct");
        println!();
    } else if error_msg.contains("connection refused") || error_msg.contains("No route to host") {
        println!("This appears to be a network connectivity issue.");
        println!();
        println!("Check:");
        println!("1. NATS server is running and accessible");
        println!("2. Firewall settings allow connections to the NATS port");
        println!("3. NATS_HOST is correct (hostname and port)");
        println!();
    } else if error_msg.contains("authentication") || error_msg.contains("authorization") {
        println!("This appears to be an authentication issue.");
        println!();
        println!("Check:");
        println!("1. NATS_USER and NATS_PASSWORD are correct");
        println!("2. User exists in NATS server configuration");
        println!("3. User has necessary permissions");
        println!();
    }
    
    println!("General debugging:");
    println!("• Enable debug logging: RUST_LOG=debug");
    println!("• Test with nats-cli first to verify server connectivity");
    println!("• Check NATS server logs for additional error details");
}