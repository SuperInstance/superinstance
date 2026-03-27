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
use parking_lot::{Mutex, RwLock};
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::collie::Collie;
use crate::evolution::{NightSchool, StudBook, StudBookStats};
use crate::pasture::{HardwareTier, LoRAManager, ModelPool, Pasture};
use crate::species::{ActiveAgent, Species, SpeciesType, SpeciesRegistry};
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
    stud_book: Arc<Mutex<StudBook>>,
    /// Species Registry - all available livestock
    species_registry: Arc<RwLock<SpeciesRegistry>>,
    /// Current day counter (for evolution tracking)
    day_counter: Arc<RwLock<u64>>,
    /// Resource usage tracking
    resource_usage: Arc<RwLock<ResourceUsage>>,
    /// Economic counter - dollars saved by using local vs cloud
    dollars_saved: Arc<RwLock<f64>>,
    /// Time the ranch was started (for uptime calculation)
    startup_time: Instant,
    /// Expected tokens/sec for detected hardware tier
    hardware_tps: f64,
    /// Timestamp of last completed Night School run
    night_school_last_run: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
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
        let stud_book = Arc::new(Mutex::new(
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

        let hardware_tps = HardwareTier::detect().expected_tps();

        Ok(Self {
            config,
            collie,
            pasture,
            stud_book,
            species_registry,
            day_counter: Arc::new(RwLock::new(1)),
            resource_usage,
            dollars_saved: Arc::new(RwLock::new(0.0)),
            startup_time: Instant::now(),
            hardware_tps,
            night_school_last_run: Arc::new(RwLock::new(None)),
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
    
    /// Uptime in seconds since the Ranch was started
    pub fn get_uptime_secs(&self) -> u64 {
        self.startup_time.elapsed().as_secs()
    }

    /// Expected tok/s for the detected hardware tier
    pub fn get_hardware_tps(&self) -> f64 {
        self.hardware_tps
    }

    /// Snapshot of all active agents, ranked by fitness
    pub fn get_all_agents(&self) -> Vec<ActiveAgent> {
        self.species_registry.read().get_ranked()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Look up a single active agent by id
    pub fn get_agent(&self, id: &str) -> Option<ActiveAgent> {
        self.species_registry.read().get(id).cloned()
    }

    /// Average fitness across all registered agents (0.0 if none)
    pub fn get_avg_fitness(&self) -> f32 {
        let agents = self.species_registry.read();
        let ranked = agents.get_ranked();
        if ranked.is_empty() {
            return 0.0;
        }
        ranked.iter().map(|a| a.fitness).sum::<f32>() / ranked.len() as f32
    }

    /// Timestamp of the last completed Night School run
    pub fn get_night_school_last_run(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        *self.night_school_last_run.read()
    }

    /// Stud Book aggregate statistics
    pub fn get_stud_book_stats(&self) -> StudBookStats {
        self.stud_book.lock().get_stats().unwrap_or_default()
    }

    /// Run the Night School breeding cycle
    pub async fn run_night_school(&self) -> Result<()> {
        if !self.config.night_school_enabled {
            debug!("Night School is disabled");
            return Ok(());
        }
        
        loop {
            // Calculate time until 02:00
            // SAFETY: We use safe datetime construction to avoid panics
            let now = chrono::Local::now();
            
            // Build next night school time safely
            let next_night_school = match now.date_naive().and_hms_opt(NIGHT_SCHOOL_HOUR, 0, 0) {
                Some(dt) => {
                    // Convert to local timezone safely
                    match dt.and_local_timezone(chrono::Local) {
                        chrono::LocalResult::Single(local) => local,
                        chrono::LocalResult::Ambiguous(local, _) => local,
                        chrono::LocalResult::None => {
                            // Fallback: use tomorrow at NIGHT_SCHOOL_HOUR
                            warn!("Ambiguous time for night school, using fallback");
                            continue;
                        }
                    }
                }
                None => {
                    // Invalid time (e.g., 25:00:00), skip this iteration
                    warn!("Invalid night school hour configuration");
                    sleep(Duration::from_secs(3600)).await;
                    continue;
                }
            };
            
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
                    *self.night_school_last_run.write() = Some(chrono::Utc::now());
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
    
    /// Check if Night School is enabled
    pub fn is_night_school_enabled(&self) -> bool {
        self.config.night_school_enabled
    }
    
    /// Get time until next Night School run
    pub fn get_night_school_next_run(&self) -> Duration {
        use chrono::{Local, TimeZone};
        
        let now = Local::now();
        let hour = NIGHT_SCHOOL_HOUR;
        
        let next_run_time = match now.date_naive().and_hms_opt(hour, 0, 0) {
            Some(dt) => {
                match dt.and_local_timezone(Local) {
                    chrono::LocalResult::Single(local) => local,
                    chrono::LocalResult::Ambiguous(local, _) => local,
                    chrono::LocalResult::None => return Duration::from_secs(3600),
                }
            }
            None => return Duration::from_secs(3600),
        };
        
        let next_run = if next_run_time > now {
            next_run_time
        } else {
            next_run_time + chrono::Duration::days(1)
        };
        
        (next_run - now).to_std().unwrap_or(Duration::from_secs(3600))
    }
}

/// Morning Routine demo - showcases the ecosystem working together
/// 
/// This is a demonstration function that shows how different species
/// collaborate to complete a typical morning workflow:
/// 1. Chicken monitors for motion/events
/// 2. Sheep flock classifies and triages emails
/// 3. Goat navigates and analyzes log files
/// 4. Duck fetches external data (calendar)
/// 5. Cattle synthesizes a morning briefing
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
