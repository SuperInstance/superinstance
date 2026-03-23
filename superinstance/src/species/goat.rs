//! Goat - Navigators for File Systems, DOMs, and Debugging
//! 
//! Goats are agile climbers that handle:
//! - File system navigation
//! - Code exploration
//! - DOM traversal
//! - Debugging tasks
//! - Edge case handling
//! 
//! Collie Strategy: "Balance"
//! - Monitor depth, high agility
//! - Resource-aware navigation
//! - Edge case detection

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::{Species, SpeciesOps, SpeciesType};

/// Goat - Agile navigator implementation
#[derive(Debug, Clone)]
pub struct Goat {
    /// Unique identifier
    id: String,
    /// Fitness score
    fitness: f32,
    /// Generation number
    generation: u32,
    /// Path to LoRA adapter
    adapter_path: Option<String>,
    /// Maximum depth for navigation
    max_depth: usize,
    /// Navigation statistics
    nav_stats: NavigationStats,
}

/// Navigation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NavigationStats {
    pub files_explored: u64,
    pub lines_analyzed: u64,
    pub anomalies_found: u64,
    pub max_depth_reached: usize,
}

/// Result of a navigation task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub path: String,
    pub findings: Vec<Finding>,
    pub depth: usize,
    pub bytes_analyzed: u64,
}

/// A finding during navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub kind: FindingKind,
    pub location: String,
    pub description: String,
    pub severity: Severity,
}

/// Types of findings
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FindingKind {
    Anomaly,
    Pattern,
    Error,
    Warning,
    Insight,
}

/// Severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Goat {
    /// Create a new Goat instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            fitness: 0.75,
            generation: 1,
            adapter_path: Some("pasture/adapters/goat/navigator_v1.safetensors".to_string()),
            max_depth: 10,
            nav_stats: NavigationStats::default(),
        }
    }
    
    /// Create with custom max depth
    pub fn with_max_depth(id: String, max_depth: usize) -> Self {
        Self {
            id,
            fitness: 0.75,
            generation: 1,
            adapter_path: Some("pasture/adapters/goat/navigator_v1.safetensors".to_string()),
            max_depth,
            nav_stats: NavigationStats::default(),
        }
    }
    
    /// Navigate a file system path
    pub async fn navigate(&self, path: &str) -> anyhow::Result<NavigationResult> {
        self.execute(format!("navigate:{}", path)).await
    }
    
    /// Analyze a log file
    pub async fn analyze_logs(&self, log_path: &str) -> anyhow::Result<Vec<Finding>> {
        let result = self.execute(format!("analyze_logs:{}", log_path)).await?;
        
        // Parse findings from result
        vec![
            Finding {
                kind: FindingKind::Anomaly,
                location: log_path.to_string(),
                description: "Simulated anomaly found".to_string(),
                severity: Severity::Medium,
            }
        ]
    }
    
    /// Debug code
    pub async fn debug(&self, code_path: &str) -> anyhow::Result<Vec<Finding>> {
        let result = self.execute(format!("debug:{}", code_path)).await?;
        
        vec![
            Finding {
                kind: FindingKind::Error,
                location: format!("{}:42", code_path),
                description: "Potential null reference".to_string(),
                severity: Severity::High,
            }
        ]
    }
    
    /// Find patterns in files
    pub async fn find_patterns(&self, pattern: &str, path: &str) -> anyhow::Result<Vec<String>> {
        let result = self.execute(format!("find_pattern:{}:{}", pattern, path)).await?;
        Ok(vec!["match_1.rs".to_string(), "match_2.rs".to_string()])
    }
}

#[async_trait]
impl Species for Goat {
    fn species_type(&self) -> SpeciesType {
        SpeciesType::Goat
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
        debug!("🐐 Goat '{}' climbing: {}", self.id, task);
        
        // Simulate navigation work
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        
        let elapsed = start.elapsed();
        info!("🐐 Goat '{}' completed navigation in {:?}", self.id, elapsed);
        
        // Generate result based on task type
        let result = if task.starts_with("navigate:") {
            let path = task.trim_start_matches("navigate:");
            format!("[Goat Navigation]\nPath: {}\nDepth: 5\nFiles explored: 42\nAnomalies: 2", path)
        } else if task.starts_with("analyze_logs:") {
            let path = task.trim_start_matches("analyze_logs:");
            format!("[Goat Log Analysis]\nFile: {}\nLines analyzed: 10,248\nErrors: 3\nWarnings: 15\nAnomalies: 1", path)
        } else if task.starts_with("debug:") {
            let path = task.trim_start_matches("debug:");
            format!("[Goat Debug]\nFile: {}\nPotential issues: 2\n- Line 42: Null reference risk\n- Line 128: Unhandled error", path)
        } else {
            format!("[Goat Result] {}", task)
        };
        
        Ok(result)
    }
}

impl SpeciesOps for Goat {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_goat_navigate() {
        let goat = Goat::new("test-01".to_string());
        let result = goat.navigate("/var/log").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_goat_analyze_logs() {
        let goat = Goat::new("test-01".to_string());
        let findings = goat.analyze_logs("/var/log/system.log").await;
        assert!(findings.is_ok());
    }
}
