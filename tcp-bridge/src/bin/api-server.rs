//! HTTP REST API server for Rodecaster control.
//! Sends JSON commands to the proxy via Unix socket.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

use tcp_bridge::commands::{Command, MixAction};
use tcp_bridge::names::{MixOutput, Source, Fader};

const SOCKET_PATH: &str = "/tmp/socket_bridge_control";

#[derive(Clone)]
struct AppState {
    socket_path: String,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ApiResponse {
    fn ok(msg: &str) -> Self {
        Self { success: true, message: Some(msg.to_string()), error: None }
    }
    fn err(msg: &str) -> Self {
        Self { success: false, message: None, error: Some(msg.to_string()) }
    }
}

#[derive(Deserialize)]
struct MixRequest {
    action: MixAction,
    mix: MixOutput,
    source: Source,
}

#[derive(Deserialize)]
struct FaderRequest {
    fader: Fader,
    #[serde(default)]
    muted: Option<bool>,
    #[serde(default)]
    source: Option<Source>,
    #[serde(default)]
    level: Option<f32>,
}

async fn send_command(socket_path: &str, cmd: &Command) -> Result<(), String> {
    let json = serde_json::to_string(cmd)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    let mut stream = UnixStream::connect(socket_path).await
        .map_err(|e| format!("Connection error: {}", e))?;
    
    stream.write_all(json.as_bytes()).await
        .map_err(|e| format!("Write error: {}", e))?;
    
    Ok(())
}

async fn health() -> Json<ApiResponse> {
    Json(ApiResponse::ok("API server running"))
}

async fn mix_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MixRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let cmd = Command::Mix { action: req.action, mix: req.mix, source: req.source };
    let msg = format!("{:?} {} in {}", req.action, req.source, req.mix);
    
    match send_command(&state.socket_path, &cmd).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(&msg))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(&e))),
    }
}

async fn fader_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FaderRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let cmd = Command::Fader { 
        fader: req.fader, 
        muted: req.muted, 
        source: req.source, 
        level: req.level 
    };
    
    let msg = format!("Updated {}", req.fader);
    
    match send_command(&state.socket_path, &cmd).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(&msg))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(&e))),
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState { socket_path: SOCKET_PATH.to_string() });

    let app = Router::new()
        .route("/health", get(health))
        .route("/mix", post(mix_handler))
        .route("/fader", post(fader_handler))
        .with_state(state);

    let addr = "0.0.0.0:8080";
    println!("[API] Starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
