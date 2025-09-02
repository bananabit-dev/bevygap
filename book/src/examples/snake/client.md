# Client Implementation

In this chapter, we'll implement the game client that provides a visual interface for players and handles user input.

## Client Architecture

The client is responsible for:
- **User Interface**: Displaying the game world and UI elements
- **Input Handling**: Capturing player input and sending it to the server
- **Rendering**: Drawing snakes, food, and game information
- **Connection Management**: Connecting to the server through BevyGap

## Basic Client Setup

Let's set up the basic client structure in `client/src/main.rs`:

```rust
use bevy::prelude::*;
use bevygap_client_plugin::prelude::*;
use snake_shared::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake Multiplayer - Demo".to_string(),
                resolution: (WORLD_WIDTH + 100.0, WORLD_HEIGHT + 200.0).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BevygapClientPlugin)
        .insert_resource(BevygapClientConfig::default())
        .add_plugins(SnakeClientPlugin)
        .run();
}

pub struct SnakeClientPlugin;

impl Plugin for SnakeClientPlugin {
    fn build(&self, app: &mut App) {
        // Add resources
        app.init_resource::<GameConfig>()
            .init_resource::<InputBuffer>()
            .init_resource::<LocalGameState>();

        // Add systems
        app.add_systems(Startup, (setup_camera, setup_ui, setup_demo_game));
        
        app.add_systems(
            Update,
            (
                handle_input,
                update_demo_game,
                render_game,
            ),
        );
    }
}
```

## Local Game State

For this demo, we'll maintain a local game state to show how the game logic works:

```rust
#[derive(Resource)]
struct LocalGameState {
    snake: Snake,
    food: Vec<Food>,
    score: u32,
    game_time: f32,
}

impl Default for LocalGameState {
    fn default() -> Self {
        Self {
            snake: Snake::new(1, Vec2::ZERO), // Will be set in setup
            food: Vec::new(),
            score: 0,
            game_time: 0.0,
        }
    }
}
```

## Camera and UI Setup

Set up the camera and basic UI:

```rust
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_ui(mut commands: Commands) {
    // Simple UI text
    commands.spawn((
        Text::new("Snake Demo - Use WASD to move | Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));
}

#[derive(Component)]
struct ScoreText;
```

## Input Handling

Capture player input and apply it to the local snake:

```rust
#[derive(Resource, Default)]
struct InputBuffer {
    last_direction: Option<Direction>,
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut input_buffer: ResMut<InputBuffer>,
    mut local_state: ResMut<LocalGameState>,
) {
    let mut new_direction = None;

    if keyboard_input.just_pressed(KeyCode::KeyW) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
        new_direction = Some(Direction::Up);
    } else if keyboard_input.just_pressed(KeyCode::KeyS) || keyboard_input.just_pressed(KeyCode::ArrowDown) {
        new_direction = Some(Direction::Down);
    } else if keyboard_input.just_pressed(KeyCode::KeyA) || keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        new_direction = Some(Direction::Left);
    } else if keyboard_input.just_pressed(KeyCode::KeyD) || keyboard_input.just_pressed(KeyCode::ArrowRight) {
        new_direction = Some(Direction::Right);
    }

    // Update snake direction
    if let Some(direction) = new_direction {
        if input_buffer.last_direction != Some(direction) {
            input_buffer.last_direction = Some(direction);
            local_state.snake.set_direction(direction);
        }
    }
}
```

## Game Logic Update

Update the demo game state locally:

```rust
fn update_demo_game(
    time: Res<Time>,
    mut local_state: ResMut<LocalGameState>,
) {
    // Simple timer for snake movement
    local_state.game_time += time.delta_secs();
    
    if local_state.game_time > 0.3 { // Move every 300ms
        local_state.game_time = 0.0;
        
        // Update snake movement
        local_state.snake.update_direction();
        
        let new_head = local_state.snake.head_position() + local_state.snake.direction.to_vec2();
        
        // Handle world bounds (wrap around)
        if utils::is_in_bounds(new_head) {
            local_state.snake.segments.push_front(new_head);
        } else {
            let wrapped = utils::clamp_to_world(new_head);
            local_state.snake.segments.push_front(wrapped);
        }
        
        // Check food collision
        let head_pos = local_state.snake.head_position();
        let mut ate_food = false;
        let mut score_increase = 0;
        
        local_state.food.retain(|food| {
            if utils::positions_overlap(head_pos, food.position) {
                ate_food = true;
                score_increase = food.value;
                false // Remove this food
            } else {
                true // Keep this food
            }
        });
        
        // Apply food effects
        if ate_food {
            local_state.snake.grow(1);
            local_state.score += score_increase;
        }
        
        // Remove tail if not growing
        if local_state.snake.grow_pending > 0 {
            local_state.snake.grow_pending -= 1;
        } else {
            local_state.snake.segments.pop_back();
        }
        
        // Spawn new food if needed
        if local_state.food.len() < 3 {
            local_state.food.push(Food::new(utils::random_grid_position()));
        }
    }
}
```

## Rendering System

Render the game world using Bevy's gizmos:

```rust
fn render_game(
    local_state: Res<LocalGameState>,
    mut gizmos: Gizmos,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    // Render snake
    let color = Color::srgb(0.0, 1.0, 0.0); // Green
    
    for (i, segment) in local_state.snake.segments.iter().enumerate() {
        let size = if i == 0 { GRID_SIZE * 0.9 } else { GRID_SIZE * 0.8 };
        let segment_color = if i == 0 { color } else { color.with_alpha(0.8) };
        
        gizmos.rect_2d(
            *segment,
            Vec2::splat(size),
            segment_color,
        );
    }
    
    // Render food
    for food in local_state.food.iter() {
        gizmos.circle_2d(
            food.position,
            GRID_SIZE * 0.4,
            Color::srgb(1.0, 0.5, 0.0), // Orange
        );
    }
    
    // Update score text
    if let Ok(mut text) = score_text_query.single_mut() {
        **text = format!("Snake Demo - Use WASD to move | Score: {} | Ready for Networking!", local_state.score);
    }
}
```

## Adding Network Support

For full multiplayer functionality, you would add connection handling:

```rust
// These systems would be added for full networking:
fn connect_to_server(mut commands: Commands) {
    // Use BevyGap to connect to the server
    commands.bevygap_connect_client();
}

fn handle_connection_state(
    state: Res<State<BevygapClientState>>,
    // Handle different connection states
    // Show appropriate UI feedback
) {
    // Implementation would go here
}

fn send_input_to_server(
    // Send player input to the server instead of applying locally
    // Use Lightyear's message system
) {
    // Implementation would go here
}

fn receive_game_state(
    // Receive authoritative game state from server
    // Update local rendering based on server data
) {
    // Implementation would go here
}
```

## Testing the Client

You can test the client demo:

```bash
cargo run -p snake_client
```

The client will:
1. Display a window with the game
2. Show a controllable snake
3. Spawn food that can be eaten
4. Track score
5. Demonstrate game mechanics

## Key Features

### Input Responsiveness
- Immediate input feedback for smooth controls
- Direction changes are buffered to prevent invalid moves
- Support for both WASD and arrow keys

### Visual Feedback
- Clear snake rendering with head/body distinction
- Attractive food rendering
- Real-time score updates
- Visual indication of game state

### Local Demo Mode
- Fully functional single-player version
- Shows how multiplayer would work
- Easy to extend with networking

## Performance Considerations

### Efficient Rendering
- Uses Bevy's gizmo system for simple shapes
- Minimal draw calls
- Smooth animations

### Responsive Input
- Input handled every frame
- Separate from game logic timing
- No input lag

## What's Next?

Now that we have both client and server implementations, let's move on to [Running the Game](./running.md) where we'll learn how to:

- Run the server and client
- Test the local demo mode
- Prepare for full networking
- Deploy with BevyGap