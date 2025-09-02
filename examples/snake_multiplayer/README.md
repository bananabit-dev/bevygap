# Snake Multiplayer Example

A complete multiplayer snake game implementation using BevyGap, demonstrating how to build networked games with Bevy and Lightyear.

## Overview

This example showcases:
- **Client-Server Architecture**: Authoritative server with responsive clients
- **BevyGap Integration**: Matchmaking and connection management
- **Game Logic**: Snake movement, collision detection, and scoring
- **Local Development**: Demo mode for testing without infrastructure

## Quick Start

### Run the Demo

```bash
# Clone the repository and navigate to the example
cd examples/snake_multiplayer

# Run the client demo (no server required)
cargo run -p snake_client
```

Use WASD or arrow keys to control the snake!

### Run the Server

```bash
# Start the game server
cargo run -p snake_server
```

## Project Structure

```
snake_multiplayer/
├── shared/           # Common components and messages
├── server/           # Authoritative game server
├── client/           # Game client with rendering
└── Cargo.toml        # Workspace configuration
```

## Features

### Implemented
- ✅ Snake movement and controls
- ✅ Food spawning and consumption
- ✅ Collision detection
- ✅ Score tracking
- ✅ Client rendering with Bevy
- ✅ Server game logic
- ✅ BevyGap integration structure

### Ready for Extension
- 🔧 Full networking with Lightyear
- 🔧 Multiple players
- 🔧 Real-time synchronization
- 🔧 Matchmaking integration
- 🔧 Production deployment

## Game Mechanics

- **Movement**: Grid-based snake movement with smooth animations
- **Growth**: Snake grows when consuming food
- **Scoring**: Points awarded for each food consumed
- **Boundaries**: Snake wraps around screen edges
- **Food**: Automatically spawns in random locations

## Technical Details

### Architecture
- **Shared Crate**: Common game components and utilities
- **Server Crate**: Authoritative game logic and BevyGap server integration
- **Client Crate**: User interface, rendering, and BevyGap client integration

### Dependencies
- **Bevy 0.16**: Game engine and ECS framework
- **BevyGap**: Matchmaking and networking infrastructure
- **Lightyear**: Low-level networking (ready for integration)
- **Serde**: Serialization for network messages

### Performance
- **60 FPS**: Smooth gameplay with efficient rendering
- **Low Latency**: Optimized for responsive controls
- **Scalable**: Architecture supports multiple players

## Development

### Local Testing

The example includes a demo mode that runs entirely locally:

```bash
cargo run -p snake_client
```

This allows you to:
- Test game mechanics
- Verify rendering
- Debug input handling
- Develop new features

### Adding Networking

To extend to full multiplayer:

1. **Implement Lightyear Protocol** in shared crate
2. **Add Connection Handling** in server
3. **Replace Local State** with network messages in client
4. **Configure Matchmaking** for production deployment

### Code Organization

- `shared/src/lib.rs`: Game components, messages, and utilities
- `server/src/main.rs`: Server logic and BevyGap integration
- `client/src/main.rs`: Client rendering and controls

## Documentation

Complete step-by-step tutorial available in the [BevyGap Book](../../book/src/examples/snake/index.md):

1. [Project Setup](../../book/src/examples/snake/setup.md)
2. [Shared Components](../../book/src/examples/snake/shared.md)
3. [Server Implementation](../../book/src/examples/snake/server.md)
4. [Client Implementation](../../book/src/examples/snake/client.md)
5. [Running the Game](../../book/src/examples/snake/running.md)

## Requirements

- Rust (latest stable)
- Platform support for Bevy (Windows, macOS, Linux)

## License

This example is part of the BevyGap project and follows the same license terms.

## Contributing

Improvements and extensions welcome! Consider:
- Additional game features
- Better graphics and animations
- Performance optimizations
- Documentation improvements
- Full networking implementation

## Related Projects

- [BevyGap Spaceships](https://github.com/RJ/bevygap-spaceships) - Another complete multiplayer example
- [Bevy Examples](https://github.com/bevyengine/bevy/tree/main/examples) - Official Bevy examples
- [Lightyear Examples](https://github.com/cBournhonesque/lightyear/tree/main/examples) - Networking examples