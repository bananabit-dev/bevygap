use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// Re-export commonly used types
pub use bevy::prelude::*;

// Game constants
pub const GRID_SIZE: f32 = 20.0;
pub const SNAKE_SPEED: f32 = 5.0;
pub const WORLD_WIDTH: f32 = 800.0;
pub const WORLD_HEIGHT: f32 = 600.0;
pub const FOOD_SPAWN_TIME: f32 = 2.0;

/// Represents a player's snake
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Snake {
    pub player_id: u64,
    pub segments: VecDeque<Vec2>,
    pub direction: Direction,
    pub next_direction: Direction,
    pub grow_pending: usize,
}

impl Snake {
    pub fn new(player_id: u64, start_position: Vec2) -> Self {
        let mut segments = VecDeque::new();
        segments.push_back(start_position);
        
        Self {
            player_id,
            segments,
            direction: Direction::Right,
            next_direction: Direction::Right,
            grow_pending: 0,
        }
    }

    pub fn head_position(&self) -> Vec2 {
        self.segments.front().copied().unwrap_or(Vec2::ZERO)
    }

    pub fn set_direction(&mut self, direction: Direction) {
        // Prevent moving in opposite direction
        if !self.direction.is_opposite(&direction) {
            self.next_direction = direction;
        }
    }

    pub fn grow(&mut self, segments: usize) {
        self.grow_pending += segments;
    }

    pub fn update_direction(&mut self) {
        self.direction = self.next_direction;
    }
}

/// Snake movement directions
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0.0, GRID_SIZE),
            Direction::Down => Vec2::new(0.0, -GRID_SIZE),
            Direction::Left => Vec2::new(-GRID_SIZE, 0.0),
            Direction::Right => Vec2::new(GRID_SIZE, 0.0),
        }
    }

    pub fn is_opposite(&self, other: &Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }
}

/// Food that snakes can eat
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Food {
    pub position: Vec2,
    pub value: u32,
}

impl Food {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            value: 1,
        }
    }
}

/// Player information and score
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub score: u32,
    pub is_alive: bool,
}

impl Player {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            score: 0,
            is_alive: true,
        }
    }
}

/// Game state component
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
    pub players_count: u32,
    pub food_count: u32,
    pub game_time: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            players_count: 0,
            food_count: 0,
            game_time: 0.0,
        }
    }
}

/// Input messages from client to server
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum InputMessage {
    Move(Direction),
    StartGame,
    RestartGame,
}

/// Server messages to clients
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ServerMessage {
    PlayerJoined { player_id: u64, name: String },
    PlayerLeft { player_id: u64 },
    GameStarted,
    GameOver { winner: Option<u64> },
    ScoreUpdate { player_id: u64, score: u32 },
}

/// Define our protocol for Lightyear
#[derive(Clone)]
pub struct SnakeProtocol;

// For now, we'll implement a simple protocol setup
// In a real application, you'd configure channels and message types here

/// Utility functions for the game
pub mod utils {
    use super::*;
    use rand::Rng;

    /// Generate a random position on the grid
    pub fn random_grid_position() -> Vec2 {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..((WORLD_WIDTH / GRID_SIZE) as i32)) as f32 * GRID_SIZE;
        let y = rng.gen_range(0..((WORLD_HEIGHT / GRID_SIZE) as i32)) as f32 * GRID_SIZE;
        
        // Center the position
        Vec2::new(
            x - WORLD_WIDTH / 2.0 + GRID_SIZE / 2.0,
            y - WORLD_HEIGHT / 2.0 + GRID_SIZE / 2.0,
        )
    }

    /// Check if two positions are on the same grid cell
    pub fn positions_overlap(pos1: Vec2, pos2: Vec2) -> bool {
        (pos1.x - pos2.x).abs() < GRID_SIZE / 2.0 && (pos1.y - pos2.y).abs() < GRID_SIZE / 2.0
    }

    /// Clamp position to world bounds
    pub fn clamp_to_world(position: Vec2) -> Vec2 {
        Vec2::new(
            position.x.clamp(-WORLD_WIDTH / 2.0, WORLD_WIDTH / 2.0 - GRID_SIZE),
            position.y.clamp(-WORLD_HEIGHT / 2.0, WORLD_HEIGHT / 2.0 - GRID_SIZE),
        )
    }

    /// Check if position is within world bounds
    pub fn is_in_bounds(position: Vec2) -> bool {
        position.x >= -WORLD_WIDTH / 2.0
            && position.x < WORLD_WIDTH / 2.0
            && position.y >= -WORLD_HEIGHT / 2.0
            && position.y < WORLD_HEIGHT / 2.0
    }

    /// Get spawn position for a new player
    pub fn get_spawn_position(player_count: u32) -> Vec2 {
        let spacing = GRID_SIZE * 5.0;
        let start_x = -WORLD_WIDTH / 4.0;
        let start_y = -WORLD_HEIGHT / 4.0;
        
        Vec2::new(
            start_x + (player_count as f32 * spacing) % (WORLD_WIDTH / 2.0),
            start_y + ((player_count as f32 * spacing) / (WORLD_WIDTH / 2.0)).floor() * spacing,
        )
    }
}

/// Timer for snake movement
#[derive(Resource)]
pub struct SnakeTimer(pub Timer);

impl Default for SnakeTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0 / SNAKE_SPEED, TimerMode::Repeating))
    }
}

/// Timer for food spawning
#[derive(Resource)]
pub struct FoodSpawnTimer(pub Timer);

impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(FOOD_SPAWN_TIME, TimerMode::Repeating))
    }
}

/// Game configuration
#[derive(Resource, Clone)]
pub struct GameConfig {
    pub max_players: u32,
    pub max_food: u32,
    pub world_width: f32,
    pub world_height: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            max_players: 4,
            max_food: 5,
            world_width: WORLD_WIDTH,
            world_height: WORLD_HEIGHT,
        }
    }
}