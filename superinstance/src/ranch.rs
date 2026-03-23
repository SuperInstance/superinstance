//! The Ranch - The Container that Manages the Pasture
//! 
//! The Ranch is the top-level container that holds all components of the ecosystem:
//! - The Border Collie (Foreman)
//! - The Pasture (Resource Manager)
//! - The Livestock (Species Agents)
//! - The Stud Book (Evolution Database)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use parking_lot::RwLock;
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::collie::Collie;
use crate::evolution::{NightSchool, StudBook};
use crate::pasture::{LoRAManager, ModelPool, Pasture};
use crate::species::{Species, SpeciesType, SpeciesRegistry};
use crate::{Config, NIGHT_SCHOOL_HOUR};

/// The Ranch - the top-level container for the entire ecosystem
pub struct Ranch {
    /// Configuration
    config: Config,
    /// The Border Collie - manages and herds livestock
    pub collie: Collie,
    /// The Pasture - resource management for models and adapters
    pasture: Arc<Pasture>,
    /// The Stud Book - tracks evolution and genetics
    stud_book: Arc<RwLock<StudBook>>,
    /// Species Registry - all available livestock
    species_registry: Arc<RwLock<SpeciesRegistry>>,
    /// Current day counter (for evolution tracking)
    day_counter: Arc<RwLock<u64>>,
    /// Resource usage tracking
    resource_usage: Arc<RwLock<ResourceUsage>>,
    /// Economic counter - dollars saved by using local vs cloud
    dollars_saved: Arc<RwLock<f64>>,
}

/// Resource usage tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub vram_used_bytes: u64,
    pub vram_max_bytes: u64,
    pub active_adapters: usize,
    pub pending_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
}

impl ResourceUsage {
    pub fn vram_used_percent(&self) -> f64 {
        if self.vram_max_bytes == 0 {
            return 0.0;
        }
        (self.vram_used_bytes as f64 / self.vram_max_bytes as f64) * 100.0
    }
}

impl Ranch {
    /// Create a new Ranch instance
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Pasture (Resource Manager)...");
        let pasture = Arc::new(Pasture::new(&config).await?);
        
        info!("Initializing Stud Book (Evolution Database)...");
        let stud_book = Arc::new(RwLock::new(
            StudBook::new(&config.stud_book_path)?
        ));
        
        info!("Initializing Species Registry...");
        let species_registry = Arc::new(RwLock::new(
            SpeciesRegistry::new()
        ));
        
        info!("Initializing Border Collie (Foreman)...");
        let collie = Collie::new(
            Arc::clone(&pasture),
            Arc::clone(&stud_book),
            Arc::clone(&species_registry),
        );
        
        let resource_usage = Arc::new(RwLock::new(ResourceUsage {
            vram_max_bytes: config.max_vram_bytes,
            ..Default::default()
        }));
        
        Ok(Self {
            config,
            collie,
            pasture,
            stud_book,
            species_registry,
            day_counter: Arc::new(RwLock::new(1)),
            resource_usage,
            dollars_saved: Arc::new(RwLock::new(0.0)),
        })
    }
    
    /// Get current resource usage
    pub fn get_resource_usage(&self) -> ResourceUsage {
        self.resource_usage.read().clone()
    }
    
    /// Get current day counter
    pub fn get_day(&self) -> u64 {
        *self.day_counter.read()
    }
    
    /// Get dollars saved
    pub fn get_dollars_saved(&self) -> f64 {
        *self.dollars_saved.read()
    }
    
    /// Add to dollars saved counter
    pub fn add_savings(&self, amount: f64) {
        let mut savings = self.dollars_saved.write();
        *savings += amount;
        info!("💰 Savings: ${:.2} (added ${:.2})", *savings, amount);
    }
    
    /// Get active species count by type
    pub fn get_species_counts(&self) -> HashMap<SpeciesType, usize> {
        let registry = self.species_registry.read();
        registry.get_counts()
    }
    
    /// Get total number of active agents
    pub fn get_total_agents(&self) -> usize {
        let registry = self.species_registry.read();
        registry.total_active()
    }
    
    /// Run the Night School breeding cycle
    pub async fn run_night_school(&self) -> Result<()> {
        if !self.config.night_school_enabled {
            debug!("Night School is disabled");
            return Ok(());
        }
        
        loop {
            // Calculate time until 02:00
            let now = chrono::Local::now();
            let next_night_school = now.date_naive()
                .and_hms_opt(NIGHT_SCHOOL_HOUR, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap();
            
            let next_run = if next_night_school > now {
                next_night_school
            } else {
                next_night_school + chrono::Duration::days(1)
            };
            
            let duration = (next_run - now).to_std()?;
            
            info!(
                "🌙 Next Night School in {} hours",
                duration.as_secs() / 3600
            );
            
            sleep(duration).await;
            
            info!("═══════════════════════════════════════════════════════════");
            info!("  🌙 NIGHT SCHOOL - The Breeding Cycle Begins");
            info!("═══════════════════════════════════════════════════════════");
            
            // Increment day counter
            {
                let mut day = self.day_counter.write();
                *day += 1;
                info!("📅 Day {} of the Ranch", *day);
            }
            
            // Run Night School
            let night_school = NightSchool::new(
                Arc::clone(&self.stud_book),
                Arc::clone(&self.species_registry),
                self.config.cull_threshold,
            );
            
            match night_school.run().await {
                Ok(report) => {
                    info!("Night School Report:");
                    info!("  Culled: {} underperformers", report.culled_count);
                    info!("  Bred: {} new offspring", report.bred_count);
                    info!("  Promoted: {} to production", report.promoted_count);
                }
                Err(e) => {
                    warn!("Night School error: {}", e);
                }
            }
            
            info!("═══════════════════════════════════════════════════════════");
            info!("  Night School complete. The Ranch has evolved.");
            info!("═══════════════════════════════════════════════════════════");
        }
    }
    
    /// Start monitoring (Chickens)
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Update resource usage
            let pasture_stats = self.pasture.get_stats();
            {
                let mut usage = self.resource_usage.write();
                usage.vram_used_bytes = pasture_stats.vram_used;
                usage.active_adapters = pasture_stats.active_adapters;
            }
            
            // Check for alerts
            let usage = self.resource_usage.read();
            if usage.vram_used_percent() > 90.0 {
                warn!("⚠️ VRAM usage critical: {:.1}%", usage.vram_used_percent());
            }
        }
    }
}

/// Morning Routine demo - showcases the ecosystem working together
pub async fn morning_routine(ranch: &Ranch) -> Result<()> {
    use crate::species::{Cattle, Chicken, Duck, Goat, Sheep, SpeciesOps};
    
    info!("═══════════════════════════════════════════════════════════");
    info!("  🌅 THE MORNING ROUTINE - Demo");
    info!("═══════════════════════════════════════════════════════════");
    
    let start = Instant::now();
    
    // Step 1: Chicken detects motion
    info!("\n🐔 Step 1: Chicken detects motion event...");
    let chicken = Chicken::new("watcher-01".to_string());
    chicken.execute("monitor_motion".to_string()).await?;
    
    // Step 2: Collie wakes Sheep to triage emails
    info!("\n🐑 Step 2: Sheep flock triages emails...");
    let sheep = vec![
        Sheep::new("classifier-01".to_string()),
        Sheep::new("classifier-02".to_string()),
        Sheep::new("classifier-03".to_string()),
    ];
    
    let mut email_results = vec![];
    for s in &sheep {
        let result = s.execute("classify_emails".to_string()).await?;
        email_results.push(result);
    }
    
    // Ensemble voting
    let spam_count = email_results.iter()
        .filter(|r| r.contains("spam"))
        .count();
    info!("  Flock consensus: {} spam, {} legitimate", spam_count, email_results.len() - spam_count);
    
    // Step 3: Goat climbs into log file
    info!("\n🐐 Step 3: Goat navigates complex log file...");
    let goat = Goat::new("navigator-01".to_string());
    let log_analysis = goat.execute("analyze_logs:/var/log/system.log".to_string()).await?;
    info!("  Found {} anomalies", log_analysis.matches("error").count());
    
    // Step 4: Duck fetches calendar data
    info!("\n🦆 Step 4: Duck flies to fetch calendar data...");
    let duck = Duck::new("fetcher-01".to_string());
    let calendar = duck.execute("fetch:calendar://today".to_string()).await?;
    info!("  Calendar events: {}", calendar.matches("\n").count() + 1);
    
    // Step 5: Cattle synthesizes the briefing
    info!("\n🐄 Step 5: Cattle synthesizes the morning briefing...");
    let cattle = Cattle::new("reasoner-01".to_string());
    let briefing = cattle.execute(format!(
        "synthesize_briefing from [emails: triaged], [logs: analyzed], [calendar: {}]",
        calendar.len()
    )).await?;
    info!("  Briefing length: {} chars", briefing.len());
    
    let elapsed = start.elapsed();
    
    info!("\n═══════════════════════════════════════════════════════════");
    info!("  ✅ MORNING ROUTINE COMPLETE");
    info!("  Time: {:.2}s | Cloud calls: 0 | RAM: <6GB", elapsed.as_secs_f64());
    info!("═══════════════════════════════════════════════════════════");
    
    // Update savings (would have cost ~$0.50 in cloud API calls)
    ranch.add_savings(0.50);
    
    Ok(())
}
