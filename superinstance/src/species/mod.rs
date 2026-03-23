//! Species - The Livestock
//! 
//! Species are specialized LoRA Adapters (Personalities), not separate processes.
//! They "possess" the Base Model when needed.
//! 
//! Each species has:
//! - A specific purpose and capability
//! - Resource requirements
//! - A herding strategy
//! - A fitness score for evolution

mod cattle;
mod sheep;
mod duck;
mod goat;
mod hog;
mod chicken;
mod horse;

pub use cattle::*;
pub use sheep::*;
pub use duck::*;
pub use goat::*;
pub use hog::*;
pub use chicken::*;
pub use horse::*;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Species Type - All available agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpeciesType {
    /// Heavy LLMs - Deep reasoning, code gen
    Cattle,
    /// Classifiers - Voting/filtering
    Sheep,
    /// Network/API - External calls
    Duck,
    /// Navigators - File systems, DOMs, debugging
    Goat,
    /// Hardware - GPIO, Sensors
    Hog,
    /// Monitors - Heartbeats, alerts
    Chicken,
    /// Pipelines - ETL, encoding
    Horse,
}

impl SpeciesType {
    /// Get the emoji for this species
    pub fn emoji(&self) -> &'static str {
        match self {
            SpeciesType::Cattle => "🐄",
            SpeciesType::Sheep => "🐑",
            SpeciesType::Duck => "🦆",
            SpeciesType::Goat => "🐐",
            SpeciesType::Hog => "🐗",
            SpeciesType::Chicken => "🐔",
            SpeciesType::Horse => "🐎",
        }
    }
    
    /// Get the name with emoji
    pub fn emoji_name(&self) -> String {
        format!("{} {:?}", self.emoji(), self)
    }
    
    /// Get typical VRAM usage in MB
    pub fn typical_vram_mb(&self) -> u64 {
        match self {
            SpeciesType::Cattle => 500,  // Heavy adapter
            SpeciesType::Sheep => 50,    // Tiny classifier
            SpeciesType::Duck => 100,    // Network utilities
            SpeciesType::Goat => 150,    // Navigation tools
            SpeciesType::Hog => 10,      // Minimal, real-time
            SpeciesType::Chicken => 5,   // Tiny monitor
            SpeciesType::Horse => 200,   // Pipeline tools
        }
    }
    
    /// Get typical execution time in ms
    pub fn typical_latency_ms(&self) -> u64 {
        match self {
            SpeciesType::Cattle => 2000,
            SpeciesType::Sheep => 100,
            SpeciesType::Duck => 500,
            SpeciesType::Goat => 200,
            SpeciesType::Hog => 10,
            SpeciesType::Chicken => 5,
            SpeciesType::Horse => 1000,
        }
    }
    
    /// Get the collie strategy name
    pub fn strategy_name(&self) -> &'static str {
        match self {
            SpeciesType::Cattle => "Strong Eye",
            SpeciesType::Sheep => "The Wear",
            SpeciesType::Duck => "Whistle Stop",
            SpeciesType::Goat => "Balance",
            SpeciesType::Hog => "The Drive",
            SpeciesType::Chicken => "Free Range",
            SpeciesType::Horse => "Saddle Up",
        }
    }
    
    /// Get all species types
    pub fn all() -> Vec<SpeciesType> {
        vec![
            SpeciesType::Cattle,
            SpeciesType::Sheep,
            SpeciesType::Duck,
            SpeciesType::Goat,
            SpeciesType::Hog,
            SpeciesType::Chicken,
            SpeciesType::Horse,
        ]
    }
}

impl fmt::Display for SpeciesType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeciesType::Cattle => write!(f, "Cattle"),
            SpeciesType::Sheep => write!(f, "Sheep"),
            SpeciesType::Duck => write!(f, "Duck"),
            SpeciesType::Goat => write!(f, "Goat"),
            SpeciesType::Hog => write!(f, "Hog"),
            SpeciesType::Chicken => write!(f, "Chicken"),
            SpeciesType::Horse => write!(f, "Horse"),
        }
    }
}

/// Intent - A task to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub kind: String,
    pub payload: String,
    pub priority: u8,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Intent {
    pub fn new(kind: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            payload: payload.into(),
            priority: 5,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Species Trait - All livestock must implement this
#[async_trait]
pub trait Species: Send + Sync {
    /// Get the species type
    fn species_type(&self) -> SpeciesType;
    
    /// Get the agent's unique ID
    fn id(&self) -> &str;
    
    /// Get the agent's fitness score (0.0 to 1.0)
    fn fitness(&self) -> f32;
    
    /// Update the fitness score
    fn set_fitness(&mut self, score: f32);
    
    /// Get the agent's generation
    fn generation(&self) -> u32;
    
    /// Get the adapter path (LoRA weights)
    fn adapter_path(&self) -> Option<&str>;
    
    /// Execute a task
    async fn execute(&self, task: String) -> anyhow::Result<String>;
}

/// Species Operations trait - Common operations for all species
#[async_trait]
pub trait SpeciesOps: Species {
    /// Check if the agent is healthy
    async fn health_check(&self) -> bool {
        self.fitness() > 0.3
    }
    
    /// Get statistics for this agent
    fn stats(&self) -> SpeciesStats {
        SpeciesStats {
            species: self.species_type(),
            id: self.id().to_string(),
            fitness: self.fitness(),
            generation: self.generation(),
        }
    }
}

/// Statistics for a species instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesStats {
    pub species: SpeciesType,
    pub id: String,
    pub fitness: f32,
    pub generation: u32,
}

/// Species Registry - Manages all active livestock
pub struct SpeciesRegistry {
    /// Active agents by ID
    agents: HashMap<String, ActiveAgent>,
    /// Count by species type
    counts: HashMap<SpeciesType, usize>,
}

/// An active agent in the registry
#[derive(Debug, Clone)]
pub struct ActiveAgent {
    pub id: String,
    pub species: SpeciesType,
    pub fitness: f32,
    pub generation: u32,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub total_tasks: u64,
    pub successful_tasks: u64,
}

impl SpeciesRegistry {
    /// Create a new species registry
    pub fn new() -> Self {
        // Initialize with some default agents
        let mut registry = Self {
            agents: HashMap::new(),
            counts: HashMap::new(),
        };
        
        // Add some initial livestock
        registry.spawn_initial();
        
        registry
    }
    
    /// Spawn initial livestock
    fn spawn_initial(&mut self) {
        // 2 Cattle
        for i in 0..2 {
            self.register(ActiveAgent {
                id: format!("cattle-{:02}", i),
                species: SpeciesType::Cattle,
                fitness: 0.8,
                generation: 1,
                last_used: chrono::Utc::now(),
                total_tasks: 0,
                successful_tasks: 0,
            });
        }
        
        // 5 Sheep
        for i in 0..5 {
            self.register(ActiveAgent {
                id: format!("sheep-{:02}", i),
                species: SpeciesType::Sheep,
                fitness: 0.85,
                generation: 1,
                last_used: chrono::Utc::now(),
                total_tasks: 0,
                successful_tasks: 0,
            });
        }
        
        // 3 Ducks
        for i in 0..3 {
            self.register(ActiveAgent {
                id: format!("duck-{:02}", i),
                species: SpeciesType::Duck,
                fitness: 0.9,
                generation: 1,
                last_used: chrono::Utc::now(),
                total_tasks: 0,
                successful_tasks: 0,
            });
        }
        
        // 2 Goats
        for i in 0..2 {
            self.register(ActiveAgent {
                id: format!("goat-{:02}", i),
                species: SpeciesType::Goat,
                fitness: 0.75,
                generation: 1,
                last_used: chrono::Utc::now(),
                total_tasks: 0,
                successful_tasks: 0,
            });
        }
        
        // 1 Hog
        self.register(ActiveAgent {
            id: "hog-00".to_string(),
            species: SpeciesType::Hog,
            fitness: 0.95,
            generation: 1,
            last_used: chrono::Utc::now(),
            total_tasks: 0,
            successful_tasks: 0,
        });
        
        // 3 Chickens
        for i in 0..3 {
            self.register(ActiveAgent {
                id: format!("chicken-{:02}", i),
                species: SpeciesType::Chicken,
                fitness: 0.9,
                generation: 1,
                last_used: chrono::Utc::now(),
                total_tasks: 0,
                successful_tasks: 0,
            });
        }
        
        // 1 Horse
        self.register(ActiveAgent {
            id: "horse-00".to_string(),
            species: SpeciesType::Horse,
            fitness: 0.85,
            generation: 1,
            last_used: chrono::Utc::now(),
            total_tasks: 0,
            successful_tasks: 0,
        });
    }
    
    /// Register a new agent
    pub fn register(&mut self, agent: ActiveAgent) {
        *self.counts.entry(agent.species).or_insert(0) += 1;
        self.agents.insert(agent.id.clone(), agent);
    }
    
    /// Get an agent by ID
    pub fn get(&self, id: &str) -> Option<&ActiveAgent> {
        self.agents.get(id)
    }
    
    /// Get all agents of a specific species
    pub fn get_by_species(&self, species: SpeciesType) -> Vec<&ActiveAgent> {
        self.agents.values()
            .filter(|a| a.species == species)
            .collect()
    }
    
    /// Get counts by species
    pub fn get_counts(&self) -> HashMap<SpeciesType, usize> {
        self.counts.clone()
    }
    
    /// Get total active agents
    pub fn total_active(&self) -> usize {
        self.agents.len()
    }
    
    /// Update agent fitness
    pub fn update_fitness(&mut self, id: &str, fitness: f32) {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.fitness = fitness;
        }
    }
    
    /// Record a task completion
    pub fn record_task(&mut self, id: &str, success: bool) {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.total_tasks += 1;
            if success {
                agent.successful_tasks += 1;
            }
            agent.last_used = chrono::Utc::now();
        }
    }
    
    /// Remove agents below fitness threshold
    pub fn cull(&mut self, threshold: f32) -> Vec<String> {
        let to_remove: Vec<String> = self.agents.values()
            .filter(|a| a.fitness < threshold)
            .map(|a| a.id.clone())
            .collect();
        
        for id in &to_remove {
            if let Some(agent) = self.agents.remove(id) {
                *self.counts.get_mut(&agent.species).unwrap() -= 1;
            }
        }
        
        to_remove
    }
    
    /// Get all agents sorted by fitness
    pub fn get_ranked(&self) -> Vec<&ActiveAgent> {
        let mut agents: Vec<_> = self.agents.values().collect();
        agents.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        agents
    }
}

impl Default for SpeciesRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_species_type() {
        assert_eq!(SpeciesType::Cattle.emoji(), "🐄");
        assert_eq!(SpeciesType::Cattle.typical_vram_mb(), 500);
    }
    
    #[test]
    fn test_registry() {
        let registry = SpeciesRegistry::new();
        assert!(registry.total_active() > 0);
    }
}
