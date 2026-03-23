//! Sheep - Classifiers for Voting and Filtering
//! 
//! Sheep work in flocks (ensembles) to:
//! - Classify content
//! - Filter data
//! - Vote on decisions
//! - Moderate content
//! 
//! Collie Strategy: "The Wear"
//! - Flock ensemble, consensus voting
//! - Low individual VRAM (50MB each)
//! - Parallel execution for voting

use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::{Species, SpeciesOps, SpeciesType};

/// Sheep - Lightweight classifier implementation
#[derive(Debug, Clone)]
pub struct Sheep {
    /// Unique identifier
    id: String,
    /// Fitness score
    fitness: f32,
    /// Generation number
    generation: u32,
    /// Path to LoRA adapter
    adapter_path: Option<String>,
    /// Classification labels this sheep knows
    labels: Vec<String>,
}

/// Vote from a sheep in a flock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheepVote {
    pub sheep_id: String,
    pub label: String,
    pub confidence: f32,
}

/// Result of a flock voting process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlockConsensus {
    pub winning_label: String,
    pub confidence: f32,
    pub votes: Vec<SheepVote>,
    pub agreement_ratio: f64,
}

impl Sheep {
    /// Create a new Sheep instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            fitness: 0.85,
            generation: 1,
            adapter_path: Some("pasture/adapters/sheep/classifier_v1.safetensors".to_string()),
            labels: vec!["spam".to_string(), "legitimate".to_string(), "urgent".to_string()],
        }
    }
    
    /// Create with custom labels
    pub fn with_labels(id: String, labels: Vec<String>) -> Self {
        Self {
            id,
            fitness: 0.85,
            generation: 1,
            adapter_path: Some("pasture/adapters/sheep/classifier_v1.safetensors".to_string()),
            labels,
        }
    }
    
    /// Classify a piece of content
    pub async fn classify(&self, content: &str) -> anyhow::Result<SheepVote> {
        let result = self.execute(format!("classify:{}", content)).await?;
        
        // Parse the classification result
        let label = self.labels.iter()
            .find(|l| result.to_lowercase().contains(&l.to_lowercase()))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        
        Ok(SheepVote {
            sheep_id: self.id.clone(),
            label,
            confidence: 0.8, // Simulated confidence
        })
    }
    
    /// Get known labels
    pub fn labels(&self) -> &[String] {
        &self.labels
    }
}

#[async_trait]
impl Species for Sheep {
    fn species_type(&self) -> SpeciesType {
        SpeciesType::Sheep
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
        debug!("🐑 Sheep '{}' classifying...", self.id);
        
        // Simulate classification
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let elapsed = start.elapsed();
        debug!("🐑 Sheep '{}' completed in {:?}", self.id, elapsed);
        
        // Return simulated classification
        let result = if task.contains("spam") || task.contains("buy now") {
            "classification: spam (confidence: 0.92)"
        } else if task.contains("urgent") || task.contains("asap") {
            "classification: urgent (confidence: 0.85)"
        } else {
            "classification: legitimate (confidence: 0.78)"
        };
        
        Ok(result.to_string())
    }
}

impl SpeciesOps for Sheep {}

/// Flock - A group of sheep that vote together
pub struct Flock {
    sheep: Vec<Sheep>,
}

impl Flock {
    /// Create a new flock with n sheep
    pub fn new(size: usize) -> Self {
        let sheep = (0..size)
            .map(|i| Sheep::new(format!("flock-{:02}", i)))
            .collect();
        Self { sheep }
    }
    
    /// Add a sheep to the flock
    pub fn add(&mut self, sheep: Sheep) {
        self.sheep.push(sheep);
    }
    
    /// Get flock size
    pub fn size(&self) -> usize {
        self.sheep.len()
    }
    
    /// Run a vote on content
    pub async fn vote(&self, content: &str) -> anyhow::Result<FlockConsensus> {
        let start = Instant::now();
        info!("🐑 Flock of {} voting on content...", self.sheep.len());
        
        // Collect votes from all sheep
        let mut votes = Vec::new();
        for sheep in &self.sheep {
            let vote = sheep.classify(content).await?;
            votes.push(vote);
        }
        
        // Calculate consensus
        let mut label_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut label_confidence: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        
        for vote in &votes {
            *label_counts.entry(vote.label.clone()).or_insert(0) += 1;
            *label_confidence.entry(vote.label.clone()).or_insert(0.0) += vote.confidence;
        }
        
        // Find winning label
        let (winning_label, count) = label_counts.iter()
            .max_by_key(|(_, c)| *c)
            .map(|(l, c)| (l.clone(), *c))
            .unwrap_or(("unknown".to_string(), 0));
        
        let avg_confidence = label_confidence.get(&winning_label)
            .copied()
            .unwrap_or(0.0) / count.max(1) as f32;
        
        let agreement_ratio = count as f64 / self.sheep.len() as f64;
        
        let elapsed = start.elapsed();
        info!("🐑 Flock consensus: {} ({:.0}% agreement) in {:?}", 
            winning_label, agreement_ratio * 100.0, elapsed);
        
        Ok(FlockConsensus {
            winning_label,
            confidence: avg_confidence,
            votes,
            agreement_ratio,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sheep_classify() {
        let sheep = Sheep::new("test-01".to_string());
        let result = sheep.classify("This is spam!").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_flock_vote() {
        let flock = Flock::new(5);
        let consensus = flock.vote("Hello, legitimate message").await;
        assert!(consensus.is_ok());
        let c = consensus.unwrap();
        assert!(c.agreement_ratio >= 0.0);
    }
}
