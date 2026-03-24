//! SuperInstance Web Dashboard - Axum + Dioxus
//!
//! Single-binary web UI that compiles into the same 4.2 MB binary.
//! No node_modules, no separate processes.

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
    pub addr: SocketAddr,
    pub enable_dashboard: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([0, 0, 0, 0], 3000)),
            enable_dashboard: true,
        }
    }
}

/// Start the web server
pub async fn start_server(
    config: WebConfig,
    ranch_state: RanchState,
    mut shutdown: broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    let app = Router::new()
        // API routes
        .route("/api/status", get(api::status))
        .route("/api/species", get(api::list_species))
        .route("/api/species/:name", get(api::get_species))
        .route("/api/breed", post(api::create_breed))
        .route("/api/night-school", post(api::run_night_school))
        // Dashboard (Dioxus SPA)
        .route("/", get(dashboard::serve_dashboard))
        .route("/assets/*path", get(dashboard::serve_assets))
        // WebSocket for real-time updates
        .route("/ws", get(api::websocket_handler))
        .layer(Extension(ranch_state));

    tracing::info!("🌐 Web server starting on {}", config.addr);

    // Use the older axum 0.6 style for compatibility
    let addr = config.addr;
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            let _ = shutdown.recv().await;
            tracing::info!("🌐 Web server shutting down");
        });

    server.await?;

    Ok(())
}
