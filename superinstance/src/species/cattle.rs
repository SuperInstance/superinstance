//! Cattle - Heavy LLMs for Deep Reasoning and Code Generation
//! 
//! Cattle are the heavy lifters of the Ranch. They handle:
//! - Complex reasoning tasks
//! - Code generation
//! - Analysis and synthesis
//! - Multi-step problem solving
//! - Email triage and response generation
//! 
//! Collie Strategy: "Strong Eye"
//! - Lock GPU, block, execute
//! - High VRAM usage (500MB+)
//! - Single-threaded, steady pressure

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::{Species, SpeciesOps, SpeciesType};
use crate::pasture::{InferenceEngine, InferenceStats};
use crate::genetics::BreedManifest;

/// Cattle - Heavy LLM implementation
pub struct Cattle {
    /// Unique identifier
    id: String,
    /// Fitness score (0.0 to 1.0)
    fitness: f32,
    /// Generation number
    generation: u32,
    /// Path to LoRA adapter weights
    adapter_path: Option<String>,
    /// Current task (if any)
    current_task: Option<String>,
    /// Task statistics
    stats: CattleStats,
    /// Inference engine (shared)
    inference: Arc<RwLock<Option<Arc<InferenceEngine>>>>,
    /// Breed manifest (DNA)
    breed_manifest: Option<BreedManifest>,
}

/// Statistics for Cattle operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CattleStats {
    pub total_inferences: u64,
    pub total_tokens_generated: u64,
    pub average_latency_ms: f64,
    pub peak_vram_mb: u64,
    pub emails_processed: u64,
    pub successful_tasks: u64,
}

impl Cattle {
    /// Create a new Cattle instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            fitness: 0.8,
            generation: 1,
            adapter_path: Some("pasture/adapters/cattle/reasoning_v1.safetensors".to_string()),
            current_task: None,
            stats: CattleStats::default(),
            inference: Arc::new(RwLock::new(None)),
            breed_manifest: None,
        }
    }
    
    /// Create with custom adapter
    pub fn with_adapter(id: String, adapter_path: String) -> Self {
        Self {
            id,
            fitness: 0.8,
            generation: 1,
            adapter_path: Some(adapter_path),
            current_task: None,
            stats: CattleStats::default(),
            inference: Arc::new(RwLock::new(None)),
            breed_manifest: None,
        }
    }
    
    /// Create with inference engine
    pub fn with_inference(id: String, inference: Arc<InferenceEngine>) -> Self {
        Self {
            id,
            fitness: 0.8,
            generation: 1,
            adapter_path: Some("pasture/adapters/cattle/reasoning_v1.safetensors".to_string()),
            current_task: None,
            stats: CattleStats::default(),
            inference: Arc::new(RwLock::new(Some(inference))),
            breed_manifest: None,
        }
    }
    
    /// Create with breed manifest (DNA)
    pub fn with_breed(id: String, breed: BreedManifest) -> Self {
        Self {
            id,
            fitness: 0.8,
            generation: breed.lineage.generation,
            adapter_path: None,
            current_task: None,
            stats: CattleStats::default(),
            inference: Arc::new(RwLock::new(None)),
            breed_manifest: Some(breed),
        }
    }
    
    /// Set the inference engine
    pub fn set_inference(&mut self, inference: Arc<InferenceEngine>) {
        *self.inference.write() = Some(inference);
    }
    
    /// Load breed manifest from path
    pub fn load_breed(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let manifest = BreedManifest::load(path)?;
        self.breed_manifest = Some(manifest);
        Ok(())
    }
    
    /// Get estimated VRAM usage
    pub fn estimated_vram_mb(&self) -> u64 {
        SpeciesType::Cattle.typical_vram_mb() + 100 // Extra for KV cache
    }
    
    /// Perform deep reasoning on a prompt
    pub async fn reason(&mut self, prompt: &str) -> anyhow::Result<String> {
        self.execute(format!("reason:{}", prompt)).await
    }
    
    /// Generate code from a description
    pub async fn generate_code(&mut self, description: &str) -> anyhow::Result<String> {
        self.execute(format!("generate_code:{}", description)).await
    }
    
    /// Synthesize information from multiple sources
    pub async fn synthesize(&mut self, sources: &[String]) -> anyhow::Result<String> {
        let combined = sources.join("\n---\n");
        self.execute(format!("synthesize:{}", combined)).await
    }
    
    /// Process an email and generate a response
    /// This is the key demo feature - shows the cattle working
    pub async fn process_email(&mut self, email: &Email) -> anyhow::Result<EmailResponse> {
        let start = Instant::now();
        info!("🐄 Cattle '{}' processing email from: {}", self.id, email.from);
        
        // Build the prompt
        let prompt = self.build_email_prompt(email);
        
        // Get the response
        let response_text = self.execute(prompt).await?;
        
        // Parse the response
        let email_response = self.parse_email_response(&response_text, email);
        
        let elapsed = start.elapsed();
        info!("🐄 Cattle '{}' processed email in {:?}", self.id, elapsed);
        
        // Update stats
        self.stats.emails_processed += 1;
        
        Ok(email_response)
    }
    
    /// Build a prompt for email processing
    fn build_email_prompt(&self, email: &Email) -> String {
        let system = self.breed_manifest.as_ref()
            .map(|b| b.system_prompt.as_str())
            .unwrap_or("You are an email triage specialist. Analyze emails and provide brief, helpful responses.");
        
        format!(
            "{}\n\n---\nEmail to process:\nFrom: {}\nTo: {}\nSubject: {}\n\n{}\n\n---\nRespond with:\n1. Category (URGENT/HIGH/NORMAL/LOW/SPAM)\n2. Brief summary (one sentence)\n3. Suggested action\n4. Draft response (if needed)",
            system,
            email.from,
            email.to.as_deref().unwrap_or("me"),
            email.subject,
            email.body
        )
    }
    
    /// Parse the AI response into structured format
    fn parse_email_response(&self, response: &str, original: &Email) -> EmailResponse {
        // Simple parsing - in production would be more robust
        let category = if response.contains("URGENT") {
            EmailCategory::Urgent
        } else if response.contains("HIGH") {
            EmailCategory::High
        } else if response.contains("LOW") {
            EmailCategory::Low
        } else if response.contains("SPAM") {
            EmailCategory::Spam
        } else {
            EmailCategory::Normal
        };
        
        EmailResponse {
            email_id: original.id.clone(),
            category,
            summary: response.lines().next().unwrap_or("").to_string(),
            action: response.to_string(),
            draft_response: None,
            processed_at: chrono::Utc::now(),
        }
    }
    
    /// Run the actual inference
    fn run_inference(&self, prompt: &str) -> anyhow::Result<String> {
        let inference_guard = self.inference.read();
        if let Some(ref engine) = *inference_guard {
            engine.generate(prompt, 500)
        } else {
            // No inference engine - return mock response
            warn!("No inference engine set, returning mock response");
            Ok(self.mock_response(prompt))
        }
    }
    
    /// Generate a mock response when no inference engine is available
    fn mock_response(&self, task: &str) -> String {
        if task.contains("Email") || task.contains("email") {
            let categories = ["URGENT", "HIGH", "NORMAL", "LOW"];
            let category = categories[self.id.hash(&std::collections::hash_map::DefaultHasher::new()) as usize % categories.len()];
            format!(
                "Category: {}\nSummary: This email requires your attention.\nAction: Review and respond within 24 hours.\nDraft: Thank you for your email. I'll get back to you shortly.",
                category
            )
        } else if task.starts_with("reason:") {
            format!("[Cattle Analysis]\nAnalyzed: {}\nConclusion: Deep reasoning complete.", 
                task.trim_start_matches("reason:"))
        } else if task.starts_with("generate_code:") {
            format!("[Cattle Code Gen]\n// Generated for: {}\nfn generated_function() {{\n    // Implementation\n}}", 
                task.trim_start_matches("generate_code:"))
        } else {
            format!("[Cattle Result]\nProcessed: {}", task)
        }
    }
}

#[async_trait]
impl Species for Cattle {
    fn species_type(&self) -> SpeciesType {
        SpeciesType::Cattle
    }
    
    fn id(&self) -> &str {
        &self.id
    }
    
    fn fitness(&self) -> f32 {
        self.fitness
    }
    
    fn set_fitness(&mut self, score: f32) {
        self.fitness = score.clamp(0.0, 1.0);
    }
    
    fn generation(&self) -> u32 {
        self.generation
    }
    
    fn adapter_path(&self) -> Option<&str> {
        self.adapter_path.as_deref()
    }
    
    async fn execute(&self, task: String) -> anyhow::Result<String> {
        let start = Instant::now();
        info!("🐄 Cattle '{}' beginning heavy computation...", self.id);
        debug!("Task: {}", task);
        
        // Run inference
        let result = self.run_inference(&task)?;
        
        let elapsed = start.elapsed();
        info!("🐄 Cattle '{}' completed in {:?}", self.id, elapsed);
        
        Ok(result)
    }
}

impl SpeciesOps for Cattle {}

// Implement std::fmt::Debug manually to handle the Arc<RwLock>
impl std::fmt::Debug for Cattle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cattle")
            .field("id", &self.id)
            .field("fitness", &self.fitness)
            .field("generation", &self.generation)
            .field("adapter_path", &self.adapter_path)
            .field("stats", &self.stats)
            .finish()
    }
}

/// Email structure for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub from: String,
    pub to: Option<String>,
    pub subject: String,
    pub body: String,
    pub received_at: chrono::DateTime<chrono::Utc>,
}

impl Email {
    /// Create a demo email for testing
    pub fn demo() -> Self {
        Self {
            id: "demo-001".to_string(),
            from: "boss@company.com".to_string(),
            to: Some("me@company.com".to_string()),
            subject: "Q4 Report Review Needed".to_string(),
            body: "Hi,\n\nPlease review the attached Q4 report before our meeting tomorrow.\n\nBest,\nBoss".to_string(),
            received_at: chrono::Utc::now(),
        }
    }
}

/// Email category for triage
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EmailCategory {
    Urgent,
    High,
    Normal,
    Low,
    Spam,
}

/// Response to an email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailResponse {
    pub email_id: String,
    pub category: EmailCategory,
    pub summary: String,
    pub action: String,
    pub draft_response: Option<String>,
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

/// Cattle Agent - A managed Cattle instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CattleAgent {
    pub id: String,
    pub fitness: f32,
    pub generation: u32,
    pub adapter_id: String,
    pub speciality: CattleSpeciality,
}

/// Speciality areas for Cattle
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CattleSpeciality {
    CodeGeneration,
    Reasoning,
    Analysis,
    Synthesis,
    EmailTriage,
    General,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cattle_execute() {
        let cattle = Cattle::new("test-01".to_string());
        let result = cattle.execute("test task".to_string()).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_cattle_reason() {
        let mut cattle = Cattle::new("test-01".to_string());
        let result = cattle.reason("Why is the sky blue?").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_cattle_process_email() {
        let mut cattle = Cattle::new("email-cow-01".to_string());
        let email = Email::demo();
        let response = cattle.process_email(&email).await;
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.email_id, "demo-001");
    }
    
    #[tokio::test]
    async fn test_cattle_with_inference() {
        let inference = Arc::new(InferenceEngine::demo());
        let cattle = Cattle::with_inference("test-01".to_string(), inference);
        let result = cattle.execute("Hello world".to_string()).await;
        assert!(result.is_ok());
    }
}
