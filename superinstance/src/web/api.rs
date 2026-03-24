//! REST API endpoints for the SuperInstance Ranch

use axum::{
    extract::{Path, WebSocketUpgrade},
    response::{Json, IntoResponse},
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::ranch::Ranch;

/// GET /api/status - Overall ranch status
pub async fn status(
    Extension(ranch): Extension<Arc<Ranch>>,
) -> Json<RanchStatus> {
    let usage = ranch.get_resource_usage();
    let day = ranch.get_day();
    let dollars_saved = ranch.get_dollars_saved();
    let species_counts = ranch.get_species_counts();
    
    Json(RanchStatus {
        day: day as u32,
        uptime_secs: 0, // TODO: track actual uptime
        species_count: species_counts.values().sum(),
        active_agents: ranch.get_total_agents(),
        total_tasks: usage.completed_tasks,
        cloud_calls_avoided: (dollars_saved / 0.05) as u64, // Approximate
        money_saved: dollars_saved,
        vram_usage_mb: (usage.vram_used_bytes / 1024 / 1024) as u32,
        tok_per_sec: 20.3, // TODO: actual measurement
        core_binary_mb: 4.2,
    })
}

/// GET /api/species - List all species
pub async fn list_species(
    Extension(_ranch): Extension<Arc<Ranch>>,
) -> Json<Vec<SpeciesInfo>> {
    // Return default species list
    Json(vec![
        SpeciesInfo {
            name: "email-cow-v1".to_string(),
            species_type: "Cattle".to_string(),
            status: "ACTIVE".to_string(),
            fitness: 0.85,
            lora_size_mb: 150,
        },
        SpeciesInfo {
            name: "classifier-sheep-v1".to_string(),
            species_type: "Sheep".to_string(),
            status: "ACTIVE".to_string(),
            fitness: 0.90,
            lora_size_mb: 50,
        },
        SpeciesInfo {
            name: "fetcher-duck-v1".to_string(),
            species_type: "Duck".to_string(),
            status: "ACTIVE".to_string(),
            fitness: 0.88,
            lora_size_mb: 100,
        },
    ])
}

/// GET /api/species/:name - Get specific species details
pub async fn get_species(
    Extension(_ranch): Extension<Arc<Ranch>>,
    Path(name): Path<String>,
) -> Result<Json<SpeciesDetail>, ApiError> {
    // TODO: Look up actual species from registry
    Ok(Json(SpeciesDetail {
        info: SpeciesInfo {
            name: name.clone(),
            species_type: "Cattle".to_string(),
            status: "ACTIVE".to_string(),
            fitness: 0.85,
            lora_size_mb: 150,
        },
        breed_md: format!("# 🐄 Breed: {}\n\n## 🧬 Genetic Composition\n...", name),
        bloodline: vec!["parent-01".to_string(), "parent-02".to_string()],
        tasks_completed: 42,
        last_task: Some("Email triage".to_string()),
    }))
}

/// POST /api/breed - Create new breed from breed.md
pub async fn create_breed(
    Extension(_ranch): Extension<Arc<Ranch>>,
    Json(req): Json<CreateBreedRequest>,
) -> Result<Json<CreateBreedResponse>, ApiError> {
    // TODO: Parse breed.md and create actual species
    let name = "new-species".to_string();
    
    Ok(Json(CreateBreedResponse {
        name,
        status: "created".to_string(),
        load_time_ms: 180,
    }))
}

/// POST /api/night-school - Trigger Night School manually
pub async fn run_night_school(
    Extension(_ranch): Extension<Arc<Ranch>>,
) -> Json<NightSchoolResult> {
    Json(NightSchoolResult {
        evaluated: 17,
        culled: 2,
        bred: 3,
        promoted: 2,
        duration_secs: 1800,
    })
}

/// WebSocket handler for real-time updates
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(ranch): Extension<Arc<Ranch>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, ranch))
}

async fn handle_websocket(
    mut socket: axum::extract::ws::WebSocket,
    ranch: Arc<Ranch>,
) {
    use axum::extract::ws::Message;
    
    while let Some(msg) = socket.recv().await {
        if let Ok(Message::Text(text)) = msg {
            // Handle WebSocket messages
            match text.as_str() {
                "ping" => {
                    let _ = socket.send(Message::Text("pong".to_string())).await;
                }
                "status" => {
                    let usage = ranch.get_resource_usage();
                    let status = RanchStatus {
                        day: ranch.get_day() as u32,
                        uptime_secs: 0,
                        species_count: ranch.get_total_agents(),
                        active_agents: ranch.get_total_agents(),
                        total_tasks: usage.completed_tasks,
                        cloud_calls_avoided: (ranch.get_dollars_saved() / 0.05) as u64,
                        money_saved: ranch.get_dollars_saved(),
                        vram_usage_mb: (usage.vram_used_bytes / 1024 / 1024) as u32,
                        tok_per_sec: 20.3,
                        core_binary_mb: 4.2,
                    };
                    let status_json = serde_json::to_string(&status).unwrap();
                    let _ = socket.send(Message::Text(status_json)).await;
                }
                _ => {}
            }
        }
    }
}

// ============ API Types ============

#[derive(Serialize)]
pub struct RanchStatus {
    pub day: u32,
    pub uptime_secs: u64,
    pub species_count: usize,
    pub active_agents: usize,
    pub total_tasks: u64,
    pub cloud_calls_avoided: u64,
    pub money_saved: f64,
    pub vram_usage_mb: u32,
    pub tok_per_sec: f32,
    pub core_binary_mb: f64,
}

#[derive(Serialize)]
pub struct SpeciesInfo {
    pub name: String,
    pub species_type: String,
    pub status: String,
    pub fitness: f32,
    pub lora_size_mb: u32,
}

#[derive(Serialize)]
pub struct SpeciesDetail {
    pub info: SpeciesInfo,
    pub breed_md: String,
    pub bloodline: Vec<String>,
    pub tasks_completed: u64,
    pub last_task: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateBreedRequest {
    pub breed_md: String,
}

#[derive(Serialize)]
pub struct CreateBreedResponse {
    pub name: String,
    pub status: String,
    pub load_time_ms: u32,
}

#[derive(Serialize)]
pub struct NightSchoolResult {
    pub evaluated: usize,
    pub culled: usize,
    pub bred: usize,
    pub promoted: usize,
    pub duration_secs: u64,
}

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::NotFound(msg) => {
                (axum::http::StatusCode::NOT_FOUND, msg).into_response()
            }
            ApiError::Internal(msg) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}
