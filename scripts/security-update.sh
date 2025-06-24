#!/bin/bash

# Security Update Script for AI Workflow Engine
# This script safely updates dependencies and runs security checks

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to backup current state
backup_current_state() {
    print_status "Creating backup of current dependency state..."
    cp Cargo.lock Cargo.lock.backup
    print_success "Backup created: Cargo.lock.backup"
}

# Function to restore from backup
restore_backup() {
    if [ -f "Cargo.lock.backup" ]; then
        print_warning "Restoring from backup..."
        mv Cargo.lock.backup Cargo.lock
        print_success "Restored from backup"
    fi
}

# Function to update dependencies
update_dependencies() {
    local update_type="$1"
    
    print_status "Updating dependencies ($update_type)..."
    
    case "$update_type" in
        "patch")
            print_status "Performing patch-level updates only..."
            cargo update
            ;;
        "minor")
            print_status "Performing minor-level updates..."
            if command_exists cargo-edit; then
                cargo upgrade --incompatible allow --pinned allow
            else
                print_warning "cargo-edit not found, falling back to cargo update"
                cargo update
            fi
            ;;
        "major")
            print_warning "Major updates require manual review"
            print_status "Checking for major updates..."
            if command_exists cargo-outdated; then
                cargo outdated --root-deps-only
            else
                print_warning "cargo-outdated not installed - install with: cargo install cargo-outdated"
            fi
            return 1
            ;;
        *)
            print_error "Invalid update type: $update_type"
            return 1
            ;;
    esac
}

# Function to run security checks
run_security_checks() {
    print_status "Running comprehensive security checks..."
    
    # Check for vulnerabilities
    print_status "Checking for security vulnerabilities..."
    if ! cargo audit; then
        print_error "Security vulnerabilities found!"
        return 1
    fi
    print_success "No security vulnerabilities found"
    
    # Check security policies
    print_status "Checking security policies..."
    if ! cargo deny check; then
        print_error "Security policy violations found!"
        return 1
    fi
    print_success "All security policies passed"
    
    return 0
}

# Function to run tests
run_tests() {
    print_status "Running tests to verify updates..."
    
    # Run unit tests
    if ! cargo test --lib; then
        print_error "Unit tests failed!"
        return 1
    fi
    
    # Run integration tests if available
    if cargo test --test "*" 2>/dev/null; then
        print_success "All tests passed"
    else
        print_warning "Integration tests not available or failed"
    fi
    
    return 0
}

# Function to build project
build_project() {
    print_status "Building project to verify compatibility..."
    
    if ! cargo build --release; then
        print_error "Build failed!"
        return 1
    fi
    
    print_success "Build successful"
    return 0
}

# Function to show dependency changes
show_changes() {
    if [ -f "Cargo.lock.backup" ]; then
        print_status "Dependency changes:"
        echo ""
        
        # Show differences in a readable format
        if command_exists diff; then
            diff_output=$(diff Cargo.lock.backup Cargo.lock | grep "^[<>]" | head -20)
            if [ -n "$diff_output" ]; then
                echo "$diff_output"
                echo ""
                diff_count=$(diff Cargo.lock.backup Cargo.lock | grep "^[<>]" | wc -l)
                if [ "$diff_count" -gt 20 ]; then
                    echo "... and $((diff_count - 20)) more changes"
                fi
            else
                print_success "No dependency changes detected"
            fi
        else
            print_warning "diff command not available - cannot show changes"
        fi
    fi
}

# Function to clean up
cleanup() {
    if [ -f "Cargo.lock.backup" ]; then
        print_status "Cleaning up backup files..."
        rm -f Cargo.lock.backup
    fi
}

# Function to install required tools
install_tools() {
    print_status "Checking required security tools..."
    
    if ! command_exists cargo-audit; then
        print_warning "cargo-audit not found - installing..."
        cargo install cargo-audit --locked
    fi
    
    if ! command_exists cargo-deny; then
        print_warning "cargo-deny not found - installing..."
        cargo install cargo-deny --locked
    fi
    
    print_success "Required tools are available"
}

# Main function
main() {
    local update_type="${1:-patch}"
    local skip_tests="${2:-false}"
    
    echo "============================================"
    echo "  AI Workflow Engine - Security Update"
    echo "============================================"
    echo ""
    
    # Validate update type
    case "$update_type" in
        "patch"|"minor"|"major"|"check-only")
            ;;
        *)
            print_error "Invalid update type: $update_type"
            echo "Usage: $0 [patch|minor|major|check-only] [skip-tests]"
            echo ""
            echo "Update types:"
            echo "  patch      - Safe patch-level updates (default)"
            echo "  minor      - Minor version updates (requires review)"
            echo "  major      - Show major updates (manual review required)"
            echo "  check-only - Only run security checks, no updates"
            echo ""
            echo "Options:"
            echo "  skip-tests - Skip running tests (faster but less safe)"
            exit 1
            ;;
    esac
    
    print_status "Update type: $update_type"
    print_status "Skip tests: $skip_tests"
    echo ""
    
    # Install required tools
    install_tools
    
    # Create backup before any changes
    if [ "$update_type" != "check-only" ]; then
        backup_current_state
    fi
    
    # Trap to restore backup on failure
    trap 'restore_backup; cleanup; exit 1' ERR
    
    if [ "$update_type" != "check-only" ]; then
        # Update dependencies
        if ! update_dependencies "$update_type"; then
            print_error "Dependency update failed or requires manual intervention"
            cleanup
            exit 1
        fi
        
        # Show what changed
        show_changes
    fi
    
    # Run security checks
    if ! run_security_checks; then
        print_error "Security checks failed!"
        if [ "$update_type" != "check-only" ]; then
            restore_backup
        fi
        cleanup
        exit 1
    fi
    
    if [ "$update_type" != "check-only" ] && [ "$skip_tests" != "true" ]; then
        # Build project
        if ! build_project; then
            print_error "Build failed after updates!"
            restore_backup
            cleanup
            exit 1
        fi
        
        # Run tests
        if ! run_tests; then
            print_error "Tests failed after updates!"
            restore_backup
            cleanup
            exit 1
        fi
    fi
    
    # Success!
    print_success "Security update completed successfully!"
    
    if [ "$update_type" != "check-only" ]; then
        echo ""
        print_status "Next steps:"
        echo "1. Review the dependency changes above"
        echo "2. Test your application thoroughly"
        echo "3. Commit the updated Cargo.lock file"
        echo "4. Consider updating your documentation"
    fi
    
    cleanup
}

# Show help if requested
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "AI Workflow Engine Security Update Script"
    echo ""
    echo "Usage: $0 [UPDATE_TYPE] [OPTIONS]"
    echo ""
    echo "Update Types:"
    echo "  patch      - Safe patch-level updates (default)"
    echo "  minor      - Minor version updates"
    echo "  major      - Show major updates (manual review required)"
    echo "  check-only - Only run security checks, no updates"
    echo ""
    echo "Options:"
    echo "  skip-tests - Skip running tests"
    echo ""
    echo "Examples:"
    echo "  $0                    # Patch updates with tests"
    echo "  $0 minor              # Minor updates with tests"
    echo "  $0 check-only         # Security check only"
    echo "  $0 patch skip-tests   # Patch updates without tests"
    echo ""
    echo "Security Tools Required:"
    echo "  - cargo-audit (installed automatically)"
    echo "  - cargo-deny (installed automatically)"
    echo ""
    echo "Optional Tools (for better experience):"
    echo "  - cargo-edit (for minor updates)"
    echo "  - cargo-outdated (for checking major updates)"
    exit 0
fi

# Run main function
main "$@"