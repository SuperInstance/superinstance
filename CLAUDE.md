# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**PastureAI / SuperInstance** is a self-evolving AI ranch ecosystem for edge deployment (NVIDIA Jetson Orin Nano 8GB primary target). Users define AI agents via Markdown "breed files" that evolve nightly at 02:00 through SLERP genetic algorithms ("Night School"). The Border Collie orchestrator routes user intents to specialized LoRA agents (species: Cattle for email, Duck for other tasks, etc.).

Architecture flow: **Cowboy (user) → Intent → Collie (orchestrator) → Route → Pasture (LoRA pool)**

## Build, Test, and Lint Commands

```bash
# Build
make build              # Build all components (enforces <5 MB binary limit)
make build-debug        # Debug build

# Run
make run                # Ranch (TUI + Web combined)
make ranch              # Full dev stack: bun dev (:3000) + backend proxy (:3001)
make run-web            # Next.js frontend only
make run-tui            # TUI dashboard only

# Test
make test               # All tests (cargo + bun)
make test-night-school  # Night School evolution tests
make test-coverage      # With tarpaulin coverage report

# Single Rust test
cd superinstance && cargo test <test_name> --release

# Lint & Format
make lint               # cargo clippy -D warnings
make fmt                # cargo fmt + bun format
bun run lint            # ESLint only

# Database (Next.js/Prisma)
bun run db:push         # Push schema changes
bun run db:generate     # Regenerate Prisma client
bun run db:migrate      # Run migrations
bun run db:reset        # Reset database

# Evolution
make breed              # Run breeding cycle
make night-school       # Manual Night School trigger
make cull               # Prune weak breeds
```

## Architecture

### Repository Structure

- **`superinstance/`** — Main Rust binary (single binary, <5 MB constraint with LTO + strip; requires Rust 1.85+)
- **`backend/`** — Heavy integrations: TensorRT, CRDT (yrs/Yjs), llama.cpp FFI
- **`src/`** — Next.js 16 frontend (TypeScript, React 19, TailwindCSS 4, Prisma/SQLite)
- **`pasture/`** — User-editable breed definitions (Markdown); hot-reloaded via `notify`
- **`genetics/traits/`** — Gene definitions (composable LoRA traits)
- **`examples/`** — Template breeds (consultancy, coder, maker, smart-home)
- **`templates/`** — Role-based templates (finance, healthcare, legal, etc.)

### Rust Binary (`superinstance/src/`)

| Module | Purpose |
|---|---|
| `collie/` | Border Collie orchestrator: routing, reflex (<1ms), anticipation |
| `species/` | Agent type implementations (Cattle, Sheep, Duck, etc.) |
| `genetics/` | `breed.md` parser, gene composer, hot-reload watcher |
| `pasture/` | Model pool, LoRA manager, hardware-aware inference engine |
| `evolution/` | Night School (02:00 cron), SLERP breeding, stud book genealogy |
| `channels/` | Protocol connectors: Discord (serenity), Telegram (teloxide, optional) |
| `web/` | Axum API server + WebSocket |
| `onboarding/` | Interactive setup wizard |

### Key Design Constraints

- **5 MB binary limit** enforced in CI. Release profile uses: `opt-level=z`, LTO, `codegen-units=1`, strip, `panic=abort`.
- **Hardware fallback chain**: Jetson TensorRT → Desktop CUDA → Laptop CPU (Candle) → Mock (demo mode)
- **breed.md files** in `pasture/` are hot-reloaded; editing them mid-run updates agent behavior
- Cargo features gate optional deps: `gpu`, `tensorrt`, `prometheus`, `telegram`, `full`

### Frontend (`src/`)

Next.js App Router, Radix UI components, TanStack Query for server state, Prisma ORM over SQLite. Auth via NextAuth. i18n via next-intl.

## Environment Setup

Copy `.env.example` to `.env` and configure:
- `DATABASE_URL` — SQLite path
- `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` — AI provider keys
- `DISCORD_TOKEN` / `TELEGRAM_BOT_TOKEN` — channel connectors
- `MAX_VRAM_GB`, `BASE_MODEL`, `NIGHT_SCHOOL_TIME` (default: `02:00`) — ranch tuning
- `DLA_ENABLED` — Jetson DLA acceleration

## CI/CD

GitHub Actions (`.github/workflows/ci.yml`) runs: Rust fmt/clippy/build/test, ESLint/build/prisma-generate, cargo audit, binary size enforcement (<5 MB), and trufflehog secret scanning.
