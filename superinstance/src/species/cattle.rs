//! Cattle - Heavy LLMs for Deep Reasoning and Code Generation
//! 
//! Cattle are the heavy lifters of the Ranch. They handle:
//! - Complex reasoning tasks
//! - Code generation
//! - Analysis and synthesis
//! - Multi-step problem solving
//! 
//! Collie Strategy: "Strong Eye"
//! - Lock GPU, block, execute
//! - High VRAM usage (500MB+)
//! - Single-threaded, steady pressure

use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::{Species, SpeciesOps, SpeciesType};

/// Cattle - Heavy LLM implementation
#[derive(Debug, Clone)]
pub struct Cattle {
    /// Unique identifier
    id: String,
    /// Fitness score (0.0 to 1.0)
    fitness: f32,
    /// Generation number
    generation: u32,
    /// Path to LoRA adapter weights
    adapter_path: Option<String>,
    /// Current task (if any)
    current_task: Option<String>,
    /// Task statistics
    stats: CattleStats,
}

/// Statistics for Cattle operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CattleStats {
    pub total_inferences: u64,
    pub total_tokens_generated: u64,
    pub average_latency_ms: f64,
    pub peak_vram_mb: u64,
}

impl Cattle {
    /// Create a new Cattle instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            fitness: 0.8,
            generation: 1,
            adapter_path: Some("pasture/adapters/cattle/reasoning_v1.safetensors".to_string()),
            current_task: None,
            stats: CattleStats::default(),
        }
    }
    
    /// Create with custom adapter
    pub fn with_adapter(id: String, adapter_path: String) -> Self {
        Self {
            id,
            fitness: 0.8,
            generation: 1,
            adapter_path: Some(adapter_path),
            current_task: None,
            stats: CattleStats::default(),
        }
    }
    
    /// Get estimated VRAM usage
    pub fn estimated_vram_mb(&self) -> u64 {
        // Base adapter size + KV cache estimate
        SpeciesType::Cattle.typical_vram_mb() + 100 // Extra for KV cache
    }
    
    /// Perform deep reasoning on a prompt
    pub async fn reason(&mut self, prompt: &str) -> anyhow::Result<String> {
        self.execute(format!("reason:{}", prompt)).await
    }
    
    /// Generate code from a description
    pub async fn generate_code(&mut self, description: &str) -> anyhow::Result<String> {
        self.execute(format!("generate_code:{}", description)).await
    }
    
    /// Synthesize information from multiple sources
    pub async fn synthesize(&mut self, sources: &[String]) -> anyhow::Result<String> {
        let combined = sources.join("\n---\n");
        self.execute(format!("synthesize:{}", combined)).await
    }
}

#[async_trait]
impl Species for Cattle {
    fn species_type(&self) -> SpeciesType {
        SpeciesType::Cattle
    }
    
    fn id(&self) -> &str {
        &self.id
    }
    
    fn fitness(&self) -> f32 {
        self.fitness
    }
    
    fn set_fitness(&mut self, score: f32) {
        self.fitness = score.clamp(0.0, 1.0);
    }
    
    fn generation(&self) -> u32 {
        self.generation
    }
    
    fn adapter_path(&self) -> Option<&str> {
        self.adapter_path.as_deref()
    }
    
    async fn execute(&self, task: String) -> anyhow::Result<String> {
        let start = Instant::now();
        info!("🐄 Cattle '{}' beginning heavy computation...", self.id);
        debug!("Task: {}", task);
        
        // In production, this would:
        // 1. Load the LoRA adapter if not already loaded
        // 2. Acquire GPU lock
        // 3. Run inference through the base model + adapter
        // 4. Release GPU lock
        
        // Simulate heavy computation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let elapsed = start.elapsed();
        info!("🐄 Cattle '{}' completed in {:?}", self.id, elapsed);
        
        // Return a simulated result based on task type
        let result = if task.starts_with("reason:") {
            format!("[Cattle Analysis]\nAnalyzed: {}\nConclusion: Deep reasoning complete.", 
                task.trim_start_matches("reason:"))
        } else if task.starts_with("generate_code:") {
            format!("[Cattle Code Gen]\n// Generated for: {}\nfn generated_function() {{\n    // Implementation\n}}", 
                task.trim_start_matches("generate_code:"))
        } else if task.starts_with("synthesize:") {
            format!("[Cattle Synthesis]\nSynthesized {} characters of input.\nKey insights extracted.", 
                task.trim_start_matches("synthesize:").len())
        } else {
            format!("[Cattle Result]\nProcessed: {}", task)
        };
        
        Ok(result)
    }
}

impl SpeciesOps for Cattle {}

/// Cattle Agent - A managed Cattle instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CattleAgent {
    pub id: String,
    pub fitness: f32,
    pub generation: u32,
    pub adapter_id: String,
    pub speciality: CattleSpeciality,
}

/// Speciality areas for Cattle
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CattleSpeciality {
    CodeGeneration,
    Reasoning,
    Analysis,
    Synthesis,
    General,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cattle_execute() {
        let cattle = Cattle::new("test-01".to_string());
        let result = cattle.execute("test task".to_string()).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_cattle_reason() {
        let mut cattle = Cattle::new("test-01".to_string());
        let result = cattle.reason("Why is the sky blue?").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Analysis"));
    }
}
