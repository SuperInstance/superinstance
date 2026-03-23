//! Anticipation Engine - The "Shadow Pup"
//! 
//! A tiny model that runs ahead of the main inference to predict user intent
//! and provide instant acknowledgement. This is the "instinct" layer - fast,
//! speculative, and always ready.
//! 
//! The Shadow Pup:
//! - Predicts what the user might want next
//! - Provides instant "acknowledgement" responses
//! - Pre-warms likely adapters
//! - Learns from patterns over time

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::species::SpeciesType;

/// The Shadow Pup - Tiny anticipation model
pub const SHADOW_PUP_NAME: &str = "shadow-pup";
pub const SHADOW_PUP_SIZE_MB: f64 = 50.0; // Tiny model

/// Anticipation Engine - Predicts user intent
pub struct AnticipationEngine {
    /// Pattern history for learning
    patterns: HashMap<String, PatternStats>,
    /// Recent user interactions (for context)
    recent_context: Vec<ContextEntry>,
    /// Maximum context length
    max_context: usize,
}

/// Statistics for a pattern
#[derive(Debug, Clone, Default)]
struct PatternStats {
    /// Number of times this pattern occurred
    occurrences: u64,
    /// What species was typically needed
    species_distribution: HashMap<SpeciesType, u32>,
    /// Average time between pattern start and user action
    avg_latency_ms: f64,
}

/// A context entry for recent interactions
#[derive(Debug, Clone)]
struct ContextEntry {
    timestamp: Instant,
    intent_kind: String,
    species: SpeciesType,
    success: bool,
}

/// Prediction result from the Shadow Pup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Predicted species that will be needed
    pub predicted_species: SpeciesType,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Predicted next intent kind
    pub predicted_intent: Option<String>,
    /// Should we pre-warm the adapter?
    pub prewarm: bool,
}

/// Acknowledgement response - instant feedback to user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acknowledgement {
    /// Message to show user
    pub message: String,
    /// Estimated time to completion
    pub estimated_ms: u64,
    /// What action is being taken
    pub action: String,
}

impl AnticipationEngine {
    /// Create a new anticipation engine
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            recent_context: Vec::new(),
            max_context: 100,
        }
    }
    
    /// Predict what the user might want next
    pub fn predict(&self, current_intent: &crate::collie::shepherd::Intent) -> Prediction {
        let start = Instant::now();
        
        // Look for patterns in recent context
        let pattern_key = self.extract_pattern_key(&current_intent.kind);
        
        let prediction = if let Some(stats) = self.patterns.get(&pattern_key) {
            // We've seen this pattern before
            let most_common_species = stats.species_distribution.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(s, _)| *s)
                .unwrap_or(SpeciesType::Cattle);
            
            let total_occurrences: u32 = stats.species_distribution.values().sum();
            let confidence = if total_occurrences > 0 {
                stats.species_distribution.get(&most_common_species)
                    .copied()
                    .unwrap_or(0) as f32 / total_occurrences as f32
            } else {
                0.5
            };
            
            Prediction {
                predicted_species: most_common_species,
                confidence,
                predicted_intent: Some(pattern_key.clone()),
                prewarm: confidence > 0.6,
            }
        } else {
            // No pattern found - use heuristics
            self.heuristic_prediction(&current_intent.kind)
        };
        
        let elapsed = start.elapsed();
        debug!("🔮 Shadow Pup prediction in {:?}: {:?}", elapsed, prediction);
        
        prediction
    }
    
    /// Generate an instant acknowledgement for the user
    pub fn acknowledge(&self, intent: &crate::collie::shepherd::Intent) -> Acknowledgement {
        let species = self.predict(intent).predicted_species;
        
        let (message, estimated_ms) = match species {
            SpeciesType::Cattle => ("Thinking deeply about this...", 2000),
            SpeciesType::Sheep => ("Gathering consensus...", 500),
            SpeciesType::Duck => ("Fetching from the wild...", 1000),
            SpeciesType::Goat => ("Climbing into the details...", 800),
            SpeciesType::Hog => ("Interfacing with hardware...", 100),
            SpeciesType::Chicken => ("Watching for changes...", 50),
            SpeciesType::Horse => ("Running the pipeline...", 1500),
        };
        
        Acknowledgement {
            message: message.to_string(),
            estimated_ms,
            action: format!("Activating {}", species.emoji_name()),
        }
    }
    
    /// Learn from a completed interaction
    pub fn learn(
        &mut self,
        intent_kind: &str,
        species: SpeciesType,
        success: bool,
    ) {
        // Add to context
        self.recent_context.push(ContextEntry {
            timestamp: Instant::now(),
            intent_kind: intent_kind.to_string(),
            species,
            success,
        });
        
        // Trim context if needed
        if self.recent_context.len() > self.max_context {
            self.recent_context.remove(0);
        }
        
        // Update pattern stats
        let pattern_key = self.extract_pattern_key(intent_kind);
        let stats = self.patterns.entry(pattern_key).or_default();
        stats.occurrences += 1;
        *stats.species_distribution.entry(species).or_insert(0) += 1;
        
        debug!("📚 Learned: {} -> {:?} (success: {})", intent_kind, species, success);
    }
    
    /// Extract a pattern key from an intent kind
    fn extract_pattern_key(&self, intent_kind: &str) -> String {
        // Simplified - in production, would use NLP or embeddings
        intent_kind.split(':').next().unwrap_or(intent_kind).to_string()
    }
    
    /// Use heuristics when no pattern is found
    fn heuristic_prediction(&self, intent_kind: &str) -> Prediction {
        let species = match intent_kind.to_lowercase().as_str() {
            s if s.contains("code") || s.contains("reason") => SpeciesType::Cattle,
            s if s.contains("fetch") || s.contains("api") => SpeciesType::Duck,
            s if s.contains("classify") || s.contains("vote") => SpeciesType::Sheep,
            s if s.contains("file") || s.contains("debug") => SpeciesType::Goat,
            s if s.contains("sensor") || s.contains("gpio") => SpeciesType::Hog,
            s if s.contains("monitor") || s.contains("watch") => SpeciesType::Chicken,
            s if s.contains("transform") || s.contains("etl") => SpeciesType::Horse,
            _ => SpeciesType::Cattle,
        };
        
        Prediction {
            predicted_species: species,
            confidence: 0.3, // Low confidence for heuristics
            predicted_intent: None,
            prewarm: false,
        }
    }
    
    /// Get memory usage of the Shadow Pup
    pub fn memory_usage_mb(&self) -> f64 {
        // Base model size
        let mut size = SHADOW_PUP_SIZE_MB;
        
        // Pattern storage
        size += self.patterns.len() as f64 * 0.001; // ~1KB per pattern
        
        // Context storage
        size += self.recent_context.len() as f64 * 0.0001; // ~100 bytes per entry
        
        size
    }
    
    /// Pre-warm an adapter based on prediction
    pub async fn prewarm_adapter(&self, _species: SpeciesType) -> bool {
        // In production, this would signal the LoRA manager to pre-load
        // the adapter for the predicted species
        
        // For now, just return success
        true
    }
    
    /// Get statistics about learned patterns
    pub fn get_stats(&self) -> AnticipationStats {
        AnticipationStats {
            total_patterns: self.patterns.len(),
            total_interactions: self.recent_context.len() as u64,
            memory_usage_mb: self.memory_usage_mb(),
        }
    }
}

/// Statistics about the anticipation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnticipationStats {
    pub total_patterns: usize,
    pub total_interactions: u64,
    pub memory_usage_mb: f64,
}

impl Default for AnticipationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prediction() {
        let engine = AnticipationEngine::new();
        let intent = crate::collie::shepherd::Intent::new("fetch:data", "test");
        
        let prediction = engine.predict(&intent);
        
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
    }
    
    #[test]
    fn test_learning() {
        let mut engine = AnticipationEngine::new();
        
        engine.learn("test", SpeciesType::Cattle, true);
        engine.learn("test", SpeciesType::Cattle, true);
        
        let stats = engine.get_stats();
        assert!(stats.total_patterns > 0);
    }
}
