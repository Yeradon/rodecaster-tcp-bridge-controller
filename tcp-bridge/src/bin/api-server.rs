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

mod names {
    include!("../names.rs");
}

use names::{MixOutput, Source, Fader};

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
    fn ok(message: &str) -> Self {
        Self { success: true, message: Some(message.to_string()), error: None }
    }

    fn err(error: &str) -> Self {
        Self { success: false, message: None, error: Some(error.to_string()) }
    }
}

// Unified request bodies with action field
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum MixAction { Link, Unlink, Disable, Enable }

#[derive(Deserialize)]
struct MixRequest {
    action: MixAction,
    mix: String,
    source: String,
}

#[derive(Deserialize)]
struct FaderRequest {
    fader: String,
    #[serde(default)]
    muted: Option<bool>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    level: Option<f32>,
}

// Helper to send command to proxy
async fn send_command(socket_path: &str, cmd: &str) -> Result<(), String> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect to proxy: {}", e))?;
    
    stream.write_all(cmd.as_bytes()).await
        .map_err(|e| format!("Failed to send command: {}", e))?;
    
    Ok(())
}

// Handlers
async fn health() -> Json<ApiResponse> {
    Json(ApiResponse::ok("API server is running"))
}

async fn mix_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MixRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mix = match req.mix.parse::<MixOutput>() {
        Ok(m) => m,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::err(&e))),
    };
    
    let source = match req.source.parse::<Source>() {
        Ok(s) => s,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::err(&e))),
    };
    
    // Build command - use callme_* commands for CallMe sources, mix_* for others
    let (cmd, msg) = if source.is_callme() {
        // CallMe sources use special commands
        match req.action {
            MixAction::Link => (
                format!("callme_link {} {}", mix.to_index(), source.to_index()),
                format!("Linked {} to {}", source, mix),
            ),
            MixAction::Unlink => (
                format!("callme_unlink {} {}", mix.to_index(), source.to_index()),
                format!("Unlinked {} from {}", source, mix),
            ),
            MixAction::Disable | MixAction::Enable => {
                return (StatusCode::BAD_REQUEST, 
                    Json(ApiResponse::err("Disable/Enable not supported for CallMe sources")));
            }
        }
    } else {
        // Regular sources
        match req.action {
            MixAction::Link => (
                format!("mix_link {} {}", mix.to_index(), source.to_index()),
                format!("Linked {} to {}", source, mix),
            ),
            MixAction::Unlink => (
                format!("mix_unlink {} {}", mix.to_index(), source.to_index()),
                format!("Unlinked {} from {}", source, mix),
            ),
            MixAction::Disable => (
                format!("mix_disable {} {} 3", mix.to_index(), source.to_index()),
                format!("Disabled {} in {}", source, mix),
            ),
            MixAction::Enable => (
                format!("mix_disable {} {} 2", mix.to_index(), source.to_index()),
                format!("Enabled {} in {}", source, mix),
            ),
        }
    };
    
    match send_command(&state.socket_path, &cmd).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(&msg))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(&e))),
    }
}

async fn fader_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FaderRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let fader = match req.fader.parse::<Fader>() {
        Ok(f) => f,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::err(&e))),
    };
    
    // Handle mute
    if let Some(muted) = req.muted {
        let cmd = format!("mute {} {}", fader.to_index(), if muted { 1 } else { 0 });
        let msg = format!("{} {}", if muted { "Muted" } else { "Unmuted" }, fader);
        return match send_command(&state.socket_path, &cmd).await {
            Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(&msg))),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(&e))),
        };
    }
    
    // Handle source
    if let Some(source_str) = &req.source {
        let source = match source_str.parse::<Source>() {
            Ok(s) => s,
            Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::err(&e))),
        };
        let cmd = format!("source {} {}", fader.to_index(), source.to_index());
        let msg = format!("Set {} source to {}", fader, source);
        return match send_command(&state.socket_path, &cmd).await {
            Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(&msg))),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(&e))),
        };
    }
    
    // Handle level
    if let Some(level) = req.level {
        let level_val = (level.clamp(0.0, 1.0) * 65535.0) as u32;
        let cmd = format!("level {} {}", fader.to_index(), level_val);
        let msg = format!("Set {} level to {:.1}%", fader, level * 100.0);
        return match send_command(&state.socket_path, &cmd).await {
            Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(&msg))),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(&e))),
        };
    }
    
    (StatusCode::BAD_REQUEST, Json(ApiResponse::err("No action specified (muted, source, or level)")))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        socket_path: SOCKET_PATH.to_string(),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/mix", post(mix_handler))
        .route("/fader", post(fader_handler))
        .with_state(state);

    let addr = "0.0.0.0:8080";
    println!("[API] Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
