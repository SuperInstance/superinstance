//! Stud Book - Genealogy Database
//! 
//! Tracks bloodlines, generations, fitness scores, and genetic history
//! using SQLite. This is the evolutionary memory of the Ranch.

use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::species::SpeciesType;

/// Stud Book - SQLite genealogy database
pub struct StudBook {
    conn: Connection,
}

/// A record of an agent in the Stud Book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    /// Unique agent ID
    pub id: String,
    /// Species type
    pub species: SpeciesType,
    /// Generation number
    pub generation: u32,
    /// Fitness score (0.0 to 1.0)
    pub fitness: f32,
    /// Parent IDs (for breeding)
    pub parent_ids: Vec<String>,
    /// Adapter path
    pub adapter_path: Option<String>,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Total tasks completed
    pub total_tasks: u64,
    /// Successful tasks
    pub successful_tasks: u64,
    /// Status
    pub status: AgentStatus,
}

/// Status of an agent
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Quarantined,
    Culled,
    Archived,
}

/// A task log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLog {
    pub id: i64,
    pub agent_id: String,
    pub species: SpeciesType,
    pub intent_kind: String,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Breeding record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreedingRecord {
    pub id: i64,
    pub sire_id: String,
    pub dam_id: String,
    pub offspring_id: String,
    pub merge_method: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

/// Statistics from the Stud Book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudBookStats {
    pub total_agents: u64,
    pub active_agents: u64,
    pub total_generations: u32,
    pub avg_fitness: f32,
    pub total_breeding_events: u64,
    pub total_culls: u64,
}

impl Default for StudBookStats {
    fn default() -> Self {
        Self {
            total_agents: 0,
            active_agents: 0,
            total_generations: 0,
            avg_fitness: 0.0,
            total_breeding_events: 0,
            total_culls: 0,
        }
    }
}

impl StudBook {
    /// Create a new Stud Book database
    pub fn new(path: &str) -> Result<Self> {
        let path = Path::new(path);
        
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(path)?;
        
        let book = Self { conn };
        book.initialize()?;
        
        info!("📖 Stud Book initialized at {}", path.display());
        Ok(book)
    }
    
    /// Initialize database schema
    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                species TEXT NOT NULL,
                generation INTEGER NOT NULL,
                fitness REAL NOT NULL,
                parent_ids TEXT,
                adapter_path TEXT,
                created_at TEXT NOT NULL,
                total_tasks INTEGER DEFAULT 0,
                successful_tasks INTEGER DEFAULT 0,
                status TEXT DEFAULT 'active'
            );
            
            CREATE TABLE IF NOT EXISTS task_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                species TEXT NOT NULL,
                intent_kind TEXT NOT NULL,
                success INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (agent_id) REFERENCES agents(id)
            );
            
            CREATE TABLE IF NOT EXISTS breeding_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sire_id TEXT NOT NULL,
                dam_id TEXT NOT NULL,
                offspring_id TEXT NOT NULL,
                merge_method TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                success INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_agents_species ON agents(species);
            CREATE INDEX IF NOT EXISTS idx_agents_fitness ON agents(fitness);
            CREATE INDEX IF NOT EXISTS idx_task_log_agent ON task_log(agent_id);
            CREATE INDEX IF NOT EXISTS idx_task_log_timestamp ON task_log(timestamp);
            "#,
        )?;
        
        Ok(())
    }
    
    /// Register a new agent
    pub fn register_agent(&self, record: &AgentRecord) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO agents 
            (id, species, generation, fitness, parent_ids, adapter_path, created_at, total_tasks, successful_tasks, status)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                record.id,
                serde_json::to_string(&record.species)?,
                record.generation,
                record.fitness,
                serde_json::to_string(&record.parent_ids)?,
                record.adapter_path,
                record.created_at.to_rfc3339(),
                record.total_tasks,
                record.successful_tasks,
                serde_json::to_string(&record.status)?,
            ],
        )?;
        
        debug!("Registered agent: {}", record.id);
        Ok(())
    }
    
    /// Get an agent by ID
    pub fn get_agent(&self, id: &str) -> Result<Option<AgentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, species, generation, fitness, parent_ids, adapter_path, created_at, total_tasks, successful_tasks, status FROM agents WHERE id = ?1"
        )?;
        
        let result = stmt.query_row(params![id], |row| {
            Ok(AgentRecord {
                id: row.get(0)?,
                species: serde_json::from_str(&row.get::<_, String>(1)?).unwrap_or(SpeciesType::Cattle),
                generation: row.get(2)?,
                fitness: row.get(3)?,
                parent_ids: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                adapter_path: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                total_tasks: row.get(7)?,
                successful_tasks: row.get(8)?,
                status: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or(AgentStatus::Active),
            })
        });
        
        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Log a task execution
    pub fn log_task(&mut self, species: SpeciesType, intent: crate::species::Intent, result: &str) -> Result<()> {
        let success = !result.contains("error") && !result.contains("failed");
        
        self.conn.execute(
            r#"
            INSERT INTO task_log (agent_id, species, intent_kind, success, duration_ms, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                format!("{}_unknown", serde_json::to_string(&species)?),
                serde_json::to_string(&species)?,
                intent.kind,
                success as i32,
                100, // Placeholder duration
                Utc::now().to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Update agent fitness
    pub fn update_fitness(&self, id: &str, fitness: f32) -> Result<()> {
        self.conn.execute(
            "UPDATE agents SET fitness = ?1 WHERE id = ?2",
            params![fitness, id],
        )?;
        
        Ok(())
    }
    
    /// Increment task counters
    pub fn increment_tasks(&self, id: &str, success: bool) -> Result<()> {
        if success {
            self.conn.execute(
                "UPDATE agents SET total_tasks = total_tasks + 1, successful_tasks = successful_tasks + 1 WHERE id = ?1",
                params![id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE agents SET total_tasks = total_tasks + 1 WHERE id = ?1",
                params![id],
            )?;
        }
        
        Ok(())
    }
    
    /// Get top performers by species
    pub fn get_top_performers(&self, species: SpeciesType, limit: usize) -> Result<Vec<AgentRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, species, generation, fitness, parent_ids, adapter_path, created_at, total_tasks, successful_tasks, status
            FROM agents
            WHERE species = ?1 AND status = 'active'
            ORDER BY fitness DESC, successful_tasks DESC
            LIMIT ?2
            "#,
        )?;
        
        let records = stmt.query_map(params![serde_json::to_string(&species)?, limit as i32], |row| {
            Ok(AgentRecord {
                id: row.get(0)?,
                species: serde_json::from_str(&row.get::<_, String>(1)?).unwrap_or(SpeciesType::Cattle),
                generation: row.get(2)?,
                fitness: row.get(3)?,
                parent_ids: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                adapter_path: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                total_tasks: row.get(7)?,
                successful_tasks: row.get(8)?,
                status: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or(AgentStatus::Active),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(records)
    }
    
    /// Get agents below fitness threshold
    pub fn get_underperformers(&self, threshold: f32) -> Result<Vec<AgentRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, species, generation, fitness, parent_ids, adapter_path, created_at, total_tasks, successful_tasks, status
            FROM agents
            WHERE fitness < ?1 AND status = 'active'
            "#,
        )?;
        
        let records = stmt.query_map(params![threshold], |row| {
            Ok(AgentRecord {
                id: row.get(0)?,
                species: serde_json::from_str(&row.get::<_, String>(1)?).unwrap_or(SpeciesType::Cattle),
                generation: row.get(2)?,
                fitness: row.get(3)?,
                parent_ids: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                adapter_path: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                total_tasks: row.get(7)?,
                successful_tasks: row.get(8)?,
                status: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or(AgentStatus::Active),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(records)
    }
    
    /// Mark agent as culled
    pub fn cull_agent(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE agents SET status = 'culled' WHERE id = ?1",
            params![id],
        )?;
        
        info!("🗑️ Culled agent: {}", id);
        Ok(())
    }
    
    /// Log a breeding event
    pub fn log_breeding(&self, sire_id: &str, dam_id: &str, offspring_id: &str, method: &str, success: bool) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO breeding_log (sire_id, dam_id, offspring_id, merge_method, timestamp, success)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![sire_id, dam_id, offspring_id, method, Utc::now().to_rfc3339(), success as i32],
        )?;
        
        Ok(())
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> Result<StudBookStats> {
        let total_agents: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM agents", [], |row| row.get(0)
        )?;
        
        let active_agents: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM agents WHERE status = 'active'", [], |row| row.get(0)
        )?;
        
        let total_generations: u32 = self.conn.query_row(
            "SELECT COALESCE(MAX(generation), 0) FROM agents", [], |row| row.get(0)
        )?;
        
        let avg_fitness: f32 = self.conn.query_row(
            "SELECT COALESCE(AVG(fitness), 0.0) FROM agents WHERE status = 'active'", [], |row| row.get(0)
        )?;
        
        let total_breeding_events: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM breeding_log WHERE success = 1", [], |row| row.get(0)
        )?;
        
        let total_culls: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM agents WHERE status = 'culled'", [], |row| row.get(0)
        )?;
        
        Ok(StudBookStats {
            total_agents,
            active_agents,
            total_generations,
            avg_fitness,
            total_breeding_events,
            total_culls,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stud_book() {
        let book = StudBook::new(":memory:").unwrap();
        let stats = book.get_stats().unwrap();
        assert_eq!(stats.total_agents, 0);
    }
}
