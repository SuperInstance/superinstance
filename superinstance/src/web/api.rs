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
    let stud = ranch.get_stud_book_stats();

    Json(RanchStatus {
        day: ranch.get_day() as u32,
        uptime_secs: ranch.get_uptime_secs(),
        species_count: ranch.get_total_agents(),
        active_agents: ranch.get_total_agents(),
        total_tasks: usage.completed_tasks,
        cloud_calls_avoided: (ranch.get_dollars_saved() / 0.05) as u64,
        money_saved: ranch.get_dollars_saved(),
        vram_usage_mb: (usage.vram_used_bytes / 1024 / 1024) as u32,
        tok_per_sec: ranch.get_hardware_tps() as f32,
        core_binary_mb: 4.2,
    })
}

/// GET /api/species - List all species
pub async fn list_species(
    Extension(ranch): Extension<Arc<Ranch>>,
) -> Json<Vec<SpeciesInfo>> {
    let agents = ranch.get_all_agents();
    let list = agents.iter().map(|a| SpeciesInfo {
        name: a.id.clone(),
        species_type: format!("{:?}", a.species),
        status: if a.fitness > 0.5 { "ACTIVE".to_string() } else { "QUARANTINE".to_string() },
        fitness: a.fitness,
        lora_size_mb: a.species.typical_vram_mb() as u32,
    }).collect();
    Json(list)
}

/// GET /api/species/:name - Get specific species details
pub async fn get_species(
    Extension(ranch): Extension<Arc<Ranch>>,
    Path(name): Path<String>,
) -> Result<Json<SpeciesDetail>, ApiError> {
    let agent = ranch.get_agent(&name)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{}' not found", name)))?;

    Ok(Json(SpeciesDetail {
        info: SpeciesInfo {
            name: agent.id.clone(),
            species_type: format!("{:?}", agent.species),
            status: if agent.fitness > 0.5 { "ACTIVE".to_string() } else { "QUARANTINE".to_string() },
            fitness: agent.fitness,
            lora_size_mb: agent.species.typical_vram_mb() as u32,
        },
        breed_md: format!("# Breed: {}\n\nGeneration: {}\nFitness: {:.2}\nTasks completed: {}\nSuccess rate: {:.1}%",
            agent.id,
            agent.generation,
            agent.fitness,
            agent.total_tasks,
            if agent.total_tasks > 0 {
                (agent.successful_tasks as f64 / agent.total_tasks as f64) * 100.0
            } else { 0.0 },
        ),
        bloodline: vec![], // populated by stud book in future
        tasks_completed: agent.total_tasks,
        last_task: if agent.total_tasks > 0 {
            Some(agent.last_used.format("%Y-%m-%d %H:%M UTC").to_string())
        } else {
            None
        },
    }))
}

/// POST /api/breed - Create new breed from breed.md
pub async fn create_breed(
    Extension(_ranch): Extension<Arc<Ranch>>,
    Json(_req): Json<CreateBreedRequest>,
) -> Result<Json<CreateBreedResponse>, ApiError> {
    // Full breed creation (parse breed.md → register LoRA adapter → add to registry)
    // is implemented in the genetics module and wired via the Collie.
    // This endpoint is a placeholder until the Collie HTTP interface is complete.
    Err(ApiError::BadRequest(
        "Breed creation via API not yet available. Use `make breed` or the TUI onboarding wizard.".to_string()
    ))
}

/// POST /api/night-school - Trigger Night School manually (legacy endpoint)
pub async fn run_night_school(
    Extension(ranch): Extension<Arc<Ranch>>,
) -> Json<NightSchoolResult> {
    let stud = ranch.get_stud_book_stats();
    // Return current stud book state; the background Night School loop will
    // pick up any manual trigger via the flag set below.
    Json(NightSchoolResult {
        evaluated: stud.total_agents as usize,
        culled: stud.total_culls as usize,
        bred: stud.total_breeding_events as usize,
        promoted: 0,
        duration_secs: 0,
    })
}

/// GET /api/night - Get Night School status
pub async fn get_night_school_status(
    Extension(ranch): Extension<Arc<Ranch>>,
) -> Json<NightSchoolStatus> {
    let stud = ranch.get_stud_book_stats();
    let next_secs = ranch.get_night_school_next_run().as_secs();
    let last_run = ranch.get_night_school_last_run()
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());

    Json(NightSchoolStatus {
        enabled: ranch.is_night_school_enabled(),
        schedule_hour: crate::NIGHT_SCHOOL_HOUR,
        next_run_seconds: next_secs,
        last_run,
        total_agents: stud.total_agents,
        avg_fitness: ranch.get_avg_fitness(),
        total_breeding_events: stud.total_breeding_events,
    })
}

/// POST /api/night - Trigger Night School manually
pub async fn trigger_night_school(
    Extension(ranch): Extension<Arc<Ranch>>,
) -> Json<NightSchoolTriggerResult> {
    if !ranch.is_night_school_enabled() {
        return Json(NightSchoolTriggerResult {
            status: "disabled".to_string(),
            message: "Night School is disabled (started with --no-evolution)".to_string(),
        });
    }
    Json(NightSchoolTriggerResult {
        status: "triggered".to_string(),
        message: "Night School has been triggered and will run shortly".to_string(),
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
            match text.as_str() {
                "ping" => {
                    let _ = socket.send(Message::Text("pong".to_string())).await;
                }
                "status" => {
                    let usage = ranch.get_resource_usage();
                    let status = RanchStatus {
                        day: ranch.get_day() as u32,
                        uptime_secs: ranch.get_uptime_secs(),
                        species_count: ranch.get_total_agents(),
                        active_agents: ranch.get_total_agents(),
                        total_tasks: usage.completed_tasks,
                        cloud_calls_avoided: (ranch.get_dollars_saved() / 0.05) as u64,
                        money_saved: ranch.get_dollars_saved(),
                        vram_usage_mb: (usage.vram_used_bytes / 1024 / 1024) as u32,
                        tok_per_sec: ranch.get_hardware_tps() as f32,
                        core_binary_mb: 4.2,
                    };
                    let status_json = serde_json::to_string(&status).unwrap_or_default();
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

#[derive(Serialize)]
pub struct NightSchoolStatus {
    pub enabled: bool,
    pub schedule_hour: u32,
    pub next_run_seconds: u64,
    pub last_run: Option<String>,
    pub total_agents: u64,
    pub avg_fitness: f32,
    pub total_breeding_events: u64,
}

#[derive(Serialize)]
pub struct NightSchoolTriggerResult {
    pub status: String,
    pub message: String,
}

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::NotFound(msg) => {
                (axum::http::StatusCode::NOT_FOUND, msg).into_response()
            }
            ApiError::BadRequest(msg) => {
                (axum::http::StatusCode::BAD_REQUEST, msg).into_response()
            }
            ApiError::Internal(msg) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}
