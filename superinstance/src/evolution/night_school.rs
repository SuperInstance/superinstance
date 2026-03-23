//! Night School - The Breeding Cycle
//! 
//! Runs at 02:00 daily to:
//! - Evaluate agent performance
//! - Cull underperformers
//! - Breed new agents from champions
//! - Distill cloud knowledge
//! - Quarantine and test new offspring

use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::{BreedingEngine, StudBook, AgentRecord, SelectionStrategy};
use crate::species::{SpeciesRegistry, SpeciesType};

/// Night School - Daily evolution cycle
pub struct NightSchool {
    /// Stud Book reference
    stud_book: Arc<RwLock<StudBook>>,
    /// Species Registry reference
    species_registry: Arc<RwLock<SpeciesRegistry>>,
    /// Culling threshold
    cull_threshold: f32,
    /// Breeding engine
    breeding_engine: BreedingEngine,
}

/// Report from a Night School session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightSchoolReport {
    /// Session timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Day number
    pub day: u32,
    /// Number of agents culled
    pub culled_count: usize,
    /// Number of new offspring bred
    pub bred_count: usize,
    /// Number promoted to production
    pub promoted_count: usize,
    /// Number quarantined
    pub quarantined_count: usize,
    /// Average fitness before
    pub avg_fitness_before: f32,
    /// Average fitness after
    pub avg_fitness_after: f32,
    /// Per-species results
    pub species_results: Vec<SpeciesEvolutionResult>,
}

/// Evolution result per species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesEvolutionResult {
    pub species: SpeciesType,
    pub population_before: usize,
    pub population_after: usize,
    pub avg_fitness_before: f32,
    pub avg_fitness_after: f32,
    pub culled: usize,
    pub bred: usize,
}

impl NightSchool {
    /// Create a new Night School instance
    pub fn new(
        stud_book: Arc<RwLock<StudBook>>,
        species_registry: Arc<RwLock<SpeciesRegistry>>,
        cull_threshold: f32,
    ) -> Self {
        Self {
            stud_book,
            species_registry,
            cull_threshold,
            breeding_engine: BreedingEngine::default_engine(),
        }
    }
    
    /// Run the Night School cycle
    pub async fn run(&self) -> Result<NightSchoolReport> {
        info!("🌙 Night School starting...");
        let start = std::time::Instant::now();
        
        // Get stats before
        let stats_before = {
            let book = self.stud_book.read();
            book.get_stats()?
        };
        
        let mut report = NightSchoolReport {
            timestamp: chrono::Utc::now(),
            day: stats_before.total_generations,
            culled_count: 0,
            bred_count: 0,
            promoted_count: 0,
            quarantined_count: 0,
            avg_fitness_before: stats_before.avg_fitness,
            avg_fitness_after: 0.0,
            species_results: Vec::new(),
        };
        
        // Phase 1: Evaluate
        info!("📊 Phase 1: Evaluating agent performance...");
        self.evaluate_agents()?;
        
        // Phase 2: Cull
        info!("🗑️ Phase 2: Culling underperformers...");
        let culled = self.cull_underperformers()?;
        report.culled_count = culled.len();
        
        // Phase 3: Breed
        info!("🧬 Phase 3: Breeding new offspring...");
        let offspring = self.breed_new_generation(stats_before.total_generations + 1)?;
        report.bred_count = offspring.len();
        
        // Phase 4: Quarantine & Test
        info!("🔬 Phase 4: Quarantine and testing...");
        let (promoted, quarantined) = self.quarantine_and_test(&offspring)?;
        report.promoted_count = promoted.len();
        report.quarantined_count = quarantined.len();
        
        // Phase 5: Promote successful offspring
        info!("⬆️ Phase 5: Promoting to production...");
        self.promote_to_production(&promoted)?;
        
        // Phase 6: Distill cloud knowledge (if any)
        info!("📚 Phase 6: Distilling cloud knowledge...");
        self.distill_cloud_knowledge().await?;
        
        // Get stats after
        let stats_after = {
            let book = self.stud_book.read();
            book.get_stats()?
        };
        report.avg_fitness_after = stats_after.avg_fitness;
        
        // Generate per-species results
        for species in SpeciesType::all() {
            let result = self.get_species_evolution_result(species)?;
            report.species_results.push(result);
        }
        
        let elapsed = start.elapsed();
        info!("🌙 Night School complete in {:?}", elapsed);
        info!("  Culled: {} | Bred: {} | Promoted: {}", 
            report.culled_count, report.bred_count, report.promoted_count);
        info!("  Fitness: {:.2} -> {:.2}", 
            report.avg_fitness_before, report.avg_fitness_after);
        
        Ok(report)
    }
    
    /// Evaluate all agents and update fitness scores
    fn evaluate_agents(&self) -> Result<()> {
        let mut registry = self.species_registry.write();
        
        // In production, this would analyze task logs and update fitness
        // For now, simulate fitness decay and random improvement
        
        let agents = registry.get_ranked();
        for agent in agents {
            // Simulate fitness evaluation based on performance
            let success_rate = if agent.total_tasks > 0 {
                agent.successful_tasks as f32 / agent.total_tasks as f32
            } else {
                0.5
            };
            
            // Update fitness (moving average)
            let new_fitness = (agent.fitness * 0.7) + (success_rate * 0.3);
            registry.update_fitness(&agent.id, new_fitness);
        }
        
        Ok(())
    }
    
    /// Cull agents below fitness threshold
    fn cull_underperformers(&self) -> Result<Vec<String>> {
        let book = self.stud_book.read();
        let underperformers = book.get_underperformers(self.cull_threshold)?;
        drop(book);
        
        let mut culled = Vec::new();
        
        for agent in underperformers {
            // Cull in Stud Book
            {
                let book = self.stud_book.read();
                // Would call cull_agent here
            }
            
            // Cull in Registry
            {
                let mut registry = self.species_registry.write();
                // Would remove from registry
            }
            
            culled.push(agent.id.clone());
            info!("  🗑️ Culled: {} (fitness: {:.2})", agent.id, agent.fitness);
        }
        
        // Also cull from registry
        let mut registry = self.species_registry.write();
        let registry_culled = registry.cull(self.cull_threshold);
        culled.extend(registry_culled);
        
        Ok(culled)
    }
    
    /// Breed new generation from top performers
    fn breed_new_generation(&self, generation: u32) -> Result<Vec<super::Offspring>> {
        let book = self.stud_book.read();
        
        let mut offspring_list = Vec::new();
        let mut engine = BreedingEngine::default_engine();
        
        // Breed for each species
        for species in SpeciesType::all() {
            let top_performers = book.get_top_performers(species, 10)?;
            
            if top_performers.len() < 2 {
                warn!("  Not enough {} for breeding (need 2, have {})", 
                    species.emoji_name(), top_performers.len());
                continue;
            }
            
            // Select breeding pairs
            let pairs = engine.select_parents(
                &top_performers,
                2, // Breed 2 offspring per species
                SelectionStrategy::Tournament,
            );
            
            drop(book); // Release lock before breeding
            
            // Breed each pair
            for pair in pairs {
                let offspring = engine.breed(pair, generation)?;
                offspring_list.push(offspring);
            }
            
            // Re-acquire lock for next species
            // (In production, would restructure to avoid this)
        }
        
        Ok(offspring_list)
    }
    
    /// Quarantine and test new offspring
    fn quarantine_and_test(
        &self,
        offspring: &[super::Offspring],
    ) -> Result<(Vec<super::Offspring>, Vec<super::Offspring>)> {
        let mut promoted = Vec::new();
        let mut quarantined = Vec::new();
        
        for child in offspring {
            // Simulate testing on yesterday's tasks
            // In production, this would run actual evaluation
            
            let test_score = child.estimated_fitness;
            
            if test_score >= 0.5 {
                info!("  ✅ Promoted: {} (score: {:.2})", child.id, test_score);
                promoted.push(child.clone());
            } else {
                warn!("  ⏸️ Quarantined: {} (score: {:.2})", child.id, test_score);
                quarantined.push(child.clone());
            }
        }
        
        Ok((promoted, quarantined))
    }
    
    /// Promote offspring to production
    fn promote_to_production(&self, offspring: &[super::Offspring]) -> Result<()> {
        let mut registry = self.species_registry.write();
        
        for child in offspring {
            // Register new agent
            registry.register(crate::species::ActiveAgent {
                id: child.id.clone(),
                species: child.species,
                fitness: child.estimated_fitness,
                generation: child.generation,
                last_used: chrono::Utc::now(),
                total_tasks: 0,
                successful_tasks: 0,
            });
            
            // Log in Stud Book
            let book = self.stud_book.read();
            // Would register agent here
        }
        
        Ok(())
    }
    
    /// Distill cloud knowledge into local adapters
    async fn distill_cloud_knowledge(&self) -> Result<()> {
        // In production, this would:
        // 1. Check if any cloud API calls were made yesterday
        // 2. Extract knowledge from cloud responses
        // 3. Train/fine-tune local adapters
        // 4. Compare performance
        
        info!("  No cloud calls to distill today");
        Ok(())
    }
    
    /// Get evolution result for a specific species
    fn get_species_evolution_result(&self, species: SpeciesType) -> Result<SpeciesEvolutionResult> {
        let registry = self.species_registry.read();
        let agents = registry.get_by_species(species);
        
        let avg_fitness = if agents.is_empty() {
            0.0
        } else {
            agents.iter().map(|a| a.fitness).sum::<f32>() / agents.len() as f32
        };
        
        Ok(SpeciesEvolutionResult {
            species,
            population_before: agents.len(),
            population_after: agents.len(), // Would track changes
            avg_fitness_before: avg_fitness,
            avg_fitness_after: avg_fitness,
            culled: 0,
            bred: 0,
        })
    }
}

/// Cloud Distillation - Extract knowledge from cloud API calls
pub struct CloudDistiller {
    /// API key for cloud services
    api_key: Option<String>,
    /// Number of samples to use for distillation
    sample_size: usize,
}

impl CloudDistiller {
    /// Create a new cloud distiller
    pub fn new() -> Self {
        Self {
            api_key: None,
            sample_size: 100,
        }
    }
    
    /// Distill knowledge from cloud responses
    pub async fn distill(&self, _task_logs: &[AgentRecord]) -> Result<DistillationResult> {
        // In production, this would:
        // 1. Identify cloud API calls in task logs
        // 2. Extract input-output pairs
        // 3. Generate training data
        // 4. Fine-tune local adapter
        
        Ok(DistillationResult {
            samples_used: 0,
            epochs_trained: 0,
            loss_improvement: 0.0,
        })
    }
}

/// Result of distillation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationResult {
    pub samples_used: usize,
    pub epochs_trained: usize,
    pub loss_improvement: f32,
}

impl Default for CloudDistiller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_night_school() {
        // This test would require mocking the StudBook and Registry
        // For now, just verify the module compiles
    }
}
