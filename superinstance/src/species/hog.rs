//! Hog - Hardware Interface Agents
//! 
//! Hogs handle real-time hardware operations:
//! - GPIO control
//! - Sensor reading
//! - Actuator control
//! - Real-time signal processing
//! 
//! Collie Strategy: "The Drive"
//! - Real-time priority
//! - Low latency (<10ms)
//! - Direct hardware access

use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::{Species, SpeciesOps, SpeciesType};

/// Hog - Real-time hardware agent
#[derive(Debug, Clone)]
pub struct Hog {
    /// Unique identifier
    id: String,
    /// Fitness score
    fitness: f32,
    /// Generation number
    generation: u32,
    /// Path to LoRA adapter
    adapter_path: Option<String>,
    /// Real-time priority enabled
    rt_priority: bool,
    /// Maximum latency budget in microseconds
    latency_budget_us: u64,
}

/// GPIO Pin state
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PinState {
    Low,
    High,
    Input,
    Output,
    Pwm { duty_cycle: f32, frequency: u32 },
}

/// Sensor reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub quality: f32,
}

/// Actuator command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorCommand {
    pub actuator_id: String,
    pub command: String,
    pub params: Vec<f64>,
}

/// Hardware statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HogStats {
    pub gpio_operations: u64,
    pub sensor_reads: u64,
    pub actuator_commands: u64,
    pub max_latency_us: u64,
    pub missed_deadlines: u64,
}

impl Hog {
    /// Create a new Hog instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            fitness: 0.95,
            generation: 1,
            adapter_path: Some("pasture/adapters/hog/hardware_v1.safetensors".to_string()),
            rt_priority: true,
            latency_budget_us: 10000, // 10ms
        }
    }
    
    /// Read a sensor
    pub async fn read_sensor(&self, sensor_id: &str) -> anyhow::Result<SensorReading> {
        let start = Instant::now();
        
        // Simulate sensor read with minimal latency
        let reading = SensorReading {
            sensor_id: sensor_id.to_string(),
            value: 23.5, // Simulated value
            unit: "celsius".to_string(),
            timestamp: chrono::Utc::now(),
            quality: 0.98,
        };
        
        let elapsed_us = start.elapsed().as_micros() as u64;
        
        if elapsed_us > self.latency_budget_us {
            warn!("⚠️ HOG LATENCY VIOLATION: {}us > {}us budget", 
                elapsed_us, self.latency_budget_us);
        }
        
        Ok(reading)
    }
    
    /// Write to a GPIO pin
    pub async fn gpio_write(&self, pin: u8, state: PinState) -> anyhow::Result<()> {
        let start = Instant::now();
        
        // Simulate GPIO operation
        tokio::time::sleep(Duration::from_micros(100)).await;
        
        let elapsed_us = start.elapsed().as_micros() as u64;
        debug!("🐗 Hog '{}' GPIO write on pin {} in {}us", self.id, pin, elapsed_us);
        
        Ok(())
    }
    
    /// Send actuator command
    pub async fn actuator_command(&self, command: ActuatorCommand) -> anyhow::Result<()> {
        let start = Instant::now();
        
        // Simulate actuator operation
        tokio::time::sleep(Duration::from_micros(500)).await;
        
        let elapsed_us = start.elapsed().as_micros() as u64;
        debug!("🐗 Hog '{}' actuator command '{}' in {}us", 
            self.id, command.command, elapsed_us);
        
        Ok(())
    }
    
    /// Check if real-time priority is set
    pub fn is_rt_priority(&self) -> bool {
        self.rt_priority
    }
}

#[async_trait]
impl Species for Hog {
    fn species_type(&self) -> SpeciesType {
        SpeciesType::Hog
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
        debug!("🐗 Hog '{}' executing: {}", self.id, task);
        
        // Parse and execute hardware task
        let result = if task.starts_with("sensor:") {
            let sensor_id = task.trim_start_matches("sensor:");
            let reading = self.read_sensor(sensor_id).await?;
            format!("{:.2} {} (quality: {:.2})", reading.value, reading.unit, reading.quality)
        } else if task.starts_with("gpio:") {
            let parts: Vec<&str> = task.split(':').collect();
            let pin: u8 = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
            let state = match parts.get(2) {
                Some(&"high") => PinState::High,
                Some(&"low") => PinState::Low,
                _ => PinState::Low,
            };
            self.gpio_write(pin, state).await?;
            format!("GPIO pin {} set to {:?}", pin, state)
        } else if task.starts_with("actuator:") {
            let parts: Vec<&str> = task.split(':').collect();
            let cmd = ActuatorCommand {
                actuator_id: parts.get(1).unwrap_or(&"0").to_string(),
                command: parts.get(2).unwrap_or(&"move").to_string(),
                params: vec![],
            };
            self.actuator_command(cmd).await?;
            "Actuator command executed".to_string()
        } else {
            // Generic hardware task
            tokio::time::sleep(Duration::from_millis(1)).await;
            format!("[Hog Result] {}", task)
        };
        
        let elapsed = start.elapsed();
        let elapsed_us = elapsed.as_micros();
        
        if elapsed_us > self.latency_budget_us as u128 {
            warn!("⚠️ HOG DEADLINE MISS: {}us", elapsed_us);
        }
        
        info!("🐗 Hog '{}' completed in {}us", self.id, elapsed_us);
        
        Ok(result)
    }
}

impl SpeciesOps for Hog {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hog_sensor() {
        let hog = Hog::new("test-01".to_string());
        let reading = hog.read_sensor("temp-01").await;
        assert!(reading.is_ok());
    }
    
    #[tokio::test]
    async fn test_hog_gpio() {
        let hog = Hog::new("test-01".to_string());
        let result = hog.gpio_write(17, PinState::High).await;
        assert!(result.is_ok());
    }
}
