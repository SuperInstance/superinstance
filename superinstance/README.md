# 🐄 SuperInstance: The Ranch Ecosystem

```
                    .-----------------------------------------.
                   /  "Don't rent an AI brain. Breed a Ranch." \
                  /      (SuperInstance v0.2.0)                 \
                 '---------------------------------------------'
                                 |
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
     ┌───▼───┐               ┌───▼───┐               ┌───▼───┐
     │ 🤠    │               │ 🐕    │               │ 🐄🦆🐐 │
     │COWBOY │               │COLLIE │               │LIVESTOCK
     │(User) │               │(AI)   │               │(LoRAs)│
     └───────┘               └───────┘               └───────┘
         │                       │                       │
         │    ┌──────────────────┼──────────────────┐    │
         │    │                  │                  │    │
         └────►  Intent ────────►│──► Route ────────►│───►│
              ┌──────────────────┴──────────────────┐    │
              │         THE PASTURE                 │    │
              │   (Base Model + LoRA Pool)          │◄───┘
              │                                     │
              │   ┌─────────────────────────────┐   │
              │   │ 🌙 NIGHT SCHOOL (02:00)     │   │
              │   │  Cull → Breed → Distill    │   │
              │   │  Your Ranch Evolves Daily  │   │
              │   └─────────────────────────────┘   │
              └─────────────────────────────────────┘
```

---

## 🎯 Why SuperInstance?

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     THE COST OF CLOUD AI                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   $$                                                                    │
│   200│                            ╭───────────────────                 │
│     │                           ╭╯                                     │
│   150│                        ╭╯                                       │
│     │                      ╭─╯    Cloud = $2,400/year                 │
│   100│                   ╭╯                                          │
│     │                 ╭─╯                                             │
│    50│              ╭─╯                                               │
│     │            ╭─╯                                                  │
│     0│──────────╯                                                     │
│     └───────────────────────────────────────────────────────► Tasks   │
│                                                                         │
│   VS                                                                    │
│                                                                         │
│   $$                                                                    │
│   500│ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─   │
│     │                                                                   │
│     │    ╭────────────────────────────────────────────────────────────│
│   0 │───╯     SuperInstance = $499 ONCE (Jetson hardware)             │
│     │             Cost stays FLAT while capability RISES              │
│     └───────────────────────────────────────────────────────► Tasks   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 📊 The Numbers Don't Lie

| Metric | Cloud Agent (OpenAI) | SuperInstance (Local) |
|:-------|:---------------------|:----------------------|
| **Latency** | 1.5s (Network + Infer) | **0.05s** (Pure Reflex) |
| **Cost (1000 emails)** | $5.00 | **$0.00** (Solar Power) |
| **Privacy** | Data leaves premises | **Never leaves the chip** |
| **Evolution** | Impossible (Black Box) | **Nightly Breeding** |
| **Offline** | No | **Yes (Bunker Mode)** |
| **VRAM (8GB Jetson)** | N/A | **5.5 GB** (Stable) |
| **Year Cost** | $2,400 | **$0** (after hardware) |

---

## 🧬 What Makes This Different?

### Not a Superintelligence. A SuperInstance.

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                    SUPERINTELLIGENCE vs SUPERINSTANCE                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  SUPERINTELLIGENCE (Them)          SUPERINSTANCE (Us)                   │
│  ════════════════════════════      ══════════════════════════           │
│                                                                         │
│  ┌─────────────────────┐          ┌─────────────────────┐              │
│  │    █████████████    │          │  🐄 ──── 🦆 ──── 🐐  │              │
│  │    █████████████    │          │   │      │      │    │              │
│  │    █████████████    │          │   └──────┼──────┘    │              │
│  │    █████████████    │          │          │           │              │
│  │    █████████████    │          │      🐕 Collie       │              │
│  └─────────────────────┘          └─────────────────────┘              │
│                                                                         │
│  • One monolithic model            • Specialized agent species         │
│  • Opaque, unchangeable            • Transparent Markdown DNA          │
│  • $200/month forever              • $499 once, evolves free           │
│  • Cloud dependency                • Works offline                     │
│  • No ownership                    • YOU own the genes                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🏔️ The Species Taxonomy

```
                         THE RANCH HIERARCHY
                              
              ┌──────────────────────────────────────┐
              │           🤠 THE COWBOY              │
              │         (You - Set Intent)           │
              └──────────────────┬───────────────────┘
                                 │
                                 ▼
              ┌──────────────────────────────────────┐
              │          🐕 THE BORDER COLLIE         │
              │    (Orchestrator - Herds Livestock)   │
              │                                       │
              │  Reflex (<1ms) → Anticipation (~10ms) │
              │     → Cognition (<50ms) → Cloud       │
              └──────────────────┬───────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │           │           │           │           │
         ▼           ▼           ▼           ▼           ▼
    ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
    │🐄 CATTLE│ │🦆  DUCK │ │🐐  GOAT │ │🐔CHICKEN│ │🐗  HOG  │
    │         │ │         │ │         │ │         │ │         │
    │ Heavy   │ │ Network │ │ Debug   │ │ Monitor │ │ Hardware│
    │Reasoning│ │ API     │ │ Navigate│ │ Watchdog│ │ GPIO    │
    │         │ │         │ │         │ │         │ │         │
    │500MB    │ │100MB    │ │150MB    │ │ 5MB     │ │ 10MB    │
    └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
         │           │           │           │           │
         └───────────┴───────────┴───────────┴───────────┘
                                 │
                    ┌────────────┴────────────┐
                    │   🌙 NIGHT SCHOOL       │
                    │   (02:00 Daily)         │
                    │                         │
                    │   1. Evaluate agents    │
                    │   2. Cull < 0.4 fitness │
                    │   3. Breed champions    │
                    │   4. Distill cloud      │
                    │   5. Quarantine test    │
                    │   6. Promote            │
                    └─────────────────────────┘
```

---

## ⚡ Quick Start (3 Commands)

```bash
# 1. Clone the Ranch
git clone https://github.com/SuperInstance/superinstance.git
cd superinstance

# 2. Install (Jetson-optimized, takes ~5 min)
make install

# 3. Start the Ranch
make run
```

**That's it.** Your Ranch is now running. Edit `pasture/cattle/email-cow-v1/breed.md` to customize your first agent.

---

## 🌱 Plant a Seed: The One-Command Install

```bash
# From any Jetson (or CUDA Linux box):
curl -sSL https://install.superinstance.ai | bash
```

### What the installer does:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     INSTALLATION FLOW                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 🔍 HARDWARE DETECTION                                               │
│     ├─ Detect Jetson Orin Nano 8GB                                     │
│     ├─ Set MAXN performance mode                                        │
│     └─ Configure 16GB swap (critical for 8GB board)                    │
│                                                                          │
│  2. ⚡ JETSON OPTIMIZATION                                               │
│     ├─ Disable GUI (headless mode) - saves 1GB RAM                     │
│     ├─ Maximize GPU clocks                                              │
│     └─ Enable DLA for power savings                                     │
│                                                                          │
│  3. 📦 DEPENDENCIES                                                      │
│     ├─ Rust (stable)                                                    │
│     ├─ Bun runtime                                                      │
│     ├─ Caddy reverse proxy                                              │
│     └─ TensorRT / CUDA                                                  │
│                                                                          │
│  4. 🏗️ BUILD                                                             │
│     ├─ Cargo build --release                                            │
│     ├─ Prisma generate                                                  │
│     └─ Download starter gene pool                                       │
│                                                                          │
│  5. 🧠 BASE MODEL                                                        │
│     └─ Phi-3 Mini (2.5GB) - optimized for 8GB VRAM                      │
│                                                                          │
│  🕐 Total time: ~5 minutes                                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🧬 Open Genomics: Edit DNA in Markdown

The breakthrough interface. Instead of binary config files, you edit `breed.md`:

```markdown
# 🐄 Breed: Email-Cow-v1

## 🧬 Genetic Composition (The Recipe)
| Gene Trait | Weight | Description |
| :--- | :--- | :--- |
| `polite_tone` | `0.8` | Strong influence on formal style |
| `json_output` | `0.1` | Light structure enforcement |
| `concise_style` | `0.5` | Medium brevity influence |

## 🧠 Genetic Code (System Prompt)
```
You are an Email Triage Specialist.
Prioritize emails from "Boss".
Always be concise.
```
```

**Save the file → Agent instantly evolves.** No restart needed.

### The Gene Pool: Periodic Table of Traits

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        GENE POOL CATALOG                                 │
├──────────────┬──────────────┬──────────────┬──────────────┬─────────────┤
│  STYLE       │  FUNCTION    │  DOMAIN      │  OUTPUT      │  TOOLS      │
├──────────────┼──────────────┼──────────────┼──────────────┼─────────────┤
│ polite_tone  │ json_output  │ rust_coder   │ concise      │ gmail_api   │
│ casual_tone  │ yaml_output  │ python_coder │ verbose      │ calendar    │
│ pirate_slang │ xml_output   │ sql_expert   │ structured   │ filesystem  │
│ formal_tone  │ markdown_out │ web_dev      │ bullet_pt    │ web_search  │
│ witty_style  │ code_block   │ data_science │ emoji_rich   │ slack_api   │
└──────────────┴──────────────┴──────────────┴──────────────┴─────────────┘

          Mix and match like LEGO blocks:
          polite_tone(0.8) + concise(0.5) = Professional Emailer
          pirate_slang(1.0) + emoji_rich(0.7) = Fun Chatbot
```

---

## 🔗 Channel Connectors

Connect your Ranch to any messaging platform:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CHANNEL ECOSYSTEM                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│                    ┌──────────────────┐                                 │
│                    │   SUPERINSTANCE  │                                 │
│                    │      RANCH       │                                 │
│                    └────────┬─────────┘                                 │
│                             │                                            │
│        ┌────────────────────┼────────────────────┐                      │
│        │         │          │          │         │                      │
│        ▼         ▼          ▼          ▼         ▼                      │
│   ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐               │
│   │Discord │ │Telegram│ │WhatsApp│ │ Slack  │ │Webhook │               │
│   │  🎮    │ │  📱    │ │  💬    │ │  💼   │ │  🔗   │               │
│   └────────┘ └────────┘ └────────┘ └────────┘ └────────┘               │
│                                                                          │
│   Two-way communication • Shared memory • Unified intent routing       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Setup Discord (2 minutes):

```bash
# 1. Copy example config
cp .env.example .env

# 2. Add your Discord bot token
echo "DISCORD_BOT_TOKEN=your_token_here" >> .env

# 3. Restart the Ranch
make run
```

---

## 🌙 Night School: Evolution While You Sleep

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     NIGHT SCHOOL (02:00 Daily)                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ PHASE 1: EVALUATE                              ⏱️ ~5 min          │  │
│  │ ─────────────────────────────────────────────────────────────── │  │
│  │ • Score each agent on yesterday's tasks                         │  │
│  │ • Calculate fitness: success_rate × impact_weight               │  │
│  │ • Identify champions (>0.8 fitness) and underperformers (<0.4)  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │                                           │
│                              ▼                                           │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ PHASE 2: CULL                                 ⏱️ ~1 min           │  │
│  │ ─────────────────────────────────────────────────────────────── │  │
│  │ • Remove agents with fitness < 0.4                              │  │
│  │ • Archive their weights to genetics/graveyard/                  │  │
│  │ • Free VRAM for new offspring                                   │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │                                           │
│                              ▼                                           │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ PHASE 3: BREED                                ⏱️ ~30 min          │  │
│  │ ─────────────────────────────────────────────────────────────── │  │
│  │ • Select top 20% as breeding stock (Sire + Dam pairs)           │  │
│  │ • Merge LoRA weights using SLERP/TIES algorithms               │  │
│  │ • Create 2-4 offspring per species                              │  │
│  │                                                                 │  │
│  │   Sire(0.85) ═══════╗                                          │  │
│  │                     ╠══► Offspring (est. 0.82)                  │  │
│  │   Dam(0.78)  ═══════╝                                          │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │                                           │
│                              ▼                                           │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ PHASE 4: DISTILL (if cloud used)             ⏱️ ~2 hours          │  │
│  │ ─────────────────────────────────────────────────────────────── │  │
│  │ • Extract knowledge from yesterday's cloud API calls            │  │
│  │ • Fine-tune local adapter on cloud responses                    │  │
│  │ • Reduce future cloud dependency                                │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │                                           │
│                              ▼                                           │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ PHASE 5: QUARANTINE & PROMOTE                ⏱️ ~10 min           │  │
│  │ ─────────────────────────────────────────────────────────────── │  │
│  │ • Test offspring on yesterday's tasks                           │  │
│  │ • Promote to production if fitness > parent average             │  │
│  │ • Log bloodline to Stud Book (SQLite)                           │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  🌅 Dawn: Your Ranch is smarter than yesterday.                         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🖥️ The Living Dashboard

```
┌─────────────────────────────────────────────────────────────────────────┐
│  SUPERINSTANCE RANCH - Day 14 - "Not a Superintelligence..."           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  SPECIES PANEL                          │  RESOURCE USAGE               │
│  ┌─────────────────────────────────┐    │  ┌─────────────────────────┐ │
│  │ 🐄 Cattle    x2 [ACTIVE]        │    │  │ GPU Memory: ████░ 45%  │ │
│  │ 🐑 Sheep     x5 [ACTIVE]        │    │  │ 2.9GB / 6.5GB          │ │
│  │ 🦆 Duck      x3 [ACTIVE]        │    │  ├─────────────────────────┤ │
│  │ 🐐 Goat      x2 [ACTIVE]        │    │  │ CPU: ██░░░░ 28%        │ │
│  │ 🐗 Hog       x1 [ACTIVE]        │    │  │ RAM: ███░░░ 3.2GB      │ │
│  │ 🐔 Chicken   x3 [ACTIVE]        │    │  └─────────────────────────┘ │
│  │ 🐎 Horse     x1 [IDLE]          │    │                              │
│  └─────────────────────────────────┘    │                              │
│                                          │  💰 SAVINGS: $127.50        │
│  ACTIVITY LOG                            │  Cloud calls avoided: 255   │
│  ┌─────────────────────────────────────┐│                              │
│  │ 10:23 🐔 Motion detected (perimeter)││  QUICK ACTIONS               │
│  │ 10:22 🦆 Fetched calendar data      ││  ┌─────────────────────────┐ │
│  │ 10:20 🐑 Consensus: 4 spam, 12 good ││  │ [D] Run Morning Routine │ │
│  │ 10:18 🐐 Navigated /var/log/syslog  ││  │ [N] Force Night School  │ │
│  │ 10:15 🐄 Briefing generated         ││  │ [B] Create New Breed    │ │
│  │ 10:10 🐔 Heartbeat OK               ││  │ [Q] Quit               │ │
│  └─────────────────────────────────────┘│  └─────────────────────────┘ │
│                                                                          │
│  Press ? for help                                                       │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 📚 Tutorials

| Tutorial | Description | Time |
|:---------|:------------|:-----|
| [Quick Start](docs/tutorials/00-quick-start.md) | Hello Ranch in 5 minutes | 5 min |
| [The Arrival](docs/tutorials/01-the-arrival.md) | Unboxing Jetson to first "Hello Cow" | 15 min |
| [Morning Routine](docs/tutorials/02-the-morning-routine.md) | Email Cattle + Calendar Duck + Summary | 20 min |
| [Night School](docs/tutorials/03-the-night-school.md) | Breed, distill, evolve | 30 min |
| [Smart Home](docs/tutorials/04-the-smart-home.md) | GPIO Hogs & Chickens | 25 min |
| [Jetson Optimization](docs/tutorials/05-jetson-optimization.md) | Swap, DLA, VRAM deep dive | 15 min |

---

## 🏗️ Architecture

```
superinstance/
├── 📁 src/                    # Rust Runtime (The Engine Room)
│   ├── 📁 collie/             # Border Collie orchestrator
│   │   ├── shepherd.rs        # Species-specific herding strategies
│   │   ├── anticipation.rs    # Shadow Pup (speculative decoding)
│   │   └── reflex.rs          # CUDA Graph cache (<1ms)
│   ├── 📁 genetics/           # Open Genomics Engine
│   │   ├── manifest.rs        # breed.md parser
│   │   ├── composer.rs        # LoRA weight merging
│   │   └── watcher.rs         # Hot-reload file watcher
│   ├── 📁 species/            # Agent implementations
│   │   ├── cattle.rs          # Heavy LLM (reasoning)
│   │   ├── sheep.rs           # Ensemble voting
│   │   ├── duck.rs            # Network/API
│   │   ├── goat.rs            # Navigation
│   │   ├── hog.rs             # Hardware/GPIO
│   │   ├── chicken.rs         # Monitoring
│   │   └── horse.rs           # Pipeline/ETL
│   ├── 📁 channels/           # Communication connectors
│   │   ├── discord.rs         # Discord bot
│   │   ├── telegram.rs        # Telegram bot
│   │   └── webhook.rs         # Generic webhook
│   ├── 📁 onboarding/         # Setup wizard
│   └── 📁 evolution/          # Night School
│       ├── stud_book.rs       # SQLite genealogy
│       ├── breeding.rs        # LoRA merge algorithms
│       └── night_school.rs    # Evolution cycle
│
├── 📁 pasture/                # Living Data (User Editable)
│   ├── 📁 cattle/             # Cattle breeds
│   │   └── email-cow-v1/
│   │       └── breed.md       # 👈 EDIT THIS!
│   └── 📁 base_models/        # Phi-3, Mamba, etc.
│
├── 📁 genetics/               # Gene Pool
│   └── 📁 traits/             # Atomic LoRA traits
│       ├── polite_tone/
│       ├── rust_coder/
│       └── json_output/
│
├── 📁 scripts/                # Installation
│   └── install_jetson.sh      # One-command install
│
├── 📁 docs/                   # Documentation
│   ├── 📁 tutorials/          # Step-by-step guides
│   ├── 📁 screenshots/        # Visual assets
│   └── 📁 gifs/               # Demo animations
│
├── 📁 examples/               # Pre-built Ranch configs
│   ├── 📁 consultancy/        # Email + Invoice + Research
│   ├── 📁 coder/              # Code + Git + Review
│   ├── 📁 smart-home/         # Sensors + Notifications
│   └── 📁 maker/              # CAD + Firmware + Docs
│
└── Makefile                   # 🌱 make install → make run
```

---

## 🔌 Integration with Other Agent Systems

SuperInstance can work alongside your existing agent frameworks:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TWO-WAY SYNERGY                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐            │
│  │  OpenClaw    │◄───►│ SuperInstance│◄───►│  LangChain   │            │
│  │   Gateway    │     │    Ranch     │     │   Chains     │            │
│  └──────────────┘     └──────────────┘     └──────────────┘            │
│         │                    │                    │                     │
│         │              Shared Memory              │                     │
│         │         (Gene Pool + Stud Book)         │                     │
│         │                    │                    │                     │
│         ▼                    ▼                    ▼                     │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     UNIFIED INTENT LAYER                        │   │
│  │   Cowboy (User) → Intent → Collie → Best Agent → Response      │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  Benefits:                                                               │
│  • Use OpenClaw channels with SuperInstance species                    │
│  • Share learned behaviors across frameworks                           │
│  • Distill cloud calls from any source into local LoRAs                │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Hardware Requirements

### Primary: Jetson Orin Nano 8GB

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    JETSON ORIN NANO 8GB                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ VRAM BUDGET (6.5GB max)                                         │   │
│  │ ─────────────────────                                           │   │
│  │                                                                 │   │
│  │ Base Model (Phi-3)    ████████████████████░░░░  2.5 GB         │   │
│  │ LoRA Pool             ████████░░░░░░░░░░░░░░░░  1.0 GB         │   │
│  │ KV Cache              ████████░░░░░░░░░░░░░░░░  1.0 GB         │   │
│  │ Collie Runtime        ████░░░░░░░░░░░░░░░░░░░░  0.5 GB         │   │
│  │ Buffer                ████████████░░░░░░░░░░░░  1.5 GB         │   │
│  │ ─────────────────────────────────────────────────────────────  │   │
│  │ TOTAL                 ████████████████████████  6.5 GB         │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  Power: <15W idle | Performance: MAXN mode                              │
│  Storage: 64GB SD card minimum (128GB recommended)                      │
│  Cooling: Active fan required                                           │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Secondary: Any CUDA Linux Box

- NVIDIA GPU with 6GB+ VRAM
- 16GB+ RAM
- Ubuntu 22.04+

---

## 📜 License

MIT License - Use it, breed it, make it yours.

---

## 🙏 Acknowledgments

Built with:
- 🦀 Rust + Tokio (runtime)
- 🎨 ratatui (TUI)
- 🤗 Phi-3 / Mamba (models)
- 🔥 TensorRT (inference)
- 🌐 Next.js + Prisma (web)

---

> *"The Collie doesn't try to be the smartest dog in the room. It tries to be the most useful."*

---

<div align="center">

**[🌱 Install Now](#-quick-start-3-commands) · [📚 Docs](docs/) · [💬 Discord](https://discord.gg/superinstance) · [🐦 Twitter](https://twitter.com/superinstance)**

Made with 🐄 by ranchers who got tired of paying $200/month for cloud AI.

</div>
