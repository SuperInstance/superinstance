# Work Ahead — Prioritized Roadmap to Release

**Date:** 2026-03-27
**Context:** v0.2.0 is planned for launch via HN, Twitter, Product Hunt, and Reddit. The codebase is a well-scaffolded alpha with several critical stubs that must be filled before the launch content (particularly dev.to) is accurate.

---

## Priority 0 — Must Fix Before Any Public Launch

### P0-1: Fix the dev.to SLERP claim
**What:** The dev.to article (`announce/launch/other/dev-to.md`) claims the function `fn slerp(a: &[f32], b: &[f32], t: f32) -> Vec<f32>` is "the actual implementation in superinstance/src/evolution/breeding.rs". This function does not exist.

**Two options (pick one):**
- **Option A (honest reframe):** Edit dev.to to say the SLERP formula is "what breeding.rs will use when LoRA loading is complete" and describe the current coefficient-based approach accurately. This is the minimum viable fix.
- **Option B (implement it):** Add the real tensor-level SLERP function to `breeding.rs`. The function itself is simple (see the formula in the article). The harder part is wiring it to actual safetensors data (P1-2). You can add the function and note it's wired when P1-2 is done.

**Files:** `announce/launch/other/dev-to.md`, `superinstance/src/evolution/breeding.rs`

### P0-2: Fix walkdir stub
**What:** `superinstance/src/genetics/watcher.rs` contains a local `mod walkdir` stub (lines 279-312) that returns `std::iter::empty()`. This means `scan_existing_breeds()` finds zero breeds at startup.

**Fix:**
1. Add `walkdir = "2"` to `superinstance/Cargo.toml` dependencies
2. Delete lines 279-312 in watcher.rs (the `mod walkdir { ... }` block)
3. Add `use walkdir::WalkDir;` at the top
4. The existing `scan_existing_breeds()` code will then work correctly since it already calls `walkdir::WalkDir::new()` with the right pattern

**Files:** `superinstance/Cargo.toml`, `superinstance/src/genetics/watcher.rs`

### P0-3: Fix Cargo.toml repository URL
**What:** `repository = "https://github.com/SuperInstance/superinstance"` should be `"https://github.com/SuperInstance/pasture-ai"`.

**File:** `superinstance/Cargo.toml`

---

## Priority 1 — Fix Before Recording Videos or Demo

### P1-1: Implement real Night School culling
**What:** `cull_underperformers()` in night_school.rs has two empty `{}` blocks where culling should happen. The registry's `cull()` IS called at the end, so the SpeciesRegistry is updated. But the StudBook record is never updated.

**Fix:** Inside the loop over `underperformers`, call:
```rust
let mut book = self.stud_book.lock();
book.set_agent_status(&agent.id, AgentStatus::Culled)?;
```
(Assuming `set_agent_status()` or equivalent exists in StudBook — check stud_book.rs and add if needed.)

**Files:** `superinstance/src/evolution/night_school.rs`

### P1-2: Add `candle` feature to Cargo.toml
**What:** The `[features]` block has no `candle` entry. The "runs on any CPU via Candle" claim requires this.

**Fix:**
```toml
[features]
candle = ["dep:candle-core", "dep:candle-nn", "dep:candle-transformers"]

[dependencies]
candle-core = { version = "0.4", optional = true }
candle-nn = { version = "0.4", optional = true }
```

Then update `CandleBackend` in inference.rs to load a real model instead of returning a placeholder string.

**Note:** This is a larger effort and will likely increase binary size. Test that release binary stays <5 MB with `--features candle` disabled (which is the default). Binary size enforcement only matters for the default build.

**Files:** `superinstance/Cargo.toml`, `superinstance/src/pasture/inference.rs`

### P1-3: Fix fitness scoring pipeline
**What:** `successful_tasks` on `ActiveAgent` is never incremented. Night School's `evaluate_agents()` therefore always sees 0/0 = 0.5 fitness. The whole selection mechanism is inert.

**Fix:** Wherever a species agent successfully processes a task, call something like:
```rust
registry.record_task_result(agent_id, success: bool)
```
This requires wiring from the species `execute()` methods back to the registry.

**Files:** `superinstance/src/species/*.rs`, `superinstance/src/species/mod.rs` (SpeciesRegistry)

### P1-4: Wire the Collie event loop
**What:** `Collie::run()` is `loop { sleep(60s) }`. It doesn't receive input. The collie is never actually routing anything.

**Fix:** Add an mpsc receiver to the Collie. When the web API or other input sources have work, they send it on this channel. The Collie's loop becomes:
```rust
loop {
    tokio::select! {
        Some(intent) = rx.recv() => self.route_intent(intent).await?,
        _ = tokio::time::sleep(Duration::from_secs(60)) => {
            debug!("Collie heartbeat");
        }
    }
}
```

**Files:** `superinstance/src/collie/mod.rs`, `superinstance/src/web/api.rs`, `superinstance/src/ranch.rs`

---

## Priority 2 — Polish and Completeness

### P2-1: Real LoRA loading via safetensors
**What:** `hot_swap()` in lora_manager.rs sleeps for 20ms instead of loading from disk. The "~180ms reload time" cannot be claimed until real file I/O is measured.

**Fix:** Use `safetensors::tensor::SafeTensors::deserialize()` to actually load the `.safetensors` file. Then apply the merge coefficients from the `Offspring` struct.

Note: This requires actual `.safetensors` adapter files to exist. The pasture/ directory structure is set up correctly but the files (other than the base model placeholder) don't exist.

**Files:** `superinstance/src/pasture/lora_manager.rs`

### P2-2: Implement real Discord Serenity integration
**What:** Discord `start()` currently sets connected=true and sends a test message. The real Serenity event handler is commented out in a block comment.

**Fix:** Uncomment and implement:
```rust
let mut client = Client::builder(&self.token, GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
    .event_handler(Handler { tx: tx.clone() })
    .await?;
client.start().await?;
```

The `DiscordHandler` struct is already defined in discord.rs. It needs the `EventHandler` impl with `async fn message()`.

**Note:** Requires a real Discord bot token. The `DISCORD_TOKEN` env var is documented in `.env.example`.

**Files:** `superinstance/src/channels/discord.rs`

### P2-3: Verify and fix debounce timing
**What:** Press materials say 50ms debounce; code uses 500ms. Measure on target hardware what the minimum safe debounce is (editors that write in multiple steps need >100ms typically).

**Recommendation:** Change to 100ms, measure on Jetson, update press materials to say "100ms debounce" instead of "50ms".

**Files:** `superinstance/src/genetics/watcher.rs:92`

### P2-4: Fix Discord embed version number
**What:** `create_status_embed()` in discord.rs has footer "SuperInstance v0.1.0".

**File:** `superinstance/src/channels/discord.rs:279`

### P2-5: Check phi3-mini.safetensors
**What:** `pasture/base_models/phi3-mini.safetensors` is tracked in git. If this is a real model file (multi-GB), it breaks git clone for anyone. If it's a placeholder, add `*.safetensors` to `.gitignore`.

**Fix:** Check size with `ls -lh pasture/base_models/phi3-mini.safetensors`. If >1 MB, immediately remove with `git rm --cached` and add to `.gitignore`.

### P2-6: Clean up memory/ directory
**What:** `memory/2026-03-25.md` and `memory/2026-03-26.md` are AI session memory logs in the repo root. These shouldn't be user-visible.

**Options:** Add `memory/` to `.gitignore` OR move to a `dev/` or `.ai/` directory that's gitignored.

### P2-7: Remove or convert pasture/cattle-v1/breed.toml
**What:** Old TOML format file from an earlier iteration, superseded by breed.md format. Could confuse users.

---

## Priority 3 — Launch Checklist (from ROADMAP.md Phase 5)

These are the items needed for the community aspects of launch:

- [ ] Discord server setup (for user community — separate from the bot)
- [ ] GitHub Discussions enabled on the repo
- [ ] Record 60-second video (script: `announce/videos/60-second/FINAL.md`)
- [ ] Record 6-minute video (script: `announce/videos/6-minute/FINAL.md`)
- [ ] Verify `superinstance --demo` exit behavior is clean (no error messages)
- [ ] crates.io publication (once P0-3 repository URL is fixed)
- [ ] Homebrew formula (for macOS users wanting `brew install superinstance`)
- [ ] Post HN (script: `announce/launch/hn/FINAL.md`)
- [ ] Post Twitter/X (script: `announce/launch/twitter-x/FINAL.md`)
- [ ] Post Product Hunt (script: `announce/launch/other/product-hunt.md`)

---

## Architecture Work (v0.3.0 territory)

These are the bigger features described in ROADMAP.md that are not blockers for v0.2.0 launch but define the v0.3.0 sprint:

1. **Full TensorRT-LLM integration** — Needs the `backend/` crate's llama_native.rs and TensorRT library wiring
2. **Discord slash commands** — `/ask`, `/breed`, `/evolve` with response streaming
3. **Pluggable fitness functions** — Allow users to define custom fitness metrics beyond "task success rate"
4. **Real-time breeding animation in TUI** — The geometric shapes merging animation described in ROADMAP.md
5. **CRDT Memory Pasture** — P2P sync between multiple Ranch instances (smartcrdt/ backend, mDNS discovery)
6. **Night School TUI log** — Live visualization of the cull/breed cycle as it runs at 02:00
7. **breed.md live editor in dashboard** — Edit in the web UI, see hot-reload in real-time

---

## Effort Estimates (rough)

| Task | Effort | Risk |
|------|--------|------|
| P0-1 dev.to reframe | 30 min | Low |
| P0-2 walkdir fix | 1 hour | Low |
| P0-3 repo URL fix | 5 min | None |
| P1-1 real culling | 2 hours | Low |
| P1-2 candle feature | 1 day | Medium (binary size risk) |
| P1-3 fitness pipeline | 1 day | Medium (wiring) |
| P1-4 Collie wiring | 4 hours | Medium |
| P2-1 real LoRA loading | 2 days | High (needs real adapters) |
| P2-2 Discord Serenity | 1 day | Low (framework handles it) |
| Videos | 2 days | None (content is ready) |
| HN/Twitter/Reddit posts | 2 hours | None (content is ready) |
