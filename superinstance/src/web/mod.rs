//! SuperInstance Web Dashboard - Axum + Dioxus
//!
//! Single-binary web UI that compiles into the same 4.2 MB binary.
//! No node_modules, no separate processes, zero Node.js runtime.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  SINGLE <5 MB BINARY                        │
//! │                                                              │
//! │   ┌───────────────┐    ┌───────────────┐    ┌───────────┐  │
//! │   │   TUI Mode    │    │   Web Mode    │    │  API Mode │  │
//! │   │   (ratatui)   │    │ (Axum+Dioxus) │    │   (Axum)  │  │
//! │   │               │    │               │    │           │  │
//! │   │  Terminal ◄───┼────┼───► Browser   │    │ REST/WS ◄─┼─ │
//! │   │               │    │               │    │           │  │
//! │   └───────────────┘    └───────────────┘    └───────────┘  │
//! │                               │                              │
//! │                    ┌──────────▼──────────┐                  │
//! │                    │    BORDER COLLIE    │                  │
//! │                    │  (Tokio Runtime)    │                  │
//! │                    └─────────────────────┘                  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Zero Node.js Philosophy
//!
//! Traditional AI stacks require: Node.js + Python + Redis + DB processes.
//! SuperInstance requires: ONE Rust binary.
//!
//! - All assets embedded in binary
//! - WebSocket for real-time updates
//! - Dioxus for reactive UI (compiles to WASM, served by Axum)
//! - No build step for users - just run the binary

pub mod api;
pub mod dashboard;

use axum::{
    routing::{get, post},
    Router,
    Extension,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::ranch::Ranch;

/// Type alias for RanchState used by the web API
pub type RanchState = Arc<Ranch>;

/// Web server configuration
pub struct WebConfig {
    pub port: u16,
    pub enable_dashboard: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            enable_dashboard: true,
        }
    }
}

impl WebConfig {
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], self.port))
    }
}

/// Start the web server
pub async fn start_server(
    config: WebConfig,
    ranch_state: RanchState,
    mut shutdown: broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    use tokio::net::TcpListener;
    
    let app = Router::new()
        // API routes
        .route("/api/status", get(api::status))
        .route("/api/species", get(api::list_species))
        .route("/api/species/:name", get(api::get_species))
        .route("/api/breed", post(api::create_breed))
        .route("/api/night-school", post(api::run_night_school))
        // Night School API
        .route("/api/night", get(api::get_night_school_status))
        .route("/api/night", post(api::trigger_night_school))
        // Dashboard (Dioxus SPA)
        .route("/", get(dashboard::serve_dashboard))
        .route("/assets/*path", get(dashboard::serve_assets))
        // WebSocket for real-time updates
        .route("/ws", get(api::websocket_handler))
        .layer(Extension(ranch_state));

    let addr = config.addr();
    tracing::info!("🌐 Web server starting on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    
    // Use axum 0.7 serve pattern
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown.recv().await;
            tracing::info!("🌐 Web server shutting down");
        })
        .await?;

    Ok(())
}
