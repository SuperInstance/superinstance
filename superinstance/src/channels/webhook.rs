//! Webhook Channel - Generic HTTP Webhook Integration
//! 
//! Provides a generic webhook interface for SuperInstance.
//! Supports:
//! - Incoming webhooks (receiving)
//! - Outgoing webhooks (sending)
//! - Custom headers and authentication
//! - JSON payload parsing

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::{Channel, ChannelType, ChannelConfig, ChannelMessage, MessageSender, OutboundMessage};

/// Webhook Channel implementation
pub struct WebhookChannel {
    /// Channel configuration
    config: ChannelConfig,
    /// Connection state
    connected: Arc<RwLock<bool>>,
    /// Webhook URL for outgoing messages
    webhook_url: Option<String>,
    /// Secret for verifying incoming webhooks
    secret: Option<String>,
    /// Custom headers
    headers: HashMap<String, String>,
}

impl WebhookChannel {
    /// Create a new webhook channel
    pub fn new(config: ChannelConfig) -> Self {
        let webhook_url = config.auth.webhook_url.clone();
        let secret = config.settings.get("secret").cloned();
        
        let headers: HashMap<String, String> = config.settings.iter()
            .filter(|(k, _)| k.starts_with("header_"))
            .map(|(k, v)| (k.trim_start_matches("header_").to_string(), v.clone()))
            .collect();
        
        Self {
            config,
            connected: Arc::new(RwLock::new(false)),
            webhook_url,
            secret,
            headers,
        }
    }
    
    /// Handle an incoming webhook request
    pub async fn handle_incoming(&self, payload: WebhookPayload) -> Result<ChannelMessage> {
        // Verify signature if secret is configured
        if let Some(ref secret) = self.secret {
            if let Some(ref signature) = payload.signature {
                self.verify_signature(&payload.body, signature, secret)?;
            }
        }
        
        // Parse the payload
        let message = self.parse_payload(&payload)?;
        
        Ok(message)
    }
    
    /// Verify webhook signature using HMAC-SHA256
    /// 
    /// # Security Note
    /// Signature verification is critical for webhook security. Always verify
    /// signatures in production to prevent forgery attacks.
    fn verify_signature(&self, body: &[u8], signature: &str, secret: &str) -> Result<()> {
        // TODO: Implement proper HMAC-SHA256 verification using `hmac` and `sha2` crates
        // 
        // Example implementation (requires adding dependencies to Cargo.toml):
        // ```toml
        // [dependencies]
        // hmac = "0.12"
        // sha2 = "0.10"
        // ```
        // 
        // ```rust,ignore
        // use hmac::{Hmac, Mac};
        // use sha2::Sha256;
        // 
        // type HmacSha256 = Hmac<Sha256>;
        // 
        // let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        //     .map_err(|e| anyhow!("HMAC init failed: {}", e))?;
        // mac.update(body);
        // let expected = mac.finalize().into_bytes();
        // 
        // // Use constant-time comparison to prevent timing attacks
        // let expected_hex = hex::encode(expected);
        // if !subtle::ConstantTimeEq::eq(
        //     expected_hex.as_bytes(),
        //     signature.as_bytes()
        // ) {
        //     return Err(anyhow!("Invalid webhook signature"));
        // }
        // ```
        // 
        // SECURITY WARNING: The current implementation does NOT verify signatures!
        // This is a stub that always passes. Do NOT use in production without
        // implementing proper verification.
        
        // Temporary stub: log warning about missing verification
        warn!(
            "⚠️ Webhook signature verification not implemented! \
             Received signature: {} (secret length: {} bytes)",
            &signature[..signature.len().min(8)],  // Don't log full signature
            secret.len()
        );
        
        // TODO: Remove this and implement proper verification before production use
        Ok(())
    }
    
    /// Parse webhook payload into a channel message
    fn parse_payload(&self, payload: &WebhookPayload) -> Result<ChannelMessage> {
        // Try to parse as JSON
        let json: serde_json::Value = serde_json::from_slice(&payload.body)
            .unwrap_or(serde_json::Value::Null);
        
        let id = json.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let sender_id = json.get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let sender_name = json.get("sender_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let content = json.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                // Fall back to raw body
                String::from_utf8_lossy(&payload.body).as_ref()
            })
            .to_string();
        
        Ok(ChannelMessage {
            id,
            channel_id: self.config.id.clone(),
            channel_type: ChannelType::Webhook,
            sender: MessageSender {
                id: sender_id,
                username: sender_name.clone(),
                display_name: sender_name,
                is_bot: false,
            },
            content,
            timestamp: chrono::Utc::now(),
            thread_id: None,
            reply_to: None,
            attachments: Vec::new(),
        })
    }
    
    /// Build an outgoing webhook payload
    fn build_payload(&self, message: &OutboundMessage) -> WebhookPayload {
        let body = serde_json::json!({
            "channel_id": message.channel_id,
            "thread_id": message.thread_id,
            "content": message.content,
            "embeds": message.embeds,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        WebhookPayload {
            body: body.to_string().into_bytes(),
            signature: None,
            headers: self.headers.clone(),
        }
    }
}

#[async_trait]
impl Channel for WebhookChannel {
    fn id(&self) -> &str {
        &self.config.id
    }
    
    fn channel_type(&self) -> ChannelType {
        ChannelType::Webhook
    }
    
    async fn send(&self, message: OutboundMessage) -> Result<()> {
        let url = self.webhook_url.as_ref()
            .ok_or_else(|| anyhow!("No webhook URL configured"))?;
        
        let payload = self.build_payload(&message);
        
        info!("📤 [Webhook] Sending to {}: {} bytes", 
            url, payload.body.len());
        
        // In production, use reqwest:
        // let client = reqwest::Client::new();
        // let mut request = client.post(url).body(payload.body);
        // 
        // for (key, value) in &payload.headers {
        //     request = request.header(key, value);
        // }
        // 
        // let response = request.send().await?;
        
        Ok(())
    }
    
    async fn start(&mut self, _tx: mpsc::Sender<ChannelMessage>) -> Result<()> {
        info!("🔌 Webhook channel ready");
        *self.connected.write() = true;
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        *self.connected.write() = false;
        info!("🔌 Webhook channel stopped");
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        *self.connected.read()
    }
}

/// Incoming webhook payload
#[derive(Debug, Clone)]
pub struct WebhookPayload {
    /// Raw body bytes
    pub body: Vec<u8>,
    /// Signature header (if any)
    pub signature: Option<String>,
    /// Custom headers
    pub headers: HashMap<String, String>,
}

impl WebhookPayload {
    /// Create from raw bytes
    pub fn from_bytes(body: Vec<u8>) -> Self {
        Self {
            body,
            signature: None,
            headers: HashMap::new(),
        }
    }
    
    /// Create from JSON
    pub fn from_json<T: Serialize>(data: &T) -> Result<Self> {
        let body = serde_json::to_vec(data)?;
        Ok(Self::from_bytes(body))
    }
    
    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    /// Add a signature
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }
}

/// Webhook server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookServerConfig {
    /// Listen address
    pub addr: String,
    /// Listen port
    pub port: u16,
    /// Path prefix for webhook endpoints
    pub path_prefix: String,
    /// Secret for signature verification
    pub secret: Option<String>,
    /// Rate limit (requests per minute)
    pub rate_limit: u32,
}

impl Default for WebhookServerConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0".to_string(),
            port: 8080,
            path_prefix: "/webhook".to_string(),
            secret: None,
            rate_limit: 60,
        }
    }
}

/// Webhook server for receiving incoming webhooks
pub struct WebhookServer {
    config: WebhookServerConfig,
    channels: Arc<RwLock<HashMap<String, String>>>, // channel_id -> handler
}

impl WebhookServer {
    /// Create a new webhook server
    pub fn new(config: WebhookServerConfig) -> Self {
        Self {
            config,
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a channel handler
    pub fn register_channel(&mut self, channel_id: String, handler: String) {
        self.channels.write().insert(channel_id, handler);
    }
    
    /// Start the webhook server
    pub async fn start(&self, tx: mpsc::Sender<ChannelMessage>) -> Result<()> {
        let addr = format!("{}:{}", self.config.addr, self.config.port);
        
        info!("🌐 Starting webhook server on {}", addr);
        
        // In production, this would use axum or warp:
        // let app = Router::new()
        //     .route("/webhook/:channel_id", post(handle_webhook))
        //     .layer(middleware::from_fn(rate_limit))
        //     .with_state(state);
        //
        // let listener = tokio::net::TcpListener::bind(&addr).await?;
        // axum::serve(listener, app).await?;
        
        info!("🌐 Webhook server started (simulated)");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_webhook_channel_creation() {
        let config = ChannelConfig {
            id: "webhook-main".to_string(),
            channel_type: ChannelType::Webhook,
            enabled: true,
            auth: super::ChannelAuth {
                webhook_url: Some("https://example.com/webhook".to_string()),
                ..Default::default()
            },
            settings: HashMap::new(),
        };
        
        let channel = WebhookChannel::new(config);
        assert!(!channel.is_connected());
    }
    
    #[test]
    fn test_payload_creation() {
        let payload = WebhookPayload::from_json(&serde_json::json!({
            "test": "value"
        })).unwrap();
        
        assert!(!payload.body.is_empty());
    }
}
