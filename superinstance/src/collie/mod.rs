//! The Border Collie - The Foreman
//! 
//! The Border Collie is the loyal foreman that manages the Ranch. It:
//! - Anticipates the Cowboy's needs
//! - Herds livestock to tasks
//! - Manages resources efficiently
//! - Routes intent to the appropriate species

mod shepherd;
mod anticipation;
mod reflex;

pub use shepherd::*;
pub use anticipation::*;
pub use reflex::*;

use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use tracing::{debug, info};

use crate::evolution::StudBook;
use crate::pasture::Pasture;
use crate::species::{Intent, SpeciesRegistry, SpeciesType};

/// The Border Collie - Manager and Orchestrator
pub struct Collie {
    /// Reference to the Pasture (resources)
    pasture: Arc<Pasture>,
    /// Reference to the Stud Book (evolution)
    stud_book: Arc<RwLock<StudBook>>,
    /// Reference to the Species Registry
    species_registry: Arc<RwLock<SpeciesRegistry>>,
    /// Anticipation engine (Shadow Pup)
    anticipation: AnticipationEngine,
    /// Reflex cache (CUDA graphs for hot paths)
    reflex: ReflexCache,
    /// Shepherd strategies per species
    shepherd: Shepherd,
}

impl Collie {
    /// Create a new Border Collie instance
    pub fn new(
        pasture: Arc<Pasture>,
        stud_book: Arc<RwLock<StudBook>>,
        species_registry: Arc<RwLock<SpeciesRegistry>>,
    ) -> Self {
        Self {
            pasture,
            stud_book,
            species_registry,
            anticipation: AnticipationEngine::new(),
            reflex: ReflexCache::new(),
            shepherd: Shepherd::new(),
        }
    }
    
    /// Main event loop - listen for intent and route
    pub async fn run(&self) -> Result<()> {
        info!("🐕 Border Collie is on duty, watching for intent...");
        
        // The Collie's main loop handles:
        // 1. Receiving intent from the Cowboy
        // 2. Checking reflex cache for instant responses
        // 3. Anticipating needs with Shadow Pup
        // 4. Routing to appropriate species
        
        loop {
            // In production, this would listen on a channel for user intent
            // For now, we'll just demonstrate the architecture
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            debug!("Collie heartbeat - all systems nominal");
        }
    }
    
    /// Route an intent to the appropriate species
    pub async fn route_intent(&self, intent: Intent) -> Result<String> {
        info!("🎯 Collie received intent: {:?}", intent.kind);
        
        // Step 1: Check reflex cache for instant response
        if let Some(cached) = self.reflex.check(&intent) {
            info!("⚡ Reflex response (cached): <1ms");
            return Ok(cached);
        }
        
        // Step 2: Use anticipation to predict and acknowledge
        let prediction = self.anticipation.predict(&intent);
        if prediction.confidence > 0.8 {
            info!("🔮 Anticipation: {:?} (confidence: {:.2})", 
                prediction.predicted_species, prediction.confidence);
        }
        
        // Step 3: Determine the appropriate species
        let species = self.classify_intent(&intent);
        info!("📋 Routing to {} species", species.emoji_name());
        
        // Step 4: Manage the species using shepherd strategies
        let result = self.shepherd.manage(species, intent.clone()).await?;
        
        // Step 5: Log the task to Stud Book for evolution tracking
        {
            let mut book = self.stud_book.write();
            book.log_task(species, intent, &result)?;
        }
        
        // Step 6: Add to reflex cache if used frequently
        self.reflex.consider_caching(intent, &result);
        
        Ok(result)
    }
    
    /// Classify intent to determine which species should handle it
    fn classify_intent(&self, intent: &Intent) -> SpeciesType {
        match intent.kind.as_str() {
            // Heavy reasoning tasks
            "reason" | "synthesize" | "generate_code" | "analyze" => {
                SpeciesType::Cattle
            }
            // Classification and voting
            "classify" | "vote" | "filter" | "moderate" => {
                SpeciesType::Sheep
            }
            // Network and API calls
            "fetch" | "api_call" | "search" | "download" => {
                SpeciesType::Duck
            }
            // Navigation and debugging
            "navigate" | "debug" | "explore" | "climb" => {
                SpeciesType::Goat
            }
            // Hardware interaction
            "gpio" | "sensor" | "actuator" | "realtime" => {
                SpeciesType::Hog
            }
            // Monitoring
            "monitor" | "watch" | "alert" | "heartbeat" => {
                SpeciesType::Chicken
            }
            // ETL and pipelines
            "transform" | "encode" | "pipeline" | "etl" => {
                SpeciesType::Horse
            }
            // Default to Cattle for general tasks
            _ => SpeciesType::Cattle,
        }
    }
    
    /// Get the anticipation engine for predictions
    pub fn get_anticipation(&self) -> &AnticipationEngine {
        &self.anticipation
    }
    
    /// Get the reflex cache
    pub fn get_reflex(&self) -> &ReflexCache {
        &self.reflex
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_classify_intent() {
        // This would require mocking the dependencies
        // For now, we just verify the module compiles
    }
}
