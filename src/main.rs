use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod glub_server;
pub mod glub_server_storage;

use glub_server_storage::*;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Create shared game storage
    let storage = Arc::new(RwLock::new(GameStorage::new()));

    // Start the move increment task
    let storage_clone = Arc::clone(&storage);
    tokio::spawn(async move {
        move_increment_task(storage_clone).await;
    });

    // build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/join_queue", post(join_queue))
        .route("/game/{game_id}/board/{player_id}", get(get_board))
        .route("/game/{game_id}/move", post(make_move))
        .route("/game/{game_i}/status", get(get_game_status))
        .with_state(storage);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Chess server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Chess Server with Fog of War - Ready!"
}

// Join the matchmaking queue
async fn join_queue(
    State(storage): State<Arc<RwLock<GameStorage>>>,
    Json(payload): Json<JoinQueueRequest>,
) -> Result<Json<JoinQueueResponse>, StatusCode> {
    let mut storage = storage.write().await;

    match storage.join_queue(payload.player_name) {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Get board state with fog of war applied
async fn get_board(
    State(storage): State<Arc<RwLock<GameStorage>>>,
    Path((game_id, player_id)): Path<(String, String)>,
) -> Result<Json<FoggedBoard>, StatusCode> {
    let game_id = Uuid::parse_str(&game_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let player_id = Uuid::parse_str(&player_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let storage = storage.read().await;

    match storage.get_fogged_board(game_id, player_id) {
        Ok(board) => Ok(Json(board)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// Make a move
async fn make_move(
    State(storage): State<Arc<RwLock<GameStorage>>>,
    Path(game_id): Path<String>,
    Json(payload): Json<MoveRequest>,
) -> Result<Json<MoveResponse>, StatusCode> {
    let game_id = Uuid::parse_str(&game_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut storage = storage.write().await;

    match storage.make_move(game_id, payload) {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            println!("Move error: {:?}", err);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// Get game status
async fn get_game_status(
    State(storage): State<Arc<RwLock<GameStorage>>>,
    Path(game_id): Path<String>,
) -> Result<Json<GameStatus>, StatusCode> {
    let game_id = Uuid::parse_str(&game_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let storage = storage.read().await;

    match storage.get_game_status(game_id) {
        Ok(status) => Ok(Json(status)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// Task that increments move points every second
async fn move_increment_task(storage: Arc<RwLock<GameStorage>>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        interval.tick().await;
        let mut storage = storage.write().await;
        storage.increment_moves();
    }
}

// Request/Response types
#[derive(Deserialize)]
pub struct JoinQueueRequest {
    pub player_name: String,
}

#[derive(Serialize)]
pub struct JoinQueueResponse {
    pub player_id: Uuid,
    pub game_id: Option<Uuid>,
    pub message: String,
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub player_id: Uuid,
    pub from: (usize, usize),
    pub to: (usize, usize),
}

#[derive(Serialize)]
pub struct MoveResponse {
    pub success: bool,
    pub message: String,
    pub remaining_moves: u64,
}

#[derive(Serialize)]
pub struct GameStatus {
    pub game_id: Uuid,
    pub player1_moves: u64,
    pub player2_moves: u64,
    pub current_turn: Option<Uuid>,
}
