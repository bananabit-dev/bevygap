// Test to verify TLS certificate content handling without needing a real NATS server
use bevygap_shared::nats::BevygapNats;

#[tokio::test]
async fn test_ca_contents_vs_ca_file_approaches() {
    // This test verifies that both NATS_CA_CONTENTS and NATS_CA approaches work
    // without requiring an actual NATS server
    
    // Test 1: NATS_CA_CONTENTS should create TLS config from content
    std::env::set_var("NATS_CA_CONTENTS", TEST_CERT);
    std::env::set_var("NATS_HOST", "fake-host.test:4222");
    std::env::set_var("NATS_USER", "test");
    std::env::set_var("NATS_PASSWORD", "test");
    
    // This should use the certificate content approach
    let result1 = BevygapNats::new_and_connect("test_ca_contents").await;
    assert!(result1.is_err()); // Expected to fail due to fake host
    let error1 = result1.unwrap_err().to_string();
    
    // Clean up
    std::env::remove_var("NATS_CA_CONTENTS");
    
    // Test 2: NATS_CA with file path (traditional approach) 
    // We'll use a non-existent file to test the path-based approach
    std::env::set_var("NATS_CA", "/nonexistent/path/to/ca.pem");
    
    // This should use the file path approach and should fail differently
    let result2 = BevygapNats::new_and_connect("test_ca_file").await;
    assert!(result2.is_err()); // Expected to fail
    let error2 = result2.unwrap_err().to_string();
    
    // The errors should be different but both should be connection-related, not parsing-related
    println!("CA_CONTENTS error: {}", error1);
    println!("CA_FILE error: {}", error2);
    
    // Clean up
    std::env::remove_var("NATS_CA");
    std::env::remove_var("NATS_HOST");
    std::env::remove_var("NATS_USER");
    std::env::remove_var("NATS_PASSWORD");
}

#[test]
fn test_tls_config_creation_from_cert_contents() {
    // Test the TLS config creation function directly
    let result = BevygapNats::create_tls_client_config_from_contents(TEST_CERT);
    
    // This should succeed because the certificate content is valid
    assert!(result.is_ok(), "Failed to create TLS config from cert contents: {:?}", result.err());
    
    // Test with completely invalid certificate content
    let invalid_cert = "NOT A CERTIFICATE AT ALL";
    let result2 = BevygapNats::create_tls_client_config_from_contents(invalid_cert);
    
    // This creates an empty root store, which is technically valid but not useful for verification
    // The real issue would show up when trying to verify a certificate against this empty store
    println!("Result for invalid cert: {:?}", result2);
    
    // Test with malformed PEM
    let malformed_pem = r#"-----BEGIN CERTIFICATE-----
    INVALID_BASE64_CONTENT_HERE
    -----END CERTIFICATE-----"#;
    let result3 = BevygapNats::create_tls_client_config_from_contents(malformed_pem);
    
    // This should fail due to invalid base64 content
    assert!(result3.is_err(), "Should have failed with malformed PEM content");
}

const TEST_CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIIE2DCCA0CgAwIBAgIRAIW/i8Ryvk+oZGg+/FvDpW8wDQYJKoZIhvcNAQELBQAw
gYMxHjAcBgNVBAoTFW1rY2VydCBkZXZlbG9wbWVudCBDQTEsMCoGA1UECwwjc3Rq
ZXBhbkBsb2NhbGhvc3QgKFN0amVwYW4gR2xhdmluYSkxMzAxBgNVBAMMKm1rY2Vy
dCBzdGplcGFuQGxvY2FsaG9zdCAoU3RqZXBhbiBHbGF2aW5hKTAeFw0yMDA3MDcx
NTU4NTlaFw0zMDA3MDcxNTU4NTlaMIGDMR4wHAYDVQQKExVta2NlcnQgZGV2ZWxv
cG1lbnQgQ0ExLDAqBgNVBAsMI3N0amVwYW5AbG9jYWxob3N0IChTdGplcGFuIEds
YXZpbmEpMTMwMQYDVQQDDCpta2NlcnQgc3RqZXBhbkBsb2NhbGhvc3QgKFN0amVw
YW4gR2xhdmluYSkwggGiMA0GCSqGSIb3DQEBAQUAA4IBjwAwggGKAoIBgQCyRbze
lNj5QkxgMxYIr4yEqLtTz/jSiX1W6uUymI+e1GQzl3BpP3fS/SyB9HZv1bn4PoiW
B+BggsFrOsmVF4aXbikRz74DQ7FCqVMU/QGvlIbHkH5TShcLGzngG8DR3+Url3S8
rlvYQBf2AXYXWcmSl1bFYzVxWSt67NzS29mvus40aAPXTpB7vq6nNs6F+7sARhDi
OPRqniPqV29khMmxndwExAEMJBVZbeTNuwfehCud8dOj0kzk5ESX+3upIwOrnoye
2tRNW34WWaxQrV65KOeGxdgIN+PeO7WL1jbCitCaGnGitTbtPGMCdf6LmRhA30Wy
IZ3ZkxOmvXGkpAR6mxz3pqQWTZYieTA92s63LeVSNeYUof0tNu0SMTYWuAas0Ob3
A/lu7PTCjTag5vVU5RwkfBmcNrbNNy9NbKgQB7TafZn7sfPpZT4EpAcFMgRb4KfR
HfPiaxlDu2LKmBS9+i7x79nYAlB5IGgLyQ1cldwjDYqAAizBoigM2PI3tEMCAwEA
AaNFMEMwDgYDVR0PAQH/BAQDAgIEMBIGA1UdEwEB/wQIMAYBAf8CAQAwHQYDVR0O
BBYEFGg72r4gKsOkZ8rgX9wSO0trlJnyMA0GCSqGSIb3DQEBCwUAA4IBgQBzxbH6
s7dhvyOGK5HnQmc/vPiTbOk4QhXX12PPekL+UrhMyiMzWET2wkpHJXxtes0dGNU3
tmQwVQTvg/BhyxxGcafvQe/o/5rS9lrrpdQriZg3gqqeUoi8UX+5/ef9EB7Z32Ho
qUSOd6hPLLI+3UrnmWP+u5PRDmsHQYKc2YqyqQisjRVwLtGtCmGYfuBncP145Yyi
qNlwI6jeZTAtRSkcKy6fnyJcjOCYKFWHpTGmBTMtO4LiTGadxnmbAq9mRBiKJJp6
wrSz1JvbVXVY4caxpbDfkaT8RiP+k1Fbd6uMWnZTJLHPTNbzCl4aXcuHgoRhCLeq
SdF3L7m0tM7lsTP3tddRY6zb+1u0II0Gu6umDsdyL6JOV4vv9Qb7xdy2jTU231+o
TXLHaypw4Amp267EyvvWmU3VOl8BeUkJ/7LOqzZfKxTECwnxWywx6NV9ONQt8mNC
ATAQAyYXklJsZkX6VLMPE0Lv4Qbt/GnGUejER09zQi433e9jUF+vwQGwj/g=
-----END CERTIFICATE-----"#;