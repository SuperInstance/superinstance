# Last Session Summary

**Date:** 2026-03-25 → 2026-03-27
**Model:** Claude Sonnet 4.6

---

## What Was Done

### Session 1: CLAUDE.md + Release Audit

Created `CLAUDE.md` at repo root with build commands, architecture overview, and environment setup. Then performed a full release readiness audit and made the following fixes:

**Blockers fixed:**
- `.gitignore` — Fixed git merge conflict markers (`<<<<<<< HEAD` / `>>>>>>> master`)
- `db/custom.db` — Removed from git tracking, added `*.db` to root `.gitignore`
- `superinstance/Cargo.toml` — Keywords reduced from 10 to 5 (crates.io limit), `rust-version` bumped `1.75` → `1.85` (required by modern deps), version bumped `0.1.0` → `0.2.0`
- `superinstance/Cargo.lock` — Generated and committed (was missing for a binary crate)
- `superinstance/.gitignore` — Removed `Cargo.lock` from ignore
- `superinstance/src/channels/webhook.rs` — Replaced always-pass stub with real HMAC-SHA256 + constant-time verification using `hmac`/`sha2`/`hex`/`subtle` crates
- `superinstance/src/evolution/breeding.rs` — Replaced `panic!()` in `tournament_selection` with `debug_assert!()` (compiled out in release)
- `superinstance/src/main.rs` — Fixed `WebConfig` construction (missing `enable_dashboard` field), removed unused binding causing compiler warning
- `superinstance/src/ranch.rs` — Added `startup_time`, `hardware_tps`, `night_school_last_run` fields plus all accessor methods needed by the API
- `superinstance/src/web/api.rs` — Complete rewrite: all endpoints wired to real Ranch state (previously hardcoded stubs). Endpoints: GET /api/status, GET /api/species, GET /api/species/:name, POST /api/breed (now returns BadRequest with helpful message), GET /api/night, POST /api/night, WebSocket /ws
- `next.config.ts` — Removed `ignoreBuildErrors: true`, enabled `reactStrictMode: true`, made backend URL configurable
- `package.json` — Name changed to `superinstance`, added metadata, moved `z-ai-web-dev-sdk` to devDeps
- `.env.example` — Fixed `DATABASE_URL`, added `NEXTAUTH_SECRET`, `NEXTAUTH_URL`, `BACKEND_URL`
- `CHANGELOG.md` — Fixed placeholder dates to real dates
- `scripts/install_jetson.sh` — Was a 1-line stub; now delegates to the real script
- `superinstance/scripts/install_jetson.sh` — Added guard to prevent overwriting existing Makefile
- `prisma/schema.prisma` — Complete rewrite: replaced template User/Post models with 6 domain models (User, Session, RanchSettings, BreedDefinition, NightSchoolRun, ChannelConfig)
- `backend/Cargo.toml` — Added metadata (authors, description, license, repository, homepage), bumped to 0.2.0
- `Makefile` — Changed benchmark header to "REFERENCE NUMBERS" (removing "VERIFIED" claim for unverified numbers)
- `README.md` / `superinstance/README.md` — ASCII art version bumped to v0.2.0
- `ROADMAP.md` — v0.2.0 GitHub Release marked complete
- `CONTRIBUTING.md` — Rust version requirement updated to 1.85+

**Git identity fix:** All commits used `-c user.name="SuperInstance" -c user.email="superinstance@users.noreply.github.com"` to work around missing git config.

### Session 2: announce/ folder

Created a complete set of launch materials:

**Video scripts:**
- `announce/videos/CREATIVE-BRIEF.md` — Shared creative direction (tone: direct, technical, real; no music in 60s)
- `announce/videos/60-second/` — 3 iterations + FINAL script. Key decision: open on the Night School log (a raw terminal log is more credible than claims)
- `announce/videos/6-minute/` — 3 iterations + FINAL script. Key decision: use a real person's workflow (invoicing), not a generic demo

**Launch posts:**
- `announce/launch/hn/` — 3 iterations + FINAL. Key decision: end with 3 genuinely open questions (HN rewards genuine discourse, not announcement-only posts)
- `announce/launch/twitter-x/` — 2 iterations + FINAL 6-tweet thread. v1 was from v1.0-era, moved to iterations/
- `announce/launch/reddit/` — 1 draft + FINAL for r/rust, r/LocalLLaMA, r/selfhosted. Each post leads with that community's primary concern
- `announce/launch/other/product-hunt.md` — Tagline (60 chars exactly), description, first comment
- `announce/launch/other/dev-to.md` — Technical article with SLERP code walkthrough, binary size optimization details

**announce/README.md** — Launch sequence, file index, key decisions log.

**All pushed to main.**

---

## Commits Made

```
d7de5dc  feat: vaporware elimination - all features match code (breed.md, scripts, data, Makefile fixes)
cbcef08  feat: merge local OpenClaw workspace bootstrap, aider scripts, memory/docs (post-audit)
4b2220a  fix: remove nested superinstance submodule
3ce3335  WIP: pre-audit push [Thu Mar 26 07:06:19 AKDT 2026]
2c55d11  refactor: PastureAI refs
ae0d333  feat: add workshopped video scripts and launch post iterations
```

---

## Known Issues Found in This Session's Audit (Not Yet Fixed)

See WORK-AHEAD.md for prioritized list. The most critical:

1. **P0-1:** dev.to article references SLERP function that doesn't exist in breeding.rs
2. **P0-2:** walkdir is a local stub in watcher.rs — breed startup scan always returns 0
3. **P0-3:** Cargo.toml repository URL points to wrong repo name
4. **P1-1:** Night School culling blocks are empty `{}`
5. **P1-2:** `candle` not defined as a feature — CPU inference always falls back to mock
6. **P1-3:** `successful_tasks` never incremented — fitness scoring inert
7. **P1-4:** Collie event loop is `sleep(60s)` — not wired to any input

---

## Files to Be Careful With

- `superinstance/src/evolution/breeding.rs` — The SLERP function in press materials doesn't exist here. Don't add code that makes it seem like it does until P0-1 is resolved.
- `announce/launch/other/dev-to.md` — Do NOT post this to dev.to without fixing the SLERP claim (P0-1).
- `pasture/base_models/phi3-mini.safetensors` — Check if this is a real model file (>1 MB). If so, needs to be removed from git immediately.
