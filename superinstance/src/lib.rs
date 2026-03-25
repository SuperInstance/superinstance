//! # SuperInstance - The AI Ranch Ecosystem
//!
//! **A self-evolving system of specialized AI agents managed by a loyal Border Collie.**
//!
//! SuperInstance is not a monolithic AI, but a ranch of specialized intelligences
//! (LoRA adapters called "livestock") managed by an orchestrator (the "Border Collie").
//! The system evolves through Night School breeding cycles, creating new capabilities
//! while staying under tight hardware constraints.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                         THE RANCH                                    │
//! │                                                                       │
//! │   ┌───────────────┐                                                  │
//! │   │   🤠 COWBOY   │  ← You (set intent, provide feedback)           │
//! │   │    (User)     │                                                  │
//! │   └───────┬───────┘                                                  │
//! │           │                                                           │
//! │           ▼                                                           │
//! │   ┌───────────────┐                                                  │
//! │   │ 🐕 COLLIE     │  ← Orchestrator (routes, anticipates, caches)    │
//! │   │  (Foreman)    │                                                  │
//! │   └───────┬───────┘                                                  │
//! │           │                                                           │
//! │     ┌─────┴─────┐                                                    │
//! │     ▼           ▼                                                    │
//! │ ┌─────────┐ ┌─────────┐                                              │
//! │ │🐄 CATTLE│ │🦆  DUCK │  ← Livestock (specialized LoRA adapters)     │
//! │ │ Heavy   │ │ Network │                                              │
//! │ │Reasoning│ │ API     │                                              │
//! │ └─────────┘ └─────────┘                                              │
//! │                                                                       │
//! │   ┌───────────────────────────────────────────────────────────────┐ │
//! │   │                    🌙 NIGHT SCHOOL                             │ │
//! │   │              (02:00 Daily Breeding Cycle)                     │ │
//! │   │   Cull → Breed → Distill → Quarantine → Promote              │ │
//! │   └───────────────────────────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use superinstance::{Ranch, Config};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create the Ranch with default configuration
//!     let config = Config::default();
//!     let ranch = Ranch::new(config).await?;
//!     
//!     // Route an intent to the appropriate species
//!     let result = ranch.collie.route_intent(
//!         superinstance::species::Intent::new("reason", "Analyze this code")
//!     ).await?;
//!     
//!     println!("Result: {}", result);
//!     Ok(())
//! }
//! ```
//!
//! ## Core Concepts
//!
//! ### Species (Livestock)
//!
//! Each species is a specialized LoRA adapter with specific capabilities:
//!
//! | Species | Role | VRAM | Latency |
//! |---------|------|------|---------|
//! | 🐄 Cattle | Heavy reasoning, code gen | 500MB | 2s |
//! | 🐑 Sheep | Classification, voting | 50MB | 100ms |
//! | 🦆 Duck | Network, API calls | 100MB | 500ms |
//! | 🐐 Goat | Navigation, debugging | 150MB | 200ms |
//! | 🐗 Hog | Hardware, GPIO | 10MB | 10ms |
//! | 🐔 Chicken | Monitoring, alerts | 5MB | 5ms |
//! | 🐎 Horse | ETL, pipelines | 200MB | 1s |
//!
//! ### Border Collie (Orchestrator)
//!
//! The Collie manages all livestock with three-tier routing:
//! 1. **Reflex** (<1ms) - Cached responses via CUDA graphs
//! 2. **Anticipation** (~10ms) - Predictive routing
//! 3. **Cognition** (<50ms) - Full reasoning pipeline
//!
//! ### Night School (Evolution)
//!
//! Every night at 02:00, the system:
//! 1. Evaluates all agents (fitness scoring)
//! 2. Culls underperformers (<0.4 fitness)
//! 3. Breeds champions (SLERP/TIES merging)
//! 4. Distills from cloud teachers
//! 5. Tests in quarantine
//! 6. Promotes best performers
//!
//! ### Open Genomics (breed.md)
//!
//! Define agent DNA in Markdown:
//!
//! ```markdown
//! # 🐄 Breed: Email-Cow-v1
//!
//! ## 🧬 Genetic Composition
//! | Gene Trait | Weight |
//! |------------|--------|
//! | `polite_tone` | `0.8` |
//! | `concise_style` | `0.5` |
//!
//! ## 🧠 System Prompt
//! ```
//! You are an email triage specialist...
//! ```
//! ```
//!
//! ## Features
//!
//! - **4.2 MB Binary** - Forever small, no runtime dependencies
//! - **Hot-Reload** - Edit breed.md, see changes instantly
//! - **Geometric Routing** - Deterministic via constraint theory
//! - **CRDT Memory** - Distributed sync without central server
//! - **TensorRT-LLM** - Optimized inference on Jetson
//! - **Axum + Dioxus** - Web UI baked into binary
//!
//! ## Hardware Target
//!
//! Designed for Jetson Orin Nano 8GB:
//! - 6.5 GB VRAM budget
//! - 2.5 GB base model
//! - 1.0 GB LoRA pool
//! - 1.0 GB KV cache
//! - 1.5 GB buffer

pub mod collie;
pub mod species;
pub mod pasture;
pub mod evolution;
pub mod genetics;
pub mod channels;

mod ranch;
mod dashboard;
mod web;

// Re-exports for ergonomic API
pub use ranch::{Ranch, ResourceUsage, morning_routine};
pub use collie::{Collie, AnticipationEngine, ReflexCache, Shepherd};
pub use species::{Species, SpeciesOps, SpeciesType, SpeciesRegistry, Intent};
pub use pasture::{Pasture, PastureStats, LoRAManager, ModelPool};
pub use evolution::{NightSchool, StudBook, BreedingPipeline};
pub use genetics::{BreedManifest, GeneWeight, Lineage, Phenotype};

/// Hardware constraints for Jetson Orin Nano 8GB
pub const MAX_VRAM_GB: f64 = 6.5;
/// Base model VRAM allocation
pub const BASE_MODEL_VRAM_GB: f64 = 2.5;
/// LoRA pool VRAM allocation  
pub const LORA_POOL_VRAM_GB: f64 = 1.0;
/// KV Cache VRAM allocation
pub const KV_CACHE_VRAM_GB: f64 = 1.0;
/// Collie runtime VRAM allocation
pub const COLLIE_RUNTIME_VRAM_GB: f64 = 0.5;
/// Buffer VRAM allocation
pub const BUFFER_VRAM_GB: f64 = 1.5;
/// Night School schedule (hour, 24-hour format)
pub const NIGHT_SCHOOL_HOUR: u32 = 2;

/// Global configuration for the Ranch
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

/// Prelude for common imports
pub mod prelude {
    pub use crate::{Config, Ranch, SpeciesType};
    pub use crate::collie::Collie;
    pub use crate::species::{Species, Intent};
    pub use crate::genetics::BreedManifest;
    pub use crate::evolution::NightSchool;
}

// Re-export commonly used crates for convenience
pub use anyhow;
pub use tokio;
pub use serde;
pub use serde_json;
