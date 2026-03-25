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
use chrono::{DateTime, Local, TimeZone, Utc};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use super::{BreedingEngine, StudBook, AgentRecord, SelectionStrategy};
use crate::species::{SpeciesRegistry, SpeciesType};

/// Default hour for Night School (02:00)
pub const NIGHT_SCHOOL_HOUR: u32 = 2;

/// Night School - Daily evolution cycle
pub struct NightSchool {
    /// Stud Book reference
    stud_book: Arc<Mutex<StudBook>>,
    /// Species Registry reference
    species_registry: Arc<RwLock<SpeciesRegistry>>,
    /// Culling threshold
    cull_threshold: f32,
    /// Breeding engine
    breeding_engine: BreedingEngine,
    /// Last run timestamp
    last_run: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// Schedule hour (24-hour format, default 2)
    schedule_hour: u32,
    /// Manual trigger flag
    manual_trigger: Arc<RwLock<bool>>,
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
        stud_book: Arc<Mutex<StudBook>>,
        species_registry: Arc<RwLock<SpeciesRegistry>>,
        cull_threshold: f32,
    ) -> Self {
        Self {
            stud_book,
            species_registry,
            cull_threshold,
            breeding_engine: BreedingEngine::default_engine(),
            last_run: Arc::new(RwLock::new(None)),
            schedule_hour: NIGHT_SCHOOL_HOUR,
            manual_trigger: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Create with custom schedule hour
    pub fn with_schedule_hour(mut self, hour: u32) -> Self {
        self.schedule_hour = hour % 24;
        self
    }
    
    /// Get last run timestamp
    pub fn last_run(&self) -> Option<DateTime<Utc>> {
        *self.last_run.read()
    }
    
    /// Trigger manual run (used by /api/night endpoint)
    pub fn trigger_manual(&self) {
        *self.manual_trigger.write() = true;
    }
    
    /// Calculate time until next scheduled run
    pub fn time_until_next_run(&self) -> Duration {
        let now = Local::now();
        
        // Build next run time safely
        let next_run_time = match now.date_naive().and_hms_opt(self.schedule_hour, 0, 0) {
            Some(dt) => {
                match dt.and_local_timezone(Local) {
                    chrono::LocalResult::Single(local) => local,
                    chrono::LocalResult::Ambiguous(local, _) => local,
                    chrono::LocalResult::None => {
                        // Fallback to 1 hour from now
                        return Duration::from_secs(3600);
                    }
                }
            }
            None => {
                return Duration::from_secs(3600);
            }
        };
        
        let next_run = if next_run_time > now {
            next_run_time
        } else {
            next_run_time + chrono::Duration::days(1)
        };
        
        (next_run - now)
            .to_std()
            .unwrap_or(Duration::from_secs(3600))
    }
    
    /// Run the cron scheduler loop (tokio-based)
    /// 
    /// This is the main entry point for the background task that:
    /// 1. Waits until 02:00 (or configured hour)
    /// 2. Runs the evolution cycle
    /// 3. Repeats forever
    pub async fn run_cron(&self) -> Result<()> {
        info!("🌙 Night School cron scheduler started (runs at {:02}:00)", self.schedule_hour);
        
        loop {
            // Check for manual trigger
            if *self.manual_trigger.read() {
                info!("🌙 Manual Night School trigger received");
                *self.manual_trigger.write() = false;
                
                if let Err(e) = self.run().await {
                    error!("Night School error: {}", e);
                }
                continue;
            }
            
            // Calculate time until next run
            let sleep_duration = self.time_until_next_run();
            
            info!(
                "🌙 Next Night School in {}h {}m",
                sleep_duration.as_secs() / 3600,
                (sleep_duration.as_secs() % 3600) / 60
            );
            
            // Sleep with periodic checks for manual trigger
            let check_interval = Duration::from_secs(60);
            let mut remaining = sleep_duration;
            
            while remaining > Duration::ZERO {
                let sleep_time = std::cmp::min(remaining, check_interval);
                sleep(sleep_time).await;
                remaining = remaining.saturating_sub(check_interval);
                
                // Check for manual trigger during wait
                if *self.manual_trigger.read() {
                    info!("🌙 Manual Night School trigger received during wait");
                    *self.manual_trigger.write() = false;
                    
                    if let Err(e) = self.run().await {
                        error!("Night School error: {}", e);
                    }
                    break;
                }
            }
            
            // Time for scheduled run
            if remaining == Duration::ZERO {
                if let Err(e) = self.run().await {
                    error!("Night School error: {}", e);
                }
            }
        }
    }
    
    /// Run the Night School cycle
    pub async fn run(&self) -> Result<NightSchoolReport> {
        info!("🌙 Night School starting...");
        let start = std::time::Instant::now();
        
        // Get stats before
        let stats_before = {
            let book = self.stud_book.lock();
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
            let book = self.stud_book.lock();
            book.get_stats()?
        };
        report.avg_fitness_after = stats_after.avg_fitness;
        
        // Generate per-species results
        for species in SpeciesType::all() {
            let result = self.get_species_evolution_result(species)?;
            report.species_results.push(result);
        }
        
        // Update last run timestamp
        {
            let mut last = self.last_run.write();
            *last = Some(report.timestamp);
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
        
        // Collect agents first to avoid borrow conflicts
        let agents: Vec<_> = registry.get_ranked().into_iter().map(|a| {
            (a.id.clone(), a.fitness, a.total_tasks, a.successful_tasks)
        }).collect();
        
        for (id, fitness, total_tasks, successful_tasks) in agents {
            // Simulate fitness evaluation based on performance
            let success_rate = if total_tasks > 0 {
                successful_tasks as f32 / total_tasks as f32
            } else {
                0.5
            };
            
            // Update fitness (moving average)
            let new_fitness = (fitness * 0.7) + (success_rate * 0.3);
            registry.update_fitness(&id, new_fitness);
        }
        
        Ok(())
    }
    
    /// Cull agents below fitness threshold
    fn cull_underperformers(&self) -> Result<Vec<String>> {
        let book = self.stud_book.lock();
        let underperformers = book.get_underperformers(self.cull_threshold)?;
        drop(book);
        
        let mut culled = Vec::new();
        
        for agent in underperformers {
            // Cull in Stud Book
            {
                let book = self.stud_book.lock();
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
        let mut offspring_list = Vec::new();
        let mut engine = BreedingEngine::default_engine();
        
        // Breed for each species
        for species in SpeciesType::all() {
            // Acquire lock inside the loop to avoid holding it across iterations
            let top_performers = {
                let book = self.stud_book.lock();
                book.get_top_performers(species, 10)?
            };
            
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
            
            // Breed each pair
            for pair in pairs {
                let offspring = engine.breed(pair, generation)?;
                offspring_list.push(offspring);
            }
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
            let book = self.stud_book.lock();
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
    use std::sync::Arc;
    use parking_lot::{Mutex, RwLock};
    
    fn create_test_night_school() -> NightSchool {
        let stud_book = Arc::new(Mutex::new(
            StudBook::new(":memory:").expect("Failed to create in-memory StudBook")
        ));
        let species_registry = Arc::new(RwLock::new(SpeciesRegistry::new()));
        NightSchool::new(stud_book, species_registry, 0.4)
    }
    
    #[tokio::test]
    async fn test_night_school_creation() {
        let night_school = create_test_night_school();
        assert!(night_school.last_run().is_none());
        assert_eq!(night_school.schedule_hour, NIGHT_SCHOOL_HOUR);
    }
    
    #[tokio::test]
    async fn test_night_school_with_custom_hour() {
        let night_school = create_test_night_school().with_schedule_hour(3);
        assert_eq!(night_school.schedule_hour, 3);
    }
    
    #[tokio::test]
    async fn test_time_until_next_run() {
        let night_school = create_test_night_school();
        let duration = night_school.time_until_next_run();
        
        // Should be less than 24 hours
        assert!(duration < Duration::from_secs(24 * 60 * 60));
        // Should be at least 0
        assert!(duration > Duration::ZERO);
    }
    
    #[tokio::test]
    async fn test_manual_trigger() {
        let night_school = create_test_night_school();
        
        // Initially not triggered
        assert!(!*night_school.manual_trigger.read());
        
        // Trigger
        night_school.trigger_manual();
        assert!(*night_school.manual_trigger.read());
    }
    
    #[tokio::test]
    async fn test_night_school_report_serialization() {
        let report = NightSchoolReport {
            timestamp: chrono::Utc::now(),
            day: 1,
            culled_count: 2,
            bred_count: 3,
            promoted_count: 1,
            quarantined_count: 2,
            avg_fitness_before: 0.5,
            avg_fitness_after: 0.6,
            species_results: vec![],
        };
        
        let json = serde_json::to_string(&report).expect("Failed to serialize");
        assert!(json.contains("culled_count"));
        assert!(json.contains("bred_count"));
        
        let deserialized: NightSchoolReport = 
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.culled_count, 2);
        assert_eq!(deserialized.bred_count, 3);
    }
    
    #[test]
    fn test_cloud_distiller_creation() {
        let distiller = CloudDistiller::new();
        assert_eq!(distiller.sample_size, 100);
    }
    
    #[tokio::test]
    async fn test_distillation_result() {
        let distiller = CloudDistiller::new();
        let result = distiller.distill(&[]).await.expect("Distillation failed");
        assert_eq!(result.samples_used, 0);
    }
    
    #[tokio::test]
    async fn test_night_school_full_cycle() {
        let night_school = create_test_night_school();
        
        // Run the full cycle
        let result = night_school.run().await;
        
        // Should succeed
        assert!(result.is_ok());
        
        let report = result.unwrap();
        assert!(report.timestamp <= chrono::Utc::now());
        
        // Last run should be updated
        assert!(night_school.last_run().is_some());
    }
}
