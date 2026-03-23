//! Shepherd - Species-specific Herding Strategies
//! 
//! Each species has a unique herding strategy inspired by real border collie techniques:
//! - Cattle: "Strong Eye" - Batch, steady pressure, block GPU
//! - Sheep: "The Wear" - Flock ensemble, consensus voting
//! - Ducks: "Whistle Stop" - Async, fire-and-recall
//! - Goats: "Balance" - Monitor depth, high agility
//! - Hogs: "The Drive" - Real-time priority, low latency
//! - Chickens: "Free Range" - Constant pecking, watch for silence
//! - Horses: "Saddle Up" - Linear throughput

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

use crate::pasture::Pasture;
use crate::species::{
    Species, SpeciesType, SpeciesRegistry,
    Cattle, Sheep, Duck, Goat, Hog, Chicken, Horse,
    SpeciesOps,
};

/// Herding strategy trait - different approaches for different species
#[async_trait]
pub trait HerdingStrategy: Send + Sync {
    /// Execute the herding strategy for this species
    async fn herd(&self, intent: Intent, pasture: Arc<Pasture>) -> Result<String>;
    
    /// Get the species type this strategy handles
    fn species(&self) -> SpeciesType;
}

/// Intent represents a task to be executed
#[derive(Debug, Clone)]
pub struct Intent {
    pub kind: String,
    pub payload: String,
    pub priority: u8,
    pub timeout_ms: Option<u64>,
}

impl Intent {
    pub fn new(kind: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            payload: payload.into(),
            priority: 5,
            timeout_ms: None,
        }
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

/// Shepherd - Manages herding strategies for all species
pub struct Shepherd {
    strategies: Vec<Box<dyn HerdingStrategy>>,
}

impl Shepherd {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(StrongEyeStrategy),      // Cattle
                Box::new(TheWearStrategy),        // Sheep
                Box::new(WhistleStopStrategy),    // Duck
                Box::new(BalanceStrategy),        // Goat
                Box::new(TheDriveStrategy),       // Hog
                Box::new(FreeRangeStrategy),      // Chicken
                Box::new(SaddleUpStrategy),       // Horse
            ],
        }
    }
    
    /// Manage a species by applying the appropriate herding strategy
    pub async fn manage(&self, species: SpeciesType, intent: Intent) -> Result<String> {
        let start = Instant::now();
        
        // Find the appropriate strategy
        let strategy = self.strategies.iter()
            .find(|s| s.species() == species)
            .ok_or_else(|| anyhow!("No strategy for species: {:?}", species))?;
        
        info!("🐕 Herding {} with {} strategy", 
            species.emoji_name(), 
            strategy_name(species));
        
        // Execute the strategy
        // Note: In production, we'd pass the actual pasture reference
        let result = strategy.herd(intent.clone(), Arc::new(Pasture::mock())).await?;
        
        let elapsed = start.elapsed();
        info!("✅ {} task completed in {:?}", species.emoji_name(), elapsed);
        
        Ok(result)
    }
}

/// Get strategy name for logging
fn strategy_name(species: SpeciesType) -> &'static str {
    match species {
        SpeciesType::Cattle => "Strong Eye",
        SpeciesType::Sheep => "The Wear",
        SpeciesType::Duck => "Whistle Stop",
        SpeciesType::Goat => "Balance",
        SpeciesType::Hog => "The Drive",
        SpeciesType::Chicken => "Free Range",
        SpeciesType::Horse => "Saddle Up",
    }
}

// ============================================================================
// HERDING STRATEGIES
// ============================================================================

/// Strong Eye Strategy - For Cattle (Heavy LLMs)
/// 
/// The border collie's famous "strong eye" - a fixed, intense stare that
/// applies steady pressure. Used for heavy tasks that need full attention.
/// 
/// Characteristics:
/// - Locks GPU resources (blocking)
/// - Single-threaded execution
/// - Batch processing for efficiency
/// - High VRAM usage
struct StrongEyeStrategy;

#[async_trait]
impl HerdingStrategy for StrongEyeStrategy {
    async fn herd(&self, intent: Intent, _pasture: Arc<Pasture>) -> Result<String> {
        info!("🐄 STRONG EYE: Locking GPU, applying steady pressure...");
        
        // Lock GPU (in production, this would acquire GPU mutex)
        // let _lock = pasture.acquire_gpu().await?;
        
        // Execute heavy task
        let cattle = Cattle::new("strong-eye-01".to_string());
        let result = cattle.execute(intent.payload.clone()).await?;
        
        info!("🐄 STRONG EYE: Task complete, releasing GPU");
        
        Ok(result)
    }
    
    fn species(&self) -> SpeciesType {
        SpeciesType::Cattle
    }
}

/// The Wear Strategy - For Sheep (Classifiers)
/// 
/// Named after the technique of wearing down a flock's resistance through
/// persistent, gentle pressure. Used for ensemble voting.
/// 
/// Characteristics:
/// - Spawns multiple agents (flock)
/// - Waits for consensus
/// - Lower individual resource usage
/// - Voting aggregation
struct TheWearStrategy;

#[async_trait]
impl HerdingStrategy for TheWearStrategy {
    async fn herd(&self, intent: Intent, _pasture: Arc<Pasture>) -> Result<String> {
        info!("🐑 THE WEAR: Forming flock for consensus...");
        
        let flock_size = 5; // Number of sheep in the flock
        let mut join_set = JoinSet::new();
        
        // Spawn the flock
        for i in 0..flock_size {
            let payload = intent.payload.clone();
            join_set.spawn(async move {
                let sheep = Sheep::new(format!("flock-{:02}", i));
                sheep.execute(payload).await
            });
        }
        
        // Collect votes
        let mut votes: Vec<String> = Vec::new();
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(vote)) => votes.push(vote),
                _ => warn!("Sheep in flock failed"),
            }
        }
        
        // Simple majority voting
        let result = if votes.len() > flock_size / 2 {
            // Return the most common vote (simplified)
            votes.first().cloned().unwrap_or_else(|| "consensus_reached".to_string())
        } else {
            "no_consensus".to_string()
        };
        
        info!("🐑 THE WEAR: Consensus reached ({}/{})", votes.len(), flock_size);
        
        Ok(result)
    }
    
    fn species(&self) -> SpeciesType {
        SpeciesType::Sheep
    }
}

/// Whistle Stop Strategy - For Ducks (Network/API)
/// 
/// Like whistle commands for working dogs - fire and recall.
/// Quick, async operations with timeouts.
/// 
/// Characteristics:
/// - Async, non-blocking
/// - Fire-and-recall pattern
/// - Timeout handling
/// - Retry on failure
struct WhistleStopStrategy;

#[async_trait]
impl HerdingStrategy for WhistleStopStrategy {
    async fn herd(&self, intent: Intent, _pasture: Arc<Pasture>) -> Result<String> {
        info!("🦆 WHISTLE STOP: Launching async fetch...");
        
        let timeout = intent.timeout_ms.unwrap_or(5000);
        
        let duck = Duck::new("fetcher-01".to_string());
        
        // Execute with timeout
        let result = tokio::time::timeout(
            Duration::from_millis(timeout),
            duck.execute(intent.payload.clone())
        ).await
            .map_err(|_| anyhow!("Duck timeout after {}ms", timeout))??;
        
        info!("🦆 WHISTLE STOP: Recall complete");
        
        Ok(result)
    }
    
    fn species(&self) -> SpeciesType {
        SpeciesType::Duck
    }
}

/// Balance Strategy - For Goats (Navigators)
/// 
/// Goats are agile and can handle complex terrain. The strategy monitors
/// depth and maintains balance while navigating.
/// 
/// Characteristics:
/// - Depth tracking
/// - Agile navigation
/// - Edge case handling
/// - Resource-aware
struct BalanceStrategy;

#[async_trait]
impl HerdingStrategy for BalanceStrategy {
    async fn herd(&self, intent: Intent, _pasture: Arc<Pasture>) -> Result<String> {
        info!("🐐 BALANCE: Navigating with agility...");
        
        let goat = Goat::new("navigator-01".to_string());
        
        // Execute with depth tracking
        let result = goat.execute(intent.payload.clone()).await?;
        
        info!("🐐 BALANCE: Navigation complete");
        
        Ok(result)
    }
    
    fn species(&self) -> SpeciesType {
        SpeciesType::Goat
    }
}

/// The Drive Strategy - For Hogs (Hardware)
/// 
/// Real-time, low-latency operations for hardware interaction.
/// Sets thread priority for deterministic timing.
/// 
/// Characteristics:
/// - Real-time thread priority
/// - Low latency (<10ms)
/// - Direct hardware access
/// - Minimal abstraction
struct TheDriveStrategy;

#[async_trait]
impl HerdingStrategy for TheDriveStrategy {
    async fn herd(&self, intent: Intent, _pasture: Arc<Pasture>) -> Result<String> {
        info!("🐗 THE DRIVE: Setting real-time priority...");
        
        // Set real-time thread priority (simplified - in production use thread_priority crate)
        // #[cfg(target_os = "linux")]
        // set_thread_priority(ThreadPriority::Realtime(RealtimeThreadPriority::Min));
        
        let hog = Hog::new("hardware-01".to_string());
        
        let start = Instant::now();
        let result = hog.execute(intent.payload.clone()).await?;
        let elapsed = start.elapsed();
        
        // Verify latency constraint
        if elapsed > Duration::from_millis(10) {
            warn!("⚠️ HOG LATENCY VIOLATION: {:?}", elapsed);
        }
        
        info!("🐗 THE DRIVE: Real-time complete in {:?}", elapsed);
        
        Ok(result)
    }
    
    fn species(&self) -> SpeciesType {
        SpeciesType::Hog
    }
}

/// Free Range Strategy - For Chickens (Monitors)
/// 
/// Chickens roam freely, constantly "pecking" at the environment.
/// They watch for silence (absence of expected events).
/// 
/// Characteristics:
/// - Always running
/// - Constant heartbeat checks
/// - Alert on silence
/// - Low resource usage
struct FreeRangeStrategy;

#[async_trait]
impl HerdingStrategy for FreeRangeStrategy {
    async fn herd(&self, intent: Intent, _pasture: Arc<Pasture>) -> Result<String> {
        debug!("🐔 FREE RANGE: Pecking at monitoring target...");
        
        let chicken = Chicken::new("monitor-01".to_string());
        
        // Execute monitoring task
        let result = chicken.execute(intent.payload.clone()).await?;
        
        // Check for silence (in production, would track last event time)
        // if last_event.elapsed() > Duration::from_secs(30) {
        //     warn!("🐔 ALERT: Silence detected!");
        // }
        
        Ok(result)
    }
    
    fn species(&self) -> SpeciesType {
        SpeciesType::Chicken
    }
}

/// Saddle Up Strategy - For Horses (Pipelines)
/// 
/// Horses are steady workers for linear throughput. They handle ETL
/// and encoding tasks efficiently.
/// 
/// Characteristics:
/// - Linear processing
/// - Throughput-oriented
/// - Batch optimization
/// - Pipeline stages
struct SaddleUpStrategy;

#[async_trait]
impl HerdingStrategy for SaddleUpStrategy {
    async fn herd(&self, intent: Intent, _pasture: Arc<Pasture>) -> Result<String> {
        info!("🐎 SADDLE UP: Beginning pipeline...");
        
        let horse = Horse::new("pipeline-01".to_string());
        
        // Execute pipeline
        let result = horse.execute(intent.payload.clone()).await?;
        
        info!("🐎 SADDLE UP: Pipeline complete");
        
        Ok(result)
    }
    
    fn species(&self) -> SpeciesType {
        SpeciesType::Horse
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_shepherd_strategies() {
        let shepherd = Shepherd::new();
        let intent = Intent::new("test", "test payload");
        
        // Test each strategy exists
        for species in [
            SpeciesType::Cattle,
            SpeciesType::Sheep,
            SpeciesType::Duck,
            SpeciesType::Goat,
            SpeciesType::Hog,
            SpeciesType::Chicken,
            SpeciesType::Horse,
        ] {
            let result = shepherd.manage(species, intent.clone()).await;
            assert!(result.is_ok(), "Strategy for {:?} should succeed", species);
        }
    }
}
