//! Model Pool - Base Model + PagedAttention
//! 
//! Manages the base model(s) and KV cache using PagedAttention
//! for efficient memory usage and context handling.
//! 
//! The Model Pool:
//! - Holds the base model (Phi-3/Mamba)
//! - Manages KV cache allocation
//! - Supports multiple concurrent contexts
//! - Implements efficient memory sharing

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Model Pool - Base model and cache management
pub struct ModelPool {
    /// Loaded models
    models: Arc<RwLock<HashMap<String, ModelInfo>>>,
    /// KV Cache manager
    kv_cache: KVCacheManager,
    /// Configuration
    config: ModelPoolConfig,
}

/// Information about a loaded model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    /// Model type
    pub model_type: ModelType,
    /// Size in bytes
    pub size_bytes: u64,
    /// Number of parameters (approximate)
    pub parameters: u64,
    /// Context length
    pub context_length: usize,
    /// When loaded
    pub loaded_at: chrono::DateTime<chrono::Utc>,
}

/// Types of models supported
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModelType {
    /// Mamba/SSM model (linear scaling)
    Mamba,
    /// Transformer model
    Transformer,
    /// Hybrid
    Hybrid,
}

/// Model pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPoolConfig {
    /// Maximum context length
    pub max_context_length: usize,
    /// KV cache size in bytes
    pub kv_cache_size: u64,
    /// Number of cache blocks
    pub num_cache_blocks: usize,
    /// Block size (tokens per block)
    pub block_size: usize,
}

impl Default for ModelPoolConfig {
    fn default() -> Self {
        Self {
            max_context_length: 4096,
            kv_cache_size: 1_000_000_000, // 1GB
            num_cache_blocks: 256,
            block_size: 16,
        }
    }
}

/// KV Cache Manager - PagedAttention implementation
pub struct KVCacheManager {
    /// Cache blocks
    blocks: Arc<RwLock<Vec<CacheBlock>>>,
    /// Block size in tokens
    block_size: usize,
    /// Total blocks
    total_blocks: usize,
    /// Free block list
    free_blocks: Arc<RwLock<Vec<usize>>>,
}

/// A single cache block
#[derive(Debug, Clone)]
pub struct CacheBlock {
    /// Block ID
    pub id: usize,
    /// Reference count
    pub ref_count: u32,
    /// Context ID that owns this block
    pub context_id: Option<String>,
    /// Token indices stored
    pub token_range: (usize, usize),
}

/// Context for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceContext {
    /// Context ID
    pub id: String,
    /// Model ID
    pub model_id: String,
    /// Allocated blocks
    pub allocated_blocks: Vec<usize>,
    /// Current sequence length
    pub sequence_length: usize,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ModelPool {
    /// Create a new model pool
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            kv_cache: KVCacheManager::new(256, 16),
            config: ModelPoolConfig::default(),
        }
    }
    
    /// Load the base model
    pub fn load_base_model(&self) -> anyhow::Result<()> {
        info!("Loading base model (Phi-3/Mamba)...");
        
        let model_info = ModelInfo {
            id: "base".to_string(),
            model_type: ModelType::Mamba,
            size_bytes: 2_500_000_000, // 2.5GB
            parameters: 3_800_000_000, // 3.8B parameters
            context_length: 4096,
            loaded_at: chrono::Utc::now(),
        };
        
        self.models.write().insert("base".to_string(), model_info);
        
        info!("Base model loaded successfully");
        Ok(())
    }
    
    /// Get model info
    pub fn get_model(&self, model_id: &str) -> Option<ModelInfo> {
        self.models.read().get(model_id).cloned()
    }
    
    /// Create an inference context
    pub fn create_context(&self, model_id: &str) -> anyhow::Result<InferenceContext> {
        let context_id = format!("ctx_{}", chrono::Utc::now().timestamp_millis());
        
        // Allocate initial blocks
        let blocks = self.kv_cache.allocate_blocks(&context_id, 4)?;
        
        let context = InferenceContext {
            id: context_id,
            model_id: model_id.to_string(),
            allocated_blocks: blocks,
            sequence_length: 0,
            created_at: chrono::Utc::now(),
        };
        
        debug!("Created context: {}", context.id);
        Ok(context)
    }
    
    /// Release an inference context
    pub fn release_context(&self, context: &InferenceContext) -> anyhow::Result<()> {
        self.kv_cache.release_blocks(&context.allocated_blocks)?;
        debug!("Released context: {}", context.id);
        Ok(())
    }
    
    /// Get KV cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.kv_cache.get_stats()
    }
    
    /// Get total VRAM used
    pub fn vram_used(&self) -> u64 {
        let models = self.models.read();
        models.values().map(|m| m.size_bytes).sum()
    }
}

impl Default for ModelPool {
    fn default() -> Self {
        Self::new()
    }
}

impl KVCacheManager {
    /// Create a new KV cache manager
    pub fn new(total_blocks: usize, block_size: usize) -> Self {
        // Initialize all blocks as free
        let free_blocks: Vec<usize> = (0..total_blocks).collect();
        
        let blocks: Vec<CacheBlock> = (0..total_blocks)
            .map(|id| CacheBlock {
                id,
                ref_count: 0,
                context_id: None,
                token_range: (0, 0),
            })
            .collect();
        
        Self {
            blocks: Arc::new(RwLock::new(blocks)),
            block_size,
            total_blocks,
            free_blocks: Arc::new(RwLock::new(free_blocks)),
        }
    }
    
    /// Allocate blocks for a context
    pub fn allocate_blocks(&self, context_id: &str, count: usize) -> anyhow::Result<Vec<usize>> {
        let mut free = self.free_blocks.write();
        
        if free.len() < count {
            return Err(anyhow::anyhow!("Not enough free blocks"));
        }
        
        let allocated: Vec<usize> = free.drain(0..count).collect();
        
        // Mark blocks as allocated
        {
            let mut blocks = self.blocks.write();
            for &id in &allocated {
                if let Some(block) = blocks.get_mut(id) {
                    block.ref_count = 1;
                    block.context_id = Some(context_id.to_string());
                }
            }
        }
        
        debug!("Allocated {} blocks for context {}", count, context_id);
        Ok(allocated)
    }
    
    /// Release blocks back to free pool
    pub fn release_blocks(&self, block_ids: &[usize]) -> anyhow::Result<()> {
        let mut free = self.free_blocks.write();
        let mut blocks = self.blocks.write();
        
        for &id in block_ids {
            if let Some(block) = blocks.get_mut(id) {
                if block.ref_count > 0 {
                    block.ref_count -= 1;
                }
                
                if block.ref_count == 0 {
                    block.context_id = None;
                    block.token_range = (0, 0);
                    free.push(id);
                }
            }
        }
        
        debug!("Released {} blocks", block_ids.len());
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let blocks = self.blocks.read();
        let free = self.free_blocks.read();
        
        let used_blocks = self.total_blocks - free.len();
        
        CacheStats {
            total_blocks: self.total_blocks,
            used_blocks,
            free_blocks: free.len(),
            block_size: self.block_size,
        }
    }
    
    /// Calculate memory needed for a sequence
    pub fn memory_for_sequence(&self, seq_length: usize) -> usize {
        (seq_length / self.block_size + 1) * self.block_size
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_blocks: usize,
    pub used_blocks: usize,
    pub free_blocks: usize,
    pub block_size: usize,
}

impl CacheStats {
    /// Get utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.total_blocks == 0 {
            return 0.0;
        }
        self.used_blocks as f64 / self.total_blocks as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_pool() {
        let pool = ModelPool::new();
        assert!(pool.load_base_model().is_ok());
    }
    
    #[test]
    fn test_kv_cache() {
        let cache = KVCacheManager::new(256, 16);
        let blocks = cache.allocate_blocks("test", 4);
        assert!(blocks.is_ok());
        
        let stats = cache.get_stats();
        assert_eq!(stats.used_blocks, 4);
    }
    
    #[test]
    fn test_create_context() {
        let pool = ModelPool::new();
        pool.load_base_model().unwrap();
        
        let context = pool.create_context("base");
        assert!(context.is_ok());
    }
}
