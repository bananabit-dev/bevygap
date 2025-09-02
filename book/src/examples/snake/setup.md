# Project Setup

In this chapter, we'll set up the project structure for our multiplayer snake game.

## Project Structure

We'll create a workspace with three crates:

```
snake_multiplayer/
├── Cargo.toml                 # Workspace configuration
├── shared/                    # Shared components and messages
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── server/                    # Game server
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── client/                    # Game client
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Creating the Workspace

First, create a new directory for your project:

```bash
mkdir snake_multiplayer
cd snake_multiplayer
```

### Workspace Cargo.toml

Create the main `Cargo.toml` file:

```toml
[workspace]
members = ["shared", "server", "client"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]

[workspace.dependencies]
# Bevy and BevyGap dependencies
bevy = { version = "0.16", default-features = false }
bevygap_client_plugin = { git = "https://github.com/bananabit-dev/bevygap.git", branch = "main" }
bevygap_server_plugin = { git = "https://github.com/bananabit-dev/bevygap.git", branch = "main" }

# Lightyear for networking (must match BevyGap's version)
lightyear = { git = "https://github.com/bananabit-dev/lightyear.git", default-features = false }

# Common dependencies
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"
log = "0.4"

# Shared crate
snake_shared = { path = "./shared" }
```

## Shared Crate

Create the shared library that both client and server will use:

```bash
mkdir shared
```

### shared/Cargo.toml

```toml
[package]
name = "snake_shared"
version.workspace = true
edition.workspace = true
authors.workspace = true

[dependencies]
bevy = { workspace = true, features = ["minimal"] }
lightyear = { workspace = true, features = ["replication"] }
serde.workspace = true
rand.workspace = true
```

### shared/src/lib.rs

We'll start with a basic structure:

```rust
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use bevy::prelude::*;
pub use lightyear::prelude::*;

// Game constants
pub const GRID_SIZE: f32 = 20.0;
pub const SNAKE_SPEED: f32 = 5.0;
pub const WORLD_WIDTH: f32 = 800.0;
pub const WORLD_HEIGHT: f32 = 600.0;

// We'll add components and messages here in the next chapter
```

## Server Crate

Create the server application:

```bash
mkdir server
```

### server/Cargo.toml

```toml
[package]
name = "snake_server"
version.workspace = true
edition.workspace = true
authors.workspace = true

[dependencies]
bevy = { workspace = true, features = [
    "minimal",
    "bevy_state",
    "bevy_time",
    "bevy_transform",
] }
bevygap_server_plugin.workspace = true
lightyear = { workspace = true, features = [
    "server",
    "webtransport",
    "netcode",
    "replication",
] }
snake_shared.workspace = true
log.workspace = true
rand.workspace = true
```

### server/src/main.rs

Basic server setup:

```rust
use bevy::prelude::*;
use bevygap_server_plugin::prelude::*;
use snake_shared::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(BevygapServerPlugin)
        // We'll add our game plugins here
        .run();
}
```

## Client Crate

Create the client application:

```bash
mkdir client
```

### client/Cargo.toml

```toml
[package]
name = "snake_client"
version.workspace = true
edition.workspace = true
authors.workspace = true

[dependencies]
bevy = { workspace = true, features = [
    "default",
    "bevy_state",
] }
bevygap_client_plugin.workspace = true
lightyear = { workspace = true, features = [
    "client",
    "webtransport",
    "netcode",
    "replication",
    "prediction",
    "interpolation",
] }
snake_shared.workspace = true
log.workspace = true
```

### client/src/main.rs

Basic client setup:

```rust
use bevy::prelude::*;
use bevygap_client_plugin::prelude::*;
use snake_shared::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevygapClientPlugin)
        .insert_resource(BevygapClientConfig::default())
        // We'll add our game plugins here
        .run();
}
```

## Local Development Setup

For local development, we want to bypass the full BevyGap matchmaking system. Add this to both client and server Cargo.toml files to enable local development:

```toml
[features]
default = []
local = []

[dependencies]
# ... existing dependencies

# For local development only
lightyear = { workspace = true, features = [
    # ... existing features
] }
```

## Testing the Setup

Let's verify our setup works:

```bash
# Test that everything compiles
cargo check --workspace

# Try running the server (it won't do much yet)
cargo run -p snake_server

# Try running the client (it will show a blank window)
cargo run -p snake_client
```

If everything compiles successfully, you're ready for the next step!

## What's Next?

Now that we have our project structure set up, we'll move on to [Shared Components](./shared.md) where we'll define the game entities, components, and messages that both the client and server will use.

This includes:
- Snake and food entities
- Input messages
- Game state components
- Network protocol setup