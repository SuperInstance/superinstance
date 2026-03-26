//! Pasture Sync Module - CRDT-based synchronization using Yjs (yrs)
//! Enables conflict-free real-time collaboration on pasture data

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use yrs::{
    updates::encoder::Encode,
    Array, ArrayRef, Doc, GetString, Map, MapRef, Text, TextRef, Transact,
};
use tracing::{info, warn};

// ============================================================================
// Types
// ============================================================================

/// Pasture sync state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastureState {
    pub breed_name: String,
    pub genes: Vec<GeneUpdate>,
    pub version: u64,
}

/// Gene update for CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneUpdate {
    pub name: String,
    pub expression: f32,
    pub dominant: bool,
    pub updated_by: String,
    pub timestamp: i64,
}

/// Sync message for distributed updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub pasture_id: String,
    pub update: Vec<u8>,
    pub timestamp: i64,
}

// ============================================================================
// CRDT Pasture Manager
// ============================================================================

/// CRDT-based pasture synchronization manager
pub struct PastureSync {
    doc: Doc,
    pasture_map: MapRef,
    genes_array: ArrayRef,
    breed_text: TextRef,
}

impl PastureSync {
    /// Create a new pasture sync manager
    pub fn new() -> Result<Self> {
        let doc = Doc::new();
        
        let pasture_map = doc.get_or_insert_map("pasture");
        let genes_array = doc.get_or_insert_array("genes");
        let breed_text = doc.get_or_insert_text("breed");

        Ok(Self {
            doc,
            pasture_map,
            genes_array,
            breed_text,
        })
    }

    /// Update breed name (CRDT text)
    pub fn update_breed_name(&self, name: &str) -> Result<Vec<u8>> {
        let mut txn = self.doc.transact_mut();
        
        // Clear existing text
        let len = self.breed_text.len(&txn);
        if len > 0 {
            self.breed_text.remove_range(&mut txn, 0, len);
        }
        
        // Insert new name
        self.breed_text.insert(&mut txn, 0, name);
        
        // Generate update for distribution
        let update = txn.encode_update();
        
        info!("Updated breed name: {}", name);
        Ok(update)
    }

    /// Get current breed name
    pub fn get_breed_name(&self) -> String {
        let txn = self.doc.transact();
        self.breed_text.get_string(&txn)
    }

    /// Add gene update (CRDT array)
    pub fn add_gene(&self, gene: GeneUpdate) -> Result<Vec<u8>> {
        let gene_json = serde_json::to_string(&gene)?;
        
        let mut txn = self.doc.transact_mut();
        
        // Create map for gene
        let gene_map = Map::new();
        gene_map.insert(&mut txn, "name", gene.name.as_str());
        gene_map.insert(&mut txn, "expression", gene.expression);
        gene_map.insert(&mut txn, "dominant", gene.dominant);
        gene_map.insert(&mut txn, "updated_by", gene.updated_by.as_str());
        gene_map.insert(&mut txn, "timestamp", gene.timestamp);
        
        // Add to array
        self.genes_array.push_back(&mut txn, gene_map);
        
        // Generate update
        let update = txn.encode_update();
        
        info!("Added gene: {}", gene.name);
        Ok(update)
    }

    /// Get all genes
    pub fn get_genes(&self) -> Result<Vec<GeneUpdate>> {
        let txn = self.doc.transact();
        let mut genes = Vec::new();

        for i in 0..self.genes_array.len(&txn) {
            if let Some(map_ref) = self.genes_array.get(&txn, i) {
                if let Some(map) = map_ref.to_map() {
                    let name = map
                        .get(&txn, "name")
                        .and_then(|v| v.to_string(&txn))
                        .unwrap_or_default();
                    
                    let expression = map
                        .get(&txn, "expression")
                        .and_then(|v| v.to_string(&txn))
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    
                    let dominant = map
                        .get(&txn, "dominant")
                        .and_then(|v| v.to_bool(&txn))
                        .unwrap_or(false);
                    
                    let updated_by = map
                        .get(&txn, "updated_by")
                        .and_then(|v| v.to_string(&txn))
                        .unwrap_or_default();
                    
                    let timestamp = map
                        .get(&txn, "timestamp")
                        .and_then(|v| v.to_string(&txn))
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    genes.push(GeneUpdate {
                        name,
                        expression,
                        dominant,
                        updated_by,
                        timestamp,
                    });
                }
            }
        }

        Ok(genes)
    }

    /// Apply remote update (CRDT merge)
    pub fn apply_update(&self, update: Vec<u8>) -> Result<()> {
        use yrs::updates::decoder::Decode;
        
        let update = yrs::Update::decode_v1(&update)?;
        let mut txn = self.doc.transact_mut();
        txn.apply_update(update);
        
        info!("Applied remote CRDT update");
        Ok(())
    }

    /// Export current state for distribution
    pub fn export_state(&self) -> Result<Vec<u8>> {
        let txn = self.doc.transact();
        let state = txn.encode_state_as_update_v1(&yrs::ID::default());
        Ok(state)
    }

    /// Get full pasture state
    pub fn get_state(&self) -> Result<PastureState> {
        let breed_name = self.get_breed_name();
        let genes = self.get_genes()?;
        
        let version = {
            let txn = self.doc.transact();
            0u64
        };

        Ok(PastureState {
            breed_name,
            genes,
            version,
        })
    }
}

// ============================================================================
// Distributed Sync Handler
// ============================================================================

/// Handles distributed pasture synchronization
pub struct PastureSyncHandler {
    sync: Arc<Mutex<PastureSync>>,
}

impl PastureSyncHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sync: Arc::new(Mutex::new(PastureSync::new()?)),
        })
    }

    /// Handle incoming sync message
    pub async fn handle_sync(&self, message: SyncMessage) -> Result<()> {
        let sync = self.sync.lock().await;
        sync.apply_update(message.update)?;
        
        info!("Synced pasture: {}", message.pasture_id);
        Ok(())
    }

    /// Broadcast update to peers
    pub async fn broadcast_update(&self, update: Vec<u8>) -> Result<SyncMessage> {
        let timestamp = chrono::Utc::now().timestamp();
        
        Ok(SyncMessage {
            pasture_id: "cattle-v1".to_string(),
            update,
            timestamp,
        })
    }

    /// Get current state
    pub async fn get_state(&self) -> Result<PastureState> {
        let sync = self.sync.lock().await;
        sync.get_state()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pasture_sync_creation() {
        let sync = PastureSync::new().unwrap();
        assert!(sync.get_breed_name().is_empty());
    }

    #[test]
    fn test_breed_name_update() {
        let sync = PastureSync::new().unwrap();
        
        let update = sync.update_breed_name("collie").unwrap();
        assert!(!update.is_empty());
        
        let name = sync.get_breed_name();
        assert_eq!(name, "collie");
    }

    #[test]
    fn test_gene_addition() {
        let sync = PastureSync::new().unwrap();
        
        let gene = GeneUpdate {
            name: "speed".to_string(),
            expression: 0.9,
            dominant: true,
            updated_by: "night-school".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let update = sync.add_gene(gene).unwrap();
        assert!(!update.is_empty());
        
        let genes = sync.get_genes().unwrap();
        assert_eq!(genes.len(), 1);
        assert_eq!(genes[0].name, "speed");
    }

    #[test]
    fn test_crdt_merge() {
        let sync1 = PastureSync::new().unwrap();
        let sync2 = PastureSync::new().unwrap();
        
        // Update sync1
        let update1 = sync1.update_breed_name("collie").unwrap();
        
        // Apply to sync2
        sync2.apply_update(update1).unwrap();
        
        // Both should have same name
        assert_eq!(sync1.get_breed_name(), sync2.get_breed_name());
    }

    #[tokio::test]
    async fn test_sync_handler() {
        let handler = PastureSyncHandler::new().unwrap();
        
        let state = handler.get_state().await.unwrap();
        assert!(state.breed_name.is_empty());
    }
}
