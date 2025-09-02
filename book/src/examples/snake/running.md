# Running the Game

In this final chapter, we'll learn how to run the multiplayer snake game and explore both local demo mode and full networking capabilities.

## Prerequisites

Before running the game, make sure you have:

1. **Rust installed** (latest stable version)
2. **Completed the previous chapters** and have all the code in place
3. **All dependencies compiled** (this may take a few minutes the first time)

## Building the Project

First, let's make sure everything compiles correctly:

```bash
cd snake_multiplayer
cargo check --workspace
```

If there are any compilation errors, review the previous chapters to ensure all code is correctly implemented.

## Running in Demo Mode

### Starting the Client Demo

The client includes a local demo mode that showcases the game mechanics without requiring a server:

```bash
cargo run -p snake_client
```

This will open a window showing:
- A controllable green snake
- Orange food items scattered around
- Score tracking in the UI
- Smooth snake movement and collision detection

### Demo Controls

- **WASD** or **Arrow Keys**: Control snake direction
- **Window Close**: Exit the game

### Demo Features

The demo demonstrates:
- **Snake Movement**: Smooth, grid-based movement
- **Food Consumption**: Snake grows when eating food
- **Score Tracking**: Points increase with each food eaten
- **Boundary Handling**: Snake wraps around screen edges
- **Collision Detection**: Accurate hit detection for food

## Running the Server

The server provides the authoritative game logic for multiplayer sessions:

```bash
cargo run -p snake_server
```

The server will:
- Start the BevyGap server plugin
- Initialize game state
- Spawn a test snake for demonstration
- Begin food spawning cycles
- Handle game logic updates

### Server Features

- **Authoritative Logic**: All game decisions made server-side
- **Collision Detection**: Server handles all collision calculations
- **Food Management**: Automatic food spawning and cleanup
- **Game State Tracking**: Maintains accurate game time and statistics

## Local Development Setup

For local development without full Edgegap infrastructure:

### Option 1: Demo Mode (Current Implementation)

Run the client in demo mode to test game mechanics:

```bash
# Terminal 1: Run the demo client
cargo run -p snake_client
```

### Option 2: Local Server Connection (Future Enhancement)

For true local multiplayer, you would:

```bash
# Terminal 1: Start local server
cargo run -p snake_server

# Terminal 2: Start client and connect to local server
cargo run -p snake_client
```

Note: Full networking requires additional implementation as outlined in the previous chapters.

## Testing Game Features

### Snake Movement
1. Start the client demo
2. Use WASD keys to control the snake
3. Verify smooth movement and direction changes
4. Test boundary wrapping

### Food Consumption
1. Move the snake to orange food items
2. Confirm the snake grows when eating food
3. Check that score increases correctly
4. Observe new food spawning

### Game Mechanics
1. Test collision detection accuracy
2. Verify score tracking
3. Confirm smooth animations
4. Check game responsiveness

## Performance Testing

### Frame Rate
- Monitor FPS in both client and server
- Should maintain 60 FPS with current implementation
- Gizmo rendering is efficient for this scale

### Memory Usage
- Basic implementation should use minimal memory
- No memory leaks expected with current design
- Resource cleanup handled by Bevy

## Troubleshooting

### Common Issues

**Window doesn't open:**
- Check that all Bevy features are properly enabled
- Verify graphics drivers are up to date
- Try running with `RUST_LOG=debug` for more information

**Controls not responding:**
- Ensure window has focus
- Check keyboard input system is running
- Verify input handling code is correct

**Poor performance:**
- Check if running in debug mode (use `--release` for better performance)
- Monitor system resources
- Ensure proper optimization settings

**Compilation errors:**
- Verify all dependencies are correctly specified
- Check that Bevy features match requirements
- Ensure workspace configuration is correct

## Next Steps for Full Multiplayer

To extend this demo into a full multiplayer game:

### 1. Network Protocol Implementation

Add proper Lightyear protocol setup:

```rust
// In shared/src/lib.rs
impl ProtocolSet for SnakeProtocol {
    fn build(builder: &mut ProtocolBuilder) {
        builder.register_component::<Snake>(ChannelKind::Reliable);
        builder.register_component::<Food>(ChannelKind::Reliable);
        builder.register_component::<Player>(ChannelKind::Reliable);
        builder.register_message::<InputMessage>(ChannelKind::Unreliable);
    }
}
```

### 2. Server Connection Handling

Implement proper connection management:

```rust
// In server/src/main.rs
fn handle_connections(
    mut connection_events: EventReader<ConnectEvent>,
    mut disconnection_events: EventReader<DisconnectEvent>,
    // ... other parameters
) {
    // Handle player joins and leaves
}
```

### 3. Client Networking

Replace local game state with server communication:

```rust
// In client/src/main.rs
fn handle_server_messages(
    mut message_events: EventReader<MessageEvent<ServerMessage>>,
    // ... other parameters
) {
    // Process messages from server
}
```

### 4. BevyGap Integration

Configure full BevyGap integration:

```rust
// Configure matchmaking
.insert_resource(BevygapClientConfig {
    game_name: "snake_multiplayer".to_string(),
    game_version: "1.0.0".to_string(),
    matchmaker_url: "your_matchmaker_url".to_string(),
})
```

## Deployment with BevyGap

For production deployment:

1. **Set up NATS server** for message routing
2. **Configure Edgegap** for server hosting
3. **Deploy matchmaker services** for player coordination
4. **Configure client** to connect to production matchmaker
5. **Test full pipeline** from client to deployed server

## Conclusion

Congratulations! You've successfully created a multiplayer snake game foundation using BevyGap. The example demonstrates:

- **Complete project structure** with shared, server, and client crates
- **Game logic implementation** with proper separation of concerns
- **Visual rendering** using Bevy's graphics capabilities
- **Input handling** and user interface
- **Extensible architecture** ready for full networking

### What You've Learned

- How to structure a multiplayer game project
- BevyGap integration patterns
- Bevy game development techniques
- Networking architecture concepts
- Local development workflows

### Further Exploration

- Add more game features (power-ups, obstacles, multiple game modes)
- Implement full networking with Lightyear
- Create more sophisticated graphics and animations
- Add sound effects and music
- Implement player authentication and persistence
- Deploy to production with Edgegap

The foundation you've built is solid and ready for extension into a full multiplayer experience!

## Resources

- [BevyGap Documentation](https://rj.github.io/bevygap/)
- [Bevy Engine Guide](https://bevyengine.org/learn/)
- [Lightyear Networking](https://github.com/cBournhonesque/lightyear)
- [Edgegap Platform](https://edgegap.com/)

Happy coding! 🐍🎮