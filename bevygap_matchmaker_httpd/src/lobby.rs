use axum::{extract::{State, Path}, Json};
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH, Duration}};
use log::*;
use async_nats::client::RequestErrorKind;

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
    /// Session information when game server is deployed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_info: Option<SessionInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: Option<String>,
    pub game_server_ip: Option<String>,
    pub game_server_port: Option<u16>,
    pub connect_token: Option<String>,
    pub deployment_status: String,
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
        session_info: None,
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

pub async fn start_room(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<LobbyRoom>, (axum::http::StatusCode, String)> {
    // First, check if room exists and is not already started
    {
        let rooms = state.lobby.rooms.lock().unwrap();
        if let Some(room) = rooms.get(&id) {
            if room.started {
                return Err((StatusCode::CONFLICT, "room already started".to_string()));
            }
        } else {
            return Err((StatusCode::NOT_FOUND, "room not found".to_string()));
        }
    }

    info!("Starting room {} - deploying game server", id);

    // Use a fake client IP for room-based deployments
    // In a real implementation, this could be the host's IP or a configured deployment region
    let client_ip = state.settings.fake_ip.to_string();
    
    // Create payload for session creation - include room info
    let payload = format!("{{\"client_ip\":\"{}\", \"room_id\":\"{}\", \"game\":\"lobby-room\"}}", client_ip, id);
    
    // Send session creation request via NATS
    let request = async_nats::client::Request::new()
        .timeout(Some(Duration::from_secs(60)))
        .payload(payload.into());

    let session_result = state
        .bgnats
        .client()
        .send_request("session.gensession", request)
        .await;

    // Update room with deployment status
    let mut rooms = state.lobby.rooms.lock().unwrap();
    if let Some(room) = rooms.get_mut(&id) {
        match session_result {
            Ok(resp) => {
                // Check if there was an error in the response
                if let Some((code, msg)) = maybe_message_error(&resp) {
                    error!("Game server deployment failed for room {}: {} - {}", id, code, msg);
                    room.session_info = Some(SessionInfo {
                        session_id: None,
                        game_server_ip: None,
                        game_server_port: None,
                        connect_token: None,
                        deployment_status: format!("Failed: {}", msg),
                    });
                    return Err((
                        StatusCode::from_u16(code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        format!("Failed to deploy game server: {}", msg),
                    ));
                } else {
                    info!("Game server deployment successful for room {}", id);
                    
                    // Parse the session response to extract connection details
                    let session_response = String::from_utf8_lossy(&resp.payload);
                    let session_info = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&session_response) {
                        SessionInfo {
                            session_id: parsed.get("session_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            game_server_ip: parsed.get("gameserver_ip").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            game_server_port: parsed.get("gameserver_port").and_then(|v| v.as_u64()).map(|n| n as u16),
                            connect_token: parsed.get("connect_token").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            deployment_status: "Ready".to_string(),
                        }
                    } else {
                        SessionInfo {
                            session_id: None,
                            game_server_ip: None,
                            game_server_port: None,
                            connect_token: None,
                            deployment_status: "Ready (details pending)".to_string(),
                        }
                    };
                    
                    room.started = true;
                    room.session_info = Some(session_info);
                    info!("Room {} marked as started with deployed game server", id);
                }
            }
            Err(e) => {
                error!("NATS error deploying game server for room {}: {:?}", id, e);
                let error_msg = match e.kind() {
                    RequestErrorKind::TimedOut => "Deployment request timeout",
                    RequestErrorKind::NoResponders => "No deployment service available",
                    RequestErrorKind::Other => "Deployment service error",
                };
                
                room.session_info = Some(SessionInfo {
                    session_id: None,
                    game_server_ip: None,
                    game_server_port: None,
                    connect_token: None,
                    deployment_status: format!("Failed: {}", error_msg),
                });
                
                return Err((
                    match e.kind() {
                        RequestErrorKind::TimedOut => StatusCode::REQUEST_TIMEOUT,
                        RequestErrorKind::NoResponders => StatusCode::SERVICE_UNAVAILABLE,
                        RequestErrorKind::Other => StatusCode::INTERNAL_SERVER_ERROR,
                    },
                    error_msg.to_string(),
                ));
            }
        }
        
        Ok(Json(room.clone()))
    } else {
        Err((StatusCode::NOT_FOUND, "room not found".to_string()))
    }
}

// Helper function to check for NATS service errors (copied from main.rs)
fn maybe_message_error(message: &async_nats::Message) -> Option<(usize, String)> {
    let h = message.headers.clone()?;
    if let Some(code) = h.get(async_nats::service::NATS_SERVICE_ERROR_CODE) {
        let msg_str = h
            .get(async_nats::service::NATS_SERVICE_ERROR)
            .unwrap()
            .to_string();
        Some((code.as_str().parse::<usize>().unwrap(), msg_str))
    } else {
        None
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
