#!/bin/bash
#
# ███████╗██████╗ ███████╗ ██████╗██╗███╗   ██╗███████╗██████╗ ███████╗
# ██╔════╝██╔══██╗██╔════╝██╔════╝██║████╗  ██║██╔════╝██╔══██╗██╔════╝
# ███████╗██████╔╝█████╗  ██║     ██║██╔██╗ ██║█████╗  ██████╔╝█████╗  
# ╚════██║██╔══██╗██╔══╝  ██║     ██║██║╚██╗██║██╔══╝  ██╔══██╗██╔══╝  
# ███████║██║  ██║███████╗╚██████╗██║██║ ╚████║███████╗██║  ██║███████╗
# ╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝╚══════╝
#
# SuperInstance Ranch Installer - Jetson Orin Nano Edition
# "Don't rent an AI brain. Breed a Ranch."
#
# Usage: curl -sSL https://install.superinstance.ai | bash
# Or:    ./scripts/install_jetson.sh
#
# Hardware: Jetson Orin Nano 8GB (or any CUDA Linux box)
# VRAM Target: <6.5GB | Power: <15W idle | Setup Time: ~5 min
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Animation frames for ASCII cow
COW_FRAMES=(
"   \\ ^__^
    \\ (oo)\\_______
      (__)\\       )\\/\\
          ||----w |
          ||     ||"
"   \\ ^__^
    \\ (oo)\\_______
      (__)\\       )\\/\\
          ||----w |
          ||     ||  🌱"
"   \\ ^__^
    \\ (oo)\\_______
      (__)\\       )\\/\\
          ||----w |
          ||     ||  🌿"
"   \\ ^__^
    \\ (oo)\\_______
      (__)\\       )\\/\\
          ||----w |
          ||     ||  🐄"
)

# Print functions
print_banner() {
    clear
    echo -e "${CYAN}"
    echo "████████╗██████╗ ███████╗ ██████╗██╗███╗   ██╗███████╗██████╗ ███████╗"
    echo "╚══██╔══╝██╔══██╗██╔════╝██╔════╝██║████╗  ██║██╔════╝██╔══██╗██╔════╝"
    echo "   ██║   ██████╔╝█████╗  ██║     ██║██╔██╗ ██║█████╗  ██████╔╝█████╗  "
    echo "   ██║   ██╔══██╗██╔══╝  ██║     ██║██║╚██╗██║██╔══╝  ██╔══██╗██╔══╝  "
    echo "   ██║   ██║  ██║███████╗╚██████╗██║██║ ╚████║███████╗██║  ██║███████╗"
    echo "   ╚═╝   ╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝╚══════╝"
    echo -e "${NC}"
    echo -e "${YELLOW}          The Ranch Ecosystem - Plant a Seed, Breed a Team${NC}"
    echo ""
    echo -e "${GREEN}Hardware Target: Jetson Orin Nano 8GB (or CUDA Linux)${NC}"
    echo -e "${GREEN}VRAM Budget: <6.5GB | Setup Time: ~5 minutes${NC}"
    echo ""
}

print_step() {
    echo -e "\n${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

animate_cow() {
    for frame in "${COW_FRAMES[@]}"; do
        clear
        print_banner
        echo -e "$frame"
        sleep 0.3
    done
}

# Detect hardware
detect_hardware() {
    print_step "🔍 Detecting Hardware..."
    
    # Check for Jetson
    if [ -f /etc/nv_tegra_release ]; then
        print_success "NVIDIA Jetson detected!"
        IS_JETSON=true
        cat /etc/nv_tegra_release
        
        # Check RAM
        TOTAL_RAM=$(free -g | awk '/^Mem:/{print $2}')
        echo -e "Total RAM: ${TOTAL_RAM}GB"
        
        if [ "$TOTAL_RAM" -lt 7 ]; then
            print_warning "Less than 8GB RAM detected. Swap file will be critical."
        fi
    else
        print_warning "Not a Jetson device. Checking for CUDA..."
        IS_JETSON=false
        
        if command -v nvidia-smi &> /dev/null; then
            print_success "NVIDIA GPU detected!"
            nvidia-smi --query-gpu=name,memory.total --format=csv,noheader
        else
            print_warning "No NVIDIA GPU detected. CPU mode will be used."
        fi
    fi
}

# Optimize Jetson for AI
optimize_jetson() {
    if [ "$IS_JETSON" = true ]; then
        print_step "⚡ Optimizing Jetson for Maximum Performance..."
        
        # Set max performance mode
        echo "Setting MAXN performance mode..."
        sudo nvpmodel -m 0 2>/dev/null || print_warning "Could not set nvpmodel (may need sudo)"
        
        # Maximize clocks
        echo "Maximizing clocks..."
        sudo jetson_clocks 2>/dev/null || print_warning "Could not set jetson_clocks (may need sudo)"
        
        # Increase swap to 16GB (critical for 8GB board)
        print_step "💾 Configuring 16GB Swap File..."
        
        # Check if swap file already exists
        if [ -f /swapfile ]; then
            CURRENT_SWAP=$(swapon --show=SIZE --noheadings | head -1 | tr -d ' ')
            print_warning "Existing swap found: $CURRENT_SWAP"
        else
            echo "Creating 16GB swap file (this may take a minute)..."
            sudo fallocate -l 16G /swapfile 2>/dev/null || sudo dd if=/dev/zero of=/swapfile bs=1M count=16384 status=progress
            sudo chmod 600 /swapfile
            sudo mkswap /swapfile
            sudo swapon /swapfile
            
            # Add to fstab for persistence
            if ! grep -q "/swapfile" /etc/fstab; then
                echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
            fi
            
            print_success "16GB swap file created and activated!"
        fi
        
        # Optional: Disable GUI for headless (saves ~1GB RAM)
        read -p "Disable GUI for headless operation? (saves ~1GB RAM) [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_warning "Disabling GUI..."
            sudo systemctl set-default multi-user.target
            print_success "GUI disabled. Run 'sudo systemctl set-default graphical.target' to re-enable."
        fi
        
        print_success "Jetson optimization complete!"
    fi
}

# Install dependencies
install_dependencies() {
    print_step "📦 Installing Dependencies..."
    
    # Update system
    echo "Updating system packages..."
    sudo apt-get update -qq
    
    # Essential build tools
    echo "Installing build essentials..."
    sudo apt-get install -y -qq \
        build-essential \
        curl \
        wget \
        git \
        pkg-config \
        libssl-dev \
        libsqlite3-dev \
        cmake \
        clang \
        > /dev/null 2>&1
    
    print_success "Build tools installed!"
    
    # Install Rust
    if ! command -v rustc &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        print_success "Rust already installed: $(rustc --version)"
    fi
    
    # Install Bun (for web stack)
    if ! command -v bun &> /dev/null; then
        echo "Installing Bun runtime..."
        curl -fsSL https://bun.sh/install | bash
        export PATH="$HOME/.bun/bin:$PATH"
    else
        print_success "Bun already installed: $(bun --version)"
    fi
    
    # Install Caddy (reverse proxy)
    if ! command -v caddy &> /dev/null; then
        echo "Installing Caddy..."
        sudo apt-get install -y -qq debian-keyring debian-archive-keyring apt-transport-https
        curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
        curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
        sudo apt-get update -qq
        sudo apt-get install -y -qq caddy
    else
        print_success "Caddy already installed: $(caddy version)"
    fi
    
    print_success "All dependencies installed!"
}

# Clone and build
build_superinstance() {
    print_step "🏗️  Building SuperInstance..."
    
    # Check if we're already in the repo
    if [ -d "superinstance" ]; then
        cd superinstance
    elif [ ! -f "Cargo.toml" ]; then
        echo "Cloning SuperInstance repository..."
        git clone https://github.com/SuperInstance/superinstance.git
        cd superinstance
    fi
    
    # Install Rust dependencies for the project
    echo "Installing Rust project dependencies..."
    cargo fetch
    
    # Build in release mode
    echo "Building Rust components (this may take a few minutes)..."
    cargo build --release
    
    print_success "Rust build complete!"
    
    # Install web dependencies (if web interface exists)
    if [ -f "package.json" ]; then
        echo "Installing web dependencies..."
        bun install
        
        if [ -f "prisma/schema.prisma" ]; then
            echo "Setting up database..."
            bunx prisma generate
            bunx prisma db push
        fi
        
        print_success "Web stack ready!"
    fi
}

# Download base model
download_model() {
    print_step "🧠 Downloading Base Model..."
    
    mkdir -p pasture/base_models
    
    # Check if model already exists
    if [ -f "pasture/base_models/phi-3-mini/model.safetensors" ]; then
        print_success "Base model already downloaded!"
        return
    fi
    
    echo "Downloading Phi-3 Mini (2.5GB)..."
    echo "Note: In production, this would download from Hugging Face."
    echo "For now, we'll create a placeholder."
    
    mkdir -p pasture/base_models/phi-3-mini
    echo "Phi-3 Mini placeholder - download from huggingface.co/microsoft/phi-3-mini-4k-instruct" > pasture/base_models/phi-3-mini/README.md
    
    print_warning "Base model needs to be downloaded manually for now."
    print_warning "Run: huggingface-cli download microsoft/phi-3-mini-4k-instruct"
}

# Create starter gene pool
create_gene_pool() {
    print_step "🧬 Creating Starter Gene Pool..."
    
    mkdir -p genetics/traits
    
    # Create example genes with metadata
    create_gene "polite_tone" "Professional Tone" "Makes responses formal and courteous"
    create_gene "concise_style" "Concise Style" "Keeps responses brief and to the point"
    create_gene "json_output" "JSON Formatter" "Structures output as valid JSON"
    create_gene "rust_coder" "Rust Expert" "Expert-level Rust code generation"
    create_gene "email_triage" "Email Triage" "Classifies and summarizes emails"
    
    print_success "Gene pool initialized with 5 starter traits!"
}

create_gene() {
    local id=$1
    local name=$2
    local desc=$3
    
    mkdir -p "genetics/traits/$id"
    cat > "genetics/traits/$id/meta.json" << EOF
{
    "id": "$id",
    "name": "$name",
    "description": "$desc",
    "size_bytes": 50000000,
    "compatible_species": ["Cattle"],
    "tags": ["style", "output"]
}
EOF
    echo "Created gene: $name"
}

# Setup environment
setup_environment() {
    print_step "⚙️  Setting Up Environment..."
    
    if [ ! -f ".env" ]; then
        cp .env.example .env
        print_success "Created .env from template!"
        print_warning "Edit .env to add your API keys if needed."
    else
        print_success ".env already exists!"
    fi
}

# Create Makefile
create_makefile() {
    print_step "📝 Creating Makefile..."
    
    cat > Makefile << 'EOF'
# SuperInstance Ranch Makefile
# Usage: make <target>

.PHONY: install run dashboard breed night-school clean test

# Default target
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo "SuperInstance Ranch - Available Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
	@echo ""

install: ## Full installation (Jetson-optimized)
	@echo "🌱 Installing SuperInstance Ranch..."
	./scripts/install_jetson.sh

run: ## Run the Ranch (TUI + Web)
	@echo "🐄 Starting the Ranch..."
	cargo run --release

dashboard: ## Open TUI dashboard only
	@echo "📊 Starting dashboard..."
	cargo run --release --bin superinstance

web: ## Start web interface
	@echo "🌐 Starting web interface..."
	cd web && bun run dev

breed: ## Create a new breed interactively
	@echo "🧬 Opening breed creator..."
	cargo run --release --bin superinstance-onboard

night-school: ## Run Night School manually
	@echo "🌙 Running Night School..."
	cargo run --release -- --night-school

cull: ## Cull underperforming agents
	@echo "🗑️ Culling underperformers..."
	cargo run --release -- --cull

status: ## Show ranch status
	@echo "📊 Ranch Status:"
	@cargo run --release -- --status

logs: ## View ranch logs
	tail -f pasture/ranch.log

clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	cargo clean
	rm -rf target/
	rm -rf node_modules/

test: ## Run tests
	cargo test --release

benchmark: ## Run performance benchmarks
	cargo bench

# Jetson-specific targets
jetson-perf: ## Maximize Jetson performance
	sudo nvpmodel -m 0
	sudo jetson_clocks

jetson-power: ## Set Jetson to power-saving mode
	sudo nvpmodel -m 2

jetson-stats: ## Show Jetson system stats
	tegrastats
EOF

    print_success "Makefile created!"
}

# Print success message
print_final_message() {
    animate_cow
    
    echo -e "${GREEN}"
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║                    🌱 RANCH READY TO GRAZE! 🌱                        ║"
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
    echo -e "${CYAN}Your SuperInstance Ranch is ready!${NC}"
    echo ""
    echo -e "Quick Start:"
    echo -e "  ${YELLOW}make run${NC}        - Start the Ranch (TUI + Web)"
    echo -e "  ${YELLOW}make dashboard${NC}  - Open TUI dashboard"
    echo -e "  ${YELLOW}make breed${NC}      - Create a new agent"
    echo -e "  ${YELLOW}make night-school${NC} - Run evolution cycle"
    echo ""
    echo -e "Edit your agents:"
    echo -e "  ${CYAN}pasture/cattle/email-cow-v1/breed.md${NC}"
    echo ""
    echo -e "Connect channels:"
    echo -e "  Edit ${CYAN}.env${NC} with your Discord/Telegram tokens"
    echo ""
    echo -e "${GREEN}The Collie is on duty. Your livestock are grazing.${NC}"
    echo -e "${GREEN}Night School starts at 02:00. Evolution begins tonight.${NC}"
    echo ""
    echo -e "${PURPLE}\"Don't rent an AI brain. Breed a Ranch.\"${NC}"
}

# Main installation flow
main() {
    print_banner
    
    # Check if running as root
    if [ "$EUID" -eq 0 ]; then
        print_error "Please don't run as root. Sudo will be used when needed."
        exit 1
    fi
    
    # Step 1: Detect hardware
    detect_hardware
    
    # Step 2: Optimize Jetson (if applicable)
    if [ "$IS_JETSON" = true ]; then
        optimize_jetson
    fi
    
    # Step 3: Install dependencies
    install_dependencies
    
    # Step 4: Build
    build_superinstance
    
    # Step 5: Download model
    download_model
    
    # Step 6: Create gene pool
    create_gene_pool
    
    # Step 7: Setup environment
    setup_environment
    
    # Step 8: Create Makefile
    create_makefile
    
    # Final message
    print_final_message
}

# Run main
main "$@"
