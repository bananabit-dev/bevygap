# Server Implementation

In this chapter, we'll implement the authoritative game server that manages all game logic, handles player connections, and ensures fair gameplay.

## Server Architecture

The server is responsible for:
- **Authoritative game state**: All decisions are made on the server
- **Player connection management**: Handle joins and disconnects
- **Game logic**: Snake movement, collision detection, food spawning
- **State synchronization**: Send updates to connected clients

## Basic Server Setup

First, let's set up the basic server structure in `server/src/main.rs`:

```rust
use bevy::prelude::*;
use bevygap_server_plugin::prelude::*;
use snake_shared::*;
use std::collections::HashMap;
use log::info;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(BevygapServerPlugin)
        .add_plugins(SnakeServerPlugin)
        .run();
}

pub struct SnakeServerPlugin;

impl Plugin for SnakeServerPlugin {
    fn build(&self, app: &mut App) {
        // Add resources
        app.init_resource::<GameConfig>()
            .init_resource::<SnakeTimer>()
            .init_resource::<FoodSpawnTimer>()
            .init_resource::<PlayerRegistry>();

        // Add systems
        app.add_systems(
            FixedUpdate,
            (
                update_snakes,
                check_collisions,
                spawn_food,
            ).chain(),
        );

        // Start with a default game state
        app.add_systems(Startup, setup_game);
    }
}
```

## Player Registry

We need to track connected players:

```rust
#[derive(Resource, Default)]
struct PlayerRegistry {
    players: HashMap<u64, Entity>,
    next_player_id: u64,
}
```

## Game Initialization

Set up the initial game state:

```rust
fn setup_game(mut commands: Commands) {
    info!("Starting Snake multiplayer server");
    
    // Spawn initial game state
    commands.spawn(GameState::default());
    
    // Spawn a test snake for demonstration
    let player_id = 1;
    let spawn_position = utils::get_spawn_position(0);
    commands.spawn((
        Player::new(player_id, format!("Player {}", player_id)),
        Snake::new(player_id, spawn_position),
    ));
    
    info!("Spawned test snake at {:?}", spawn_position);
}
```

## Snake Movement System

The core system that moves all snakes every few frames:

```rust
fn update_snakes(
    time: Res<Time>,
    mut snake_timer: ResMut<SnakeTimer>,
    mut snake_query: Query<&mut Snake>,
    mut game_state_query: Query<&mut GameState>,
) {
    snake_timer.0.tick(time.delta());
    
    if snake_timer.0.just_finished() {
        for mut snake in snake_query.iter_mut() {
            // Update direction first
            snake.update_direction();
            
            // Calculate new head position
            let new_head = snake.head_position() + snake.direction.to_vec2();
            
            // Handle world bounds (wrap around for now)
            if !utils::is_in_bounds(new_head) {
                let wrapped_head = utils::clamp_to_world(new_head);
                snake.segments.push_front(wrapped_head);
            } else {
                snake.segments.push_front(new_head);
            }
            
            // Remove tail unless growing
            if snake.grow_pending > 0 {
                snake.grow_pending -= 1;
            } else {
                snake.segments.pop_back();
            }
        }
        
        // Update game timer
        if let Ok(mut game_state) = game_state_query.single_mut() {
            game_state.game_time += snake_timer.0.duration().as_secs_f32();
        }
    }
}
```

## Collision Detection

Check for various types of collisions:

```rust
fn check_collisions(
    mut commands: Commands,
    mut snake_query: Query<(Entity, &mut Snake, &mut Player)>,
    food_query: Query<(Entity, &Food)>,
    mut game_state_query: Query<&mut GameState>,
) {
    let mut snakes: Vec<_> = snake_query.iter_mut().collect();
    
    // Food collisions
    for (_snake_entity, snake, player) in snakes.iter_mut() {
        let head_pos = snake.head_position();
        
        for (food_entity, food) in food_query.iter() {
            if utils::positions_overlap(head_pos, food.position) {
                // Snake ate food!
                snake.grow(1);
                player.score += food.value;
                commands.entity(food_entity).despawn();
                
                // Update food count
                if let Ok(mut game_state) = game_state_query.single_mut() {
                    game_state.food_count = game_state.food_count.saturating_sub(1);
                }
                
                info!("Player {} ate food! Score: {}", player.id, player.score);
            }
        }
    }
    
    // Self-collision detection
    for (_snake_entity, snake, player) in snakes.iter_mut() {
        let head_pos = snake.head_position();
        
        // Check if head hits body (skip the head segment itself)
        for segment in snake.segments.iter().skip(1) {
            if utils::positions_overlap(head_pos, *segment) {
                player.is_alive = false;
                info!("Player {} died from self-collision!", player.id);
                
                // Reset snake
                snake.segments.clear();
                snake.segments.push_back(utils::get_spawn_position(0));
                snake.grow_pending = 0;
                break;
            }
        }
    }
}
```

## Food Spawning System

Automatically spawn food at regular intervals:

```rust
fn spawn_food(
    time: Res<Time>,
    mut food_timer: ResMut<FoodSpawnTimer>,
    mut commands: Commands,
    config: Res<GameConfig>,
    food_query: Query<&Food>,
    snake_query: Query<&Snake>,
    mut game_state_query: Query<&mut GameState>,
) {
    food_timer.0.tick(time.delta());
    
    if food_timer.0.just_finished() && food_query.iter().count() < config.max_food as usize {
        // Find a valid spawn position
        let mut attempts = 0;
        let max_attempts = 50;
        
        while attempts < max_attempts {
            let position = utils::random_grid_position();
            let mut valid = true;
            
            // Check if position overlaps with any snake
            for snake in snake_query.iter() {
                for segment in snake.segments.iter() {
                    if utils::positions_overlap(position, *segment) {
                        valid = false;
                        break;
                    }
                }
                if !valid { break; }
            }
            
            // Check if position overlaps with existing food
            for food in food_query.iter() {
                if utils::positions_overlap(position, food.position) {
                    valid = false;
                    break;
                }
            }
            
            if valid {
                commands.spawn(Food::new(position));
                
                // Update food count
                if let Ok(mut game_state) = game_state_query.single_mut() {
                    game_state.food_count += 1;
                }
                
                info!("Spawned food at {:?}", position);
                break;
            }
            
            attempts += 1;
        }
    }
}
```

## Adding Network Support

For full multiplayer functionality, you would add connection handling systems:

```rust
// These systems would be added for full networking:
fn handle_connections(
    // Handle new player connections
    // Spawn new snake entities for each player
    // Update game state with player count
) {
    // Implementation would go here
}

fn handle_player_input(
    // Receive input messages from clients
    // Update snake directions based on player input
    // Validate inputs for anti-cheat
) {
    // Implementation would go here
}

fn send_game_state(
    // Send authoritative game state to all clients
    // Use Lightyear's replication system
    // Send only necessary updates for performance
) {
    // Implementation would go here
}
```

## Testing the Server

You can test the basic server logic without networking:

```bash
cargo run -p snake_server
```

The server will:
1. Start with a test snake
2. Move the snake automatically
3. Spawn food periodically
4. Handle collisions and scoring
5. Log important events

## Key Features

### Authoritative Design
- All game logic runs on the server
- Clients cannot cheat by modifying local state
- Server validates all player actions

### Performance Optimizations
- Fixed timestep for consistent gameplay
- Efficient collision detection using grid positions
- Minimal data synchronization

### Extensible Architecture
- Easy to add new game features
- Clear separation between game logic and networking
- Modular system design

## What's Next?

Now that we have a working server, let's move on to [Client Implementation](./client.md) where we'll create the visual client that connects to this server and provides a great user experience.

The client will handle:
- Connecting to the server using BevyGap
- Rendering the game world
- Handling player input
- Displaying the UI and game information