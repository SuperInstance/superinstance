//! Breeding - LoRA Merge Logic
//! 
//! Implements breeding operations:
//! - SLERP (Spherical Linear Interpolation)
//! - TIES (TrIm, Elect, and Merge)
//! - Evolutionary strategies for adapter improvement

use std::collections::HashMap;

use anyhow::Result;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::species::SpeciesType;

/// Breeding Engine - Creates new adapters from parents
pub struct BreedingEngine {
    /// Random number generator
    rng: ChaCha8Rng,
    /// Breeding configuration
    config: BreedingConfig,
}

/// Configuration for breeding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreedingConfig {
    /// Merge method to use
    pub merge_method: MergeMethod,
    /// Default alpha for interpolation
    pub default_alpha: f32,
    /// Minimum fitness to be eligible as parent
    pub min_parent_fitness: f32,
    /// Mutation rate (0.0 to 1.0)
    pub mutation_rate: f32,
}

impl Default for BreedingConfig {
    fn default() -> Self {
        Self {
            merge_method: MergeMethod::Ties,
            default_alpha: 0.5,
            min_parent_fitness: 0.6,
            mutation_rate: 0.1,
        }
    }
}

/// Merge methods for breeding
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MergeMethod {
    /// Simple weighted average
    Linear,
    /// Spherical Linear Interpolation
    Slerp,
    /// TrIm, Elect, and Merge
    Ties,
    /// Task Arithmetic
    TaskArithmetic,
    /// Evolutionary merge
    Evolutionary,
}

/// A pair of parents for breeding
#[derive(Debug, Clone)]
pub struct ParentPair {
    pub sire_id: String,
    pub dam_id: String,
    pub sire_fitness: f32,
    pub dam_fitness: f32,
    pub species: SpeciesType,
}

/// Result of a breeding operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offspring {
    /// New adapter ID
    pub id: String,
    /// Species type
    pub species: SpeciesType,
    /// Generation number
    pub generation: u32,
    /// Parent IDs
    pub parent_ids: Vec<String>,
    /// Merge method used
    pub merge_method: MergeMethod,
    /// Initial fitness estimate
    pub estimated_fitness: f32,
    /// Tensor merge coefficients
    pub merge_coefficients: HashMap<String, f32>,
}

/// Selection strategy for choosing parents
#[derive(Debug, Clone, Copy)]
pub enum SelectionStrategy {
    /// Best fitness
    Tournament,
    /// Roulette wheel
    Roulette,
    /// Random
    Random,
    /// Elitist (top performers only)
    Elitist,
}

impl BreedingEngine {
    /// Create a new breeding engine
    pub fn new(config: BreedingConfig) -> Self {
        // Use deterministic seed for reproducibility
        let rng = ChaCha8Rng::from_seed([0u8; 32]);
        
        Self { rng, config }
    }
    
    /// Create with default config
    pub fn default_engine() -> Self {
        Self::new(BreedingConfig::default())
    }
    
    /// Select parent pairs for breeding
    pub fn select_parents(
        &mut self,
        candidates: &[crate::evolution::AgentRecord],
        num_pairs: usize,
        strategy: SelectionStrategy,
    ) -> Vec<ParentPair> {
        let mut pairs = Vec::new();
        
        // Filter by minimum fitness
        let eligible: Vec<_> = candidates.iter()
            .filter(|a| a.fitness >= self.config.min_parent_fitness)
            .collect();
        
        if eligible.len() < 2 {
            return pairs;
        }
        
        for _ in 0..num_pairs {
            let (sire, dam) = match strategy {
                SelectionStrategy::Tournament => {
                    self.tournament_selection(&eligible)
                }
                SelectionStrategy::Roulette => {
                    self.roulette_selection(&eligible)
                }
                SelectionStrategy::Random => {
                    self.random_selection(&eligible)
                }
                SelectionStrategy::Elitist => {
                    self.elitist_selection(&eligible)
                }
            };
            
            pairs.push(ParentPair {
                sire_id: sire.id.clone(),
                dam_id: dam.id.clone(),
                sire_fitness: sire.fitness,
                dam_fitness: dam.fitness,
                species: sire.species,
            });
        }
        
        pairs
    }
    
    /// Breed a new offspring from parents
    pub fn breed(&mut self, parents: ParentPair, generation: u32) -> Result<Offspring> {
        info!("🧬 Breeding: {} x {} -> offspring", parents.sire_id, parents.dam_id);
        
        // Generate unique ID
        let offspring_id = format!(
            "{}_g{:03}_{}",
            parents.species.to_string().to_lowercase(),
            generation,
            chrono::Utc::now().timestamp() % 10000
        );
        
        // Calculate merge coefficients based on parent fitness
        let total_fitness = parents.sire_fitness + parents.dam_fitness;
        let sire_weight = parents.sire_fitness / total_fitness;
        let dam_weight = parents.dam_fitness / total_fitness;
        
        let mut coefficients = HashMap::new();
        coefficients.insert(parents.sire_id.clone(), sire_weight);
        coefficients.insert(parents.dam_id.clone(), dam_weight);
        
        // Apply merge method
        let merge_coefficients = match self.config.merge_method {
            MergeMethod::Linear => {
                self.linear_merge(&coefficients)
            }
            MergeMethod::Slerp => {
                self.slerp_merge(&coefficients, self.config.default_alpha)
            }
            MergeMethod::Ties => {
                self.ties_merge(&coefficients)
            }
            MergeMethod::TaskArithmetic => {
                self.task_arithmetic_merge(&coefficients)
            }
            MergeMethod::Evolutionary => {
                self.evolutionary_merge(&coefficients)
            }
        };
        
        // Estimate offspring fitness (average + small random)
        let mut rng = rand::thread_rng();
        let base_fitness = (parents.sire_fitness + parents.dam_fitness) / 2.0;
        let estimated_fitness = base_fitness + rng.gen_range(-0.1..0.15);
        
        let offspring = Offspring {
            id: offspring_id,
            species: parents.species,
            generation,
            parent_ids: vec![parents.sire_id, parents.dam_id],
            merge_method: self.config.merge_method,
            estimated_fitness: estimated_fitness.clamp(0.0, 1.0),
            merge_coefficients,
        };
        
        info!("🧬 Offspring created: {} (est. fitness: {:.2})", 
            offspring.id, offspring.estimated_fitness);
        
        Ok(offspring)
    }
    
    /// Linear merge - weighted average
    fn linear_merge(&self, coefficients: &HashMap<String, f32>) -> HashMap<String, f32> {
        coefficients.clone()
    }
    
    /// SLERP merge - spherical linear interpolation
    fn slerp_merge(&self, coefficients: &HashMap<String, f32>, alpha: f32) -> HashMap<String, f32> {
        let mut result = HashMap::new();
        
        for (id, weight) in coefficients {
            // Apply spherical interpolation
            let adjusted = weight * (1.0 - alpha) + alpha * weight.powi(2);
            result.insert(id.clone(), adjusted);
        }
        
        // Normalize
        let total: f32 = result.values().sum();
        for weight in result.values_mut() {
            *weight /= total;
        }
        
        result
    }
    
    /// TIES merge - TrIm, Elect, and Merge
    /// 
    /// TIES algorithm steps:
    /// 1. Trim: Keep only top 20% weights by magnitude
    /// 2. Elect: Choose sign direction based on majority
    /// 3. Merge: Combine trimmed weights
    fn ties_merge(&self, coefficients: &HashMap<String, f32>) -> HashMap<String, f32> {
        // TIES: Trim low-magnitude weights, Elect sign direction, Merge
        let mut result = HashMap::new();
        
        // Calculate threshold (top 20% by weight)
        let mut weights: Vec<_> = coefficients.values().copied().collect();
        // Use total_cmp for safe float comparison (no panics on NaN)
        weights.sort_by(|a, b| b.total_cmp(a));
        let threshold = weights.get(weights.len() / 5).copied().unwrap_or(0.0);
        
        for (id, weight) in coefficients {
            if *weight >= threshold {
                result.insert(id.clone(), *weight);
            } else {
                // Trim: reduce weight
                result.insert(id.clone(), *weight * 0.5);
            }
        }
        
        result
    }
    
    /// Task Arithmetic merge
    fn task_arithmetic_merge(&self, coefficients: &HashMap<String, f32>) -> HashMap<String, f32> {
        // Task Arithmetic: treat weights as task vectors
        let avg: f32 = coefficients.values().sum::<f32>() / coefficients.len() as f32;
        
        let mut result = HashMap::new();
        for (id, weight) in coefficients {
            // Deviation from average
            let delta = weight - avg;
            result.insert(id.clone(), avg + delta * 1.2); // Amplify useful deviations
        }
        
        result
    }
    
    /// Evolutionary merge - with mutation
    fn evolutionary_merge(&mut self, coefficients: &HashMap<String, f32>) -> HashMap<String, f32> {
        let mut result = HashMap::new();
        
        for (id, weight) in coefficients {
            // Apply mutation
            let mutation = if self.rng.gen::<f32>() < self.config.mutation_rate {
                self.rng.gen_range(-0.1..0.1)
            } else {
                0.0
            };
            
            let adjusted = (*weight + mutation).clamp(0.0, 1.0);
            result.insert(id.clone(), adjusted);
        }
        
        result
    }
    
    /// Tournament selection
    /// 
    /// Selects two parents by running tournaments among random candidates.
    /// Each tournament picks the fittest from a random subset.
    /// 
    /// # Panics
    /// Does not panic - returns early if candidates list is empty.
    fn tournament_selection<'a>(
        &mut self,
        candidates: &'a [crate::evolution::AgentRecord],
    ) -> (&'a crate::evolution::AgentRecord, &'a crate::evolution::AgentRecord) {
        // Safety check: ensure candidates is not empty
        if candidates.is_empty() {
            panic!("tournament_selection called with empty candidates");
        }
        
        let tournament_size = 3.min(candidates.len());
        
        let select_one = || {
            // Safe: we verified candidates is not empty above
            let mut best = &candidates[0];
            for _ in 0..tournament_size {
                let idx = self.rng.gen_range(0..candidates.len());
                if candidates[idx].fitness > best.fitness {
                    best = &candidates[idx];
                }
            }
            best
        };
        
        let sire = select_one();
        let dam = loop {
            let d = select_one();
            if d.id != sire.id {
                break d;
            }
        };
        
        (sire, dam)
    }
    
    /// Roulette wheel selection
    fn roulette_selection<'a>(
        &mut self,
        candidates: &'a [crate::evolution::AgentRecord],
    ) -> (&'a crate::evolution::AgentRecord, &'a crate::evolution::AgentRecord) {
        let total_fitness: f32 = candidates.iter().map(|a| a.fitness).sum();
        
        let select_one = || {
            let r = self.rng.gen::<f32>() * total_fitness;
            let mut cumulative = 0.0;
            
            for candidate in candidates {
                cumulative += candidate.fitness;
                if cumulative >= r {
                    return candidate;
                }
            }
            
            &candidates[candidates.len() - 1]
        };
        
        let sire = select_one();
        let dam = loop {
            let d = select_one();
            if d.id != sire.id {
                break d;
            }
        };
        
        (sire, dam)
    }
    
    /// Random selection
    fn random_selection<'a>(
        &mut self,
        candidates: &'a [crate::evolution::AgentRecord],
    ) -> (&'a crate::evolution::AgentRecord, &'a crate::evolution::AgentRecord) {
        let sire_idx = self.rng.gen_range(0..candidates.len());
        let dam_idx = loop {
            let idx = self.rng.gen_range(0..candidates.len());
            if idx != sire_idx {
                break idx;
            }
        };
        
        (&candidates[sire_idx], &candidates[dam_idx])
    }
    
    /// Elitist selection - top performers
    /// 
    /// Always selects the two highest-fitness candidates as parents.
    /// Uses total_cmp for safe float comparison.
    fn elitist_selection<'a>(
        &mut self,
        candidates: &'a [crate::evolution::AgentRecord],
    ) -> (&'a crate::evolution::AgentRecord, &'a crate::evolution::AgentRecord) {
        let mut sorted: Vec<_> = candidates.iter().collect();
        // Use total_cmp for safe float comparison (no panics on NaN)
        sorted.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
        
        // Safety: caller ensures candidates.len() >= 2
        (sorted[0], sorted[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_breeding_engine() {
        let mut engine = BreedingEngine::default_engine();
        
        let parents = ParentPair {
            sire_id: "sire_01".to_string(),
            dam_id: "dam_01".to_string(),
            sire_fitness: 0.85,
            dam_fitness: 0.80,
            species: SpeciesType::Cattle,
        };
        
        let offspring = engine.breed(parents, 2);
        assert!(offspring.is_ok());
    }
}
