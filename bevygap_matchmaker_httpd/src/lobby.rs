use axum::{extract::{State, Path}, Json};
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};
use log::*;

use crate::AppState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LobbyRoom {
    pub id: String,
    pub host_name: String,
    pub game_mode: String,
    pub created_at: u64,
    pub started: bool,
    pub current_players: u32,
    pub max_players: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub host_name: String,
    pub game_mode: String,
    #[serde(default)]
    pub max_players: Option<u32>,
}

#[derive(Default)]
pub struct LobbyStore {
    pub rooms: Mutex<HashMap<String, LobbyRoom>>, // id -> room
    pub max_rooms: usize,
}

impl LobbyStore {
    pub fn new(max_rooms: usize) -> Self { Self { rooms: Mutex::new(HashMap::new()), max_rooms } }
}

#[derive(Clone, Debug, Serialize)]
pub struct LobbyStatus {
    pub max_rooms: usize,
    pub active_rooms: usize,
    pub total_rooms: usize,
}

fn now_secs() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() }

pub async fn list_rooms(State(state): State<Arc<AppState>>) -> Json<Vec<LobbyRoom>> {
    let rooms = state.lobby.rooms.lock().unwrap();
    let mut v: Vec<LobbyRoom> = rooms.values().filter(|r| !r.started).cloned().collect();
    v.sort_by_key(|r| r.created_at);
    Json(v)
}

pub async fn create_room(State(state): State<Arc<AppState>>, Json(req): Json<CreateRoomRequest>) -> Result<Json<LobbyRoom>, (axum::http::StatusCode, String)> {
    let mut rooms = state.lobby.rooms.lock().unwrap();
    let max = state.lobby.max_rooms;
    let active_count = rooms.values().filter(|r| !r.started).count();
    if active_count >= max { 
        return Err((axum::http::StatusCode::TOO_MANY_REQUESTS, format!("maximum active rooms reached ({})", max)));
    }

    let id = format!("ROOM{:03}", (rooms.len() as u32 + 1));
    let room = LobbyRoom { 
        id: id.clone(),
        host_name: req.host_name,
        game_mode: req.game_mode,
        created_at: now_secs(),
        started: false,
        current_players: 1,
        max_players: req.max_players.unwrap_or(4).min(16),
    };
    rooms.insert(id.clone(), room.clone());
    info!("Created lobby room {}", id);
    Ok(Json(room))
}

pub async fn lobby_status(State(state): State<Arc<AppState>>) -> Json<LobbyStatus> {
    let rooms = state.lobby.rooms.lock().unwrap();
    let total = rooms.len();
    let active = rooms.values().filter(|r| !r.started).count();
    Json(LobbyStatus { max_rooms: state.lobby.max_rooms, active_rooms: active, total_rooms: total })
}

pub async fn start_room(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Result<(), (axum::http::StatusCode, String)> {
    let mut rooms = state.lobby.rooms.lock().unwrap();
    if let Some(room) = rooms.get_mut(&id) {
        room.started = true;
        info!("Room {} marked as started", id);
        Ok(())
    } else {
        Err((axum::http::StatusCode::NOT_FOUND, "room not found".to_string()))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct JoinRoomRequest {
    pub player_name: Option<String>,
}

pub async fn join_room(State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(_req): Json<JoinRoomRequest>) -> Result<Json<LobbyRoom>, (StatusCode, String)> {
    let mut rooms = state.lobby.rooms.lock().unwrap();
    if let Some(room) = rooms.get_mut(&id) {
        if room.started {
            return Err((StatusCode::CONFLICT, "room already started".to_string()));
        }
        if room.current_players >= room.max_players {
            return Err((StatusCode::CONFLICT, "room full".to_string()));
        }
        room.current_players += 1;
        info!("Player joined room {}, current players {}", id, room.current_players);
        Ok(Json(room.clone()))
    } else {
        Err((StatusCode::NOT_FOUND, "room not found".to_string()))
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct LeaveRoomRequest {
    pub player_name: Option<String>,
}

pub async fn leave_room(State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(_req): Json<LeaveRoomRequest>) -> Result<StatusCode, (StatusCode, String)> {
    let mut rooms = state.lobby.rooms.lock().unwrap();
    if let Some(room) = rooms.get_mut(&id) {
        if room.current_players > 0 { room.current_players -= 1; }
        info!("Player left room {}, current players {}", id, room.current_players);
        if room.current_players == 0 && !room.started {
            rooms.remove(&id);
            info!("Removed empty not-started room {}", id);
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "room not found".to_string()))
    }
}
