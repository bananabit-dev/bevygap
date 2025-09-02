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

#[derive(Resource, Default)]
struct InputBuffer {
    last_direction: Option<Direction>,
}

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

fn setup_demo_game(mut local_state: ResMut<LocalGameState>) {
    // Create a demo snake
    local_state.snake = Snake::new(1, Vec2::new(-100.0, 0.0));
    
    // Add some demo food
    local_state.food.push(Food::new(Vec2::new(100.0, 50.0)));
    local_state.food.push(Food::new(Vec2::new(-50.0, -100.0)));
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

fn update_demo_game(
    time: Res<Time>,
    mut local_state: ResMut<LocalGameState>,
) {
    // Simple timer for snake movement
    local_state.game_time += time.delta_secs();
    
    if local_state.game_time > 0.3 { // Move every 300ms
        local_state.game_time = 0.0;
        
        // Update snake
        local_state.snake.update_direction();
        
        let new_head = local_state.snake.head_position() + local_state.snake.direction.to_vec2();
        
        // Handle world bounds
        if utils::is_in_bounds(new_head) {
            local_state.snake.segments.push_front(new_head);
        } else {
            // Wrap around
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
        if local_state.food.len() < 3 { // Keep at least 3 food items
            local_state.food.push(Food::new(utils::random_grid_position()));
        }
    }
}

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