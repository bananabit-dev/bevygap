#[cfg(feature = "nats")]
pub mod nats;

pub mod protocol;

#[cfg(test)]
mod tests {
    #[cfg(feature = "nats")]
    mod nats_tests {
        use crate::nats::BevygapNats;

        #[test]
        fn test_generate_connection_hosts_with_ip() {
            let hosts = BevygapNats::generate_connection_hosts("192.168.1.1:4222");
            assert_eq!(hosts.len(), 1);
            assert_eq!(hosts[0], ("original".to_string(), "192.168.1.1:4222".to_string()));
        }

        #[test]
        fn test_generate_connection_hosts_with_hostname() {
            let hosts = BevygapNats::generate_connection_hosts("localhost:4222");
            // Should have at least the original
            assert!(!hosts.is_empty());
            assert_eq!(hosts[0], ("original".to_string(), "localhost:4222".to_string()));
            
            // Should have IPv6 and IPv4 variants (if localhost resolves to both)
            // The exact number depends on the system, but we expect at least 2 (original + at least one resolved)
            assert!(hosts.len() >= 1);
        }

        #[test]
        fn test_generate_connection_hosts_without_port() {
            let hosts = BevygapNats::generate_connection_hosts("example.com");
            assert!(!hosts.is_empty());
            assert_eq!(hosts[0], ("original".to_string(), "example.com".to_string()));
        }
    }
}
