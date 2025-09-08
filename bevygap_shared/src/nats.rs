use async_nats::jetstream::stream::Stream;
use async_nats::jetstream::{self, stream};
use async_nats::Client;
use std::time::Duration;
use std::net::{SocketAddr, ToSocketAddrs};

use log::*;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct BevygapNats {
    client: Client,
    kv_s2c: jetstream::kv::Store,
    kv_c2s: jetstream::kv::Store,
    kv_cert_digests: jetstream::kv::Store,
    kv_active_connections: jetstream::kv::Store,
    kv_unclaimed_sessions: jetstream::kv::Store,
    delete_session_stream: Stream,
}

const DELETE_SESSION_STREAM: &str = "edgegap_delete_session_q";

impl BevygapNats {
    /// Connects to NATS based on environment variables.
    /// 
    /// This method performs a complete setup including Jetstream key-value stores.
    /// If you only need to test basic NATS connectivity, use `connect_to_nats()` directly.
    pub async fn new_and_connect(nats_client_name: &str) -> Result<Self, async_nats::Error> {
        let client = Self::connect_to_nats(nats_client_name).await?;
        
        // Test Jetstream availability before proceeding
        info!("NATS: Testing Jetstream availability...");
        let jetstream = jetstream::new(client.clone());
        
        // Try to create a simple test operation to verify Jetstream is working
        let test_bucket_name = format!("test_connectivity_{}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
        
        match jetstream.create_key_value(async_nats::jetstream::kv::Config {
            bucket: test_bucket_name.clone(),
            ..Default::default()
        }).await {
            Ok(_kv) => {
                info!("NATS: Jetstream is available and working");
                // Clean up test bucket
                if let Err(e) = jetstream.delete_key_value(&test_bucket_name).await {
                    warn!("NATS: Failed to clean up test bucket (this is normal if server doesn't support deletion): {}", e);
                }
            }
            Err(e) => {
                error!("NATS: Jetstream is not available or not enabled: {}", e);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Jetstream is required but not available: {}. Please enable Jetstream on your NATS server.", e)
                )));
            }
        }
        
        // Create Jetstream resources with retry logic
        info!("NATS: Creating Jetstream key-value stores...");
        let (kv_s2c, kv_c2s) = Self::create_kv_buckets_for_session_mappings(client.clone()).await
            .map_err(|e| {
                error!("NATS: Failed to create session mapping KV stores: {}", e);
                e
            })?;
            
        let kv_active_connections = Self::create_kv_active_connections(client.clone()).await
            .map_err(|e| {
                error!("NATS: Failed to create active connections KV store: {}", e);
                e
            })?;
            
        let kv_cert_digests = Self::create_kv_cert_digests(client.clone()).await
            .map_err(|e| {
                error!("NATS: Failed to create cert digests KV store: {}", e);
                e
            })?;
            
        let kv_unclaimed_sessions = Self::create_kv_unclaimed_sessions(client.clone()).await
            .map_err(|e| {
                error!("NATS: Failed to create unclaimed sessions KV store: {}", e);
                e
            })?;
            
        let delete_session_stream = Self::create_session_delete_queue(&client).await
            .map_err(|e| {
                error!("NATS: Failed to create delete session stream: {}", e);
                e
            })?;
            
        info!("NATS: Successfully created all Jetstream resources");
        
        Ok(Self {
            client,
            kv_s2c,
            kv_c2s,
            kv_cert_digests,
            kv_active_connections,
            kv_unclaimed_sessions,
            delete_session_stream,
        })
    }
    
    /// Test only the basic NATS connection without Jetstream functionality.
    /// This is useful for diagnostic purposes and environments where Jetstream is not available.
    pub async fn test_basic_connection(nats_client_name: &str) -> Result<Client, async_nats::Error> {
        Self::connect_to_nats(nats_client_name).await
    }

    pub fn client(&self) -> Client {
        self.client.clone()
    }
    pub fn kv_s2c(&self) -> &jetstream::kv::Store {
        &self.kv_s2c
    }
    pub fn kv_c2s(&self) -> &jetstream::kv::Store {
        &self.kv_c2s
    }
    pub fn kv_active_connections(&self) -> &jetstream::kv::Store {
        &self.kv_active_connections
    }
    pub fn kv_unclaimed_sessions(&self) -> &jetstream::kv::Store {
        &self.kv_unclaimed_sessions
    }
    pub fn kv_cert_digests(&self) -> &jetstream::kv::Store {
        &self.kv_cert_digests
    }
    pub fn delete_session_stream(&self) -> &Stream {
        &self.delete_session_stream
    }

    /// Enqueues a job to delete a session id via the edgegap API
    pub async fn enqueue_session_delete(
        &self,
        session_id: String,
    ) -> Result<(), async_nats::Error> {
        let js = jetstream::new(self.client.clone());
        js.publish(
            format!("{DELETE_SESSION_STREAM}.{session_id}"),
            session_id.into(),
        )
        .await?
        .await?;
        Ok(())
    }

    /// Connects to NATS with TLS certificate verification support.
    /// 
    /// This method supports multiple connection modes for different deployment scenarios:
    /// 
    /// ## Production Mode (Trusted Certificates)
    /// For servers with trusted certificates (e.g., LetsEncrypt):
    /// - Set `NATS_HOST`, `NATS_USER`, `NATS_PASSWORD`
    /// - TLS verification uses the system's trusted CA store
    /// 
    /// ## Self-Signed Certificate Mode
    /// For servers with self-signed certificates, you must provide the root CA:
    /// 
    /// ### Option 1: CA File Path
    /// ```bash
    /// export NATS_CA="/path/to/rootCA.pem"
    /// ```
    /// The certificate file must be accessible on the filesystem.
    /// 
    /// ### Option 2: CA Contents (useful for containers/embedded deployments)
    /// ```bash
    /// export NATS_CA_CONTENTS="$(cat /path/to/rootCA.pem)"
    /// ```
    /// The certificate contents are written to a temporary file and loaded.
    /// 
    /// ## Insecure Mode (Development Only)
    /// Disable TLS verification entirely:
    /// ```bash
    /// export NATS_INSECURE=1
    /// ```
    /// ⚠️ **WARNING:** Not recommended for production use.
    /// 
    /// ## Troubleshooting
    /// If you see "unknown certificate authority" errors:
    /// 1. Verify the CA certificate file exists and is readable
    /// 2. Ensure the CA certificate is the one that signed your NATS server certificate
    /// 3. Check file permissions and paths
    /// 4. Enable debug logging with `RUST_LOG=debug`
    /// 
    /// ## Environment Variables
    /// - `NATS_HOST`: Server address (required)
    /// - `NATS_USER`: Username (required)  
    /// - `NATS_PASSWORD`: Password (required)
    /// - `NATS_CA`: Path to CA certificate file (optional)
    /// - `NATS_CA_CONTENTS`: CA certificate contents (optional)
    /// - `NATS_INSECURE`: Disable TLS verification (optional)
    /// 
    /// ## Retry Behavior
    /// Connection retries are handled automatically by async_nats when `retry_on_initial_connect()` is enabled.
    /// The function will try multiple host variants (original, IPv6, IPv4) with async_nats handling retries for each.
    async fn connect_to_nats(nats_client_name: &str) -> Result<Client, async_nats::Error> {
        info!("NATS: setting up, client name: {nats_client_name}");

        let nats_insecure = std::env::var("NATS_INSECURE").is_ok();
        
        // Load root CA certificate for TLS verification with self-signed certificates
        // This supports two methods:
        // 1. NATS_CA: Path to a CA certificate file on the filesystem
        // 2. NATS_CA_CONTENTS: Certificate contents passed as environment variable
        let nats_self_signed_ca: Option<String> = std::env::var("NATS_CA").ok().or_else(|| {
            if let Ok(ca_contents) = std::env::var("NATS_CA_CONTENTS") {
                // For NATS_CA_CONTENTS, we write the certificate to a temporary file
                // This is useful for container deployments where the certificate
                // content is passed as an environment variable rather than a file
                let sanitised_nats_client_name = nats_client_name
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>();
                let tmp_file =
                    std::env::temp_dir().join(format!("rootCA-{sanitised_nats_client_name}.pem"));
                
                // Write the CA certificate contents to the temporary file
                match std::fs::write(&tmp_file, ca_contents) {
                    Ok(_) => {
                        info!("NATS: CA certificate written to temporary file: {}", tmp_file.display());
                        Some(tmp_file.to_string_lossy().to_string())
                    }
                    Err(e) => {
                        warn!("NATS: Failed to write CA certificate to temp file: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        });

        let nats_host = std::env::var("NATS_HOST").expect("Missing NATS_HOST env");
        let nats_user = std::env::var("NATS_USER").expect("Missing NATS_USER env");
        let nats_pass = std::env::var("NATS_PASSWORD").expect("Missing NATS_PASSWORD env");

        if nats_insecure {
            warn!("😬 NATS: insecure mode - TLS verification is disabled. Not recommended for production!");
        } else {
            info!("NATS: TLS is enabled");
            if let Some(ref ca_path) = nats_self_signed_ca {
                info!("NATS: Using custom CA certificate for TLS verification: {}", ca_path);
            } else {
                info!("NATS: Using system trusted CA store for TLS verification");
            }
        }

        info!("NATS: connecting as '{nats_user}' to {nats_host} (using async_nats retry mechanism)");

        // Generate multiple host variants (original, IPv6, IPv4) to try
        let hosts_to_try = Self::generate_connection_hosts(&nats_host);
        let mut last_error: Option<async_nats::Error> = None;

        // Try each host variant once - async_nats will handle retries for each host
        for (host_description, host_to_try) in &hosts_to_try {
            info!("NATS: trying connection to {} ({})", host_to_try, host_description);
            
            // Create connection options with retry_on_initial_connect enabled
            let mut connection_opts = async_nats::ConnectOptions::new()
                .name(nats_client_name)
                .user_and_password(nats_user.clone(), nats_pass.clone())
                .max_reconnects(10)
                .require_tls(!nats_insecure)
                .retry_on_initial_connect(); // Let async_nats handle retries

            // Configure TLS with custom root CA certificate if provided
            // This is essential for connecting to NATS servers with self-signed certificates
            if let Some(ref ca) = nats_self_signed_ca {
                info!("NATS: Adding root certificate for TLS verification: {}", ca);
                connection_opts = connection_opts.add_root_certificates(ca.clone().into());
            }

            match connection_opts.connect(host_to_try).await {
                Ok(client) => {
                    info!("🟢 NATS: connected OK to {} ({})", host_to_try, host_description);
                    return Ok(client);
                }
                Err(e) => {
                    warn!("NATS: connection failed to {} ({}): {}", host_to_try, host_description, e);
                    // Check if this might be a certificate verification error
                    let error_msg = format!("{}", e);
                    if error_msg.contains("certificate") || error_msg.contains("tls") || error_msg.contains("handshake") {
                        warn!("NATS: TLS certificate error detected. Ensure NATS_CA or NATS_CA_CONTENTS is set for self-signed certificates.");
                    }
                    last_error = Some(Box::new(e) as async_nats::Error);
                }
            }
        }

        error!("NATS: all host variants failed to connect");
        // Return the last error we got, converting the type as needed
        Err(last_error.unwrap_or_else(|| {
            let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "All host variants failed to connect");
            Box::new(io_error) as async_nats::Error
        }))
    }

    /// Generate list of hosts to try, including IPv6 and IPv4 variants if the host is a domain name
    pub fn generate_connection_hosts(host: &str) -> Vec<(String, String)> {
        let mut hosts = Vec::new();
        
        // First, try the original host as-is
        hosts.push(("original".to_string(), host.to_string()));
        
        // If the host contains a port, separate it
        let (hostname, port) = if let Some(colon_pos) = host.rfind(':') {
            let potential_port = &host[colon_pos + 1..];
            if potential_port.parse::<u16>().is_ok() {
                (&host[..colon_pos], Some(&host[colon_pos..]))
            } else {
                (host, None)
            }
        } else {
            (host, None)
        };
        
        // Try to resolve hostname to get IPv6 and IPv4 addresses
        // We'll use a dummy port for resolution if none is provided
        let resolve_host = if port.is_some() {
            host.to_string()
        } else {
            format!("{}:4222", hostname) // Use default NATS port for resolution
        };
        
        if let Ok(addrs) = resolve_host.to_socket_addrs() {
            let mut ipv6_addrs = Vec::new();
            let mut ipv4_addrs = Vec::new();
            
            for addr in addrs {
                match addr {
                    SocketAddr::V6(_) => ipv6_addrs.push(addr),
                    SocketAddr::V4(_) => ipv4_addrs.push(addr),
                }
            }
            
            // Add IPv6 addresses first (prefer IPv6)
            for addr in ipv6_addrs {
                let host_str = if port.is_some() {
                    addr.to_string()
                } else {
                    format!("[{}]", addr.ip())
                };
                hosts.push(("IPv6".to_string(), host_str));
            }
            
            // Then add IPv4 addresses as fallback
            for addr in ipv4_addrs {
                let host_str = if port.is_some() {
                    addr.to_string()
                } else {
                    addr.ip().to_string()
                };
                hosts.push(("IPv4".to_string(), host_str));
            }
        }
        
        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        hosts.retain(|(_, host)| seen.insert(host.clone()));
        
        hosts
    }

    pub async fn create_kv_active_connections(
        client: Client,
    ) -> Result<jetstream::kv::Store, async_nats::Error> {
        let jetstream = jetstream::new(client);
        let kv = jetstream
            .create_key_value(async_nats::jetstream::kv::Config {
                bucket: "active_connections".to_string(),
                ..Default::default()
            })
            .await?;
        Ok(kv)
    }

    pub async fn create_kv_unclaimed_sessions(
        client: Client,
    ) -> Result<jetstream::kv::Store, async_nats::Error> {
        let jetstream = jetstream::new(client);
        let kv = jetstream
            .create_key_value(async_nats::jetstream::kv::Config {
                bucket: "unclaimed_sessions".to_string(),
                max_value_size: 1024,
                description: "Any session ids we get from the API are stored here, and if they key age gets too big, we delete the session via the API.".to_string(),
                ..Default::default()
            })
            .await?;
        Ok(kv)
    }

    pub async fn create_session_delete_queue(client: &Client) -> Result<Stream, async_nats::Error> {
        let js = jetstream::new(client.clone());
        let stream = js
            .create_stream(jetstream::stream::Config {
                name: "DELETE_SESSION_STREAM".to_string(),
                retention: stream::RetentionPolicy::WorkQueue,
                subjects: vec![format!("{DELETE_SESSION_STREAM}.*").to_string()],
                ..Default::default()
            })
            .await?;
        Ok(stream)
    }

    pub async fn create_kv_cert_digests(
        client: Client,
    ) -> Result<jetstream::kv::Store, async_nats::Error> {
        let jetstream = jetstream::new(client);
        let kv = jetstream
            .create_key_value(async_nats::jetstream::kv::Config {
                bucket: "cert_digests".to_string(),
                description: "Maps server public ip to their self-signed cert digests".to_string(),
                max_age: Duration::from_secs(86400 * 14),
                max_value_size: 1024,

                ..Default::default()
            })
            .await?;
        Ok(kv)
    }

    /// Creates two buckets for mapping between LY client ids and Edgegap session tokens
    async fn create_kv_buckets_for_session_mappings(
        client: Client,
    ) -> Result<(jetstream::kv::Store, jetstream::kv::Store), async_nats::Error> {
        let jetstream = jetstream::new(client);

        let kv_s2c = jetstream
            .create_key_value(async_nats::jetstream::kv::Config {
                bucket: "sessions_eg2ly".to_string(),
                description: "Maps Edgegap Session IDs to Lightyear Client IDs".to_string(),
                max_value_size: 1024,
                // shouldn't need long for the client to receive token, and make connection to gameserver.
                max_age: Duration::from_millis(30000),
                // storage: StorageType::File,
                ..Default::default()
            })
            .await?;

        let kv_c2s = jetstream
            .create_key_value(async_nats::jetstream::kv::Config {
                bucket: "sessions_ly2eg".to_string(),
                description: "Maps Lightyear Client IDs to Edgegap Session IDs".to_string(),
                max_value_size: 1024,
                // shouldn't need long for the client to receive token, and make connection to gameserver.
                max_age: Duration::from_millis(30000),
                // storage: StorageType::File,
                ..Default::default()
            })
            .await?;

        Ok((kv_s2c, kv_c2s))
    }
}
