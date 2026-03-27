# Agent Onboarding — SuperInstance / PastureAI

**Read this file first.** It tells you what to read next based on your goal.

---

## What Is This Project?

SuperInstance is a self-evolving AI agent ranch for local hardware (NVIDIA Jetson Orin Nano primary target). Users define AI agents as Markdown files (`breed.md`). Every night at 02:00, a genetic algorithm ("Night School") evaluates agent fitness, culls underperformers, and breeds new agents by interpolating their LoRA adapter weights. The result: agents that get better at your specific tasks over time, entirely offline.

**The pitch in one sentence:** "Don't rent an AI brain. Breed a Ranch that evolves forever."

**The metaphor throughout:** ranch = system, livestock = agents, breed.md = DNA, Night School = evolution, Border Collie = orchestrator, Cowboy = user.

---

## Quick Navigation

| If you want to...                                 | Read...                                             |
|---------------------------------------------------|-----------------------------------------------------|
| Understand the project deeply                     | [PROJECT-OVERVIEW.md](PROJECT-OVERVIEW.md)          |
| Know where every file is and what state it's in   | [CODEBASE-MAP.md](CODEBASE-MAP.md)                  |
| See what's real vs. what's stubbed                | [RELEASE-AUDIT.md](RELEASE-AUDIT.md)                |
| Know what needs to be done next                   | [WORK-AHEAD.md](WORK-AHEAD.md)                      |
| See what was done in the last session             | [LAST-SESSION.md](LAST-SESSION.md)                  |

---

## Current Status at a Glance (as of 2026-03-27)

| Component                | Status         | Notes                                              |
|--------------------------|----------------|----------------------------------------------------|
| Binary compiles          | ✅ Working     | `cargo build --release` in `superinstance/`        |
| Demo mode                | ✅ Working     | `superinstance --demo` — processes email, no GPU   |
| breed.md parser          | ✅ Working     | pulldown-cmark, parses all sections                |
| File watcher (inotify)   | ✅ Working     | Fires on changes; startup scan broken (walkdir stub)|
| Night School cron        | ✅ Working     | 02:00 cron loop, manual trigger, report struct     |
| TUI dashboard            | ✅ Working     | ratatui-based, shows status                        |
| REST API                 | ✅ Working     | All endpoints wired to real Ranch state            |
| HMAC webhook auth        | ✅ Working     | Real HMAC-SHA256 with constant-time compare        |
| Hardware detection       | ✅ Working     | Jetson / CUDA GPU / CPU / Demo tiers               |
| Night School culling     | ⚠️ Stub        | Logs the action but empty `{}` blocks — no removal |
| SLERP breeding           | ⚠️ Wrong algo  | slerp_merge operates on coefficients, not tensors  |
| LoRA loading             | ⚠️ Simulated   | `sleep(20ms)` — no actual safetensors file I/O     |
| Startup breed scan       | ❌ Broken      | walkdir is a local stub returning `iter::empty()`  |
| Candle CPU backend       | ❌ Not compiled| No `candle` feature in Cargo.toml features block   |
| Discord integration      | ❌ Simulated   | Sets `connected=true`, no Serenity event handler   |
| Collie routing           | ❌ Stub        | `sleep(60s)` loop — not wired to any input         |
| TensorRT-LLM             | ❌ Not compiled| `tensorrt` feature never enabled in default build  |

---

## Key Numbers

- Binary size: 4.2 MB (enforced by CI — fails build if >5 MB)
- Target hardware: NVIDIA Jetson Orin Nano 8GB (~$499)
- Model: Phi-3 Mini 4K (~20 tok/s on Jetson in MAXN mode)
- Night School schedule: 02:00 daily
- Culling threshold: 0.40 fitness
- Hot-swap target: <50ms (currently simulated)
- Debounce on file watcher: 500ms (press materials claim 50ms — discrepancy)
- Rust version required: 1.85+

---

## Repo Layout (top level)

```
pasture-ai/
├── superinstance/     ← Main Rust binary (the actual product)
├── backend/           ← Heavy integrations: TensorRT, CRDT, llama.cpp FFI
├── src/               ← Next.js 16 frontend dashboard
├── pasture/           ← User-editable breed.md definitions
├── genetics/          ← Gene trait definitions (LoRA traits)
├── examples/          ← Template breeds (coder, consultancy, maker, smart-home)
├── templates/         ← Role-based templates (10 types)
├── announce/          ← Launch materials: video scripts, HN post, Reddit, PH, dev.to
├── agent-onboarding/  ← You are here
├── docs/              ← Architecture docs, tutorials, screenshots
├── CHANGELOG.md
├── ROADMAP.md
└── CLAUDE.md          ← The source-of-truth for build commands
```

---

## How to Build and Run

```bash
# From superinstance/ directory
cargo build --release     # Build binary (enforces <5 MB)
cargo test                # Run all tests

# Run demo (no GPU needed)
./target/release/superinstance --demo

# Run full ranch
./target/release/superinstance --port 3001

# Single test
cargo test <test_name> --release
```

See `CLAUDE.md` (at repo root) for the full make command reference.

---

## What the Last Agent Did

See [LAST-SESSION.md](LAST-SESSION.md) for a detailed log of all changes made in the previous conversation.
