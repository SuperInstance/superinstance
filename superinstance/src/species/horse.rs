//! Horse - Pipeline Agents for ETL and Encoding
//! 
//! Horses are steady workers for linear throughput:
//! - ETL pipelines
//! - Data transformation
//! - Encoding/decoding
//! - Batch processing
//! 
//! Collie Strategy: "Saddle Up"
//! - Linear throughput
//! - Batch optimization
//! - Pipeline stages

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::{Species, SpeciesOps, SpeciesType};

/// Horse - Pipeline agent
#[derive(Debug, Clone)]
pub struct Horse {
    /// Unique identifier
    id: String,
    /// Fitness score
    fitness: f32,
    /// Generation number
    generation: u32,
    /// Path to LoRA adapter
    adapter_path: Option<String>,
    /// Pipeline stages
    stages: Vec<PipelineStage>,
    /// Batch size for optimization
    batch_size: usize,
}

/// A stage in a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub name: String,
    pub operation: String,
    pub params: Vec<String>,
}

/// Pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub stages_completed: usize,
    pub items_processed: usize,
    pub items_failed: usize,
    pub duration_ms: u64,
    pub output: Vec<PipelineOutput>,
}

/// Output from a pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutput {
    pub stage: String,
    pub items: usize,
    pub errors: Vec<String>,
}

/// ETL Job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ETLJob {
    pub name: String,
    pub source: String,
    pub transformations: Vec<String>,
    pub destination: String,
}

/// Horse statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HorseStats {
    pub pipelines_run: u64,
    pub items_processed: u64,
    pub bytes_processed: u64,
    pub avg_throughput_mbps: f64,
}

impl Horse {
    /// Create a new Horse instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            fitness: 0.85,
            generation: 1,
            adapter_path: Some("pasture/adapters/horse/pipeline_v1.safetensors".to_string()),
            stages: Vec::new(),
            batch_size: 100,
        }
    }
    
    /// Create with custom stages
    pub fn with_stages(id: String, stages: Vec<PipelineStage>) -> Self {
        Self {
            id,
            fitness: 0.85,
            generation: 1,
            adapter_path: Some("pasture/adapters/horse/pipeline_v1.safetensors".to_string()),
            stages,
            batch_size: 100,
        }
    }
    
    /// Add a stage to the pipeline
    pub fn add_stage(&mut self, stage: PipelineStage) {
        self.stages.push(stage);
    }
    
    /// Run a full pipeline
    pub async fn run_pipeline(&self, input: Vec<String>) -> anyhow::Result<PipelineResult> {
        let start = Instant::now();
        info!("🐎 Horse '{}' saddling up for pipeline...", self.id);
        
        let mut current_data = input.clone();
        let mut outputs = Vec::new();
        let mut total_processed = 0;
        let mut total_failed = 0;
        
        for stage in &self.stages {
            debug!("🐎 Stage: {}", stage.name);
            
            let (processed, failed) = self.run_stage(&stage, &current_data).await?;
            
            outputs.push(PipelineOutput {
                stage: stage.name.clone(),
                items: processed.len(),
                errors: vec![],
            });
            
            total_processed += processed.len();
            total_failed += failed.len();
            current_data = processed;
        }
        
        let elapsed = start.elapsed();
        info!("🐎 Horse '{}' pipeline complete in {:?}", self.id, elapsed);
        
        Ok(PipelineResult {
            stages_completed: self.stages.len(),
            items_processed: total_processed,
            items_failed: total_failed,
            duration_ms: elapsed.as_millis() as u64,
            output: outputs,
        })
    }
    
    /// Run a single stage
    async fn run_stage(&self, stage: &PipelineStage, input: &[String]) -> anyhow::Result<(Vec<String>, Vec<String>)> {
        // Simulate stage processing
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let processed: Vec<String> = input.iter()
            .map(|item| format!("[{}] {}", stage.name, item))
            .collect();
        
        Ok((processed, vec![]))
    }
    
    /// Run an ETL job
    pub async fn run_etl(&self, job: ETLJob) -> anyhow::Result<PipelineResult> {
        info!("🐎 Horse '{}' running ETL job: {}", self.id, job.name);
        
        // Build pipeline from ETL job
        let stages: Vec<PipelineStage> = job.transformations.iter()
            .map(|t| PipelineStage {
                name: t.clone(),
                operation: t.clone(),
                params: vec![],
            })
            .collect();
        
        // Simulate source data
        let input: Vec<String> = (0..self.batch_size)
            .map(|i| format!("item_{}", i))
            .collect();
        
        let horse = Horse::with_stages(self.id.clone(), stages);
        horse.run_pipeline(input).await
    }
    
    /// Encode data
    pub async fn encode(&self, data: &str, format: &str) -> anyhow::Result<String> {
        self.execute(format!("encode:{}:{}", format, data)).await
    }
    
    /// Decode data
    pub async fn decode(&self, data: &str, format: &str) -> anyhow::Result<String> {
        self.execute(format!("decode:{}:{}", format, data)).await
    }
    
    /// Transform data
    pub async fn transform(&self, data: &str, transformation: &str) -> anyhow::Result<String> {
        self.execute(format!("transform:{}:{}", transformation, data)).await
    }
}

#[async_trait]
impl Species for Horse {
    fn species_type(&self) -> SpeciesType {
        SpeciesType::Horse
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
        debug!("🐎 Horse '{}' executing: {}", self.id, task);
        
        // Parse task type
        let result = if task.starts_with("pipeline:") {
            let items: Vec<String> = task.trim_start_matches("pipeline:")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            let result = self.run_pipeline(items).await?;
            format!("[Horse Pipeline]\nStages: {}\nProcessed: {}\nDuration: {}ms",
                result.stages_completed, result.items_processed, result.duration_ms)
        } else if task.starts_with("etl:") {
            let job_name = task.trim_start_matches("etl:");
            let job = ETLJob {
                name: job_name.to_string(),
                source: "default_source".to_string(),
                transformations: vec!["extract".to_string(), "transform".to_string(), "load".to_string()],
                destination: "default_dest".to_string(),
            };
            let result = self.run_etl(job).await?;
            format!("[Horse ETL]\nJob: {}\nItems: {}\nDuration: {}ms",
                job_name, result.items_processed, result.duration_ms)
        } else if task.starts_with("encode:") {
            let parts: Vec<&str> = task.splitn(3, ':').collect();
            let format = parts.get(1).unwrap_or(&"json");
            let data = parts.get(2).unwrap_or(&"");
            format!("[Horse Encode]\nFormat: {}\nData: {} bytes", format, data.len())
        } else if task.starts_with("transform:") {
            let parts: Vec<&str> = task.splitn(3, ':').collect();
            let transformation = parts.get(1).unwrap_or(&"default");
            let data = parts.get(2).unwrap_or(&"");
            format!("[Horse Transform]\nOperation: {}\nResult: transformed_{}", transformation, data)
        } else {
            // Generic pipeline execution
            tokio::time::sleep(Duration::from_millis(50)).await;
            format!("[Horse Result] {}", task)
        };
        
        let elapsed = start.elapsed();
        info!("🐎 Horse '{}' completed in {:?}", self.id, elapsed);
        
        Ok(result)
    }
}

impl SpeciesOps for Horse {}

/// Stable - Multiple horses for parallel pipeline processing
pub struct Stable {
    horses: Vec<Horse>,
    work_queue: VecDeque<ETLJob>,
}

impl Stable {
    /// Create a new stable
    pub fn new(size: usize) -> Self {
        let horses = (0..size)
            .map(|i| Horse::new(format!("horse-{:02}", i)))
            .collect();
        Self {
            horses,
            work_queue: VecDeque::new(),
        }
    }
    
    /// Add a job to the queue
    pub fn enqueue(&mut self, job: ETLJob) {
        self.work_queue.push_back(job);
    }
    
    /// Process all queued jobs
    pub async fn process_all(&self) -> Vec<anyhow::Result<PipelineResult>> {
        let mut results = Vec::new();
        
        for (i, job) in self.work_queue.iter().enumerate() {
            let horse = &self.horses[i % self.horses.len()];
            results.push(horse.run_etl(job.clone()).await);
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_horse_pipeline() {
        let mut horse = Horse::new("test-01".to_string());
        horse.add_stage(PipelineStage {
            name: "extract".to_string(),
            operation: "extract".to_string(),
            params: vec![],
        });
        horse.add_stage(PipelineStage {
            name: "transform".to_string(),
            operation: "transform".to_string(),
            params: vec![],
        });
        
        let input = vec!["item1".to_string(), "item2".to_string()];
        let result = horse.run_pipeline(input).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_horse_etl() {
        let horse = Horse::new("test-01".to_string());
        let job = ETLJob {
            name: "test_job".to_string(),
            source: "source".to_string(),
            transformations: vec!["transform".to_string()],
            destination: "dest".to_string(),
        };
        let result = horse.run_etl(job).await;
        assert!(result.is_ok());
    }
}
