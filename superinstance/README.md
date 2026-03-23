# SuperInstance - The Ranch Ecosystem

> *"I replaced my $200/month AI subscription with a breeding program on a $499 Jetson. It's not a Superintelligence; it's a loyal team that evolves every night."*

## What is SuperInstance?

SuperInstance is a **Ranch Ecosystem** that evolves into a "SuperInstance"—a system that becomes SuperUseful over time, rather than a static SuperIntelligence.

### The Core Philosophy

We are not building a Superintelligence (a monolithic, expensive god-in-a-box). We are building a **Ranch**—an ecosystem of specialized intelligences managed by a loyal Border Collie that starts capable but generic, and becomes **superuseful** through evolution, breeding, and culling.

## Architecture

### The Cast

| Role | Description |
|------|-------------|
| **🤠 The Cowboy (User)** | Sets high-level intent. Low bandwidth. |
| **🐕 The Border Collie (Foreman)** | Orchestrates resources, routes intent, herds livestock. |
| **🐄 The Livestock (Agents)** | Specialized LoRA Adapters that "possess" the Base Model. |

### The Species Taxonomy

| Species | Role | Collie Strategy | VRAM |
|---------|------|-----------------|------|
| 🐄 Cattle | Heavy LLMs - Deep reasoning, code gen | "Strong Eye" (Block, steady pressure) | 500MB |
| 🐑 Sheep | Classifiers - Voting/filtering | "The Wear" (Flock ensemble) | 50MB |
| 🦆 Ducks | Network/API - External calls | "Whistle Stop" (Async, fire-and-recall) | 100MB |
| 🐐 Goats | Navigators - File systems, debugging | "Balance" (Monitor depth) | 150MB |
| 🐗 Hogs | Hardware - GPIO, Sensors | "The Drive" (Real-time priority) | 10MB |
| 🐔 Chickens | Monitors - Heartbeats, alerts | "Free Range" (Constant pecking) | 5MB |
| 🐎 Horses | Pipelines - ETL, encoding | "Saddle Up" (Linear throughput) | 200MB |

## Speed Layers

1. **Muscle Memory (Reflex)** - Compiled CUDA Graphs for routines used >3x. Execution <1ms.
2. **Anticipation (Instinct)** - Speculative Decoding using "Shadow Pup" model.
3. **Cognition (Thought)** - LoRA Hot-Swap in <50ms.
4. **Escalation (Deliberation)** - Cloud API fallback with nightly distillation.

## Night School

At 02:00 daily:

1. **Evaluate** - Score agents on task performance
2. **Cull** - Remove agents with fitness < 0.4
3. **Breed** - Merge top performers using SLERP/TIES
4. **Distill** - Compress cloud knowledge into local weights
5. **Quarantine** - Test offspring on yesterday's tasks
6. **Promote** - Only if utility improves

## Project Structure

```
superinstance/
├── Cargo.toml              # Rust dependencies
├── manifesto.md            # The Philosophy
├── src/
│   ├── main.rs             # Entry: Initialize Ranch
│   ├── ranch.rs            # The Container
│   ├── collie/
│   │   ├── mod.rs          # Foreman Logic
│   │   ├── shepherd.rs     # Species-specific herding
│   │   ├── anticipation.rs # Shadow Pup (speculative decoding)
│   │   └── reflex.rs       # CUDA Graph cache
│   ├── species/
│   │   ├── mod.rs          # Species trait & registry
│   │   ├── cattle.rs       # Heavy LLM
│   │   ├── sheep.rs        # Ensemble voting
│   │   ├── duck.rs         # Async network
│   │   ├── goat.rs         # Navigation/debugging
│   │   ├── hog.rs          # Real-time hardware
│   │   ├── chicken.rs      # Monitoring
│   │   └── horse.rs        # Pipeline processing
│   ├── pasture/
│   │   ├── mod.rs          # Resource management
│   │   ├── lora_manager.rs # Hot-swap logic (<50ms)
│   │   └── model_pool.rs   # Base model + PagedAttention
│   ├── evolution/
│   │   ├── stud_book.rs    # SQLite genealogy
│   │   ├── breeding.rs     # LoRA merge logic
│   │   └── night_school.rs # Breeding cycle
│   └── dashboard.rs        # Terminal UI (ratatui)
├── pasture/
│   └── adapters/           # LoRA .safetensors by species
└── night_school/
    └── breed.py            # Python distillation scripts
```

## Building

```bash
# Build the project
cargo build --release

# Run the SuperInstance
cargo run --release
```

## Hardware Requirements

- **Platform:** Jetson Orin Nano 8GB
- **RAM Ceiling:** 6.5GB max usage
  - Base Model: 2.5GB
  - LoRA Pool: 1GB
  - KV Cache: 1GB
  - Collie Runtime: 0.5GB
  - Buffer: 1.5GB
- **Power:** DLA runs "Subconscious" at <2W

## Technology Stack

- **Runtime:** Rust (Tokio async)
- **Inference:** candle / llama.cpp (Rust bindings)
- **Models:** Mamba/SSM for Collie, Phi-3 for Livestock
- **UI:** ratatui (Terminal Dashboard)
- **Database:** SQLite (Stud Book)

## Demo: The Morning Routine

1. 🐔 **Chicken** detects motion (Event)
2. 🐑 **Sheep** flock triages emails
3. 🐐 **Goat** climbs into log file
4. 🦆 **Duck** fetches calendar data
5. 🐄 **Cattle** synthesizes the briefing

**Result:** <5 seconds, 0 Cloud calls, <6GB RAM

## License

MIT

---

*"The Collie doesn't try to be the smartest dog in the room. It tries to be the most useful."*
