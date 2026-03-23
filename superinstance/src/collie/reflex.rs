//! Reflex Cache - CUDA Graph Cache for Instant Responses
//! 
//! The "Muscle Memory" layer - compiled CUDA graphs for routines used >3x.
//! Execution <1ms for cached responses.
//! 
//! This is the fastest layer of the Collie's speed stack:
//! 1. Reflex (this module) - <1ms for cached patterns
//! 2. Anticipation (Shadow Pup) - ~10ms for predictions
//! 3. Cognition (LoRA Hot-Swap) - <50ms for species activation
//! 4. Deliberation (Cloud) - seconds, but distills locally

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::collie::shepherd::Intent;

/// Minimum usage count before caching
const CACHE_THRESHOLD: u32 = 3;

/// Maximum cache entries (memory constraint)
const MAX_CACHE_ENTRIES: usize = 100;

/// Cache entry TTL (time to live)
const CACHE_TTL_SECS: u64 = 3600; // 1 hour

/// Reflex Cache - Fast response cache using CUDA graphs
pub struct ReflexCache {
    /// Cache entries
    entries: Arc<RwLock<HashMap<IntentHash, CacheEntry>>>,
    /// Usage counters (for determining what to cache)
    usage: Arc<RwLock<HashMap<IntentHash, u32>>>,
    /// Statistics
    stats: Arc<RwLock<ReflexStats>>,
}

/// A cached response entry
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached response
    response: String,
    /// When this entry was created
    created_at: Instant,
    /// Number of times this entry was used
    hits: u64,
    /// CUDA graph handle (in production)
    _cuda_graph_id: Option<u64>,
}

/// Statistics for the reflex cache
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReflexStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Current number of entries
    pub entries: usize,
    /// Average hit latency in microseconds
    pub avg_hit_latency_us: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
}

/// Hash for an intent (for cache key)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct IntentHash(u64);

impl IntentHash {
    fn new(intent: &Intent) -> Self {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        intent.kind.hash(&mut hasher);
        intent.payload.hash(&mut hasher);
        
        Self(hasher.finish())
    }
}

impl ReflexCache {
    /// Create a new reflex cache
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            usage: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ReflexStats::default())),
        }
    }
    
    /// Check if we have a cached response for this intent
    pub fn check(&self, intent: &Intent) -> Option<String> {
        let hash = IntentHash::new(intent);
        let start = Instant::now();
        
        // Increment usage counter
        {
            let mut usage = self.usage.write();
            *usage.entry(hash).or_insert(0) += 1;
        }
        
        // Check cache
        let entries = self.entries.read();
        if let Some(entry) = entries.get(&hash) {
            // Check TTL
            if entry.created_at.elapsed() > Duration::from_secs(CACHE_TTL_SECS) {
                drop(entries);
                self.evict(hash);
                return None;
            }
            
            // Cache hit!
            let elapsed = start.elapsed();
            
            // Update stats
            {
                let mut stats = self.stats.write();
                stats.hits += 1;
                stats.avg_hit_latency_us = 
                    (stats.avg_hit_latency_us * (stats.hits - 1) as f64 + elapsed.as_micros() as f64)
                    / stats.hits as f64;
            }
            
            debug!("⚡ REFLEX HIT: {:?} in {:?}", entry.response.len(), elapsed);
            
            Some(entry.response.clone())
        } else {
            // Cache miss
            let mut stats = self.stats.write();
            stats.misses += 1;
            
            debug!("❌ REFLEX MISS: {:?}", intent.kind);
            
            None
        }
    }
    
    /// Consider caching a response after execution
    pub fn consider_caching(&self, intent: Intent, response: &str) {
        let hash = IntentHash::new(&intent);
        
        // Check usage count
        let usage = self.usage.read().get(&hash).copied().unwrap_or(0);
        
        if usage < CACHE_THRESHOLD {
            debug!("🔍 Reflex candidate: {} uses ({}/{} needed)", 
                intent.kind, usage, CACHE_THRESHOLD);
            return;
        }
        
        // Check if already cached
        {
            let entries = self.entries.read();
            if entries.contains_key(&hash) {
                return;
            }
        }
        
        // Check cache size
        {
            let entries = self.entries.read();
            if entries.len() >= MAX_CACHE_ENTRIES {
                drop(entries);
                self.evict_lru();
            }
        }
        
        // Add to cache
        self.insert(hash, response);
        
        info!("✅ CACHED REFLEX: {} ({} uses)", intent.kind, usage);
    }
    
    /// Insert an entry into the cache
    fn insert(&self, hash: IntentHash, response: &str) {
        let entry = CacheEntry {
            response: response.to_string(),
            created_at: Instant::now(),
            hits: 0,
            _cuda_graph_id: None, // In production, would compile CUDA graph
        };
        
        {
            let mut entries = self.entries.write();
            entries.insert(hash, entry);
        }
        
        {
            let mut stats = self.stats.write();
            stats.entries = self.entries.read().len();
        }
    }
    
    /// Evict a specific entry
    fn evict(&self, hash: IntentHash) {
        let mut entries = self.entries.write();
        entries.remove(&hash);
        
        let mut stats = self.stats.write();
        stats.entries = entries.len();
        
        debug!("🗑️ Evicted expired cache entry");
    }
    
    /// Evict the least recently used entry
    fn evict_lru(&self) {
        let mut entries = self.entries.write();
        
        // Find the entry with the oldest creation time and fewest hits
        if let Some((hash_to_evict, _)) = entries.iter()
            .min_by_key(|(_, e)| (e.hits, e.created_at))
        {
            let hash_to_evict = *hash_to_evict;
            entries.remove(&hash_to_evict);
            debug!("🗑️ Evicted LRU cache entry");
        }
        
        let mut stats = self.stats.write();
        stats.entries = entries.len();
    }
    
    /// Clear the entire cache
    pub fn clear(&self) {
        let mut entries = self.entries.write();
        entries.clear();
        
        let mut stats = self.stats.write();
        stats.entries = 0;
        
        info!("🧹 Reflex cache cleared");
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> ReflexStats {
        let mut stats = self.stats.read().clone();
        
        // Calculate memory usage
        let entries = self.entries.read();
        stats.memory_usage_mb = entries.values()
            .map(|e| e.response.len() as f64 / 1_000_000.0)
            .sum();
        
        stats
    }
    
    /// Check if a pattern is a "reflex" (highly cached)
    pub fn is_reflex(&self, intent: &Intent) -> bool {
        let hash = IntentHash::new(intent);
        self.entries.read().contains_key(&hash)
    }
    
    /// Compile a CUDA graph for a cached response
    /// 
    /// In production, this would:
    /// 1. Parse the cached response
    /// 2. Compile a minimal compute graph
    /// 3. Store the graph handle for <1ms execution
    #[cfg(feature = "gpu")]
    pub fn compile_cuda_graph(&self, _intent: &Intent) -> Result<u64, &'static str> {
        // Placeholder for CUDA graph compilation
        // Real implementation would use:
        // - TensorRT for optimized inference
        // - CUDA graphs for fixed computation patterns
        // - cuDNN for optimized tensor operations
        
        Err("CUDA graph compilation not implemented")
    }
}

impl Default for ReflexCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Reflex pattern - a learned pattern that triggers instant response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflexPattern {
    /// Pattern name
    pub name: String,
    /// Trigger condition
    pub trigger: String,
    /// Response template
    pub response_template: String,
    /// Use count
    pub use_count: u32,
    /// Last used
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_hit_miss() {
        let cache = ReflexCache::new();
        let intent = Intent::new("test", "payload");
        
        // First check - miss
        assert!(cache.check(&intent).is_none());
        
        // Increment usage to threshold
        for _ in 0..CACHE_THRESHOLD {
            cache.check(&intent);
        }
        
        // Now cache it
        cache.consider_caching(intent.clone(), "cached response");
        
        // Should hit now
        let result = cache.check(&intent);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "cached response");
    }
    
    #[test]
    fn test_stats() {
        let cache = ReflexCache::new();
        let intent = Intent::new("test", "payload");
        
        // Miss
        cache.check(&intent);
        
        let stats = cache.get_stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);
    }
}
