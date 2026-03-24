//! SuperInstance - The Ranch Ecosystem
//! 
//! A system that evolves into a SuperInstance through specialization, evolution,
//! and loyalty. Not a monolithic AI, but a ranch of specialized agents managed
//! by a loyal Border Collie.

mod ranch;
mod collie;
mod species;
mod pasture;
mod evolution;
mod dashboard;
mod web;  // Axum + Dioxus web dashboard
mod genetics;
mod channels;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::ranch::Ranch;
use crate::dashboard::Dashboard;
use crate::web::{WebConfig, start_server};

/// Hardware constraints for Jetson Orin Nano 8GB
pub const MAX_VRAM_GB: f64 = 6.5;
pub const BASE_MODEL_VRAM_GB: f64 = 2.5;
pub const LORA_POOL_VRAM_GB: f64 = 1.0;
pub const KV_CACHE_VRAM_GB: f64 = 1.0;
pub const COLLIE_RUNTIME_VRAM_GB: f64 = 0.5;
pub const BUFFER_VRAM_GB: f64 = 1.5;

/// Night School schedule (02:00 daily)
pub const NIGHT_SCHOOL_HOUR: u32 = 2;

#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum VRAM usage in bytes
    pub max_vram_bytes: u64,
    /// Path to LoRA adapters directory
    pub adapters_path: String,
    /// Path to Stud Book database
    pub stud_book_path: String,
    /// Night School enabled
    pub night_school_enabled: bool,
    /// Minimum fitness score threshold for culling
    pub cull_threshold: f32,
    /// Hot-swap timeout in milliseconds
    pub hot_swap_timeout_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_vram_bytes: (MAX_VRAM_GB * 1024.0 * 1024.0 * 1024.0) as u64,
            adapters_path: "pasture/adapters".to_string(),
            stud_book_path: "pasture/stud_book.sqlite".to_string(),
            night_school_enabled: true,
            cull_threshold: 0.4,
            hot_swap_timeout_ms: 50,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .pretty()
        .init();
    
    info!("═══════════════════════════════════════════════════════════");
    info!("  SUPERINSTANCE RANCH - Initializing...");
    info!("  'Not a Superintelligence, but a loyal team that evolves'");
    info!("═══════════════════════════════════════════════════════════");
    
    // Load configuration
    let config = Config::default();
    info!("Configuration loaded:");
    info!("  Max VRAM: {:.1} GB", MAX_VRAM_GB);
    info!("  Buffer: {:.1} GB", BUFFER_VRAM_GB);
    info!("  Cull threshold: {:.2}", config.cull_threshold);
    
    // Initialize the Ranch
    info!("\nOpening the Ranch gates...");
    let ranch = Arc::new(Ranch::new(config).await?);
    
    // Spawn the Night School background task
    let ranch_clone = Arc::clone(&ranch);
    tokio::spawn(async move {
        if let Err(e) = ranch_clone.run_night_school().await {
            tracing::error!("Night School error: {}", e);
        }
    });
    
    // Start the Collie's main event loop
    info!("Releasing the Border Collie to herd the livestock...");
    let ranch_clone = Arc::clone(&ranch);
    tokio::spawn(async move {
        if let Err(e) = ranch_clone.collie.run().await {
            tracing::error!("Collie error: {}", e);
        }
    });
    
    // Start the monitoring chickens
    info!("Setting the Chickens to watch the perimeter...");
    let ranch_clone = Arc::clone(&ranch);
    tokio::spawn(async move {
        if let Err(e) = ranch_clone.start_monitoring().await {
            tracing::error!("Monitoring error: {}", e);
        }
    });
    
    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
    
    // Start the Web Dashboard (Axum + Dioxus)
    info!("Starting Web Dashboard on :3000...");
    let web_config = WebConfig::default();
    let ranch_clone = Arc::clone(&ranch);
    let shutdown_rx_web = shutdown_tx.subscribe();
    tokio::spawn(async move {
        if let Err(e) = start_server(web_config, ranch_clone, shutdown_rx_web).await {
            tracing::error!("Web server error: {}", e);
        }
    });
    
    info!("\n✓ Ranch is open for business!");
    info!("  The Cowboy (User) may now set intent.");
    info!("  The Collie is watching and anticipating.");
    info!("  The Livestock are grazing in the pasture.");
    info!("  Web Dashboard: http://localhost:3000\n");
    
    // Launch the Terminal Dashboard
    info!("Launching the Living Ranch Dashboard...");
    let mut dashboard = Dashboard::new(ranch);
    
    // Handle graceful shutdown
    tokio::select! {
        result = dashboard.run() => {
            if let Err(e) = result {
                tracing::error!("Dashboard error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            let _ = shutdown_tx.send(());
            info!("\nReceived shutdown signal. Closing the Ranch...");
        }
    }
    
    info!("Goodbye, Cowboy. The Ranch will be here when you return.");
    Ok(())
}
