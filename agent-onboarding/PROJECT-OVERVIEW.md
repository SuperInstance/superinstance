# Project Overview — SuperInstance / PastureAI

## The Vision

SuperInstance is a self-evolving AI ranch that runs entirely on hardware you own. The core insight: instead of a monolithic AI model that stays the same, you have a *ranch* of specialized agents that continuously improve at your specific tasks through genetic selection — automatically, overnight, without cloud API calls or training runs.

The system is designed around three constraints that make everything else follow:

1. **Single binary, ≤5 MB** — Ships as one Rust binary. `scp` it anywhere. No Docker, no Python, no Node.js.
2. **breed.md as DNA** — Every agent's personality lives in a Markdown file you can read, edit, and version-control. Save it, the agent reloads in milliseconds.
3. **Night School evolves while you sleep** — At 02:00, agents are evaluated, weak ones culled, strong ones bred via SLERP LoRA weight interpolation. You wake up to a smarter ranch.

---

## Architecture Flow

```
Cowboy (User)
     │
     ▼
Intent (what the user wants)
     │
     ▼
Border Collie (Orchestrator)
  ├── Reflex Cache (<1ms hot paths)
  ├── Anticipation Engine (Shadow Pup)
  └── Shepherd (routing strategy per species)
     │
     ▼
Pasture (Resource Pool)
  ├── ModelPool (base model management)
  ├── LoRAManager (adapter hot-swap <50ms)
  └── InferenceEngine (TensorRT / Candle / Mock)
     │
     ▼
Species Agent (the worker)
  ├── Cattle   — email handling, synthesis
  ├── Sheep    — classification, triage (can form flocks)
  ├── Duck     — fetch/retrieve from external sources
  ├── Goat     — file navigation, log analysis
  ├── Hog      — batch/compute-heavy tasks
  ├── Chicken  — monitoring/alerting (always watching)
  └── Horse    — long documents, content generation
     │
     ▼
Response → back to Cowboy

Nightly (02:00):
Species Registry + Stud Book → NightSchool
  ├── Evaluate fitness (task success rate)
  ├── Cull (agents below 0.40 threshold deleted)
  ├── Breed (SLERP merge top performers' LoRA weights)
  ├── Quarantine (new offspring tested for 24h)
  └── Promote (winners to production)
```

---

## Key Technical Decisions

### Why Rust
Single-binary constraint makes Rust the only viable choice. No runtime to ship, no garbage collector pauses, predictable memory layout for VRAM management on the Jetson.

### Why breed.md in Markdown (not TOML or JSON)
Non-technical users can read and edit it. "A farmer doesn't write config files. A farmer writes about their livestock." The pulldown-cmark parser extracts the trait weight table, system prompt, lineage, and phenotype settings from structured Markdown.

### Why SLERP (not LERP) for LoRA merging
LERP (linear interpolation) shrinks the norm of weight vectors when α ≠ 0 or 1. For a unit vector, LERP at α=0.5 produces a vector with norm ~0.71. SLERP travels along the surface of the unit sphere, preserving norm. This produces "halfway between both parents" rather than "weaker than either parent."

**Note:** The SLERP function shown in press materials is the *aspirational* implementation. The current code in `breeding.rs` operates on blend coefficients (HashMap<String, f32>), not directly on tensor weight vectors. The actual tensor-level SLERP needs to be wired when real safetensors loading is implemented.

### Why parking_lot::RwLock over tokio::sync::RwLock
Most state reads happen in synchronous accessor methods (API handlers pulling ranch state). The parking_lot lock avoids async overhead for these paths. The tradeoff is blocking behavior if a write is long-held — acceptable for current usage.

### Why 500ms debounce (not 50ms)
The 50ms claim in press materials appears to be aspirational. The current watcher uses 500ms to handle editors that write files in multiple steps (save → rename pattern). Could be reduced; needs measurement on the actual Jetson target to verify the claimed 180ms total hot-reload time.

---

## The Metaphor System (Important for Consistency)

Every public-facing name uses the ranch metaphor. **Do not break this in code or docs.**

| Technical Term    | Ranch Term        |
|-------------------|-------------------|
| AI agent          | Livestock / Animal|
| Agent config      | breed.md / DNA    |
| Delete agent      | Cull              |
| Merge agents      | Breed             |
| New merged agent  | Offspring         |
| Testing period    | Quarantine        |
| Nightly evolution | Night School      |
| Orchestrator      | Border Collie     |
| User              | Cowboy            |
| Agent pool        | Pasture           |
| Genealogy DB      | Stud Book         |
| Agent fitness     | Fitness score     |
| LoRA weights      | Genes / DNA       |
| System prompt     | Genetic Code      |
| Hardware          | Ranch hardware    |

---

## Repository Split (Important)

The product is split across two notional repos but lives in one actual repo (`pasture-ai`):

- `superinstance/` — The main binary (what ships to users). This is the public-facing product.
- `backend/` — Heavy integrations (TensorRT, CRDT, llama.cpp FFI). Not compiled into the main binary by default; used by specialized builds.
- `src/` — The Next.js dashboard frontend (separate deployment).

The `superinstance/` binary is self-contained. It embeds its own web server (Axum), TUI (ratatui), and everything else. The Next.js frontend is a richer alternative dashboard, not required.

---

## Current Codebase Health

The code represents a **well-scaffolded alpha**. The architecture is sound and the module boundaries are clean. The main gap between press materials and reality is that several modules have correct structure and logging, but their core operations are stubs — they run without errors but don't do the real work yet.

The most important stubs to fill before a genuine v0.2.0:
1. Real walkdir scan on startup (currently a local stub that returns nothing)
2. Real culling in NightSchool (currently logs but doesn't remove agents)
3. Real SLERP on tensor weight vectors (currently on coefficients only)
4. Real candle/safetensors loading (currently `sleep(20ms)`)
5. Real Collie routing (currently `sleep(60s)` loop)
6. Real Discord Serenity event handler (currently simulated connection)
