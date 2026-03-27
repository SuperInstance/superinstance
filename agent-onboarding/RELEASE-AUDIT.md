# Release Audit — Three Passes

**Date:** 2026-03-27
**Auditor:** Claude Sonnet 4.6
**Purpose:** Compare press release materials to actual codebase state. Three independent passes.

---

## PASS 1: Press Claims vs. Code Reality

This pass reads every specific claim made in `announce/` and checks the code.

### Claim: "4.2 MB single binary"
**Reality:** ✅ TRUE. CI enforces <5 MB via `stat` check. Release profile uses `opt-level=z`, `lto=true`, `codegen-units=1`, `strip=true`, `panic=abort`. 4.2 MB is a plausible real number for this profile with the listed deps.
**Risk:** Binary size has never been actually verified by building — there is no CI run confirming it. The CI workflow *is* set up to check it, but the build has not been run in this environment.

### Claim: "Night School at 02:00 nightly"
**Reality:** ✅ TRUE. `run_night_school()` in ranch.rs correctly computes sleep duration until 02:00. The loop is correct. Manual trigger also works.
**Caveat:** Night School *runs* but culling and breeding are stubs (see Pass 2).

### Claim: "Agents below 0.40 threshold are culled"
**Reality:** ❌ STUB. `cull_underperformers()` in night_school.rs calls `stud_book.get_underperformers(0.4)` and iterates the result, but the actual removal blocks are empty:
```rust
{
    let book = self.stud_book.lock();
    // Would call cull_agent here
}
{
    let mut registry = self.species_registry.write();
    // Would remove from registry
}
```
**No agent is ever actually removed.** The registry's own `cull()` method IS called at the end, so some culling happens via the registry path. But the stud_book record is never updated to `AgentStatus::Culled`.

### Claim: "SLERP LoRA weight interpolation"
**Reality:** ❌ WRONG ALGORITHM. The dev.to article shows this claimed-to-be-real function:
```rust
// The actual implementation in superinstance/src/evolution/breeding.rs
fn slerp(a: &[f32], b: &[f32], t: f32) -> Vec<f32> { ... }
```
**This function does not exist in breeding.rs.** The actual `slerp_merge()` operates on a `HashMap<String, f32>` of blend coefficients, not on tensor weight arrays. The formula used is:
```rust
let adjusted = weight * (1.0 - alpha) + alpha * weight.powi(2);
```
This is not SLERP. SLERP requires the angle between two vectors: `sin((1-t)θ)/sin(θ) × a + sin(tθ)/sin(θ) × b`. There are no two weight vectors here, only a single coefficient per parent.

**Additionally:** The `Offspring` struct stores `merge_coefficients` (the ratio at which to blend) but there is no code anywhere that applies those coefficients to actual safetensors tensor data. The LoRA loading is simulated.

**This is the most critical press-to-reality discrepancy.** The dev.to article describes code that doesn't exist. Before publishing dev.to, either implement real tensor-level SLERP or revise the article.

### Claim: "breed.md hot-reload in ~180ms"
**Reality:** ⚠️ UNVERIFIED + PARTIALLY BROKEN.
- The file watcher fires correctly on changes.
- BUT `scan_existing_breeds()` always returns 0 breeds because the `walkdir` module at the bottom of `genetics/watcher.rs` (lines 279-312) is a **local stub** returning `std::iter::empty()`. `walkdir` is not in Cargo.toml.
- Debounce window is **500ms**, not 50ms as stated in the dev.to article.
- The "~180ms" reload time includes safetensors tensor loading — which is currently `std::thread::sleep(20ms)` (simulated). The actual time on real hardware with a 150 MB adapter is unknown.

### Claim: "`notify-debouncer-full` with 50ms debounce"
**Reality:** ❌ WRONG NUMBER. Code uses `Duration::from_millis(500)`.

### Claim: "safetensors for LoRA tensor loading"
**Reality:** ⚠️ DEP PRESENT BUT NOT USED. `safetensors = "0.4"` is in Cargo.toml but `hot_swap()` in lora_manager.rs does:
```rust
let load_time = Duration::from_millis(20); // Simulated
std::thread::sleep(load_time);
```
No actual safetensors file I/O occurs.

### Claim: "`Arc<RwLock<BreedManifest>>` swapped atomically"
**Reality:** ✅ STRUCTURALLY TRUE. The watcher fires and calls `breeds.write().insert(...)` to atomically swap the breed in the cache. This part is correct. The missing piece is that the inference engine doesn't actually pick up the new manifest and apply it — there's no pipeline from "watcher fires" to "model weights reloaded."

### Claim: "Hardware: Jetson → CUDA GPU → CPU Candle → Demo"
**Reality:** ⚠️ PARTIALLY TRUE.
- ✅ Jetson detection: reads `/proc/device-tree/model` — correct.
- ✅ CUDA GPU detection: `nvidia-smi` check — correct.
- ❌ Candle CPU backend: The `#[cfg(feature = "candle")]` block exists but **`candle` is not a defined feature in Cargo.toml**. The candle feature block only lists: `gpu`, `tensorrt`, `prometheus`, `telegram`, `full`. To enable the candle backend, someone must add `candle = ["candle-core", "candle-nn"]` (or similar) to the features block AND add candle as a dep. Currently, CPU mode always falls back to Mock (Demo).
- ✅ Demo mode: Works.

### Claim: "Discord channel connector (scaffolded, needs real bot integration)"
**Reality:** ✅ ACCURATELY DESCRIBED in reddit/FINAL.md. The product hunt and video scripts are more bullish — they present it as a working feature. The actual state is that the Serenity `Client::builder()` call is commented out and `start()` just sets connected=true.

### Claim: "Single binary includes TUI, web server, inference engine, evolution scheduler, Discord/Telegram connectors"
**Reality:** ⚠️ PARTIALLY TRUE. TUI ✅, web server ✅, inference engine (mock) ✅, evolution scheduler ✅. Discord: compiled in (serenity is a dep) but simulated. Telegram: optional feature, not in default build. The "4.2 MB including Discord client" claim is accurate since serenity IS compiled in by default.

### Claim: "`superinstance --demo` to try without hardware"
**Reality:** ✅ WORKS. Demo mode runs clean, processes a sample email, exits cleanly.

### Claim: "Fitness function is currently 'human approval rate'"
**Reality:** ⚠️ NOT IMPLEMENTED. There is no mechanism for humans to approve/reject outputs and record it as fitness data. The `evaluate_agents()` function uses `task.successful_tasks / task.total_tasks` where `successful_tasks` is never incremented anywhere in the codebase (it's always 0).

### Claim: "Generation 6 or 8 after a few weeks"
**Reality:** ❌ ASPIRATIONAL. Since culling doesn't remove agents and breeding doesn't apply actual weight merges, the genealogy tracking in the Stud Book would show generations incrementing but the agents would be functionally identical across generations.

---

## PASS 2: What Actually Runs vs. What's Stubbed

### Things That Actually Work End-to-End

| Feature | Evidence |
|---------|----------|
| `--demo` mode | `run_demo_mode()` creates InferenceEngine::demo(), initializes Cattle, calls `process_email()`, gets back a real response from MockBackend |
| breed.md parsing | `BreedManifest::parse()` tests pass; parses all sections including trait weight tables |
| File watcher (notification only) | `notify-debouncer-full` watcher fires correctly; change events sent on channel |
| Hardware detection | Reads `/proc/device-tree/model`, runs `nvidia-smi`, reads `/proc/meminfo` |
| Night School cron | Sleep-until-02:00 loop correct; manual trigger works; report struct populated |
| REST API | All endpoints return real data from Ranch state |
| HMAC webhook verification | Real HMAC-SHA256 + constant-time compare |
| TUI dashboard | ratatui renders, shows status |
| Stud Book SQLite | Schema created, CRUD operations work, `get_stats()` returns real data |

### Things That Are Correctly Described as "Scaffolded" or "In Progress"

| Feature | Status |
|---------|--------|
| TensorRT-LLM | `tensorrt` feature gate exists; not callable without the actual TensorRT library |
| Discord full integration | Serenity dep present; event handler commented out |
| Telegram | `telegram` feature exists; `teloxide` dep optional |
| CRDT memory sync | `smartcrdt/` in backend/ has README; not implemented |

### Things That Are Claimed as Working But Aren't

| Claim | Reality |
|-------|---------|
| Culling removes agents | Empty implementation blocks |
| SLERP on LoRA weights | Operates on coefficients only; no tensor math |
| LoRA loading from disk | `sleep(20ms)` — no file I/O |
| breed.md startup scan | walkdir is a local stub; always finds 0 breeds |
| Candle CPU inference | `candle` feature not defined; always falls back to mock |
| Fitness from task outcomes | `successful_tasks` never incremented |
| 50ms debounce | Code uses 500ms |

---

## PASS 3: Release Readiness Gap Analysis

### Blockers for Honest v0.2.0 (must fix before launching)

**B1 — walkdir stub**
File: `superinstance/src/genetics/watcher.rs:279-312`
Fix: Add `walkdir = "2"` to Cargo.toml, remove the local stub module, replace with real `walkdir::WalkDir`.
Impact: Without this, the watcher never populates the breed cache on startup. Hot-reload only works for files changed after startup.

**B2 — Press materials reference non-existent SLERP function**
File: `announce/launch/other/dev-to.md`
The article claims the SLERP function is "the actual implementation in superinstance/src/evolution/breeding.rs" and shows it. This code does not exist in that file.
Fix option A: Implement the real tensor-level SLERP function and put it in breeding.rs.
Fix option B: Revise the dev.to article to accurately describe the coefficient-based approach and be honest it's aspirational ("this is what breeding.rs will implement when LoRA loading is complete").
Do not publish dev.to with this discrepancy.

**B3 — Cargo.toml repository URL wrong**
File: `superinstance/Cargo.toml`
`repository = "https://github.com/SuperInstance/superinstance"` should be `"https://github.com/SuperInstance/pasture-ai"`.
Must fix before crates.io publish.

### Important (fix before video recording)

**I1 — candle feature not defined**
File: `superinstance/Cargo.toml`
The `[features]` block has no `candle` entry. The CPU fallback claim ("runs on any Linux x86_64 CPU — Candle backend") is accurate only if this feature is defined and a candle dep is added. Without it, CPU mode = mock mode = no actual inference.
Fix: Add `candle = ["dep:candle-core", "dep:candle-nn"]` to features and the candle deps as optional. This is a significant addition but is needed to make the "runs on any CPU" claim true.

**I2 — Night School culling empty**
File: `superinstance/src/evolution/night_school.rs:319-333`
The two `{}` blocks need actual implementation: remove from stud_book, update agent status.
Without this, Night School never actually changes the population.

**I3 — Fitness scoring not wired**
There is no path from "agent processes a task" to "fitness score updated." The `successful_tasks` field on `ActiveAgent` is never incremented. Night School therefore evaluates all agents at 0.5 fitness and the whole selection mechanism is inert.

**I4 — Debounce discrepancy**
`watcher.rs:92`: `Duration::from_millis(500)` — press says 50ms. Either change to 50ms (test on Jetson that it doesn't cause thrashing) or correct the press materials.

### Minor (clean up before or after launch)

**m1 — Discord status embed has wrong version**
File: `superinstance/src/channels/discord.rs:279`
`SuperInstance v0.1.0` should be `v0.2.0`.

**m2 — pasture/base_models/phi3-mini.safetensors in git**
Check the size: `ls -lh pasture/base_models/phi3-mini.safetensors`. If it's a real model file (>1 GB), it must not be in git. If it's a placeholder/zero-byte, add a .gitignore for `*.safetensors` and document how users get the real model.

**m3 — cattle-v1/breed.toml (old format)**
File: `pasture/cattle-v1/breed.toml`
Old TOML format, superseded by breed.md. Should be removed or converted.

**m4 — memory/ directory in repo**
Files `memory/2026-03-25.md` and `memory/2026-03-26.md` appear to be AI session memory logs, not product files. Should be in .gitignore or a dev-only directory.

**m5 — channels/discord/mod.rs at repo root**
File: `channels/discord/mod.rs` (repo root)
This appears to be a stray file at the repo root, separate from `superinstance/src/channels/discord.rs`. The root-level file may be a remnant from an earlier structure. Verify whether it's needed or dead code.

---

## Audit Summary Table

| Area                  | Claimed State | Actual State | Gap |
|-----------------------|---------------|--------------|-----|
| Binary size           | 4.2 MB        | Unverified (CI set up correctly) | Low |
| Demo mode             | Working       | Working      | None |
| breed.md parsing      | Working       | Working      | None |
| File watcher          | Working       | Partially broken (walkdir stub) | High |
| Night School cron     | Working       | Working      | None |
| Night School culling  | Working       | Empty blocks | High |
| SLERP breeding        | "Working"     | Wrong algorithm + no tensor math | High |
| LoRA loading          | ~180ms        | sleep(20ms) simulated | High |
| Candle CPU backend    | Available     | Feature not defined | High |
| Fitness scoring       | Human approval| Never incremented | High |
| Discord bot           | Scaffolded    | Simulated (accurate in some posts) | Medium |
| Telegram              | Scaffolded    | Feature flag only | Low |
| TensorRT-LLM          | In progress   | Feature flag only | Low |
| Collie routing        | Working       | sleep(60s) stub | High |
| API endpoints         | Working       | Working (fixed last session) | None |
| Webhook HMAC          | Working       | Working (fixed last session) | None |
