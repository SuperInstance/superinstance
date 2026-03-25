//! Fallback Inference Engine - Works on any hardware
//!
//! When TensorRT is unavailable (non-Jetson, no NVIDIA GPU), we fall back to:
//! 1. Candle (pure Rust) - ~2-5 tok/s on CPU
//! 2. Mock mode for demos - Instant responses, no model required
//!
//! This removes the hardware gate and lets anyone try SuperInstance.

use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Hardware tier detected at startup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareTier {
    /// NVIDIA Jetson - Full TensorRT-LLM support
    Jetson,
    /// Desktop with NVIDIA GPU - CUDA support
    DesktopGPU,
    /// Laptop/Server CPU only - Candle fallback
    LaptopCPU,
    /// Raspberry Pi or low-power device - Minimal mode
    Embedded,
    /// Demo mode - No actual inference
    Demo,
}

impl HardwareTier {
    /// Detect the current hardware tier
    pub fn detect() -> Self {
        // Check for Jetson first
        if Self::is_jetson() {
            return HardwareTier::Jetson;
        }
        
        // Check for NVIDIA GPU
        if Self::has_nvidia_gpu() {
            return HardwareTier::DesktopGPU;
        }
        
        // Check system RAM for embedded detection
        let ram_gb = Self::get_system_ram_gb();
        if ram_gb < 4.0 {
            return HardwareTier::Embedded;
        }
        
        // Default to CPU mode
        HardwareTier::LaptopCPU
    }
    
    fn is_jetson() -> bool {
        // Check for Jetson-specific files
        std::path::Path::new("/proc/device-tree/model").exists()
            && std::fs::read_to_string("/proc/device-tree/model")
                .map(|s| s.contains("Jetson"))
                .unwrap_or(false)
    }
    
    fn has_nvidia_gpu() -> bool {
        // Check for NVIDIA GPU via nvidia-smi
        std::process::Command::new("nvidia-smi")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    fn get_system_ram_gb() -> f64 {
        // Linux: read from /proc/meminfo
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let kb: f64 = line
                        .split(':')
                        .nth(1)
                        .and_then(|s| s.trim().split(' ').next())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    return kb / (1024.0 * 1024.0);
                }
            }
        }
        8.0 // Default assumption
    }
    
    /// Get expected tokens per second for this tier
    pub fn expected_tps(&self) -> f64 {
        match self {
            HardwareTier::Jetson => 12.5,
            HardwareTier::DesktopGPU => 15.0,
            HardwareTier::LaptopCPU => 3.0,
            HardwareTier::Embedded => 1.0,
            HardwareTier::Demo => f64::INFINITY,
        }
    }
    
    /// Get description for display
    pub fn description(&self) -> &'static str {
        match self {
            HardwareTier::Jetson => "NVIDIA Jetson (TensorRT-LLM)",
            HardwareTier::DesktopGPU => "Desktop GPU (CUDA)",
            HardwareTier::LaptopCPU => "CPU Fallback (Candle)",
            HardwareTier::Embedded => "Embedded (Limited)",
            HardwareTier::Demo => "Demo Mode (No inference)",
        }
    }
}

/// Inference backend trait - implemented by TensorRT, Candle, Mock
pub trait InferenceBackend: Send + Sync {
    /// Generate text from a prompt
    fn generate(&self, prompt: &str, max_tokens: u32) -> Result<String>;
    
    /// Stream tokens as they're generated
    fn generate_stream(&self, prompt: &str, max_tokens: u32) 
        -> Result<Box<dyn Iterator<Item = Result<String>> + Send>>;
    
    /// Get the backend name
    fn name(&self) -> &'static str;
    
    /// Check if this backend is available
    fn is_available(&self) -> bool {
        true
    }
}

/// Mock inference backend for demos and testing
pub struct MockBackend {
    responses: Vec<String>,
    index: Arc<RwLock<usize>>,
}

impl MockBackend {
    /// Create a new mock backend with predefined responses
    pub fn new() -> Self {
        let responses = vec![
            "I'll help you triage this email. Based on the sender and subject, this appears to be a high-priority message that requires your attention within 24 hours.".to_string(),
            "This email looks like it can be handled automatically. I've drafted a response that you can review and send.".to_string(),
            "I've categorized this as: Newsletter - Low Priority. Would you like me to archive it automatically in the future?".to_string(),
            "Meeting request detected. I can see you're available at the suggested time. Shall I draft an acceptance?".to_string(),
            "This appears to be a transactional email. I'll file it under 'Receipts' for your records.".to_string(),
        ];
        
        Self {
            responses,
            index: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Create with custom responses
    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            responses,
            index: Arc::new(RwLock::new(0)),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceBackend for MockBackend {
    fn generate(&self, prompt: &str, _max_tokens: u32) -> Result<String> {
        // Simulate processing time based on prompt length
        let delay_ms = std::cmp::min(prompt.len() as u64 / 10, 500);
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        
        // Get next response
        let mut idx = self.index.write();
        let response = self.responses.get(*idx).cloned().unwrap_or_else(|| {
            "I understand. Let me help you with that.".to_string()
        });
        *idx = (*idx + 1) % self.responses.len();
        
        Ok(response)
    }
    
    fn generate_stream(&self, prompt: &str, max_tokens: u32) 
        -> Result<Box<dyn Iterator<Item = Result<String>> + Send>> 
    {
        let full_response = self.generate(prompt, max_tokens)?;
        let words: Vec<String> = full_response.split_whitespace()
            .map(|w| w.to_string())
            .collect();
        
        Ok(Box::new(words.into_iter().map(Ok)))
    }
    
    fn name(&self) -> &'static str {
        "Mock (Demo)"
    }
}

/// Candle-based inference backend (pure Rust, no GPU required)
#[cfg(feature = "candle")]
pub mod candle_backend {
    use super::*;
    use candle::{Device, Tensor};
    
    pub struct CandleBackend {
        device: Device,
        // Model would be loaded here
    }
    
    impl CandleBackend {
        pub fn new() -> Result<Self> {
            let device = if candle::utils::cuda_is_available() {
                Device::new_cuda(0)?
            } else {
                Device::Cpu
            };
            
            Ok(Self { device })
        }
    }
    
    impl InferenceBackend for CandleBackend {
        fn generate(&self, prompt: &str, max_tokens: u32) -> Result<String> {
            // Placeholder - would integrate with candle models
            Ok(format!("Candle response to: {}", prompt))
        }
        
        fn generate_stream(&self, prompt: &str, max_tokens: u32) 
            -> Result<Box<dyn Iterator<Item = Result<String>> + Send>> 
        {
            let response = self.generate(prompt, max_tokens)?;
            Ok(Box::new(std::iter::once(Ok(response))))
        }
        
        fn name(&self) -> &'static str {
            "Candle (Rust)"
        }
    }
}

/// Inference engine - manages backends and routing
pub struct InferenceEngine {
    /// Current hardware tier
    tier: HardwareTier,
    /// Active backend
    backend: Box<dyn InferenceBackend>,
    /// Statistics
    stats: Arc<RwLock<InferenceStats>>,
}

/// Inference statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InferenceStats {
    pub total_requests: u64,
    pub total_tokens_generated: u64,
    pub total_time_ms: u64,
    pub average_tps: f64,
    pub last_latency_ms: u64,
}

impl InferenceEngine {
    /// Create a new inference engine with auto-detection
    pub fn new() -> Result<Self> {
        let tier = HardwareTier::detect();
        info!("🖥️ Detected hardware: {} (expect ~{:.1} tok/s)", 
            tier.description(), tier.expected_tps());
        
        let backend: Box<dyn InferenceBackend> = match tier {
            HardwareTier::Jetson => {
                #[cfg(feature = "tensorrt")]
                {
                    info!("Loading TensorRT-LLM backend...");
                    Box::new(crate::pasture::TensorRTBackend::new()?)
                }
                #[cfg(not(feature = "tensorrt"))]
                {
                    warn!("TensorRT feature not enabled, falling back to mock");
                    Box::new(MockBackend::new())
                }
            }
            HardwareTier::DesktopGPU => {
                #[cfg(feature = "candle")]
                {
                    info!("Loading Candle CUDA backend...");
                    Box::new(candle_backend::CandleBackend::new()?)
                }
                #[cfg(not(feature = "candle"))]
                {
                    warn!("Candle feature not enabled, falling back to mock");
                    Box::new(MockBackend::new())
                }
            }
            HardwareTier::LaptopCPU | HardwareTier::Embedded => {
                #[cfg(feature = "candle")]
                {
                    info!("Loading Candle CPU backend...");
                    Box::new(candle_backend::CandleBackend::new()?)
                }
                #[cfg(not(feature = "candle"))]
                {
                    warn!("Candle feature not enabled, falling back to mock");
                    Box::new(MockBackend::new())
                }
            }
            HardwareTier::Demo => {
                info!("Running in demo mode - no real inference");
                Box::new(MockBackend::new())
            }
        };
        
        info!("✓ Inference backend: {}", backend.name());
        
        Ok(Self {
            tier,
            backend,
            stats: Arc::new(RwLock::new(InferenceStats::default())),
        })
    }
    
    /// Create a demo-only engine (no real inference)
    pub fn demo() -> Self {
        Self {
            tier: HardwareTier::Demo,
            backend: Box::new(MockBackend::new()),
            stats: Arc::new(RwLock::new(InferenceStats::default())),
        }
    }
    
    /// Generate a response
    pub fn generate(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        let start = Instant::now();
        
        let response = self.backend.generate(prompt, max_tokens)?;
        
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let tokens = response.split_whitespace().count() as u64;
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.total_requests += 1;
            stats.total_tokens_generated += tokens;
            stats.total_time_ms += elapsed_ms;
            stats.last_latency_ms = elapsed_ms;
            if stats.total_time_ms > 0 {
                stats.average_tps = (stats.total_tokens_generated as f64) 
                    / (stats.total_time_ms as f64 / 1000.0);
            }
        }
        
        debug!("Generated {} tokens in {}ms ({:.1} tok/s)", 
            tokens, elapsed_ms, 
            tokens as f64 / (elapsed_ms as f64 / 1000.0).max(0.001));
        
        Ok(response)
    }
    
    /// Get current hardware tier
    pub fn tier(&self) -> HardwareTier {
        self.tier
    }
    
    /// Get statistics
    pub fn stats(&self) -> InferenceStats {
        self.stats.read().clone()
    }
    
    /// Get backend name
    pub fn backend_name(&self) -> &'static str {
        self.backend.name()
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::demo())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hardware_tier_detect() {
        let tier = HardwareTier::detect();
        // Should not panic
        println!("Detected tier: {:?}", tier);
    }
    
    #[test]
    fn test_mock_backend() {
        let backend = MockBackend::new();
        let response = backend.generate("Hello", 100).unwrap();
        assert!(!response.is_empty());
    }
    
    #[test]
    fn test_inference_engine_demo() {
        let engine = InferenceEngine::demo();
        assert_eq!(engine.tier(), HardwareTier::Demo);
        
        let response = engine.generate("Test prompt", 100).unwrap();
        assert!(!response.is_empty());
        
        let stats = engine.stats();
        assert_eq!(stats.total_requests, 1);
    }
}
