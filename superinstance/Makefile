# SuperInstance Ranch Makefile
# "Don't rent an AI brain. Breed a Ranch."
#
# Usage: make <target>

.PHONY: install run dashboard breed night-school clean test help

# Default target
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo ""
	@echo "  ███████╗██████╗ ███████╗ ██████╗██╗███╗   ██╗███████╗██████╗ ███████╗"
	@echo "  ██╔════╝██╔══██╗██╔════╝██╔════╝██║████╗  ██║██╔════╝██╔══██╗██╔════╝"
	@echo "  ███████╗██████╔╝█████╗  ██║     ██║██╔██╗ ██║█████╗  ██████╔╝█████╗  "
	@echo "  ╚════██║██╔══██╗██╔══╝  ██║     ██║██║╚██╗██║██╔══╝  ██╔══██╗██╔══╝  "
	@echo "  ███████║██║  ██║███████╗╚██████╗██║██║ ╚████║███████╗██║  ██║███████╗"
	@echo "  ╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝╚══════╝"
	@echo ""
	@echo "  Available Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "    \033[36m%-18s\033[0m %s\n", $$1, $$2}'
	@echo ""

# ============================================
# INSTALLATION
# ============================================

install: ## Full installation (Jetson-optimized)
	@echo "🌱 Installing SuperInstance Ranch..."
	./scripts/install_jetson.sh

install-deps: ## Install system dependencies only
	@echo "📦 Installing dependencies..."
	sudo apt-get update
	sudo apt-get install -y build-essential curl wget git cmake clang
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	curl -fsSL https://bun.sh/install | bash

# ============================================
# RUNNING
# ============================================

run: ## Run the Ranch (TUI + Web)
	@echo "🐄 Starting the Ranch..."
	cargo run --release

run-web: ## Start web interface only
	@echo "🌐 Starting web interface..."
	bun run dev

run-tui: ## Start TUI dashboard only
	@echo "📊 Starting TUI dashboard..."
	cargo run --release --bin superinstance

# ============================================
# BREEDING & EVOLUTION
# ============================================

breed: ## Create a new breed interactively
	@echo "🧬 Opening breed creator..."
	cargo run --release --bin superinstance-onboard

night-school: ## Run Night School manually
	@echo "🌙 Running Night School..."
	cargo run --release -- --night-school

cull: ## Cull underperforming agents
	@echo "🗑️ Culling underperformers..."
	cargo run --release -- --cull

evolve: ## Run full evolution cycle
	@echo "🧬 Running full evolution cycle..."
	$(MAKE) night-school
	$(MAKE) cull

# ============================================
# MONITORING
# ============================================

status: ## Show ranch status
	@echo "📊 Ranch Status:"
	@cargo run --release -- --status

logs: ## View ranch logs
	tail -f pasture/ranch.log

stats: ## Show system stats
	@echo "📈 System Statistics:"
	@echo "VRAM: $(shell cat /sys/class/drm/card0/device/mem_info_vram_total 2>/dev/null || echo 'N/A')"
	@echo "Swap: $(shell free -h | grep Swap | awk '{print $$2}')"
	@echo "Uptime: $(shell uptime -p)"

# ============================================
# JETSON-SPECIFIC
# ============================================

jetson-perf: ## Maximize Jetson performance (MAXN mode)
	@echo "⚡ Setting MAXN performance mode..."
	sudo nvpmodel -m 0
	sudo jetson_clocks
	@echo "✅ Jetson running at maximum performance!"

jetson-power: ## Set Jetson to power-saving mode (15W)
	@echo "🔋 Setting 15W power mode..."
	sudo nvpmodel -m 1
	@echo "✅ Jetson in power-saving mode!"

jetson-stats: ## Show Jetson system stats
	@echo "📊 Jetson Stats:"
	tegrastats

jetson-swap: ## Configure 16GB swap file
	@echo "💾 Configuring 16GB swap..."
	sudo fallocate -l 16G /swapfile || sudo dd if=/dev/zero of=/swapfile bs=1M count=16384
	sudo chmod 600 /swapfile
	sudo mkswap /swapfile
	sudo swapon /swapfile
	echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
	@echo "✅ 16GB swap configured!"

# ============================================
# DEVELOPMENT
# ============================================

build: ## Build all components
	@echo "🏗️ Building..."
	cargo build --release

build-debug: ## Build in debug mode
	cargo build

clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	cargo clean
	rm -rf target/
	rm -rf node_modules/

test: ## Run tests
	cargo test --release

test-night-school: ## Run Night School tests only
	@echo "🧪 Running Night School tests..."
	cargo test --release night_school

test-api: ## Run API tests
	@echo "🧪 Running API tests..."
	cargo test --release api

test-coverage: ## Run tests with coverage
	cargo tarpaulin --out Html

benchmark: ## Run performance benchmarks
	cargo bench

lint: ## Run linters
	cargo clippy -- -D warnings

fmt: ## Format code
	cargo fmt

# ============================================
# DOCKER
# ============================================

docker-build: ## Build Docker image
	docker build -t superinstance:latest .

docker-run: ## Run in Docker
	docker run -it --rm --gpus all superinstance:latest

# ============================================
# UTILITIES
# ============================================

new-species: ## Create a new species template
	@read -p "Species name (e.g., my-agent): " name; \
	mkdir -p pasture/cattle/$$name; \
	cp templates/breed.md.template pasture/cattle/$$name/breed.md; \
	echo "✅ Created pasture/cattle/$$name/breed.md"

export-genes: ## Export gene pool to file
	tar -czf gene_pool_$$(date +%Y%m%d).tar.gz genetics/

import-genes: ## Import gene pool from file
	@read -p "Gene pool archive: " archive; \
	tar -xzf $$archive -C /

backup: ## Backup entire ranch configuration
	tar -czf ranch_backup_$$(date +%Y%m%d_%H%M%S).tar.gz \
		pasture/ genetics/ .env

restore: ## Restore from backup
	@read -p "Backup file: " backup; \
	tar -xzf $$backup
