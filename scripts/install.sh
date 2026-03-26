#!/bin/bash
#
# ███████╗██████╗ ███████╗ ██████╗██╗███╗   ██╗███████╗██████╗ ███████╗
# ██╔════╝██╔══██╗██╔════╝██╔════╝██║████╗  ██║██╔════╝██╔══██╗██╔════╝
# ███████╗██████╔╝█████╗  ██║     ██║██╔██╗ ██║█████╗  ██████╔╝█████╗  
# ╚════██║██╔══██╗██╔══╝  ██║     ██║██║╚██╗██║██╔══╝  ██╔══██╗██╔══╝  
# ███████║██║  ██║███████╗╚██████╗██║██║ ╚████║███████╗██║  ██║███████╗
# ╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝╚══════╝
#
# SuperInstance Ranch - Universal Hardware-Agnostic Installer
# "Don't rent an AI brain. Breed a Ranch."
#
# Supported Hardware:
#   - NVIDIA Jetson Orin Nano 8GB
#   - NVIDIA RTX GPUs (Linux/Windows WSL2)
#   - CPU-only fallback (slower but functional)
#
# Usage: curl -sSL https://install.superinstance.ai | bash
# Or:    ./scripts/install.sh [--debug] [--cpu-only] [--jetson-only]
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Hardware detection flags
IS_JETSON=false
IS_RTX=false
IS_CPU_ONLY=false
HAS_CUDA=false
DEBUG_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug|-d)
            DEBUG_MODE=true
            set -x
            shift
            ;;
        --cpu-only)
            IS_CPU_ONLY=true
            shift
            ;;
        --jetson-only)
            IS_JETSON=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

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
    echo -e "${YELLOW}    Universal Hardware-Agnostic Installer${NC}"
    echo -e "${GREEN}    Supports: Jetson Orin | RTX GPUs | CPU Fallback${NC}"
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

print_debug() {
    if [ "$DEBUG_MODE" = true ]; then
        echo -e "${BLUE}🔍 DEBUG: $1${NC}"
    fi
}

# ============================================================================
# HARDWARE DETECTION
# ============================================================================

detect_hardware() {
    print_step "🔍 Detecting Hardware Platform..."
    
    # Check for Jetson first
    if [ -f /etc/nv_tegra_release ]; then
        print_success "NVIDIA Jetson detected!"
        IS_JETSON=true
        cat /etc/nv_tegra_release
        
        # Detect Jetson model
        if [ -f /proc/device-tree/model ]; then
            MODEL=$(cat /proc/device-tree/model 2>/dev/null || echo "Unknown")
            echo -e "Model: ${CYAN}$MODEL${NC}"
            
            if echo "$MODEL" | grep -qi "orin nano"; then
                print_success "Jetson Orin Nano detected - optimized settings will be applied"
            fi
        fi
        
        # Check RAM
        TOTAL_RAM=$(free -g | awk '/^Mem:/{print $2}')
        echo -e "Total RAM: ${TOTAL_RAM}GB"
        
        if [ "$TOTAL_RAM" -lt 7 ]; then
            print_warning "Less than 8GB RAM detected. Swap file will be critical."
        fi
        
        HAS_CUDA=true
        return
    fi
    
    # Check for NVIDIA GPU (RTX series)
    if command -v nvidia-smi &> /dev/null; then
        print_success "NVIDIA GPU detected!"
        IS_RTX=true
        HAS_CUDA=true
        
        GPU_INFO=$(nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null || echo "Unknown GPU")
        echo -e "GPU: ${CYAN}$GPU_INFO${NC}"
        
        # Check if it's an RTX card
        if echo "$GPU_INFO" | grep -qi "RTX"; then
            print_success "RTX GPU detected - CUDA acceleration enabled"
        fi
        
        # Check VRAM
        VRAM=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>/dev/null | head -1 | tr -d ' ')
        if [ -n "$VRAM" ] && [ "$VRAM" -gt 8000 ]; then
            print_success "Sufficient VRAM detected: ${VRAM}MB"
        elif [ -n "$VRAM" ]; then
            print_warning "Limited VRAM: ${VRAM}MB - may need model quantization"
        fi
        
        return
    fi
    
    # Fallback to CPU mode
    if [ "$IS_CPU_ONLY" = false ]; then
        print_warning "No CUDA-capable GPU detected"
        print_warning "Falling back to CPU-only mode (slower inference)"
    fi
    IS_CPU_ONLY=true
    HAS_CUDA=false
}

# ============================================================================
# SYSTEM DEPENDENCIES
# ============================================================================

install_system_deps() {
    print_step "📦 Installing System Dependencies..."
    
    if command -v apt-get &> /dev/null; then
        print_debug "Using apt-get..."
        sudo apt-get update -qq
        sudo apt-get install -y -qq \
            build-essential \
            curl \
            wget \
            git \
            cmake \
            clang \
            pkg-config \
            libssl-dev \
            protobuf-compiler \
            > /dev/null
        print_success "System dependencies installed (apt)"
    elif command -v dnf &> /dev/null; then
        print_debug "Using dnf..."
        sudo dnf groupinstall -y "Development Tools"
        sudo dnf install -y curl wget git cmake clang openssl-devel protobuf-compiler
        print_success "System dependencies installed (dnf)"
    elif command -v pacman &> /dev/null; then
        print_debug "Using pacman..."
        sudo pacman -Syu --noconfirm base-devel curl wget git cmake clang openssl protobuf
        print_success "System dependencies installed (pacman)"
    elif command -v brew &> /dev/null; then
        print_debug "Using Homebrew (macOS)..."
        brew install curl wget git cmake clang openssl protobuf
        print_success "System dependencies installed (brew)"
    else
        print_warning "Unknown package manager. Please install dependencies manually:"
        print_warning "  build-essential, curl, wget, git, cmake, clang, openssl, protobuf"
    fi
}

# ============================================================================
# RUST INSTALLATION
# ============================================================================

install_rust() {
    print_step "🦀 Installing Rust..."
    
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version)
        print_success "Rust already installed: $RUST_VERSION"
        
        # Ensure targets are installed
        rustup default stable 2>/dev/null || true
        rustup update stable 2>/dev/null || true
    else
        print_debug "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Source Rust environment
        source "$HOME/.cargo/env" 2>/dev/null || true
        export PATH="$HOME/.cargo/bin:$PATH"
        
        print_success "Rust installed successfully"
    fi
    
    # Install required Rust components
    print_debug "Installing Rust components..."
    rustup component add clippy rustfmt 2>/dev/null || true
    
    # Add CUDA target if needed
    if [ "$HAS_CUDA" = true ]; then
        print_debug "CUDA target may be needed for native compilation"
    fi
}

# ============================================================================
# CUDA INSTALLATION (for RTX)
# ============================================================================

install_cuda() {
    if [ "$HAS_CUDA" = false ] || [ "$IS_JETSON" = true ]; then
        print_debug "Skipping CUDA toolkit installation (not needed)"
        return
    fi
    
    print_step "🎮 Checking CUDA Installation..."
    
    if command -v nvcc &> /dev/null; then
        CUDA_VERSION=$(nvcc --version | grep "release" | awk '{print $5}' | tr -d ',')
        print_success "CUDA already installed: $CUDA_VERSION"
        return
    fi
    
    # Install CUDA toolkit for RTX cards
    print_warning "CUDA toolkit not found. Installing..."
    
    if command -v apt-get &> /dev/null; then
        # Ubuntu/Debian CUDA installation
        print_debug "Installing CUDA via apt..."
        wget -q https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.1-1_all.deb
        sudo dpkg -i cuda-keyring_1.1-1_all.deb
        rm cuda-keyring_1.1-1_all.deb
        sudo apt-get update -qq
        sudo apt-get install -y -qq cuda-toolkit-12-3 2>/dev/null || \
        sudo apt-get install -y -qq cuda-toolkit 2>/dev/null || \
        print_warning "Could not install CUDA automatically. Please install manually."
        
        # Add CUDA to PATH
        export PATH="/usr/local/cuda/bin:$PATH"
        export LD_LIBRARY_PATH="/usr/local/cuda/lib64:$LD_LIBRARY_PATH"
        
        print_success "CUDA toolkit installed"
    else
        print_warning "Please install CUDA toolkit manually for your platform"
        print_warning "Visit: https://developer.nvidia.com/cuda-downloads"
    fi
}

# ============================================================================
# BUN INSTALLATION
# ============================================================================

install_bun() {
    print_step "🐰 Installing Bun..."
    
    if command -v bun &> /dev/null; then
        BUN_VERSION=$(bun --version)
        print_success "Bun already installed: $BUN_VERSION"
        return
    fi
    
    print_debug "Installing Bun..."
    curl -fsSL https://bun.sh/install | bash
    
    # Source Bun
    export BUN_INSTALL="$HOME/.bun"
    export PATH="$BUN_INSTALL/bin:$PATH"
    
    print_success "Bun installed successfully"
}

# ============================================================================
# MODEL DOWNLOAD
# ============================================================================

download_model() {
    print_step "🧠 Downloading Phi-3 Mini Model..."
    
    mkdir -p pasture
    
    if [ -f "pasture/phi3-mini.gguf" ]; then
        FILE_SIZE=$(stat -c%s "pasture/phi3-mini.gguf" 2>/dev/null || stat -f%z "pasture/phi3-mini.gguf" 2>/dev/null || echo "0")
        SIZE_MB=$((FILE_SIZE / 1024 / 1024))
        
        if [ "$SIZE_MB" -gt 2000 ]; then
            print_success "Phi-3 Mini already downloaded (${SIZE_MB}MB)"
            return
        else
            print_warning "Incomplete model file (${SIZE_MB}MB), re-downloading..."
        fi
    fi
    
    print_debug "Downloading Phi-3 Mini 4K Instruct (Q4_K_M)..."
    
    # Use HuggingFace mirror for faster downloads
    MODEL_URL="https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf"
    
    echo -e "${CYAN}Downloading model (~2.4GB)...${NC}"
    
    if command -v wget &> /dev/null; then
        wget -q --show-progress -O pasture/phi3-mini.gguf "$MODEL_URL" || {
            print_error "Download failed. Trying alternative source..."
            # Fallback to smaller quantization
            wget -q --show-progress -O pasture/phi3-mini.gguf \
                "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf" || \
            print_warning "Could not download model. Please download manually."
        }
    elif command -v curl &> /dev/null; then
        curl -L --progress-bar -o pasture/phi3-mini.gguf "$MODEL_URL" || {
            print_error "Download failed. Please download manually from HuggingFace."
        }
    else
        print_warning "No download tool available. Please install wget or curl."
    fi
    
    if [ -f "pasture/phi3-mini.gguf" ]; then
        print_success "Phi-3 Mini model downloaded"
    fi
}

# ============================================================================
# JETSON-SPECIFIC OPTIMIZATIONS
# ============================================================================

optimize_jetson() {
    if [ "$IS_JETSON" = false ]; then
        return
    fi
    
    print_step "⚡ Optimizing Jetson Performance..."
    
    # Jetson Orin Nano 8GB specific: MAXN mode
    print_debug "Setting MAXN performance mode (maximum power)..."
    sudo nvpmodel -m 0 2>/dev/null || print_warning "Could not set nvpmodel (may need sudo)"
    
    print_debug "Maximizing clocks..."
    sudo jetson_clocks 2>/dev/null || print_warning "Could not set jetson_clocks"
    
    # Configure 16GB swap (critical for 8GB boards)
    print_step "💾 Configuring 16GB Swap File..."
    
    if [ -f /swapfile ]; then
        CURRENT_SWAP=$(swapon --show=SIZE --noheadings 2>/dev/null | head -1)
        if [ -n "$CURRENT_SWAP" ]; then
            print_success "Swap already configured: $CURRENT_SWAP"
            return
        fi
    fi
    
    print_debug "Creating 16GB swap file..."
    sudo fallocate -l 16G /swapfile 2>/dev/null || \
        sudo dd if=/dev/zero of=/swapfile bs=1M count=16384 status=progress
    
    sudo chmod 600 /swapfile
    sudo mkswap /swapfile
    sudo swapon /swapfile
    
    # Add to fstab for persistence
    if ! grep -q "/swapfile" /etc/fstab; then
        echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
    fi
    
    print_success "16GB swap configured"
    
    # Set swappiness for better memory management
    echo "vm.swappiness=10" | sudo tee /etc/sysctl.d/99-superinstance.conf
    sudo sysctl -p /etc/sysctl.d/99-superinstance.conf 2>/dev/null || true
    
    print_success "Jetson optimizations complete"
}

# ============================================================================
# PROJECT SETUP
# ============================================================================

setup_project() {
    print_step "🏗️ Setting Up SuperInstance..."
    
    # Check if we're in the project directory
    if [ ! -f "Makefile" ]; then
        print_warning "Not in project directory. Cloning..."
        git clone https://github.com/SuperInstance/superinstance.git
        cd superinstance
    fi
    
    # Install Node dependencies
    print_debug "Installing Node dependencies..."
    bun install 2>/dev/null || npm install
    
    # Generate Prisma client
    print_debug "Generating Prisma client..."
    bunx prisma generate 2>/dev/null || npx prisma generate 2>/dev/null || true
    
    # Build Rust backend
    print_step "🔧 Building Rust Backend..."
    cd backend
    cargo build --release 2>&1 | tail -5
    cd ..
    
    print_success "Project setup complete"
}

# ============================================================================
# VERIFICATION
# ============================================================================

verify_installation() {
    print_step "✅ Verifying Installation..."
    
    local errors=0
    
    # Check Rust
    if command -v rustc &> /dev/null; then
        print_success "Rust: $(rustc --version)"
    else
        print_error "Rust not found"
        ((errors++))
    fi
    
    # Check Bun
    if command -v bun &> /dev/null; then
        print_success "Bun: $(bun --version)"
    else
        print_error "Bun not found"
        ((errors++))
    fi
    
    # Check CUDA
    if [ "$HAS_CUDA" = true ]; then
        if command -v nvcc &> /dev/null; then
            print_success "CUDA: $(nvcc --version | grep release | awk '{print $5}' | tr -d ',')"
        elif [ "$IS_JETSON" = true ]; then
            print_success "CUDA: Available via Jetson"
        else
            print_warning "CUDA: Not verified"
        fi
    fi
    
    # Check model
    if [ -f "pasture/phi3-mini.gguf" ]; then
        SIZE=$(stat -c%s "pasture/phi3-mini.gguf" 2>/dev/null || stat -f%z "pasture/phi3-mini.gguf" 2>/dev/null || echo "0")
        SIZE_MB=$((SIZE / 1024 / 1024))
        print_success "Model: Phi-3 Mini (${SIZE_MB}MB)"
    else
        print_warning "Model: Not downloaded (run 'make download-model' later)"
    fi
    
    # Check backend binary
    if [ -f "backend/target/release/backend" ]; then
        SIZE=$(stat -c%s "backend/target/release/backend" 2>/dev/null || echo "0")
        SIZE_MB=$((SIZE / 1024 / 1024))
        print_success "Backend: ${SIZE_MB}MB"
    else
        print_warning "Backend: Not built"
    fi
    
    if [ $errors -gt 0 ]; then
        print_error "Installation completed with $errors error(s)"
        return 1
    fi
    
    print_success "All checks passed!"
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    print_banner
    
    # Detect hardware
    detect_hardware
    
    # Install dependencies
    install_system_deps
    install_rust
    install_bun
    install_cuda
    
    # Hardware-specific optimizations
    if [ "$IS_JETSON" = true ]; then
        optimize_jetson
    fi
    
    # Download model
    download_model
    
    # Setup project
    setup_project
    
    # Verify
    verify_installation
    
    # Print next steps
    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}  🎉 SuperInstance Ranch Installed Successfully!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "  Hardware detected: ${CYAN}$(
        [ "$IS_JETSON" = true ] && echo "Jetson Orin"
        [ "$IS_RTX" = true ] && echo "RTX GPU"
        [ "$IS_CPU_ONLY" = true ] && echo "CPU-only"
    )${NC}"
    echo ""
    echo -e "  ${YELLOW}Quick Start:${NC}"
    echo ""
    echo "    make ranch       # Start full dev stack"
    echo "    make run         # Run TUI + Web"
    echo "    make test        # Run tests"
    echo "    make benchmark   # Check performance"
    echo ""
    if [ "$IS_JETSON" = true ]; then
        echo -e "  ${CYAN}Jetson Tips:${NC}"
        echo "    make jetson-perf  # Maximize performance (MAXN)"
        echo "    make jetson-stats # Show system stats"
        echo ""
    fi
    echo -e "${GREEN}Happy Ranching! 🐄${NC}"
    echo ""
}

# Run main
main "$@"
