# Backend Integrations

```
   ╭──────────────────────────────────────────────────────────────╮
   │                    BACKEND PHILOSOPHY                        │
   │                                                              │
   │    "All heavy lifting lives here;                            │
   │     the core binary stays tiny forever."                     │
   │                                                              │
   │   ┌─────────────────────────────────────────────────────┐    │
   │   │                 CORE (4.2 MB)                       │    │
   │   │   • Border Collie orchestrator                      │    │
   │   │   • breed.md parser                                 │    │
   │   │   • LoRA hot-swap engine                            │    │
   │   │   • CRDT memory primitives                          │    │
   │   └─────────────────────────────────────────────────────┘    │
   │                         │                                    │
   │                         ▼                                    │
   │   ┌─────────────────────────────────────────────────────┐    │
   │   │              BACKEND (optional deps)                │    │
   │   │                                                      │    │
   │   │   cudaclaw/      TensorRT-LLM acceleration         │    │
   │   │   constraint-theory/  Geometric routing            │    │
   │   │   smartcrdt/    Persistent CRDT memory             │    │
   │   │   lucineer/     RAG integration                    │    │
   │   │                                                      │    │
   │   │   Each is optional; core works without them.       │    │
   │   └─────────────────────────────────────────────────────┘    │
   │                                                              │
   ╰──────────────────────────────────────────────────────────────╯
```

## Architecture Principle

**The core binary never exceeds 5 MB.** All capabilities beyond basic orchestration are:
1. Loaded dynamically from `breed.md` files
2. Hot-swapped via LoRA adapters
3. Optionally compiled from backend/ crates

This ensures:
- Fast startup (<100ms)
- Low memory footprint
- Infinite extensibility without bloat
- User-editable abilities (no recompilation needed)

## Integration Crates

### cudaclaw (TensorRT-LLM)
- **Purpose**: 2× faster inference via TensorRT
- **Size**: ~2 MB compiled
- **Optional**: Core falls back to CPU inference
- **Use case**: Production deployments on Jetson

### constraint-theory
- **Purpose**: Geometric determinism for routing
- **Size**: ~500 KB compiled
- **Optional**: Core uses simple rule-based routing
- **Use case**: Complex multi-agent workflows

### smartcrdt
- **Purpose**: Persistent CRDT memory
- **Size**: ~300 KB compiled
- **Optional**: Core uses in-memory state
- **Use case**: Agents that need to remember across reboots

### lucineer
- **Purpose**: RAG over Memory Pasture
- **Size**: ~400 KB compiled
- **Optional**: Core has no RAG capability
- **Use case**: Contextual agents with long memory

## Cargo.toml Structure

```toml
[package]
name = "superinstance"
version = "0.1.0"

[dependencies]
# Core dependencies only (keeps binary small)
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }
pulldown-cmark = "0.9"

[features]
default = []
tensorrt = ["cudaclaw"]
geometric = ["constraint-theory"]
persistent = ["smartcrdt"]
rag = ["lucineer"]
full = ["tensorrt", "geometric", "persistent", "rag"]

[dependencies.cudaclaw]
path = "backend/cudaclaw"
optional = true

[dependencies.constraint-theory]
path = "backend/constraint-theory"
optional = true

[dependencies.smartcrdt]
path = "backend/smartcrdt"
optional = true

[dependencies.lucineer]
path = "backend/lucineer"
optional = true
```

## Build Commands

```bash
# Minimal core (4.2 MB)
cargo build --release

# With TensorRT acceleration (6.3 MB)
cargo build --release --features tensorrt

# Full featured (7.5 MB, still small!)
cargo build --release --features full
```

## Adding New Capabilities

**Never modify the core.** Instead:

1. Create a new `breed.md` in `pasture/`
2. Add LoRA adapters to `genetics/traits/`
3. Optionally, create a backend crate for heavy lifting

The core binary stays the same size forever.
