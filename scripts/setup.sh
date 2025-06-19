#!/bin/bash
# AI Workflow System - Master Setup Script
# This script sets up the complete development environment

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}AI Workflow System - Development Environment Setup${NC}"
echo "=================================================="
echo

# Functions for colored output
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_section() {
    echo
    echo -e "${BLUE}=== $1 ===${NC}"
    echo
}

# OS Detection
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/debian_version ]; then
            echo "debian"
        elif [ -f /etc/redhat-release ]; then
            echo "redhat"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

OS_TYPE=$(detect_os)
print_info "Detected OS: $OS_TYPE"

# Check if running with appropriate permissions
if [[ "$OS_TYPE" != "windows" ]] && [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root!"
   print_info "Please run as a regular user. The script will request sudo when needed."
   exit 1
fi

# Step 1: Install Prerequisites
print_section "Step 1: Installing Prerequisites"

install_rust() {
    if ! command -v rustc &> /dev/null; then
        print_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        print_status "Rust installed successfully"
    else
        print_status "Rust is already installed ($(rustc --version))"
        # Update Rust
        rustup update stable
    fi
}

install_postgresql() {
    if ! command -v psql &> /dev/null; then
        print_info "Installing PostgreSQL..."
        
        case $OS_TYPE in
            "macos")
                if command -v brew &> /dev/null; then
                    brew install postgresql
                    brew services start postgresql
                else
                    print_error "Homebrew not found. Please install from https://brew.sh"
                    exit 1
                fi
                ;;
            "debian")
                sudo apt-get update
                sudo apt-get install -y postgresql postgresql-contrib
                sudo systemctl start postgresql
                sudo systemctl enable postgresql
                ;;
            "redhat")
                sudo yum install -y postgresql postgresql-server postgresql-contrib
                sudo postgresql-setup initdb
                sudo systemctl start postgresql
                sudo systemctl enable postgresql
                ;;
            *)
                print_error "Automatic PostgreSQL installation not supported for $OS_TYPE"
                print_info "Please install PostgreSQL manually from https://www.postgresql.org/download/"
                exit 1
                ;;
        esac
        
        print_status "PostgreSQL installed successfully"
    else
        print_status "PostgreSQL is already installed ($(psql --version))"
    fi
}

install_python() {
    if ! command -v python3 &> /dev/null; then
        print_info "Installing Python 3..."
        
        case $OS_TYPE in
            "macos")
                if command -v brew &> /dev/null; then
                    brew install python@3.11
                else
                    print_error "Homebrew not found. Please install from https://brew.sh"
                    exit 1
                fi
                ;;
            "debian")
                sudo apt-get update
                sudo apt-get install -y python3.11 python3.11-venv python3-pip
                ;;
            "redhat")
                sudo yum install -y python3.11 python3.11-venv python3-pip
                ;;
            *)
                print_error "Automatic Python installation not supported for $OS_TYPE"
                print_info "Please install Python 3.11+ manually from https://www.python.org"
                exit 1
                ;;
        esac
        
        print_status "Python installed successfully"
    else
        print_status "Python is already installed ($(python3 --version))"
    fi
}

install_uv() {
    if ! command -v uv &> /dev/null; then
        print_info "Installing uv (Python package manager)..."
        curl -LsSf https://astral.sh/uv/install.sh | sh
        export PATH="$HOME/.local/bin:$PATH"
        print_status "uv installed successfully"
    else
        print_status "uv is already installed ($(uv --version))"
    fi
}

# Install all prerequisites
install_rust
install_postgresql
install_python
install_uv

# Step 2: Setup Database
print_section "Step 2: Setting up Database"

cd "$PROJECT_ROOT"
if [ -f "scripts/database-setup.sh" ]; then
    print_info "Running database setup script..."
    chmod +x scripts/database-setup.sh
    ./scripts/database-setup.sh
    print_status "Database setup completed"
else
    print_error "Database setup script not found!"
    exit 1
fi

# Step 3: Setup Environment Configuration
print_section "Step 3: Setting up Environment Configuration"

if [ ! -f ".env" ]; then
    if [ -f ".env.example" ]; then
        print_info "Creating .env file from template..."
        cp .env.example .env
        print_status ".env file created"
        print_warning "Please update .env with your configuration values"
    else
        print_error ".env.example not found!"
        exit 1
    fi
else
    print_status ".env file already exists"
fi

# Step 4: Install Rust Dependencies
print_section "Step 4: Installing Rust Dependencies"

print_info "Installing Rust dependencies..."
cargo fetch
print_status "Rust dependencies downloaded"

print_info "Building project..."
cargo build
if [ $? -eq 0 ]; then
    print_status "Project built successfully"
else
    print_error "Build failed! Please check the errors above"
    exit 1
fi

# Step 5: Setup Python MCP Servers
print_section "Step 5: Setting up Python MCP Servers"

if [ -d "scripts" ] && [ -f "scripts/pyproject.toml" ]; then
    cd scripts
    print_info "Installing Python dependencies for MCP servers..."
    
    if [ -f "pyproject.toml" ]; then
        uv sync
        print_status "Python dependencies installed"
    else
        print_warning "pyproject.toml not found in scripts/"
    fi
    
    cd "$PROJECT_ROOT"
else
    print_warning "MCP server dependencies not found in scripts directory"
fi

# Step 6: Run Validation
print_section "Step 6: Running Environment Validation"

if [ -f "scripts/validate-environment.sh" ]; then
    chmod +x scripts/validate-environment.sh
    print_info "Validating environment setup..."
    ./scripts/validate-environment.sh
else
    print_warning "Validation script not found, skipping validation"
fi

# Step 7: Optional Docker Setup
print_section "Step 7: Docker Setup (Optional)"

if command -v docker &> /dev/null; then
    print_status "Docker is installed"
    
    if docker info &> /dev/null; then
        print_status "Docker daemon is running"
        
        # Check for docker-compose
        if command -v docker-compose &> /dev/null; then
            print_status "Docker Compose is installed"
        else
            print_info "Installing Docker Compose..."
            case $OS_TYPE in
                "macos")
                    # Docker Desktop includes docker-compose
                    print_status "Docker Compose should be included with Docker Desktop"
                    ;;
                "linux"|"debian"|"redhat")
                    sudo curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
                    sudo chmod +x /usr/local/bin/docker-compose
                    print_status "Docker Compose installed"
                    ;;
            esac
        fi
    else
        print_warning "Docker daemon is not running. Start it to use containerized development"
    fi
else
    print_info "Docker not found. Install Docker Desktop for containerized development"
fi

# Step 8: Create Development Shortcuts
print_section "Step 8: Creating Development Shortcuts"

# Create a development helper script
cat > "$PROJECT_ROOT/dev.sh" << 'EOF'
#!/bin/bash
# Development helper script

case "$1" in
    "start")
        echo "Starting development servers..."
        ./scripts/start_test_servers.sh &
        cargo watch -x "run --bin workflow-engine"
        ;;
    "test")
        echo "Running tests..."
        cargo test
        ;;
    "mcp")
        echo "Starting MCP test servers..."
        ./scripts/start_test_servers.sh
        ;;
    "db")
        echo "Connecting to database..."
        source .env
        psql "$DATABASE_URL"
        ;;
    "logs")
        echo "Tailing logs..."
        tail -f logs/*.log
        ;;
    "clean")
        echo "Cleaning build artifacts..."
        cargo clean
        rm -rf target/
        ;;
    *)
        echo "Usage: ./dev.sh {start|test|mcp|db|logs|clean}"
        ;;
esac
EOF

chmod +x "$PROJECT_ROOT/dev.sh"
print_status "Created dev.sh helper script"

# Final Summary
print_section "Setup Complete! 🎉"

echo "Your AI Workflow System development environment is ready!"
echo
echo "Next steps:"
echo "1. Review and update your .env file with appropriate values"
echo "2. Run tests: ${GREEN}cargo test${NC}"
echo "3. Start the server: ${GREEN}cargo run --bin workflow-engine${NC}"
echo "4. Start MCP servers: ${GREEN}./scripts/start_test_servers.sh${NC}"
echo
echo "Helpful commands:"
echo "- ${BLUE}./dev.sh start${NC}   - Start development servers"
echo "- ${BLUE}./dev.sh test${NC}    - Run all tests"
echo "- ${BLUE}./dev.sh mcp${NC}     - Start MCP test servers"
echo "- ${BLUE}./dev.sh db${NC}      - Connect to database"
echo "- ${BLUE}./dev.sh logs${NC}    - Tail application logs"
echo
echo "Documentation:"
echo "- README.md         - Project overview and examples"
echo "- CLAUDE.md         - Development guidelines"
echo "- docs/             - Additional documentation"
echo
print_status "Happy coding! 🚀"