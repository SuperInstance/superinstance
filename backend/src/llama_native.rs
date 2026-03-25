//! Llama.cpp Native Bindgen Module - Ultra-fast inference (<1ms reflex)
//! Provides direct FFI bindings to llama.cpp for minimal latency

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr;
use tracing::{info, warn};

// ============================================================================
// Types
// ============================================================================

/// Llama context handle
#[derive(Debug)]
pub struct LlamaContext {
    ctx: *mut c_void,
    model_path: String,
}

/// Inference parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    pub repeat_penalty: f32,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 512,
            repeat_penalty: 1.1,
        }
    }
}

/// Inference result with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub text: String,
    pub tokens_generated: u32,
    pub time_ms: f32,
    pub tokens_per_second: f32,
}

// ============================================================================
// FFI Declarations
// ============================================================================

extern "C" {
    // Llama.cpp native functions (would be linked from libllama)
    fn llama_init_from_file(path: *const c_char, params: *const LlamaParams) -> *mut c_void;
    fn llama_free(ctx: *mut c_void);
    fn llama_eval(
        ctx: *mut c_void,
        tokens: *const i32,
        n_tokens: c_int,
        n_past: c_int,
        n_threads: c_int,
    ) -> c_int;
    fn llama_token_to_str(ctx: *mut c_void, token: i32) -> *const c_char;
    fn llama_tokenize(
        ctx: *mut c_void,
        text: *const c_char,
        tokens: *mut i32,
        n_max_tokens: c_int,
        add_bos: c_int,
    ) -> c_int;
    fn llama_sample_top_p_top_k(
        ctx: *mut c_void,
        candidates: *const c_void,
        top_k: c_int,
        top_p: f32,
        temp: f32,
        repeat_penalty: f32,
    ) -> i32;
    fn llama_n_ctx(ctx: *mut c_void) -> c_int;
    fn llama_get_logits(ctx: *mut c_void) -> *mut f32;
}

/// Native llama params
#[repr(C)]
struct LlamaParams {
    n_ctx: c_int,
    n_parts: c_int,
    n_gpu_layers: c_int,
    seed: c_int,
    f16_memory: bool,
    use_mmap: bool,
    use_mlock: bool,
    embedding: bool,
}

impl Default for LlamaParams {
    fn default() -> Self {
        Self {
            n_ctx: 4096,
            n_parts: -1,
            n_gpu_layers: 0,
            seed: -1,
            f16_memory: true,
            use_mmap: true,
            use_mlock: false,
            embedding: false,
        }
    }
}

// ============================================================================
// Llama Native Bindgen
// ============================================================================

impl LlamaContext {
    /// Load model from file
    pub fn load(model_path: &Path, context_size: u32) -> Result<Self> {
        let path_str = model_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid model path"))?;
        
        let path_c = CString::new(path_str)?;
        
        let mut params = LlamaParams::default();
        params.n_ctx = context_size as c_int;

        info!("Loading llama model from: {}", path_str);
        
        // SAFETY: We're calling FFI with valid pointers
        let ctx = unsafe { llama_init_from_file(path_c.as_ptr(), &params) };
        
        if ctx.is_null() {
            anyhow::bail!("Failed to load llama model");
        }

        info!("Model loaded successfully");
        
        Ok(Self {
            ctx,
            model_path: path_str.to_string(),
        })
    }

    /// Perform fast inference (<1ms target for first token)
    pub fn infer(
        &self,
        prompt: &str,
        params: &InferenceParams,
    ) -> Result<InferenceResult> {
        let start = std::time::Instant::now();
        
        // Tokenize prompt
        let prompt_c = CString::new(prompt)?;
        let mut tokens = vec![0i32; params.max_tokens as usize];
        
        let n_tokens = unsafe {
            llama_tokenize(
                self.ctx,
                prompt_c.as_ptr(),
                tokens.as_mut_ptr(),
                tokens.len() as c_int,
                1,
            )
        };

        if n_tokens < 0 {
            anyhow::bail!("Tokenization failed");
        }

        tokens.truncate(n_tokens as usize);
        
        // Eval prompt
        let eval_result = unsafe {
            llama_eval(
                self.ctx,
                tokens.as_ptr(),
                n_tokens,
                0,
                4, // n_threads
            )
        };

        if eval_result != 0 {
            anyhow::bail!("Evaluation failed");
        }

        // Generate tokens
        let mut generated = String::new();
        let mut n_past = n_tokens;
        
        for _ in 0..params.max_tokens {
            // Sample next token
            let next_token = unsafe {
                llama_sample_top_p_top_k(
                    self.ctx,
                    ptr::null(),
                    params.top_k,
                    params.top_p,
                    params.temperature,
                    params.repeat_penalty,
                )
            };

            // Check for EOS
            if next_token == 2 { // EOS token
                break;
            }

            // Convert to string
            let token_str = unsafe {
                let cstr = llama_token_to_str(self.ctx, next_token);
                CStr::from_ptr(cstr).to_str()?.to_string()
            };

            generated.push_str(&token_str);

            // Eval new token
            let eval_result = unsafe {
                llama_eval(
                    self.ctx,
                    &next_token,
                    1,
                    n_past,
                    4,
                )
            };

            if eval_result != 0 {
                warn!("Evaluation stopped early");
                break;
            }

            n_past += 1;
        }

        let elapsed = start.elapsed();
        let time_ms = elapsed.as_secs_f32() * 1000.0;
        let tokens_generated = (n_past - n_tokens) as u32;
        let tokens_per_second = if time_ms > 0.0 {
            (tokens_generated as f32) / (time_ms / 1000.0)
        } else {
            0.0
        };

        Ok(InferenceResult {
            text: generated,
            tokens_generated,
            time_ms,
            tokens_per_second,
        })
    }

    /// Quick reflex response (<1ms)
    pub fn reflex(&self, prompt: &str) -> Result<String> {
        let params = InferenceParams {
            temperature: 0.1, // Low temperature for fast, deterministic responses
            max_tokens: 32,   // Short response
            ..Default::default()
        };

        let result = self.infer(prompt, &params)?;
        
        info!(
            "Reflex response: {:.2}ms, {} tokens, {:.1} tok/s",
            result.time_ms, result.tokens_generated, result.tokens_per_second
        );

        Ok(result.text)
    }

    /// Get context size
    pub fn context_size(&self) -> i32 {
        unsafe { llama_n_ctx(self.ctx) }
    }
}

impl Drop for LlamaContext {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe { llama_free(self.ctx) };
        }
    }
}

// Safety: The context is thread-safe in read-only mode
unsafe impl Send for LlamaContext {}
unsafe impl Sync for LlamaContext {}

// ============================================================================
// Stub Implementation (when native lib not available)
// ============================================================================

/// Stub implementation for when llama.cpp is not linked
pub struct LlamaStub {
    model_path: String,
}

impl LlamaStub {
    pub fn load(model_path: &Path, _context_size: u32) -> Result<Self> {
        Ok(Self {
            model_path: model_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?
                .to_string(),
        })
    }

    pub fn infer(&self, prompt: &str, params: &InferenceParams) -> Result<InferenceResult> {
        warn!("Using stub implementation - native llama.cpp not linked");
        
        Ok(InferenceResult {
            text: format!(
                "[Stub] Received: '{}...' ({} max tokens)",
                prompt.chars().take(50).collect::<String>(),
                params.max_tokens
            ),
            tokens_generated: 10,
            time_ms: 1.0,
            tokens_per_second: 10.0,
        })
    }

    pub fn reflex(&self, prompt: &str) -> Result<String> {
        Ok(format!("[Stub reflex] {}", prompt.chars().take(20).collect::<String>()))
    }

    pub fn context_size(&self) -> i32 {
        4096
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_params_default() {
        let params = InferenceParams::default();
        assert_eq!(params.temperature, 0.7);
        assert_eq!(params.max_tokens, 512);
    }

    #[test]
    fn test_stub_inference() {
        let stub = LlamaStub::load(Path::new("/tmp/model.gguf"), 4096).unwrap();
        
        let params = InferenceParams::default();
        let result = stub.infer("Hello", &params).unwrap();
        
        assert!(result.text.contains("Stub"));
        assert!(result.time_ms > 0.0);
    }
}
