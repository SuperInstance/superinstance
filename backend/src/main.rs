//! Superinstance Backend MVP
//! Axum server with breed parsing, LLM chat stub, and UI support

pub mod cattle;
pub mod night_school;
pub mod discord;
pub mod pasture_sync;
pub mod llama_native;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    env,
};
use tokio::sync::Mutex;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::SqlitePool;

// ============================================================================
// Types
// ============================================================================

/// Breed configuration parsed from TOML breed.md files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breed {
    pub name: String,
    pub genes: Vec<Gene>,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gene {
    pub name: String,
    pub expression: f32,
    #[serde(default)]
    pub dominant: bool,
}

/// Chat request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub breed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub model: String,
    pub breed: Option<String>,
}

/// Application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub models_dir: PathBuf,
    pub breeds_dir: PathBuf,
    pub llm_loaded: bool,
    pub db_pool: Option<SqlitePool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            models_dir: PathBuf::from("models"),
            breeds_dir: PathBuf::from("breeds"),
            llm_loaded: false,
            db_pool: None,
        }
    }

    pub fn with_db(mut self, pool: SqlitePool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    pub fn model_path(&self) -> PathBuf {
        self.models_dir.join("phi3-mini.gguf")
    }
}

// ============================================================================
// API Handlers
// ============================================================================

/// GET /api/breeds - List all available breeds
pub async fn list_breeds() -> impl IntoResponse {
    let breeds_dir = PathBuf::from("breeds");
    
    let mut breeds = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&breeds_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(breed) = parse_breed_md(&content) {
                        breeds.push(breed);
                    }
                }
            }
        }
    }
    
    Json(breeds)
}

/// GET /api/breeds/:name - Get specific breed
pub async fn get_breed(Path(name): Path<String>) -> impl IntoResponse {
    let breed_path = PathBuf::from("breeds").join(format!("{}.md", name));
    
    match fs::read_to_string(&breed_path) {
        Ok(content) => match parse_breed_md(&content) {
            Ok(breed) => Json(breed).into_response(),
            Err(e) => {
                error!("Failed to parse breed {}: {}", name, e);
                (StatusCode::BAD_REQUEST, format!("Parse error: {}", e)).into_response()
            }
        },
        Err(e) => {
            warn!("Breed not found: {} ({})", name, e);
            (StatusCode::NOT_FOUND, format!("Breed '{}' not found", name)).into_response()
        }
    }
}

/// Parse a breed.md file containing TOML frontmatter
fn parse_breed_md(content: &str) -> Result<Breed, String> {
    // Look for TOML between --- markers
    let content = content.trim();
    
    if !content.starts_with("---") {
        return Err("No TOML frontmatter found".to_string());
    }
    
    let end = content[3..]
        .find("---")
        .ok_or("Unclosed TOML frontmatter")?;
    
    let toml_content = &content[3..end + 3];
    
    #[derive(Deserialize)]
    struct BreedToml {
        name: String,
        #[serde(default)]
        genes: Vec<Gene>,
        #[serde(default)]
        system_prompt: String,
    }
    
    let parsed: BreedToml = toml::from_str(toml_content)
        .map_err(|e| format!("TOML parse error: {}", e))?;
    
    Ok(Breed {
        name: parsed.name,
        genes: parsed.genes,
        system_prompt: parsed.system_prompt,
    })
}

/// POST /api/cattle - LLM chat endpoint (llama-cpp stub)
pub async fn cattle_chat(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ChatResponse>)> {
    info!("Cattle chat request: {:?}", req.message.chars().take(50).collect::<String>());
    
    // Check if model exists, download if missing
    let model_path = {
        let s = state.lock().await;
        s.model_path()
    };
    
    if !model_path.exists() {
        info!("Model not found, downloading phi3-mini.gguf...");
        if let Err(e) = download_model(&model_path).await {
            error!("Failed to download model: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatResponse {
                    response: format!("Model download failed: {}", e),
                    model: "phi3-mini".to_string(),
                    breed: req.breed,
                }),
            ));
        }
    }
    
    // Stub: llama-cpp-rs integration
    // In production, this would load and run the model:
    // let llama = llama_cpp_rs::Llama::new(&model_path)?;
    // let response = llama.chat(&req.message)?;
    
    let stub_response = format!(
        "[phi3-mini stub] I received: \"{}\"\n\nThe LLM integration is ready. \
         To enable actual inference, build with --features llm and ensure llama.cpp native libs are available.",
        req.message.chars().take(100).collect::<String>()
    );
    
    Ok(Json(ChatResponse {
        response: stub_response,
        model: "phi3-mini".to_string(),
        breed: req.breed,
    }))
}

/// POST /api/collie - Route to cattle (breed-specific chat)
pub async fn collie_chat(
    state: State<Arc<Mutex<AppState>>>,
    Json(mut req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ChatResponse>)> {
    info!("Collie routing to cattle for chat");
    
    // Collie is a herding breed - route to cattle endpoint
    req.breed = Some("collie".to_string());
    
    cattle_chat(state, Json(req)).await
}

/// POST /api/chat - Main chat endpoint (Collie -> Cattle pipeline)
pub async fn api_chat(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<cattle::ChatRequest>,
) -> Result<Json<cattle::ChatResponse>, (StatusCode, Json<cattle::ChatResponse>)> {
    info!("API chat request: {:?}", req.prompt.chars().take(50).collect::<String>());
    
    // Get cattle state
    let cattle_state = {
        let s = state.lock().await;
        let mut cs = cattle::CattleState::new(s.model_path());
        
        // Load cattle-v1 breed config
        if let Ok(breed) = cattle::load_default_breed() {
            cs.breed_config = Some(breed);
        }
        Arc::new(Mutex::new(cs))
    };
    
    // Collie herds the request to Cattle for inference
    info!("Collie -> Cattle: routing chat request");
    
    match cattle::chat_completion(&cattle_state, req).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Cattle chat completion failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(cattle::ChatResponse {
                    response: format!("Error: {}", e),
                    model: "phi3-mini".to_string(),
                    tokens_used: None,
                }),
            ))
        }
    }
}

/// Download phi3-mini model
async fn download_model(model_path: &PathBuf) -> Result<(), String> {
    use std::io::Write;
    
    // Create models directory
    if let Some(parent) = model_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create models dir: {}", e))?;
    }
    
    let url = "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf";
    
    info!("Downloading model from {}", url);
    
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }
    
    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let mut file = fs::File::create(model_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    info!("Model downloaded: {:?} ({} bytes)", model_path, bytes.len());
    Ok(())
}

/// POST /api/sync - Sync pasture via CRDT
pub async fn sync_pasture(
    Json(msg): Json<pasture_sync::SyncMessage>,
) -> Result<Json<pasture_sync::PastureState>, (StatusCode, String)> {
    info!("Received pasture sync for: {}", msg.pasture_id);
    
    let handler = pasture_sync::PastureSyncHandler::new()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    handler.handle_sync(msg).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let state = handler.get_state().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(state))
}

/// POST /api/evolution - Trigger Night School evolution manually
pub async fn trigger_evolution(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<night_school::EvolutionResult>, (StatusCode, String)> {
    info!("Manual evolution trigger");
    
    let db_pool = {
        let s = state.lock().await;
        s.db_pool.clone()
    };
    
    let pool = db_pool.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database not configured".to_string(),
    ))?;
    
    let night_school = night_school::NightSchool::new(pool, PathBuf::from("pasture"));
    
    let result = night_school.run_evolution().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

/// GET / - Dashboard stub (Dioxus)
pub async fn dashboard() -> impl IntoResponse {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Superinstance Dashboard</title>
    <style>
        body { font-family: system-ui; max-width: 800px; margin: 2rem auto; padding: 1rem; }
        h1 { color: #333; }
        .status { background: #f0f0f0; padding: 1rem; border-radius: 8px; }
        code { background: #e0e0e0; padding: 2px 6px; border-radius: 4px; }
    </style>
</head>
<body>
    <h1>🐕 Superinstance Backend</h1>
    <div class="status">
        <h2>Status: Running</h2>
        <p>Dioxus dashboard stub - full UI coming soon!</p>
        <h3>Available Endpoints:</h3>
        <ul>
            <li><code>GET /api/breeds</code> - List all breeds</li>
            <li><code>GET /api/breeds/:name</code> - Get specific breed</li>
            <li><code>POST /api/cattle</code> - LLM chat</li>
            <li><code>POST /api/collie</code> - Collie breed chat (routes to cattle)</li>
        </ul>
    </div>
</body>
</html>
"#)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting Superinstance Backend...");
    
    // Create breeds directory with sample breeds
    create_sample_breeds()?;
    
    // Initialize database
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:pasture.db".to_string());
    let db_pool = SqlitePool::connect(&db_url).await?;
    info!("Database connected: {}", db_url);
    
    // Initialize app state
    let state = Arc::new(Mutex::new(AppState::new().with_db(db_pool.clone())));
    
    // Start Night School cron (if enabled)
    let night_school_enabled = env::var("NIGHT_SCHOOL_ENABLED")
        .unwrap_or_else(|_| "false".to_string()) == "true";
    
    if night_school_enabled {
        let night_school = night_school::NightSchool::new(
            db_pool.clone(),
            PathBuf::from("pasture"),
        );
        
        tokio::spawn(async move {
            if let Err(e) = night_school::start_night_school(night_school).await {
                error!("Night School error: {}", e);
            }
        });
        
        info!("🌙 Night School enabled - running at 02:00 daily");
    }
    
    // Start Discord bot (if configured)
    if let Ok(discord_token) = env::var("DISCORD_TOKEN") {
        let discord_config = discord::DiscordConfig {
            token: discord_token,
            channel_id: env::var("DISCORD_CHANNEL_ID")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
            collie_endpoint: env::var("COLLIE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            stream_responses: true,
        };
        
        tokio::spawn(async move {
            if let Err(e) = discord::start_discord_bot(discord_config).await {
                error!("Discord bot error: {}", e);
            }
        });
        
        info!("🐕 Discord bot enabled");
    }
    
    // Build router
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/api/breeds", get(list_breeds))
        .route("/api/breeds/:name", get(get_breed))
        .route("/api/cattle", post(cattle_chat))
        .route("/api/collie", post(collie_chat))
        .route("/api/chat", post(api_chat))
        .route("/api/sync", post(sync_pasture))
        .route("/api/evolution", post(trigger_evolution))
        .with_state(state);
    
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Create sample breed.md files
fn create_sample_breeds() -> anyhow::Result<()> {
    fs::create_dir_all("breeds")?;
    
    // Collie breed
    let collie = r#"---
name = "collie"

[[genes]]
name = "herding"
expression = 0.95
dominant = true

[[genes]]
name = "intelligence"
expression = 0.9

[[genes]]
name = "loyalty"
expression = 0.85

system_prompt = "You are a Collie assistant - intelligent, loyal, and helpful. You excel at organizing and herding information."
---
# Collie

A herding breed known for intelligence and loyalty. Routes requests to the cattle endpoint.
"#;
    fs::write("breeds/collie.md", collie)?;
    
    // Cattle breed (working dog)
    let cattle = r#"---
name = "cattle"

[[genes]]
name = "working"
expression = 0.9
dominant = true

[[genes]]
name = "stamina"
expression = 0.85

[[genes]]
name = "protectiveness"
expression = 0.75

system_prompt = "You are a Cattle Dog assistant - tough, reliable, and direct. You handle the heavy lifting of LLM inference."
---
# Cattle Dog

A working breed. Handles LLM inference and model management.
"#;
    fs::write("breeds/cattle.md", cattle)?;
    
    info!("Created sample breeds in breeds/");
    Ok(())
}
