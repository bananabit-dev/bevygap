# Developing your Game

This section covers how to integrate Bevygap into your Bevy game and how to develop locally without requiring the full Edgegap infrastructure.

## Quick Start for Local Development

For local development, you can bypass the matchmaking system and connect directly to your local gameserver. This is useful during development when you don't want to deploy to Edgegap for every test.

### Option 1: Disable Bevygap Features

In your game's `Cargo.toml`, you can build without the bevygap features to use standard Lightyear networking:

```toml
# For local development - build without bevygap features
bevygap_client_plugin = { git = "https://github.com/bananabit-dev/bevygap.git", branch = "main", default-features = false }
bevygap_server_plugin = { git = "https://github.com/bananabit-dev/bevygap.git", branch = "main", default-features = false }
```

### Option 2: Local Bypass Configuration

Alternatively, you can configure bevygap to bypass the matchmaker for local development. (TODO: Document specific configuration parameters)

## Integrating Bevygap into Your Game

### Client Integration

Add the bevygap client plugin to your game client:

```rust
use bevygap_client_plugin::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevygapClientPlugin)
        .insert_resource(BevygapClientConfig::default())
        // ... your other plugins
        .run();
}
```

### Server Integration

Add the bevygap server plugin to your game server:

```rust
use bevygap_server_plugin::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(BevygapServerPlugin)
        // ... your other plugins
        .run();
}
```

## Example Game

The [bevygap-spaceships](https://github.com/RJ/bevygap-spaceships) repository contains a complete example showing how to integrate Bevygap with a Bevy game using Lightyear networking.

## Development Workflow

1. **Local Development**: Use direct connections or bypass features for rapid iteration
2. **Testing**: Deploy to Edgegap when you need to test the full matchmaking flow
3. **Production**: Use the full Bevygap stack with proper NATS setup and Edgegap deployment

For more details on each component and configuration options, see the Installation section.