// Integration tests for bevygap_matchmaker_httpd lobby API endpoints
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    routing::{get, post},
    Router,
};
use bevygap_matchmaker_httpd::{CreateRoomRequest, LobbyRoom, LobbyStatus, LeaveRoomRequest, LobbyStore, HasLobby};
use serde_json;
use std::sync::Arc;
use tower::ServiceExt;

// Mock minimal AppState for testing lobby endpoints
struct TestAppState {
    pub lobby: LobbyStore,
}

impl HasLobby for TestAppState {
    fn lobby(&self) -> &LobbyStore {
        &self.lobby
    }
}

// Helper function to create a test app with just lobby routes
fn create_test_app() -> Router {
    let app_state = Arc::new(TestAppState {
        lobby: LobbyStore::new(10), // max 10 rooms
    });

    Router::new()
        .route("/lobby/api/rooms", get(bevygap_matchmaker_httpd::lobby::list_rooms::<TestAppState>).post(bevygap_matchmaker_httpd::lobby::create_room::<TestAppState>))
        .route("/lobby/api/status", get(bevygap_matchmaker_httpd::lobby::lobby_status::<TestAppState>))
        .route("/lobby/api/rooms/:id/start", post(bevygap_matchmaker_httpd::lobby::start_room::<TestAppState>))
        .route("/lobby/api/rooms/:id/leave", post(bevygap_matchmaker_httpd::lobby::leave_room::<TestAppState>))
        .with_state(app_state)
}

#[tokio::test]
async fn test_lobby_status() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/lobby/api/status")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let status: LobbyStatus = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(status.max_rooms, 10);
    assert_eq!(status.active_rooms, 0);
    assert_eq!(status.total_rooms, 0);
}

#[tokio::test]
async fn test_list_empty_rooms() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/lobby/api/rooms")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let rooms: Vec<LobbyRoom> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(rooms.len(), 0);
}

#[tokio::test]
async fn test_create_room() {
    let app = create_test_app();

    let create_request = CreateRoomRequest {
        host_name: "TestHost".to_string(),
        game_mode: "FreeForAll".to_string(),
        max_players: Some(4),
    };

    let response = app
        .oneshot(
            Request::builder()
                .uri("/lobby/api/rooms")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&create_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let room: LobbyRoom = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(room.host_name, "TestHost");
    assert_eq!(room.game_mode, "FreeForAll");
    assert_eq!(room.max_players, 4);
    assert_eq!(room.current_players, 1);
    assert_eq!(room.started, false);
    assert!(room.id.starts_with("ROOM"));
}

#[tokio::test]
async fn test_start_room_not_found() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/lobby/api/rooms/NONEXISTENT/start")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_leave_room_not_found() {
    let app = create_test_app();

    let leave_request = LeaveRoomRequest {
        player_name: Some("TestPlayer".to_string()),
    };

    let response = app
        .oneshot(
            Request::builder()
                .uri("/lobby/api/rooms/NONEXISTENT/leave")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&leave_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_full_lobby_workflow() {
    let app = create_test_app();

    // 1. Check initial status
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/lobby/api/status")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let initial_status: LobbyStatus = serde_json::from_slice(&body).unwrap();
    assert_eq!(initial_status.active_rooms, 0);

    // 2. Create a room
    let create_request = CreateRoomRequest {
        host_name: "WorkflowHost".to_string(),
        game_mode: "Capture".to_string(),
        max_players: Some(6),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/lobby/api/rooms")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&create_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_room: LobbyRoom = serde_json::from_slice(&body).unwrap();
    let room_id = created_room.id.clone();

    // 3. List rooms - should show our new room
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/lobby/api/rooms")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let rooms: Vec<LobbyRoom> = serde_json::from_slice(&body).unwrap();
    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0].id, room_id);

    // 4. Start the room
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/lobby/api/rooms/{}/start", room_id))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. List rooms - should be empty now (started rooms are filtered out)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/lobby/api/rooms")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let rooms: Vec<LobbyRoom> = serde_json::from_slice(&body).unwrap();
    assert_eq!(rooms.len(), 0); // Started rooms are filtered out from list
}

#[tokio::test]
async fn test_max_rooms_limit() {
    let app = create_test_app();

    // Create rooms up to the limit (10)
    for i in 1..=10 {
        let create_request = CreateRoomRequest {
            host_name: format!("Host{}", i),
            game_mode: "Test".to_string(),
            max_players: Some(4),
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/lobby/api/rooms")
                    .method("POST")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&create_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // Try to create one more room - should fail
    let create_request = CreateRoomRequest {
        host_name: "TooManyHost".to_string(),
        game_mode: "Test".to_string(),
        max_players: Some(4),
    };

    let response = app
        .oneshot(
            Request::builder()
                .uri("/lobby/api/rooms")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&create_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}