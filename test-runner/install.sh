#!/bin/bash
# Installation script for AI Workflow Test Runner with UV

set -e

echo "🚀 AI Workflow Test Runner - UV Installation"
echo "==========================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if UV is installed
if ! command -v uv &> /dev/null; then
    echo -e "${YELLOW}UV is not installed. Installing UV...${NC}"
    
    # Detect OS
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
        echo "Windows detected. Please run this command in PowerShell:"
        echo "irm https://astral.sh/uv/install.ps1 | iex"
        exit 1
    else
        # macOS/Linux installation
        curl -LsSf https://astral.sh/uv/install.sh | sh
        
        # Add to PATH for current session
        export PATH="$HOME/.cargo/bin:$PATH"
        
        echo -e "${GREEN}✓ UV installed successfully${NC}"
        echo
        echo "Add UV to your PATH by adding this to your shell config:"
        echo 'export PATH="$HOME/.cargo/bin:$PATH"'
        echo
    fi
else
    echo -e "${GREEN}✓ UV is already installed${NC}"
fi

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
MIN_VERSION="3.8"

if [[ "$(printf '%s\n' "$MIN_VERSION" "$PYTHON_VERSION" | sort -V | head -n1)" != "$MIN_VERSION" ]]; then
    echo -e "${RED}Error: Python 3.8+ is required. Found: $PYTHON_VERSION${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Python $PYTHON_VERSION detected${NC}"
echo

# Create virtual environment
echo "Creating virtual environment..."
uv venv

# Detect activation script
if [[ -f ".venv/bin/activate" ]]; then
    ACTIVATE_SCRIPT=".venv/bin/activate"
elif [[ -f ".venv/Scripts/activate" ]]; then
    ACTIVATE_SCRIPT=".venv/Scripts/activate"
else
    echo -e "${RED}Error: Could not find venv activation script${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Virtual environment created${NC}"
echo

# Install the package
echo "Installing ai-workflow-test-runner..."
source "$ACTIVATE_SCRIPT"
uv pip install -e .

echo -e "${GREEN}✓ Package installed successfully${NC}"
echo

# Create test config if it doesn't exist
if [[ ! -f "test-config.yaml" && -f "test-config.example.yaml" ]]; then
    echo "Creating test-config.yaml from example..."
    cp test-config.example.yaml test-config.yaml
    echo -e "${GREEN}✓ Created test-config.yaml${NC}"
    echo
fi

# Installation complete
echo -e "${GREEN}🎉 Installation complete!${NC}"
echo
echo "To get started:"
echo "1. Activate the virtual environment:"
echo "   source $ACTIVATE_SCRIPT"
echo
echo "2. Run the test runner:"
echo "   workflow-test --help"
echo
echo "3. Run with default configuration:"
echo "   workflow-test"
echo
echo "4. Run with custom config:"
echo "   workflow-test --config test-config.yaml"
echo
echo "For more information, see README.md"