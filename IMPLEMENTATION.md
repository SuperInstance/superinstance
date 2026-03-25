# SuperInstance v1.0 - Phase 1-2 Implementation

## Overview

This implementation adds Phase 1-2 features from ROADMAP-v1.0.md:

### Phase 1: Core (Night School + Discord)
- **night_school.rs**: Tokio cron job running at 02:00 for SLERP breed/cull operations
- **discord.rs**: Serenity bot for Collie route streaming
- **llama_native.rs**: Native llama.cpp bindgen for <1ms reflex responses

### Phase 2: Integration
- **Makefile**: Enhanced `make ranch` command with proper proxy setup
- **pasture_sync.rs**: CRDT (Yjs) for conflict-free pasture synchronization
- **API endpoints**: `/api/sync` and `/api/evolution`

## Key Features

### 🌙 Night School - Autonomous Evolution

Night School runs daily at 02:00 AM to:

1. **Collect metrics** from agent performance database
2. **Identify top performers** (success_rate > 0.75) for breeding
3. **Identify bottom performers** (success_rate < 0.25) for culling
4. **Breed agents** using SLERP interpolation for smooth genetic transitions
5. **Cull underperformers** to maintain herd quality

#### SLERP Interpolation

SLERP (Spherical Linear Interpolation) ensures smooth genetic transitions:

```rust
// Smooth interpolation between parent genes
let omega = (g1.expression / g2.expression).acos();
let child_expr = s1 * g1.expression + s2 * g2.expression;
```

This prevents jarring jumps in gene expression values.

### 🐕 Discord Integration

Collie Discord bot provides:

- **Real-time streaming** responses to Discord messages
- **Collie routing** - routes messages through `/api/collie` endpoint
- **Typing indicators** - shows bot is processing
- **Chunked responses** - handles Discord's 2000 char limit

#### Configuration

```bash
export DISCORD_TOKEN="your-bot-token"
export DISCORD_CHANNEL_ID="1234567890"
export COLLIE_ENDPOINT="http://localhost:3001"
```

### ⚡ Llama.cpp Native Bindgen

Ultra-fast inference with <1ms target for reflex responses:

```rust
// Load model
let llama = LlamaContext::load(&model_path, 4096)?;

// Reflex response (<1ms)
let response = llama.reflex("Quick question?")?;

// Full inference
let result = llama.infer(prompt, &params)?;
```

Features:
- Native FFI bindings to llama.cpp
- Thread-safe context
- Configurable inference parameters
- Performance metrics (tokens/sec, time_ms)

### 🔄 CRDT Pasture Sync

Conflict-free synchronization using Yjs (yrs):

```rust
// Create sync manager
let sync = PastureSync::new()?;

// Update breed name
let update = sync.update_breed_name("collie")?;

// Add gene
let gene_update = sync.add_gene(GeneUpdate {
    name: "speed".to_string(),
    expression: 0.9,
    dominant: true,
    updated_by: "night-school".to_string(),
    timestamp: chrono::Utc::now().timestamp(),
})?;

// Apply remote update
sync.apply_update(remote_update)?;
```

Benefits:
- **No conflicts** - CRDTs merge automatically
- **Real-time sync** - changes propagate immediately
- **Offline support** - sync when reconnected
- **Distributed** - multiple nodes can edit simultaneously

## API Endpoints

### POST /api/chat
Main chat endpoint (Collie → Cattle pipeline)

```bash
curl -X POST http://localhost:3001/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Hello!"}'
```

### POST /api/sync
Sync pasture via CRDT

```bash
curl -X POST http://localhost:3001/api/sync \
  -H "Content-Type: application/json" \
  -d '{
    "pasture_id": "cattle-v1",
    "update": "...",
    "timestamp": 1234567890
  }'
```

### POST /api/evolution
Trigger Night School evolution manually

```bash
curl -X POST http://localhost:3001/api/evolution
```

Response:
```json
{
  "timestamp": "2026-03-25T02:00:00Z",
  "agents_bred": 5,
  "agents_culled": 2,
  "genes_mutated": 15,
  "improvements": [
    "Bred agent-1 + agent-2 -> 3 mutations",
    "Culled underperformer: agent-99"
  ]
}
```

## Running the Ranch

### Start Development Stack

```bash
make ranch
```

This starts:
- Frontend on http://localhost:3000 (Bun/Next.js)
- Backend on http://localhost:3001 (Axum/Rust)

### Enable Night School

```bash
export NIGHT_SCHOOL_ENABLED=true
export DATABASE_URL="sqlite:pasture.db"
make ranch
```

### Enable Discord Bot

```bash
export DISCORD_TOKEN="your-token"
export DISCORD_CHANNEL_ID="1234567890"
make ranch
```

### Test API

```bash
# Run all tests
./test-api.sh

# Test specific endpoint
curl -X POST http://localhost:3001/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Hello"}'
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│              SuperInstance Ranch                 │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────┐      ┌──────────┐                │
│  │  Discord │──────│  Collie  │                │
│  │   Bot    │      │  Route   │                │
│  └──────────┘      └────┬─────┘                │
│                         │                        │
│                    ┌────▼─────┐                 │
│                    │  Cattle  │                 │
│                    │  LLM     │                 │
│                    └────┬─────┘                 │
│                         │                        │
│         ┌───────────────┼───────────────┐      │
│         │               │               │       │
│    ┌────▼────┐    ┌────▼────┐    ┌────▼────┐ │
│    │  Night  │    │ Pasture │    │  Llama  │ │
│    │ School  │    │  Sync   │    │ Native  │ │
│    └─────────┘    └─────────┘    └─────────┘ │
│         │               │                      │
│         └───────────────┼───────────────┘      │
│                         │                       │
│                    ┌────▼─────┐                │
│                    │ Database │                │
│                    │ (SQLite) │                │
│                    └──────────┘                │
└─────────────────────────────────────────────────┘
```

## Performance Targets

- **Reflex response**: <1ms first token
- **Chat completion**: 20+ tokens/sec (Jetson Orin Nano)
- **Night School cycle**: <5 minutes
- **CRDT sync**: <10ms for merge

## Next Steps

Phase 3-4 from ROADMAP-v1.0.md:

1. **Jetson Production**
   - install_jetson.sh optimization
   - TensorRT integration
   - MAXN performance mode

2. **Testing & Release**
   - CI workflows (size/test)
   - Documentation
   - Screenshots
   - Announcement thread

## Dependencies

### Rust (Cargo.toml)
- `tokio` - Async runtime
- `axum` - Web framework
- `serenity` - Discord bot
- `yrs` - CRDT (Yjs)
- `sqlx` - Database
- `chrono` - Date/time

### System
- `llama.cpp` - Native LLM inference
- `libssl` - TLS support
- `sqlite3` - Database

## License

MIT
