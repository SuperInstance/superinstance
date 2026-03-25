//! Discord Module - Serenity bot for Collie route streaming
//! Enables Discord integration with real-time streaming responses

use anyhow::Result;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        id::ChannelId,
    },
    prelude::*,
    builder::CreateMessage,
    http::Http,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{info, warn, error};

// ============================================================================
// Types
// ============================================================================

/// Discord bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub channel_id: u64,
    pub collie_endpoint: String,
    pub stream_responses: bool,
}

/// Stream chunk for real-time responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub done: bool,
}

/// Collie chat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollieRequest {
    pub message: String,
    pub breed: Option<String>,
    pub stream: bool,
}

/// Collie chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollieResponse {
    pub response: String,
    pub model: String,
    pub breed: Option<String>,
}

// ============================================================================
// Discord Bot Handler
// ============================================================================

/// Collie Discord Bot
pub struct CollieBot {
    config: DiscordConfig,
    http_client: reqwest::Client,
    stream_tx: Option<mpsc::Sender<StreamChunk>>,
}

impl CollieBot {
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
            stream_tx: None,
        }
    }

    /// Route message to Collie endpoint
    async fn route_to_collie(&self, message: &str) -> Result<String> {
        info!(" Routing to Collie: {:?}", message.chars().take(50).collect::<String>());

        let request = CollieRequest {
            message: message.to_string(),
            breed: Some("collie".to_string()),
            stream: false,
        };

        let response = self
            .http_client
            .post(&format!("{}/api/collie", self.config.collie_endpoint))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Collie endpoint returned: {}", response.status());
        }

        let collie_response: CollieResponse = response.json().await?;
        
        Ok(collie_response.response)
    }

    /// Stream response to Discord channel
    async fn stream_response(&self, ctx: &Context, msg: &Message, response: String) -> Result<()> {
        // Split long responses into chunks (Discord has 2000 char limit)
        let chunks = self.chunk_response(&response, 1900);

        for chunk in chunks {
            msg.channel_id.say(&ctx.http, &chunk).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Split response into Discord-compatible chunks
    fn chunk_response(&self, response: &str, max_size: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current = String::new();

        for line in response.lines() {
            if current.len() + line.len() + 1 > max_size {
                if !current.is_empty() {
                    chunks.push(current.clone());
                    current.clear();
                }
            }
            current.push_str(line);
            current.push('\n');
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }
}

#[async_trait]
impl EventHandler for CollieBot {
    /// Handle incoming Discord messages
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot messages
        if msg.author.bot {
            return;
        }

        // Only respond in configured channel or when mentioned
        let should_respond = msg.channel_id.0 == self.config.channel_id
            || msg.mentions.iter().any(|u| u.bot);

        if !should_respond {
            return;
        }

        info!("Received message: {:?}", msg.content);

        // Show typing indicator
        if let Err(e) = msg.channel_id.broadcast_typing(&ctx.http).await {
            warn!("Failed to show typing: {}", e);
        }

        // Route to Collie endpoint
        match self.route_to_collie(&msg.content).await {
            Ok(response) => {
                if let Err(e) = self.stream_response(&ctx, &msg, response).await {
                    error!("Failed to send response: {}", e);
                    let _ = msg.channel_id.say(&ctx.http, "❌ Error sending response").await;
                }
            }
            Err(e) => {
                error!("Collie routing failed: {}", e);
                let _ = msg.channel_id.say(&ctx.http, &format!("❌ Collie error: {}", e)).await;
            }
        }
    }

    /// Bot ready event
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("🐕 Collie Discord Bot ready: {}", ready.user.name);
    }
}

// ============================================================================
// Discord Integration
// ============================================================================

/// Start Discord bot
pub async fn start_discord_bot(config: DiscordConfig) -> Result<()> {
    info!("🚀 Starting Collie Discord bot...");

    let bot = CollieBot::new(config.clone());
    
    let mut client = Client::builder(&config.token, GatewayIntents::all())
        .event_handler(bot)
        .await?;

    info!("🐕 Discord bot connecting...");

    if let Err(e) = client.start().await {
        error!("Discord client error: {}", e);
        return Err(e.into());
    }

    Ok(())
}

/// Send message to Discord channel (for notifications)
pub async fn send_discord_notification(
    token: &str,
    channel_id: u64,
    message: &str,
) -> Result<()> {
    let http = Http::new(token);
    let channel = ChannelId::from(channel_id);

    channel.say(&http, message).await?;

    Ok(())
}

// ============================================================================
// Stream Support (for real-time responses)
// ============================================================================

/// Stream handler for real-time Collie responses
pub struct StreamHandler {
    tx: mpsc::Sender<StreamChunk>,
}

impl StreamHandler {
    pub fn new() -> (Self, mpsc::Receiver<StreamChunk>) {
        let (tx, rx) = mpsc::channel(100);
        (Self { tx }, rx)
    }

    /// Send a stream chunk
    pub async fn send_chunk(&self, delta: String, done: bool) -> Result<()> {
        self.tx.send(StreamChunk { delta, done }).await?;
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_response() {
        let bot = CollieBot::new(DiscordConfig {
            token: "test".to_string(),
            channel_id: 0,
            collie_endpoint: "http://localhost".to_string(),
            stream_responses: true,
        });

        let response = "Line 1\nLine 2\nLine 3\n";
        let chunks = bot.chunk_response(response, 10);

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.len() <= 10);
        }
    }

    #[tokio::test]
    async fn test_stream_handler() {
        let (handler, mut rx) = StreamHandler::new();

        handler.send_chunk("Hello ".to_string(), false).await.unwrap();
        handler.send_chunk("world!".to_string(), true).await.unwrap();

        let chunk1 = rx.recv().await.unwrap();
        assert_eq!(chunk1.delta, "Hello ");
        assert!(!chunk1.done);

        let chunk2 = rx.recv().await.unwrap();
        assert_eq!(chunk2.delta, "world!");
        assert!(chunk2.done);
    }
}
