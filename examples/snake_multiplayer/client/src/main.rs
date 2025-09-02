use bevy::prelude::*;
use bevygap_client_plugin::prelude::*;
use lightyear::prelude::*;
use snake_shared::*;

fn main() {
    env_logger::init();
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake Multiplayer - Client".to_string(),
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
        // Add Lightyear client plugin
        app.add_plugins(
            ClientPlugin::new(NetConfig::Local { auth: ClientAuthentication::Unsecured })
                .with_protocol(build_client_protocol())
        );

        // Add resources
        app.init_resource::<GameConfig>()
            .init_resource::<InputBuffer>();

        // Add systems
        app.add_systems(Startup, (setup_camera, setup_ui));
        
        app.add_systems(
            Update,
            (
                handle_input,
                render_snakes,
                render_food,
                update_ui,
                handle_connection_state,
            ),
        );

        // Connect to server on startup
        app.add_systems(Startup, connect_to_server);
    }
}

#[derive(Resource, Default)]
struct InputBuffer {
    last_direction: Option<Direction>,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Root UI node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Top info panel
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Snake Multiplayer - Use WASD to move",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 24.0,
                                color: Color::WHITE,
                            },
                        ),
                        GameInfoText,
                    ));
                });

            // Bottom score panel
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Score: 0 | Players: 0",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        ),
                        ScoreText,
                    ));
                });
        });
}

#[derive(Component)]
struct GameInfoText;

#[derive(Component)]
struct ScoreText;

fn connect_to_server(mut commands: Commands) {
    // For local development, we'll bypass the full BevyGap flow
    // and connect directly to a local server
    info!("Attempting to connect to server...");
    commands.bevygap_connect_client();
}

fn handle_connection_state(
    state: Res<State<BevygapClientState>>,
    mut next_state: ResMut<NextState<BevygapClientState>>,
) {
    match state.get() {
        BevygapClientState::Disconnected => {
            info!("Client is disconnected");
        }
        BevygapClientState::Request => {
            info!("Requesting connection...");
        }
        BevygapClientState::AwaitingResponse(_) => {
            info!("Awaiting server response...");
        }
        BevygapClientState::ReadyToConnect => {
            info!("Ready to connect to game server");
        }
        BevygapClientState::Finished => {
            info!("Connection attempt finished");
        }
        BevygapClientState::Error(code, msg) => {
            error!("Connection error {}: {}", code, msg);
            // For local development, retry connection
            // In production, you might want different behavior
        }
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut input_buffer: ResMut<InputBuffer>,
    mut client: ResMut<Client>,
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

    // Send input to server if direction changed
    if let Some(direction) = new_direction {
        if input_buffer.last_direction != Some(direction) {
            input_buffer.last_direction = Some(direction);
            
            if client.is_connected() {
                let message = InputMessage::Move(direction);
                if let Err(e) = client.send_message(message) {
                    error!("Failed to send input message: {:?}", e);
                }
            }
        }
    }

    // Handle other inputs
    if keyboard_input.just_pressed(KeyCode::Space) {
        if client.is_connected() {
            let message = InputMessage::StartGame;
            if let Err(e) = client.send_message(message) {
                error!("Failed to send start game message: {:?}", e);
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        if client.is_connected() {
            let message = InputMessage::RestartGame;
            if let Err(e) = client.send_message(message) {
                error!("Failed to send restart game message: {:?}", e);
            }
        }
    }
}

fn render_snakes(
    mut commands: Commands,
    snake_query: Query<(Entity, &Snake), (With<Replicated>, Added<Snake>)>,
    existing_snakes: Query<Entity, (With<SnakeVisual>, Without<Snake>)>,
    mut gizmos: Gizmos,
    all_snakes: Query<&Snake, With<Replicated>>,
) {
    // Clean up old visuals
    for entity in existing_snakes.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Render all snakes
    for snake in all_snakes.iter() {
        let color = get_player_color(snake.player_id);
        
        // Render snake segments
        for (i, segment) in snake.segments.iter().enumerate() {
            let size = if i == 0 { GRID_SIZE * 0.9 } else { GRID_SIZE * 0.8 }; // Head slightly larger
            let segment_color = if i == 0 { color } else { color.with_alpha(0.8) };
            
            gizmos.rect_2d(
                Vec2::new(segment.x, segment.y),
                0.0,
                Vec2::splat(size),
                segment_color,
            );
        }
    }
}

fn render_food(
    food_query: Query<&Food, With<Replicated>>,
    mut gizmos: Gizmos,
) {
    for food in food_query.iter() {
        gizmos.circle_2d(
            food.position,
            GRID_SIZE * 0.4,
            Color::srgb(1.0, 0.5, 0.0), // Orange color for food
        );
    }
}

fn update_ui(
    game_state_query: Query<&GameState, With<Replicated>>,
    player_query: Query<&Player, With<Replicated>>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    client: Res<Client>,
) {
    if let Ok(mut text) = score_text_query.get_single_mut() {
        let mut display_text = String::new();
        
        // Find our player's score
        let mut our_score = 0;
        if let Some(client_id) = client.id() {
            // In a real implementation, you'd need to track which player entity belongs to this client
            // For now, we'll just show the first player's score
            if let Some(player) = player_query.iter().next() {
                our_score = player.score;
            }
        }
        
        // Get game state info
        let players_count = if let Ok(game_state) = game_state_query.get_single() {
            game_state.players_count
        } else {
            0
        };
        
        display_text.push_str(&format!("Score: {} | Players: {}", our_score, players_count));
        
        if !client.is_connected() {
            display_text.push_str(" | Disconnected");
        }
        
        text.sections[0].value = display_text;
    }
}

#[derive(Component)]
struct SnakeVisual;

fn get_player_color(player_id: u64) -> Color {
    match player_id % 4 {
        0 => Color::srgb(0.0, 1.0, 0.0), // Green
        1 => Color::srgb(0.0, 0.0, 1.0), // Blue
        2 => Color::srgb(1.0, 1.0, 0.0), // Yellow
        3 => Color::srgb(1.0, 0.0, 1.0), // Magenta
        _ => Color::WHITE,
    }
}