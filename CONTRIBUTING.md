# Contributing to SuperInstance

First off, thank you for considering contributing to SuperInstance! It's people like you that will make this Ranch grow.

## 🌱 Current Status

SuperInstance is in **Early Development (Alpha)**. The architecture is solid, but many features are still being built. This is a great time to get involved!

### What's Ready
- Core architecture (Collie orchestrator, species registry, routing)
- breed.md parser with hot-reload
- Species type system
- Installation scripts
- Documentation structure

### What Needs Help
- TensorRT-LLM integration
- Channel connectors (Discord, Telegram, WhatsApp)
- Night School evolution pipeline
- Web dashboard (Dioxus components)
- Testing infrastructure
- Example breeds and tutorials

## 🚀 Getting Started

### Prerequisites

- Rust 1.75+ (use `rustup`)
- A CUDA-capable device (Jetson Orin Nano recommended, or any NVIDIA GPU)
- Git

### Development Setup

```bash
# Clone the repository
git clone https://github.com/SuperInstance/superinstance.git
cd superinstance

# Build the project
cd superinstance
cargo build

# Run tests
cargo test

# Run the development version
cargo run
```

### Project Structure

```
superinstance/
├── src/
│   ├── collie/        # Border Collie orchestrator
│   ├── species/       # Agent implementations (Cattle, Sheep, etc.)
│   ├── genetics/      # breed.md parser, LoRA composition
│   ├── pasture/       # Model pool, LoRA manager
│   ├── channels/      # Discord, Telegram, WhatsApp connectors
│   ├── evolution/     # Night School, breeding algorithms
│   ├── web/           # Axum API + Dioxus dashboard
│   └── onboarding/    # Setup wizard
├── pasture/           # User-editable breed files
├── genetics/          # LoRA trait library
├── backend/           # Heavy integrations (TensorRT, CRDT)
└── docs/              # Tutorials and documentation
```

## 🎯 How to Contribute

### Reporting Bugs

If you find a bug, please open an issue with:
1. A clear title
2. Steps to reproduce
3. Expected behavior
4. Actual behavior
5. Your environment (OS, Rust version, hardware)

### Suggesting Features

Open an issue with the `enhancement` label. Describe:
1. What the feature would do
2. Why it's valuable for the Ranch
3. Any implementation ideas

### Contributing Code

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** with clear commit messages
3. **Add tests** if applicable
4. **Update documentation** if you change behavior
5. **Submit a pull request**

#### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Follow the existing code patterns
- Add comments for complex logic

#### Commit Messages

Format: `type: description`

Types:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only
- `style:` Formatting, no code change
- `refactor:` Code restructuring
- `test:` Adding tests
- `chore:` Maintenance

Examples:
- `feat: add Discord channel connector`
- `fix: handle missing breed.md gracefully`
- `docs: update TensorRT-LLM integration guide`

### Adding a New Species

1. Create a new file in `src/species/` (e.g., `llama.rs`)
2. Implement the `Species` trait
3. Register in `src/species/mod.rs`
4. Add routing logic in `src/collie/mod.rs`
5. Create example breed files in `pasture/`
6. Update documentation

### Adding a Channel Connector

1. Create a new file in `src/channels/` (e.g., `slack.rs`)
2. Implement the message handling
3. Add configuration to `.env.example`
4. Create setup documentation

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_species_routing

# Run with coverage
cargo tarpaulin --out Html
```

## 📚 Documentation

- Tutorials go in `docs/tutorials/`
- API documentation uses rustdoc comments
- Update README.md for user-facing changes

## 🏆 Recognition

Contributors will be:
- Listed in the README
- Credited in release notes
- Invited to the Ranch Discord (when available)

## 📜 Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something amazing together.

## ❓ Questions?

Open a discussion on GitHub or reach out to the maintainers.

---

Thank you for helping SuperInstance evolve! 🐄🐕
