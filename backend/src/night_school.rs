//! Night School Module - Autonomous evolution via Tokio cron
//! Runs at 02:00 daily to breed/cull agents based on performance
//! Uses SLERP interpolation for smooth genetic transitions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use chrono::{DateTime, Utc, Timelike};

// ============================================================================
// Types
// ============================================================================

/// Agent performance metrics for evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: String,
    pub breed: String,
    pub success_rate: f32,
    pub avg_response_time_ms: f32,
    pub total_interactions: u32,
    pub user_satisfaction: f32,
    pub last_active: DateTime<Utc>,
}

/// Gene for SLERP interpolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gene {
    pub name: String,
    pub expression: f32,
    pub dominant: bool,
}

/// Evolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    pub timestamp: DateTime<Utc>,
    pub agents_bred: u32,
    pub agents_culled: u32,
    pub genes_mutated: u32,
    pub improvements: Vec<String>,
}

// ============================================================================
// Night School Core
// ============================================================================

/// Night School state
pub struct NightSchool {
    db_pool: SqlitePool,
    pasture_path: PathBuf,
    breeding_threshold: f32,
    culling_threshold: f32,
}

impl NightSchool {
    pub fn new(db_pool: SqlitePool, pasture_path: PathBuf) -> Self {
        Self {
            db_pool,
            pasture_path,
            breeding_threshold: 0.75,
            culling_threshold: 0.25,
        }
    }

    /// Run Night School evolution cycle
    pub async fn run_evolution(&self) -> Result<EvolutionResult> {
        info!("🌙 Night School starting evolution cycle...");
        
        let start_time = Utc::now();
        let mut agents_bred = 0;
        let mut agents_culled = 0;
        let mut genes_mutated = 0;
        let mut improvements = Vec::new();

        // 1. Collect agent metrics
        let metrics = self.collect_metrics().await?;
        info!("Collected metrics for {} agents", metrics.len());

        // 2. Identify top performers for breeding
        let top_performers: Vec<_> = metrics
            .iter()
            .filter(|m| m.success_rate > self.breeding_threshold)
            .collect();

        // 3. Identify bottom performers for culling
        let bottom_performers: Vec<_> = metrics
            .iter()
            .filter(|m| m.success_rate < self.culling_threshold)
            .collect();

        // 4. Breed top performers using SLERP
        for i in (0..top_performers.len()).step_by(2) {
            if i + 1 < top_performers.len() {
                let parent1 = &top_performers[i];
                let parent2 = &top_performers[i + 1];
                
                match self.breed_agents(parent1, parent2).await {
                    Ok(mutations) => {
                        agents_bred += 1;
                        genes_mutated += mutations;
                        improvements.push(format!(
                            "Bred {} + {} -> {} mutations",
                            parent1.agent_id, parent2.agent_id, mutations
                        ));
                    }
                    Err(e) => error!("Breeding failed: {}", e),
                }
            }
        }

        // 5. Cull bottom performers
        for agent in bottom_performers {
            if self.cull_agent(&agent.agent_id).await? {
                agents_culled += 1;
                improvements.push(format!("Culled underperformer: {}", agent.agent_id));
            }
        }

        let result = EvolutionResult {
            timestamp: start_time,
            agents_bred,
            agents_culled,
            genes_mutated,
            improvements,
        };

        info!(
            "✅ Night School complete: bred={}, culled={}, mutated={}",
            result.agents_bred, result.agents_culled, result.genes_mutated
        );

        Ok(result)
    }

    /// Collect performance metrics from database
    async fn collect_metrics(&self) -> Result<Vec<AgentMetrics>> {
        let metrics = sqlx::query_as!(
            AgentMetrics,
            r#"
            SELECT 
                agent_id,
                breed,
                success_rate,
                avg_response_time_ms,
                total_interactions,
                user_satisfaction,
                last_active as "last_active: DateTime<Utc>"
            FROM agent_metrics
            WHERE last_active > datetime('now', '-7 days')
            ORDER BY success_rate DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(metrics)
    }

    /// Breed two agents using SLERP interpolation
    async fn breed_agents(&self, parent1: &AgentMetrics, parent2: &AgentMetrics) -> Result<u32> {
        info!("🧬 Breeding {} + {}", parent1.agent_id, parent2.agent_id);

        // Load parent genes
        let genes1 = self.load_agent_genes(&parent1.agent_id).await?;
        let genes2 = self.load_agent_genes(&parent2.agent_id).await?;

        // SLERP interpolation for smooth genetic transition
        let child_genes = self.slerp_genes(&genes1, &genes2, 0.5);

        // Create child agent
        let child_id = format!("{}-{}", parent1.agent_id, parent2.agent_id);
        self.save_agent_genes(&child_id, &child_genes).await?;

        Ok(child_genes.len() as u32)
    }

    /// SLERP interpolation between two gene sets
    /// Enables smooth genetic transitions without jarring jumps
    fn slerp_genes(&self, genes1: &[Gene], genes2: &[Gene], t: f32) -> Vec<Gene> {
        let mut child_genes = Vec::new();

        for g1 in genes1 {
            if let Some(g2) = genes2.iter().find(|g| g.name == g1.name) {
                // SLERP formula for smooth interpolation
                let omega = (g1.expression / g2.expression).acos();
                let sin_omega = omega.sin();

                let expr = if sin_omega.abs() > 0.0001 {
                    let s1 = ((1.0 - t) * omega).sin() / sin_omega;
                    let s2 = (t * omega).sin() / sin_omega;
                    s1 * g1.expression + s2 * g2.expression
                } else {
                    // Linear interpolation for small angles
                    g1.expression * (1.0 - t) + g2.expression * t
                };

                child_genes.push(Gene {
                    name: g1.name.clone(),
                    expression: expr.clamp(0.0, 1.0),
                    dominant: g1.dominant || g2.dominant,
                });
            }
        }

        child_genes
    }

    /// Load agent genes from pasture
    async fn load_agent_genes(&self, agent_id: &str) -> Result<Vec<Gene>> {
        let gene_path = self.pasture_path.join(agent_id).join("breed.toml");
        
        if gene_path.exists() {
            let content = tokio::fs::read_to_string(&gene_path).await?;
            let config: toml::Value = toml::from_str(&content)?;
            
            let genes = config
                .get("genes")
                .and_then(|g| g.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| {
                            Some(Gene {
                                name: g.get("name")?.as_str()?.to_string(),
                                expression: g.get("expression")?.as_float()? as f32,
                                dominant: g.get("dominant")?.as_bool().unwrap_or(false),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            Ok(genes)
        } else {
            Ok(vec![])
        }
    }

    /// Save agent genes to pasture
    async fn save_agent_genes(&self, agent_id: &str, genes: &[Gene]) -> Result<()> {
        let agent_path = self.pasture_path.join(agent_id);
        tokio::fs::create_dir_all(&agent_path).await?;

        let gene_path = agent_path.join("breed.toml");
        let content = self.serialize_genes(genes);
        
        tokio::fs::write(&gene_path, content).await?;
        info!("Saved genes for agent: {}", agent_id);

        Ok(())
    }

    /// Serialize genes to TOML format
    fn serialize_genes(&self, genes: &[Gene]) -> String {
        let mut toml = String::new();
        toml.push_str("# Auto-generated by Night School\n\n");

        for gene in genes {
            toml.push_str(&format!(
                "[[genes]]\nname = \"{}\"\nexpression = {:.4}\ndominant = {}\n\n",
                gene.name, gene.expression, gene.dominant
            ));
        }

        toml
    }

    /// Cull an underperforming agent
    async fn cull_agent(&self, agent_id: &str) -> Result<bool> {
        warn!("🗑️ Culling agent: {}", agent_id);

        let agent_path = self.pasture_path.join(agent_id);
        
        if agent_path.exists() {
            tokio::fs::remove_dir_all(&agent_path).await?;
            info!("Removed agent directory: {:?}", agent_path);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// ============================================================================
// Cron Scheduler
// ============================================================================

/// Start Night School cron job (runs at 02:00 daily)
pub async fn start_night_school(night_school: NightSchool) -> Result<()> {
    info!("🌙 Night School cron scheduler started");

    loop {
        let now = Utc::now();
        let next_run = calculate_next_2am(&now);
        let duration_until = (next_run - now).to_std()?;

        info!(
            "Next Night School run in: {:?}",
            duration_until
        );

        sleep(duration_until).await;

        info!("🌙 Night School cron triggered!");
        
        if let Err(e) = night_school.run_evolution().await {
            error!("Night School evolution failed: {}", e);
        }
    }
}

/// Calculate next 02:00 AM
fn calculate_next_2am(now: &DateTime<Utc>) -> DateTime<Utc> {
    let next_2am = now
        .with_hour(2)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .unwrap();

    if next_2am > *now {
        next_2am
    } else {
        next_2am + chrono::Duration::days(1)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slerp_interpolation() {
        let ns = NightSchool::new(
            SqlitePool::connect(":memory:").await.unwrap(),
            PathBuf::from("/tmp"),
        );

        let genes1 = vec![
            Gene { name: "speed".to_string(), expression: 0.9, dominant: true },
            Gene { name: "accuracy".to_string(), expression: 0.7, dominant: false },
        ];

        let genes2 = vec![
            Gene { name: "speed".to_string(), expression: 0.5, dominant: false },
            Gene { name: "accuracy".to_string(), expression: 0.95, dominant: true },
        ];

        let child = ns.slerp_genes(&genes1, &genes2, 0.5);

        assert_eq!(child.len(), 2);
        // Child should have traits between parents
        assert!(child[0].expression > 0.5 && child[0].expression < 0.9);
        assert!(child[1].expression > 0.7 && child[1].expression < 0.95);
    }

    #[test]
    fn test_next_2am_calculation() {
        let now = Utc::now();
        let next = calculate_next_2am(&now);

        assert!(next > now);
        assert_eq!(next.hour(), 2);
        assert_eq!(next.minute(), 0);
    }
}
