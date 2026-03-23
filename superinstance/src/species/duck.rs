//! Duck - Network/API Agents
//! 
//! Ducks handle all external communications:
//! - API calls
//! - Web scraping
//! - Data fetching
//! - External service integration
//! 
//! Collie Strategy: "Whistle Stop"
//! - Async, fire-and-recall
//! - Timeout handling
//! - Retry logic

use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::{Species, SpeciesOps, SpeciesType};

/// Duck - Async network agent
#[derive(Debug, Clone)]
pub struct Duck {
    /// Unique identifier
    id: String,
    /// Fitness score
    fitness: f32,
    /// Generation number
    generation: u32,
    /// Path to LoRA adapter
    adapter_path: Option<String>,
    /// Default timeout
    default_timeout_ms: u64,
    /// Max retries
    max_retries: u32,
}

/// Result of a Duck fetch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckFetch {
    pub status: u16,
    pub data: String,
    pub latency_ms: u64,
    pub source: String,
}

/// Duck configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckConfig {
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub user_agent: String,
}

impl Default for DuckConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_retries: 3,
            user_agent: "SuperInstance-Duck/1.0".to_string(),
        }
    }
}

impl Duck {
    /// Create a new Duck instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            fitness: 0.9,
            generation: 1,
            adapter_path: Some("pasture/adapters/duck/network_v1.safetensors".to_string()),
            default_timeout_ms: 5000,
            max_retries: 3,
        }
    }
    
    /// Create with custom config
    pub fn with_config(id: String, config: DuckConfig) -> Self {
        Self {
            id,
            fitness: 0.9,
            generation: 1,
            adapter_path: Some("pasture/adapters/duck/network_v1.safetensors".to_string()),
            default_timeout_ms: config.timeout_ms,
            max_retries: config.max_retries,
        }
    }
    
    /// Fetch from a URL
    pub async fn fetch(&self, url: &str) -> anyhow::Result<DuckFetch> {
        let start = Instant::now();
        info!("🦆 Duck '{}' flying to: {}", self.id, url);
        
        // In production, this would use reqwest
        // Simulate network latency
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let latency = start.elapsed().as_millis() as u64;
        
        // Simulate successful fetch
        Ok(DuckFetch {
            status: 200,
            data: format!("Fetched data from {}", url),
            latency_ms: latency,
            source: url.to_string(),
        })
    }
    
    /// Fetch with timeout
    pub async fn fetch_with_timeout(&self, url: &str, timeout_ms: u64) -> anyhow::Result<DuckFetch> {
        tokio::time::timeout(
            Duration::from_millis(timeout_ms),
            self.fetch(url)
        ).await?
    }
    
    /// Make an API call
    pub async fn api_call(&self, endpoint: &str, payload: &str) -> anyhow::Result<String> {
        self.execute(format!("api:{}:{}", endpoint, payload)).await
    }
}

#[async_trait]
impl Species for Duck {
    fn species_type(&self) -> SpeciesType {
        SpeciesType::Duck
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
        debug!("🦆 Duck '{}' executing: {}", self.id, task);
        
        // Parse task type
        if task.starts_with("fetch:") {
            let url = task.trim_start_matches("fetch:");
            let result = self.fetch_with_timeout(url, self.default_timeout_ms).await?;
            
            let elapsed = start.elapsed();
            info!("🦆 Duck '{}' returned in {:?}", self.id, elapsed);
            
            return Ok(result.data);
        }
        
        if task.starts_with("api:") {
            // Simulate API call
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let elapsed = start.elapsed();
            info!("🦆 Duck '{}' API call complete in {:?}", self.id, elapsed);
            
            return Ok(format!("API Response: {{\"status\": \"success\"}}"));
        }
        
        // Generic execution
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        Ok(format!("[Duck Result] {}", task))
    }
}

impl SpeciesOps for Duck {}

/// Duck Flock - Multiple ducks for parallel fetching
pub struct DuckFlock {
    ducks: Vec<Duck>,
}

impl DuckFlock {
    /// Create a new flock of ducks
    pub fn new(size: usize) -> Self {
        let ducks = (0..size)
            .map(|i| Duck::new(format!("duck-{:02}", i)))
            .collect();
        Self { ducks }
    }
    
    /// Fetch multiple URLs in parallel
    pub async fn fetch_all(&self, urls: &[&str]) -> Vec<anyhow::Result<DuckFetch>> {
        let start = Instant::now();
        info!("🦆 Duck flock of {} fetching {} URLs", self.ducks.len(), urls.len());
        
        let mut tasks = Vec::new();
        for (i, url) in urls.iter().enumerate() {
            let duck = &self.ducks[i % self.ducks.len()];
            let url = url.to_string();
            tasks.push(async move {
                duck.fetch(&url).await
            });
        }
        
        // Execute all fetches concurrently
        let results: Vec<_> = futures::future::join_all(tasks).await;
        
        let elapsed = start.elapsed();
        info!("🦆 Duck flock completed in {:?}", elapsed);
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_duck_fetch() {
        let duck = Duck::new("test-01".to_string());
        let result = duck.fetch("https://example.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, 200);
    }
    
    #[tokio::test]
    async fn test_duck_execute() {
        let duck = Duck::new("test-01".to_string());
        let result = duck.execute("fetch:https://example.com".to_string()).await;
        assert!(result.is_ok());
    }
}
