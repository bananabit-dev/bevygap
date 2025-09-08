# Bevygap Development Instructions

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

Bevygap is a Rust workspace for deploying Bevy + Lightyear multiplayer games on Edgegap cloud infrastructure. It consists of multiple services, Docker deployment capabilities, and example integrations.

## Working Effectively

### System Dependencies
Install required system packages before building:
```bash
sudo apt update
sudo apt install -y libasound2-dev libudev-dev pkg-config
```

### Bootstrap, Build, and Test
- **Check workspace**: `cargo check` -- takes 5m 12s. NEVER CANCEL. Set timeout to 10+ minutes.
- **Build workspace**: `cargo build` -- takes 15m 19s. NEVER CANCEL. Set timeout to 25+ minutes.
- **Build release**: `cargo build --release` -- takes similar time to debug build. NEVER CANCEL. Set timeout to 25+ minutes.
- **Run tests**: `cargo test` -- takes 11m 21s. NEVER CANCEL. Set timeout to 20+ minutes.

### Example Projects
- **Check examples**: `cd examples/snake_multiplayer && cargo check` -- takes 2m 18s. Set timeout to 5+ minutes.
- **Build examples**: `cd examples/snake_multiplayer && cargo build` -- takes 3m 17s. Set timeout to 5+ minutes.

### Documentation
- **Install mdbook**: `cargo install mdbook`
- **Build documentation**: `cd book && mdbook build` -- takes 0.091s.
- **Serve documentation locally**: `cd book && mdbook serve -o` -- opens browser to view docs.

### Docker Builds
Docker builds available for core services:
- `bevygap_matchmaker/Dockerfile`
- `bevygap_matchmaker_httpd/Dockerfile`  
- `bevygap_webhook_sink/Dockerfile`

Example build: `docker build -f bevygap_matchmaker/Dockerfile -t bevygap_matchmaker .`
Note: Docker builds will take significantly longer (20+ minutes) as they rebuild the entire Rust workspace inside containers.

## Validation

### Manual Testing
- **Test matchmaker help**: `./target/debug/bevygap_matchmaker_httpd --help` -- shows CLI options
- **Test binaries**: Most services require NATS environment variables to run properly
- Always run `cargo check` after code changes to catch compilation errors quickly
- Run `cargo test` to validate that existing functionality still works

### Build Dependencies
The workspace depends on:
- **Bevy 0.16** for game engine functionality
- **Lightyear** (git dependency) for networking
- **NATS** for message brokering between services
- **Edgegap API** client for cloud deployment
- Various system audio/input libraries (hence the libasound2-dev, libudev-dev requirements)

### Environment Setup for Services
Services require environment variables:
- `NATS_HOST` - NATS server hostname
- `NATS_USER` - NATS username  
- `NATS_PASS` - NATS password
- `NATS_INSECURE` - Set for non-TLS connections
- Edgegap API credentials for deployment functionality

## Key Components

### Core Services
- **bevygap_matchmaker** - Core matchmaking service that manages Edgegap sessions
- **bevygap_matchmaker_httpd** - HTTP frontend providing WebSocket API for clients
- **bevygap_webhook_sink** - Webhook handling service for Edgegap callbacks
- **bevygap_shared** - Shared utilities and NATS integration code

### Game Integration
- **bevygap_client_plugin** - Bevy plugin for game clients to connect via matchmaker
- **bevygap_server_plugin** - Bevy plugin for game servers running on Edgegap
- **bevy_nfws** - WebSocket client library for Bevy applications

### Infrastructure
- **edgegap_async** - Auto-generated Edgegap API client (regenerated via utils/gen-edgegap-client.sh)
- **examples/snake_multiplayer** - Complete example game showing integration patterns

## Common Tasks

### Development Workflow
1. **Make code changes** in relevant crate
2. **Quick validation**: `cargo check` (5+ minutes)
3. **Full build test**: `cargo build` (25+ minutes) 
4. **Run tests**: `cargo test` (20+ minutes)
5. **Test specific binary**: `cargo run -p <crate_name>`

### Adding New Features
- Modify relevant workspace member in `Cargo.toml`
- Add new dependencies to workspace dependencies section
- Update documentation in `book/src/` and rebuild with `mdbook build`
- Add integration tests to validate end-to-end functionality

### Expected Build Warnings
The workspace generates various "dead code" warnings - this is normal for library crates with public APIs that may not be fully utilized internally. Key warnings to expect:
- Dead code warnings in bevygap_server_plugin (expected for library APIs)
- Unused import warnings in example code
- Deprecated method warnings (e.g., bevy's `get_single` vs `single`)

### File Locations
- **Main workspace**: `/` (root Cargo.toml defines all members)
- **Documentation source**: `book/src/`
- **Example games**: `examples/`
- **Utility scripts**: `utils/` (requires internet access for some scripts)
- **Container definitions**: `docker-compose.yml` and individual Dockerfiles
- **CI/CD**: `.github/workflows/` (publishes Docker images and documentation)

### Time Expectations
- **cargo check**: ~5 minutes (dependency heavy, but no compilation)
- **cargo build**: ~15 minutes (full compilation with optimizations)
- **cargo test**: ~11 minutes (limited test suite, mostly compilation time)
- **example builds**: ~3 minutes (smaller scope)
- **mdbook build**: <1 second (markdown to HTML)
- **Docker builds**: 20+ minutes (rebuilds everything in container)

### Critical Validation Scenarios
After making any changes, always test these scenarios:
1. **Basic compilation**: `cargo check` to catch syntax/type errors quickly
2. **Full workspace build**: `cargo build` to ensure all dependencies resolve
3. **Test suite**: `cargo test` to verify existing functionality
4. **Binary functionality**: `./target/debug/bevygap_matchmaker_httpd --help` to test CLI
5. **Example integration**: `cd examples/snake_multiplayer && cargo check` to validate game integration patterns

### End-to-End Testing
When possible, verify the complete flow:
1. Build all services with `cargo build`
2. Test that help commands work for key binaries
3. Validate that documentation builds with `mdbook build`
4. Check that example projects compile successfully

Remember: NEVER CANCEL long-running builds. Bevy projects with complex dependencies require substantial compilation time. Set generous timeouts and let builds complete.