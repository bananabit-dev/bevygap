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
            )
                .chain(),
        );

        // Start with a default game state
        app.add_systems(Startup, setup_game);
    }
}

#[derive(Resource, Default)]
struct PlayerRegistry {
    players: HashMap<u64, Entity>,
    next_player_id: u64,
}

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

fn update_snakes(
    time: Res<Time>,
    mut snake_timer: ResMut<SnakeTimer>,
    mut snake_query: Query<&mut Snake>,
    mut game_state_query: Query<&mut GameState>,
) {
    snake_timer.0.tick(time.delta());
    
    if snake_timer.0.just_finished() {
        for mut snake in snake_query.iter_mut() {
            // Update direction
            snake.update_direction();
            
            // Move snake
            let new_head = snake.head_position() + snake.direction.to_vec2();
            
            // Check world bounds
            if !utils::is_in_bounds(new_head) {
                // Snake hit wall - for now just wrap around
                let wrapped_head = utils::clamp_to_world(new_head);
                snake.segments.push_front(wrapped_head);
            } else {
                snake.segments.push_front(new_head);
            }
            
            // Remove tail if not growing
            if snake.grow_pending > 0 {
                snake.grow_pending -= 1;
            } else {
                snake.segments.pop_back();
            }
        }
        
        // Update game time
        if let Ok(mut game_state) = game_state_query.single_mut() {
            game_state.game_time += snake_timer.0.duration().as_secs_f32();
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    mut snake_query: Query<(Entity, &mut Snake, &mut Player)>,
    food_query: Query<(Entity, &Food)>,
    mut game_state_query: Query<&mut GameState>,
) {
    let mut snakes: Vec<_> = snake_query.iter_mut().collect();
    
    // Check food collisions
    for (_snake_entity, snake, player) in snakes.iter_mut() {
        let head_pos = snake.head_position();
        
        for (food_entity, food) in food_query.iter() {
            if utils::positions_overlap(head_pos, food.position) {
                // Snake ate food
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
    
    // Check snake self-collision
    for (_snake_entity, snake, player) in snakes.iter_mut() {
        let head_pos = snake.head_position();
        
        // Check collision with own body (skip head)
        for segment in snake.segments.iter().skip(1) {
            if utils::positions_overlap(head_pos, *segment) {
                player.is_alive = false;
                info!("Player {} died from self-collision!", player.id);
                // Reset snake to spawn position
                snake.segments.clear();
                snake.segments.push_back(utils::get_spawn_position(0));
                snake.grow_pending = 0;
                break;
            }
        }
    }
}

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
        // Find a position that doesn't overlap with snakes
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
                if !valid {
                    break;
                }
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