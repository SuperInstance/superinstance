//! LoRA Manager - Hot-Swap Logic for Adapter Loading
//! 
//! Manages LoRA adapter loading, unloading, and hot-swapping.
//! Target: <50ms for adapter swap operations.
//! 
//! The LoRA Manager:
//! - Loads adapters from disk to GPU
//! - Manages VRAM allocation
//! - Tracks adapter usage and fitness
//! - Supports hot-swapping without model reload

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::species::SpeciesType;

/// Maximum time for hot-swap operation
const HOT_SWAP_TIMEOUT_MS: u64 = 50;

/// LoRA Manager - Handles adapter hot-swapping
pub struct LoRAManager {
    /// Currently loaded adapters (adapter_id -> AdapterInfo)
    loaded: Arc<RwLock<HashMap<String, AdapterInfo>>>,
    /// Adapter cache on disk
    cache: AdapterCache,
    /// Maximum VRAM in bytes
    max_vram: u64,
    /// Current VRAM usage
    vram_used: Arc<RwLock<u64>>,
}

/// Information about a loaded adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
    /// Unique adapter ID
    pub id: String,
    /// Species type this adapter is for
    pub species: SpeciesType,
    /// Path to the adapter file
    pub path: PathBuf,
    /// Size in bytes
    pub size_bytes: u64,
    /// When it was loaded
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    /// Number of times used
    pub use_count: u64,
    /// Fitness score
    pub fitness: f32,
    /// Generation number
    pub generation: u32,
}

/// Adapter cache on disk
#[derive(Debug, Clone)]
pub struct AdapterCache {
    /// Base path for adapter files
    base_path: PathBuf,
    /// Index of available adapters
    index: HashMap<String, AdapterMetadata>,
}

/// Metadata for an adapter in cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterMetadata {
    pub id: String,
    pub species: SpeciesType,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub generation: u32,
    pub parent_ids: Vec<String>,
}

/// Result of a hot-swap operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSwapResult {
    pub adapter_id: String,
    pub species: SpeciesType,
    pub duration_ms: u64,
    pub vram_used: u64,
    pub success: bool,
}

impl LoRAManager {
    /// Create a new LoRA Manager
    pub fn new(max_vram: u64) -> Result<Self> {
        let cache = AdapterCache::new("pasture/adapters")?;
        
        Ok(Self {
            loaded: Arc::new(RwLock::new(HashMap::new())),
            cache,
            max_vram,
            vram_used: Arc::new(RwLock::new(0)),
        })
    }
    
    /// Create a mock manager for testing
    pub fn mock() -> Self {
        Self {
            loaded: Arc::new(RwLock::new(HashMap::new())),
            cache: AdapterCache::mock(),
            max_vram: 6_500_000_000, // 6.5 GB
            vram_used: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Hot-swap to a new adapter
    /// 
    /// This is the core operation - swapping adapters in <50ms:
    /// 1. Check if adapter is already loaded
    /// 2. Unload current adapter (free VRAM)
    /// 3. Load new adapter from disk
    /// 4. Update tracking
    pub fn hot_swap(&self, adapter_id: &str) -> Result<HotSwapResult> {
        let start = Instant::now();
        info!("🔄 Hot-swap requested: {}", adapter_id);
        
        // Check if already loaded
        {
            let loaded = self.loaded.read();
            if let Some(info) = loaded.get(adapter_id) {
                debug!("Adapter {} already loaded", adapter_id);
                return Ok(HotSwapResult {
                    adapter_id: adapter_id.to_string(),
                    species: info.species,
                    duration_ms: start.elapsed().as_millis() as u64,
                    vram_used: info.size_bytes,
                    success: true,
                });
            }
        }
        
        // Get adapter metadata
        let metadata = self.cache.get(adapter_id)
            .ok_or_else(|| anyhow!("Adapter not found: {}", adapter_id))?;
        
        // Check VRAM availability
        let vram_needed = metadata.size_bytes;
        {
            let used = self.vram_used.read();
            let available = self.max_vram.saturating_sub(*used);
            
            if available < vram_needed {
                // Need to unload something
                drop(used);
                self.evict_lru(vram_needed)?;
            }
        }
        
        // Simulate loading the adapter
        // In production, this would use candle/safetensors to load weights
        let load_time = Duration::from_millis(20); // Simulated
        std::thread::sleep(load_time);
        
        // Create adapter info
        let info = AdapterInfo {
            id: adapter_id.to_string(),
            species: metadata.species,
            path: metadata.path.clone(),
            size_bytes: metadata.size_bytes,
            loaded_at: chrono::Utc::now(),
            use_count: 1,
            fitness: 0.8,
            generation: metadata.generation,
        };
        
        // Update state
        {
            let mut loaded = self.loaded.write();
            loaded.insert(adapter_id.to_string(), info.clone());
        }
        
        {
            let mut used = self.vram_used.write();
            *used += vram_needed;
        }
        
        let elapsed = start.elapsed();
        let duration_ms = elapsed.as_millis() as u64;
        
        if duration_ms > HOT_SWAP_TIMEOUT_MS {
            warn!("⚠️ Hot-swap exceeded target: {}ms > {}ms", 
                duration_ms, HOT_SWAP_TIMEOUT_MS);
        } else {
            info!("✅ Hot-swap complete: {}ms", duration_ms);
        }
        
        Ok(HotSwapResult {
            adapter_id: adapter_id.to_string(),
            species: info.species,
            duration_ms,
            vram_used: info.size_bytes,
            success: true,
        })
    }
    
    /// Unload an adapter
    pub fn unload(&self, adapter_id: &str) -> Result<()> {
        let mut loaded = self.loaded.write();
        
        if let Some(info) = loaded.remove(adapter_id) {
            let mut used = self.vram_used.write();
            *used = used.saturating_sub(info.size_bytes);
            info!("🗑️ Unloaded adapter: {}", adapter_id);
        }
        
        Ok(())
    }
    
    /// Evict least recently used adapters to free space
    fn evict_lru(&self, needed_bytes: u64) -> Result<()> {
        let mut loaded = self.loaded.write();
        let mut freed = 0u64;
        
        // Sort by use count (ascending) and loaded time (ascending)
        let mut adapters: Vec<_> = loaded.iter()
            .map(|(id, info)| (id.clone(), info.use_count, info.loaded_at, info.size_bytes))
            .collect();
        
        adapters.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .then_with(|| a.2.cmp(&b.2))
        });
        
        let mut to_remove = Vec::new();
        
        for (id, _, _, size) in adapters {
            if freed >= needed_bytes {
                break;
            }
            to_remove.push(id);
            freed += size;
        }
        
        for id in to_remove {
            if let Some(info) = loaded.remove(&id) {
                freed += info.size_bytes;
                debug!("Evicted adapter: {}", id);
            }
        }
        
        drop(loaded);
        
        let mut used = self.vram_used.write();
        *used = used.saturating_sub(freed);
        
        Ok(())
    }
    
    /// Get adapter info
    pub fn get(&self, adapter_id: &str) -> Option<AdapterInfo> {
        self.loaded.read().get(adapter_id).cloned()
    }
    
    /// Check if adapter is loaded
    pub fn is_loaded(&self, adapter_id: &str) -> bool {
        self.loaded.read().contains_key(adapter_id)
    }
    
    /// Get number of active adapters
    pub fn active_count(&self) -> usize {
        self.loaded.read().len()
    }
    
    /// Get current VRAM usage
    pub fn vram_used(&self) -> u64 {
        *self.vram_used.read()
    }
    
    /// Get list of loaded adapters
    pub fn list_loaded(&self) -> Vec<AdapterInfo> {
        self.loaded.read().values().cloned().collect()
    }
    
    /// Update adapter fitness score
    pub fn update_fitness(&self, adapter_id: &str, fitness: f32) {
        if let Some(info) = self.loaded.write().get_mut(adapter_id) {
            info.fitness = fitness;
        }
    }
    
    /// Increment adapter use count
    pub fn increment_use(&self, adapter_id: &str) {
        if let Some(info) = self.loaded.write().get_mut(adapter_id) {
            info.use_count += 1;
        }
    }
}

impl AdapterCache {
    /// Create a new adapter cache
    pub fn new(base_path: &str) -> Result<Self> {
        let base = PathBuf::from(base_path);
        
        // Build index from disk
        let mut index = HashMap::new();
        
        // Add default adapters for each species
        for species in SpeciesType::all() {
            let id = format!("{}_v1", species.to_string().to_lowercase());
            let path = base.join(species.to_string().to_lowercase())
                .join(format!("{}.safetensors", id));
            
            index.insert(id.clone(), AdapterMetadata {
                id,
                species,
                path,
                size_bytes: species.typical_vram_mb() as u64 * 1024 * 1024,
                generation: 1,
                parent_ids: vec![],
            });
        }
        
        Ok(Self {
            base_path: base,
            index,
        })
    }
    
    /// Create a mock cache
    pub fn mock() -> Self {
        let mut index = HashMap::new();
        
        for species in SpeciesType::all() {
            let id = format!("{}_v1", species.to_string().to_lowercase());
            index.insert(id.clone(), AdapterMetadata {
                id,
                species,
                path: PathBuf::from("mock.safetensors"),
                size_bytes: species.typical_vram_mb() as u64 * 1024 * 1024,
                generation: 1,
                parent_ids: vec![],
            });
        }
        
        Self {
            base_path: PathBuf::from("pasture/adapters"),
            index,
        }
    }
    
    /// Get adapter metadata
    pub fn get(&self, adapter_id: &str) -> Option<AdapterMetadata> {
        self.index.get(adapter_id).cloned()
    }
    
    /// List all available adapters
    pub fn list_all(&self) -> Vec<AdapterMetadata> {
        self.index.values().cloned().collect()
    }
    
    /// Add a new adapter to cache
    pub fn add(&mut self, metadata: AdapterMetadata) {
        self.index.insert(metadata.id.clone(), metadata);
    }
    
    /// Remove an adapter from cache
    pub fn remove(&mut self, adapter_id: &str) -> Option<AdapterMetadata> {
        self.index.remove(adapter_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hot_swap() {
        let manager = LoRAManager::mock();
        let result = manager.hot_swap("cattle_v1");
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
    
    #[test]
    fn test_is_loaded() {
        let manager = LoRAManager::mock();
        assert!(!manager.is_loaded("cattle_v1"));
        
        manager.hot_swap("cattle_v1").unwrap();
        assert!(manager.is_loaded("cattle_v1"));
    }
}
