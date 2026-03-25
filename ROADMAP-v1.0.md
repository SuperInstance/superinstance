# SuperInstance v1.0 Production Roadmap

## Current State
- Backend: 2.5MB Axum live (:3001 /api/breeds/cattle)
- Pushes: 2 (backend-mvp, cattle-working)
- Next.js frontend scaffold
- ✅ Phase 1-2 COMPLETE (feat: core-night-discord)

## Phases

### ✅ 1. Core (Night School + Discord) 1h [COMPLETE]
- ✅ night_school.rs: Tokio cron 02:00 SLERP breed/cull SQLite
- ✅ discord.rs: Serenity bot Collie route streaming
- ✅ llama_native.rs: <1ms reflex via native bindgen

### ✅ 2. Integration 1h [COMPLETE]
- ✅ Makefile: make ranch (bun+backend proxy)
- ✅ pasture_sync.rs: CRDT yjs for conflict-free sync
- ✅ API endpoints: /api/sync, /api/evolution

### 3. Jetson Prod 1h [NEXT]
- install_jetson.sh: rustup/TensorRT/MAXN/swap
- Performance optimization

### 4. Tests/Release 30min
- CI workflows (size/test)
- Docs/screenshots
- Announce thread

## Agent Spawns
- grok-4.20-0309-reasoning: Phases 1-2 code ✅ COMPLETE
- grok-4.20-0309-non-reasoning: Phases 3-4 edits
- grok-imagine-video: Onboarding video (ranch anim)

## Implementation Details

See [IMPLEMENTATION.md](./IMPLEMENTATION.md) for:
- Night School SLERP algorithm
- Discord bot setup
- Llama.cpp native bindgen
- CRDT pasture sync
- API documentation
- Testing guide
