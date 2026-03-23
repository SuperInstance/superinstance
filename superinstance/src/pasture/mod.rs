//! Pasture - Resource Management
//! 
//! The Pasture manages the computational resources of the Ranch:
//! - Base model (Phi-3/Mamba)
//! - LoRA adapters (Species personalities)
//! - KV Cache (Context memory)
//! - VRAM allocation

mod lora_manager;
mod model_pool;

pub use lora_manager::*;
pub use model_pool::*;

use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::Config;

/// Pasture - Resource Manager
pub struct Pasture {
    /// LoRA Manager
    lora_manager: LoRAManager,
    /// Model Pool
    model_pool: ModelPool,
    /// Resource statistics
    stats: Arc<RwLock<PastureStats>>,
    /// Configuration
    config: Config,
}

/// Statistics about pasture resources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PastureStats {
    /// VRAM currently used in bytes
    pub vram_used: u64,
    /// Number of active adapters
    pub active_adapters: usize,
    /// Number of loaded models
    pub loaded_models: usize,
    /// KV cache size in bytes
    pub kv_cache_size: u64,
    /// Number of pending requests
    pub pending_requests: usize,
}

impl Pasture {
    /// Create a new Pasture
    pub async fn new(config: &Config) -> Result<Self> {
        info!("Initializing LoRA Manager...");
        let lora_manager = LoRAManager::new(config.max_vram_bytes)?;
        
        info!("Initializing Model Pool...");
        let model_pool = ModelPool::new();
        
        Ok(Self {
            lora_manager,
            model_pool,
            stats: Arc::new(RwLock::new(PastureStats::default())),
            config: config.clone(),
        })
    }
    
    /// Create a mock pasture for testing
    pub fn mock() -> Self {
        Self {
            lora_manager: LoRAManager::mock(),
            model_pool: ModelPool::new(),
            stats: Arc::new(RwLock::new(PastureStats::default())),
            config: Config::default(),
        }
    }
    
    /// Get the LoRA manager
    pub fn lora(&self) -> &LoRAManager {
        &self.lora_manager
    }
    
    /// Get the model pool
    pub fn models(&self) -> &ModelPool {
        &self.model_pool
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> PastureStats {
        let mut stats = self.stats.read().clone();
        stats.active_adapters = self.lora_manager.active_count();
        stats.vram_used = self.lora_manager.vram_used();
        stats
    }
    
    /// Check if resources are available for a task
    pub fn can_allocate(&self, vram_needed: u64) -> bool {
        let stats = self.stats.read();
        let available = self.config.max_vram_bytes.saturating_sub(stats.vram_used);
        available >= vram_needed
    }
    
    /// Acquire GPU lock (for Cattle operations)
    pub async fn acquire_gpu(&self) -> Result<GPUGuard> {
        // In production, this would acquire a mutex for GPU access
        Ok(GPUGuard { _held: true })
    }
}

/// Guard for GPU access
pub struct GPUGuard {
    _held: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pasture_mock() {
        let pasture = Pasture::mock();
        let stats = pasture.get_stats();
        assert!(stats.active_adapters >= 0);
    }
}
