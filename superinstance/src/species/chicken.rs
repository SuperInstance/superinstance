//! Chicken - Monitor Agents
//! 
//! Chickens are always watching:
//! - System health monitoring
//! - Event detection
//! - Alert generation
//! - Heartbeat tracking
//! 
//! Collie Strategy: "Free Range"
//! - Constant pecking
//! - Watch for silence
//! - Low resource usage

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::{Species, SpeciesOps, SpeciesType};

/// Chicken - Monitoring agent
#[derive(Debug, Clone)]
pub struct Chicken {
    /// Unique identifier
    id: String,
    /// Fitness score
    fitness: f32,
    /// Generation number
    generation: u32,
    /// Path to LoRA adapter
    adapter_path: Option<String>,
    /// Monitoring interval in milliseconds
    peck_interval_ms: u64,
    /// Silence threshold (alert if no event for this duration)
    silence_threshold_ms: u64,
    /// Alert callbacks
    alert_handlers: Vec<String>,
}

/// Monitoring event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    pub source: String,
    pub kind: EventKind,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: AlertSeverity,
}

/// Types of monitor events
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EventKind {
    Heartbeat,
    Motion,
    Silence,
    Anomaly,
    Threshold,
    Custom,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// System health snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub vram_used_percent: f64,
    pub cpu_percent: f64,
    pub active_agents: usize,
    pub pending_tasks: usize,
    pub uptime_secs: u64,
}

/// Monitoring statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChickenStats {
    pub events_detected: u64,
    pub alerts_sent: u64,
    pub silences_detected: u64,
    pub heartbeats_received: u64,
    pub false_positives: u64,
}

impl Chicken {
    /// Create a new Chicken instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            fitness: 0.9,
            generation: 1,
            adapter_path: Some("pasture/adapters/chicken/monitor_v1.safetensors".to_string()),
            peck_interval_ms: 5000, // 5 seconds
            silence_threshold_ms: 30000, // 30 seconds
            alert_handlers: vec!["collie".to_string()],
        }
    }
    
    /// Create with custom monitoring interval
    pub fn with_interval(id: String, peck_interval_ms: u64) -> Self {
        Self {
            id,
            fitness: 0.9,
            generation: 1,
            adapter_path: Some("pasture/adapters/chicken/monitor_v1.safetensors".to_string()),
            peck_interval_ms,
            silence_threshold_ms: peck_interval_ms * 6,
            alert_handlers: vec!["collie".to_string()],
        }
    }
    
    /// Start monitoring loop
    pub async fn start_monitoring(&self) -> anyhow::Result<()> {
        info!("🐔 Chicken '{}' starting watch...", self.id);
        
        loop {
            self.peck().await?;
            tokio::time::sleep(Duration::from_millis(self.peck_interval_ms)).await;
        }
    }
    
    /// Single monitoring check ("peck")
    pub async fn peck(&self) -> anyhow::Result<Vec<MonitorEvent>> {
        let start = Instant::now();
        debug!("🐔 Chicken '{}' pecking...", self.id);
        
        let mut events = Vec::new();
        
        // Check system health
        let health = self.check_health().await;
        
        // Generate simulated events based on health
        if health.vram_used_percent > 90.0 {
            events.push(MonitorEvent {
                source: "system".to_string(),
                kind: EventKind::Threshold,
                message: format!("VRAM usage critical: {:.1}%", health.vram_used_percent),
                timestamp: chrono::Utc::now(),
                severity: AlertSeverity::Critical,
            });
        }
        
        // Simulate motion detection
        if rand::random::<f32>() < 0.1 {
            events.push(MonitorEvent {
                source: "motion_sensor".to_string(),
                kind: EventKind::Motion,
                message: "Motion detected".to_string(),
                timestamp: chrono::Utc::now(),
                severity: AlertSeverity::Info,
            });
        }
        
        let elapsed = start.elapsed();
        debug!("🐔 Chicken '{}' peck complete in {:?}", self.id, elapsed);
        
        Ok(events)
    }
    
    /// Check system health
    pub async fn check_health(&self) -> HealthSnapshot {
        // Simulate health check
        HealthSnapshot {
            timestamp: chrono::Utc::now(),
            vram_used_percent: 45.0 + rand::random::<f64>() * 20.0,
            cpu_percent: 30.0 + rand::random::<f64>() * 40.0,
            active_agents: 15,
            pending_tasks: 2,
            uptime_secs: 86400,
        }
    }
    
    /// Send alert
    pub async fn send_alert(&self, event: MonitorEvent) -> anyhow::Result<()> {
        match event.severity {
            AlertSeverity::Emergency => {
                warn!("🚨 EMERGENCY: {}", event.message);
            }
            AlertSeverity::Critical => {
                warn!("⚠️ CRITICAL: {}", event.message);
            }
            AlertSeverity::Warning => {
                info!("⚠️ WARNING: {}", event.message);
            }
            AlertSeverity::Info => {
                debug!("ℹ️ INFO: {}", event.message);
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl Species for Chicken {
    fn species_type(&self) -> SpeciesType {
        SpeciesType::Chicken
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
        debug!("🐔 Chicken '{}' executing: {}", self.id, task);
        
        let result = if task.starts_with("monitor:") {
            let target = task.trim_start_matches("monitor:");
            let health = self.check_health().await;
            format!("[Chicken Monitor]\nTarget: {}\nHealth: VRAM {:.1}%, CPU {:.1}%", 
                target, health.vram_used_percent, health.cpu_percent)
        } else if task.starts_with("watch:") {
            let target = task.trim_start_matches("watch:");
            format!("[Chicken Watch]\nWatching: {}\nStatus: Active", target)
        } else if task.starts_with("heartbeat") {
            "[Chicken Heartbeat] All systems nominal".to_string()
        } else {
            let events = self.peck().await?;
            format!("[Chicken Peck] {} events detected", events.len())
        };
        
        let elapsed = start.elapsed();
        debug!("🐔 Chicken '{}' completed in {:?}", self.id, elapsed);
        
        Ok(result)
    }
}

impl SpeciesOps for Chicken {}

/// Coop - A group of chickens coordinating monitoring
pub struct Coop {
    chickens: Vec<Chicken>,
    /// Last event times per source
    last_events: HashMap<String, Instant>,
}

impl Coop {
    /// Create a new coop
    pub fn new(size: usize) -> Self {
        let chickens = (0..size)
            .map(|i| Chicken::new(format!("chicken-{:02}", i)))
            .collect();
        Self {
            chickens,
            last_events: HashMap::new(),
        }
    }
    
    /// Check for silence across all sources
    pub fn check_silence(&self, threshold: Duration) -> Vec<String> {
        self.last_events.iter()
            .filter(|(_, &last)| last.elapsed() > threshold)
            .map(|(source, _)| source.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_chicken_peck() {
        let chicken = Chicken::new("test-01".to_string());
        let events = chicken.peck().await;
        assert!(events.is_ok());
    }
    
    #[tokio::test]
    async fn test_chicken_health() {
        let chicken = Chicken::new("test-01".to_string());
        let health = chicken.check_health().await;
        assert!(health.vram_used_percent >= 0.0);
    }
}
