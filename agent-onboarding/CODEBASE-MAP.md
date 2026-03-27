# Codebase Map — What Lives Where

This is a file-by-file reference of what each module does and what state it's actually in.
"State" means: does the code do the thing it says it does, or is it scaffolded/stubbed?

---

## superinstance/src/ (The Main Binary)

### main.rs
- **Does:** Parses CLI args (--demo, --bunker, --port, --pasture, --no-evolution, --verbose), initializes Ranch, spawns Night School task, starts Collie, starts web server, launches TUI dashboard.
- **State:** ✅ Working. The startup flow is complete.
- **Note:** Demo mode (`--demo`) correctly processes a sample email end-to-end.

### ranch.rs
- **Does:** The top-level container holding all components. Accessor methods for all status data used by the API.
- **State:** ✅ Working. All accessor methods implemented. Night School integrated.
- **Accessors added in last session:** `get_uptime_secs()`, `get_hardware_tps()`, `get_all_agents()`, `get_agent()`, `get_avg_fitness()`, `get_night_school_last_run()`, `get_stud_book_stats()`.

### collie/mod.rs
- **Does:** Defines the Border Collie struct, runs the main event loop.
- **State:** ⚠️ Stub. `run()` is `loop { sleep(60s); debug!("heartbeat") }`. `route_intent()` has logic but no input wire.
- **Fix needed:** Wire an mpsc channel or websocket input into the Collie's `run()` loop.

### collie/shepherd.rs, anticipation.rs, reflex.rs
- **Does:** Routing strategy (shepherd), anticipation engine (shadow pup), reflex cache.
- **State:** ⚠️ Scaffolded. Structs and logic exist; not exercised in the live system yet.

### species/mod.rs + cattle.rs, sheep.rs, duck.rs, goat.rs, hog.rs, chicken.rs, horse.rs
- **Does:** Implements each species type with `execute()` method.
- **State:** ✅ Working for demo (Cattle processes email in demo mode). Others are functional as stubs that return string responses.
- **Note:** `Cattle::with_inference()` is the most complete. Other species use mock inference.

### genetics/manifest.rs
- **Does:** Parses breed.md files using pulldown-cmark. Extracts lineage, phenotype, genetic composition (LoRA trait weights), system prompt, tool access.
- **State:** ✅ Working. Parser handles all sections. Has tests that pass.
- **Note:** Correctly strips backticks from gene IDs and weights.

### genetics/watcher.rs
- **Does:** Watches pasture/ directory for breed.md changes, fires BreedChangeEvent on change.
- **State:** ⚠️ Partially broken. The file system watching via `notify-debouncer-full` works. BUT:
  - **Critical bug:** `walkdir` module (lines 279-312) is a LOCAL STUB that returns `std::iter::empty()`. `scan_existing_breeds()` will never find any breeds on startup.
  - **Debounce:** 500ms (not 50ms as claimed in press materials).
- **Fix needed:** Add `walkdir = "2"` to Cargo.toml and remove the local stub module.

### genetics/composer.rs, anatomy.rs
- **Does:** Gene composition, anatomy trait system.
- **State:** ⚠️ Scaffolded. Structures present; not fully wired.

### evolution/night_school.rs
- **Does:** Cron loop (runs at configurable hour, default 02:00), manual trigger, full 6-phase cycle (evaluate → cull → breed → quarantine → promote → distill).
- **State:** ⚠️ Partially stubbed.
  - ✅ Cron scheduler works, manual trigger works, report struct populated.
  - ❌ `evaluate_agents()`: "simulate fitness decay and random improvement" — not using task logs.
  - ❌ `cull_underperformers()`: Gets list of underperformers, but the culling blocks are empty `{}` — no actual removal from registry or stud book.
  - ⚠️ `quarantine_and_test()`: Uses `child.estimated_fitness` (a random f32) — not real testing.
  - ❌ `distill_cloud_knowledge()`: Always logs "No cloud calls to distill today."

### evolution/breeding.rs
- **Does:** Parent selection (tournament, roulette, random, elitist), offspring creation, merge method dispatch.
- **State:** ⚠️ Partially stubbed.
  - ✅ `tournament_selection()`, `elitist_selection()`, etc. are correct.
  - ❌ `slerp_merge()`: This is NOT spherical linear interpolation on weight tensors. It operates on HashMap<String, f32> blend coefficients and uses `weight * (1.0-alpha) + alpha * weight.powi(2)`. The actual tensor-level SLERP shown in the dev.to press article DOES NOT EXIST in this file.
  - ⚠️ `ties_merge()`: TIES algorithm described correctly but operating on coefficients, not tensors.
  - Note: The `Offspring` struct stores `merge_coefficients` (the blend ratios) but there is no code that applies those coefficients to actual safetensors weight data.

### evolution/stud_book.rs
- **Does:** SQLite-backed genealogy tracking. Creates schema, records agents/breeding/tasks.
- **State:** ✅ Working. SQL schema correctly defined, CRUD operations implemented.
- **Note:** `get_stats()` returns `StudBookStats { total_agents, total_generations, avg_fitness, total_culls, total_breeding_events }`.

### pasture/inference.rs
- **Does:** Hardware tier detection (Jetson/GPU/CPU/Demo), inference engine routing, mock backend.
- **State:** ✅ Works for demo/mock. Real backends gated:
  - ❌ `#[cfg(feature = "tensorrt")]` — `tensorrt` feature exists in Cargo.toml but the actual `TensorRTBackend` struct is not in `superinstance/src/`. It may be in `backend/` crate.
  - ❌ `#[cfg(feature = "candle")]` — **`candle` is NOT a defined feature in Cargo.toml**. The candle feature block in `[features]` only has `gpu`, `tensorrt`, `prometheus`, `telegram`, `full`. So the candle backend code NEVER compiles. Default inference is always mock.

### pasture/lora_manager.rs
- **Does:** LoRA adapter loading, hot-swap, VRAM management, LRU eviction.
- **State:** ⚠️ Simulated. Core data structures are correct. But `hot_swap()` uses `std::thread::sleep(Duration::from_millis(20))` instead of actual safetensors loading. No real file I/O to `.safetensors` files occurs.

### pasture/model_pool.rs, mod.rs
- **Does:** Model pool management.
- **State:** ⚠️ Scaffolded. Structure present.

### web/api.rs
- **Does:** REST API endpoints: GET /api/status, GET /api/species, GET /api/species/:name, POST /api/breed, GET /api/night, POST /api/night, WebSocket /ws.
- **State:** ✅ Working. All endpoints wired to real Ranch state (fixed in last session).
- **Note:** POST /api/breed returns BadRequest with helpful message (not fake success). POST /api/night triggers Night School if enabled.

### web/mod.rs, dashboard.rs
- **Does:** Axum router setup, WebSocket handling.
- **State:** ✅ Working.

### channels/discord.rs
- **Does:** Discord channel implementation.
- **State:** ❌ Simulated.
  - `start()` sets `connected.write() = true` and sends ONE hardcoded test message.
  - The real Serenity event handler (`Client::builder(...).event_handler(Handler)...`) is commented out in a block comment.
  - Outbound `send()` logs but doesn't use Serenity HTTP client.
- **Fix needed:** Uncomment and implement the Serenity client. The `serenity` dep IS in Cargo.toml.

### channels/telegram.rs
- **Does:** Telegram channel via teloxide.
- **State:** ❌ Stub. `teloxide` is an optional feature.

### channels/webhook.rs
- **Does:** Webhook receiver with HMAC-SHA256 signature verification.
- **State:** ✅ Working. Real HMAC-SHA256 with constant-time compare (fixed in last session).

### onboarding/
- **Does:** Interactive setup wizard, hardware detection, connector configuration.
- **State:** ⚠️ Scaffolded. Wizard logic exists; wizard is a separate binary (`superinstance-onboard`).

### dashboard.rs
- **Does:** TUI dashboard using ratatui.
- **State:** ✅ Working (basic). Shows status panels.

---

## Cargo.toml Features State

```
[features]
default = []         ← No GPU, no candle, no telegram, no prometheus
gpu = []             ← Gate for GPU-specific code (not used yet)
tensorrt = []        ← Gate for TensorRT-LLM backend (backend/ crate)
prometheus = [...]   ← Prometheus metrics
telegram = [teloxide] ← Telegram bot
full = [...]         ← All optional features
```

**Missing feature:** `candle` — This feature needs to be added to the features block for the candle CPU backend to ever compile.

---

## Cargo.toml Known Wrong Value

`repository = "https://github.com/SuperInstance/superinstance"`

The actual repo is `https://github.com/SuperInstance/pasture-ai`. This will matter when publishing to crates.io.

---

## backend/ crate

Heavy integrations that don't need to ship in the main binary:
- `src/cattle.rs` — alternative cattle impl
- `src/discord.rs` — alternative discord integration
- `src/llama_native.rs` — llama.cpp FFI
- `src/night_school.rs` — alternative night school
- `src/pasture_sync.rs` — CRDT sync
- `src/tui.rs` — alternative TUI

State: ⚠️ All scaffolded. The backend crate builds separately and is not linked by default.

Subdirectories with READMEs (planned integrations):
- `constraint-theory/` — constraint theory framework
- `cudaclaw/` — CUDA-level operations
- `lucineer/` — memory/search
- `smartcrdt/` — P2P CRDT sync

---

## src/ (Next.js Frontend)

Standard Next.js 16 App Router setup with:
- Radix UI + TailwindCSS 4 + shadcn components
- TanStack Query for server state
- Prisma ORM over SQLite
- NextAuth

State: ✅ Builds. The Prisma schema was rewritten in the last session to match domain models (User, Session, RanchSettings, BreedDefinition, NightSchoolRun, ChannelConfig).

---

## pasture/ (Breed Definitions)

```
pasture/
├── base_models/phi3-mini.safetensors   ← PLACEHOLDER (zero-byte or minimal file)
├── cattle-v1/breed.toml                ← Old TOML format (superseded by breed.md)
├── cattle/email-cow-v1/breed.md        ← Working example
├── cattle/hello-cow-v1/breed.md        ← Working example
├── cattle/template/breed.md            ← Template
└── duck/calendar-duck-v1/breed.md      ← Working example
```

Note: `phi3-mini.safetensors` is in git which is a problem — real model weights are multi-GB and must not be committed. Check if this is a placeholder (likely zero bytes).

---

## announce/ (Launch Materials)

Fully written and committed. Contains workshopped video scripts, HN post, Twitter thread, Reddit posts, Product Hunt, and dev.to article. See `announce/README.md` for the index.

**Critical note for press consistency:** The dev.to article shows a SLERP function `fn slerp(a: &[f32], b: &[f32], t: f32) -> Vec<f32>` claimed to be "the actual implementation in superinstance/src/evolution/breeding.rs". This function does NOT exist in that file. Before publishing the dev.to article, either implement the real function or revise the article to accurately describe the current coefficient-based approach.
