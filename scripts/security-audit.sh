#!/bin/bash

# Security Audit Script for AI Workflow Engine
# This script performs comprehensive security auditing of dependencies and generates reports

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
REPORT_DIR="$PROJECT_ROOT/target/security-reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="$REPORT_DIR/security-audit-$TIMESTAMP.md"

# Ensure report directory exists
mkdir -p "$REPORT_DIR"

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

# Function to install required tools
install_tools() {
    print_status "Checking and installing required security tools..."
    
    if ! command_exists cargo-audit; then
        print_status "Installing cargo-audit..."
        cargo install cargo-audit --locked
    fi
    
    if ! command_exists cargo-deny; then
        print_status "Installing cargo-deny..."
        cargo install cargo-deny --locked
    fi
    
    if ! command_exists cargo-outdated; then
        print_status "Installing cargo-outdated..."
        cargo install cargo-outdated --locked
    fi
    
    print_success "All required tools are installed"
}

# Function to generate report header
generate_report_header() {
    cat > "$REPORT_FILE" << EOF
# Security Audit Report

**Generated:** $(date)  
**Project:** AI Workflow Engine  
**Version:** $(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | cut -d'"' -f2)  
**Audit Type:** Comprehensive Security Audit

## Executive Summary

This report provides a comprehensive security audit of the AI Workflow Engine project's dependencies and configurations.

---

EOF
}

# Function to run security vulnerability scan
run_vulnerability_scan() {
    print_status "Running security vulnerability scan..."
    
    echo "## Security Vulnerabilities" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    if cargo audit --format json > "$REPORT_DIR/audit-$TIMESTAMP.json" 2>/dev/null; then
        local vuln_count=$(jq '.vulnerabilities.count // 0' "$REPORT_DIR/audit-$TIMESTAMP.json")
        
        if [ "$vuln_count" -eq 0 ]; then
            echo "✅ **No security vulnerabilities found**" >> "$REPORT_FILE"
            print_success "No security vulnerabilities detected"
        else
            echo "❌ **$vuln_count security vulnerabilities found**" >> "$REPORT_FILE"
            print_error "Found $vuln_count security vulnerabilities"
            
            echo "" >> "$REPORT_FILE"
            echo "### Vulnerability Details" >> "$REPORT_FILE"
            echo '```json' >> "$REPORT_FILE"
            jq '.vulnerabilities.list' "$REPORT_DIR/audit-$TIMESTAMP.json" >> "$REPORT_FILE" 2>/dev/null || echo "Error parsing vulnerabilities" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
        fi
    else
        print_error "Failed to run vulnerability scan"
        echo "❌ **Vulnerability scan failed**" >> "$REPORT_FILE"
    fi
    
    echo "" >> "$REPORT_FILE"
}

# Function to check for unmaintained dependencies
check_unmaintained() {
    print_status "Checking for unmaintained dependencies..."
    
    echo "## Unmaintained Dependencies" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    if cargo audit --stale --format json > "$REPORT_DIR/stale-$TIMESTAMP.json" 2>/dev/null; then
        local stale_count=$(jq '.warnings.count // 0' "$REPORT_DIR/stale-$TIMESTAMP.json")
        
        if [ "$stale_count" -eq 0 ]; then
            echo "✅ **No unmaintained dependencies found**" >> "$REPORT_FILE"
            print_success "No unmaintained dependencies detected"
        else
            echo "⚠️ **$stale_count unmaintained dependencies found**" >> "$REPORT_FILE"
            print_warning "Found $stale_count unmaintained dependencies"
            
            echo "" >> "$REPORT_FILE"
            echo "### Unmaintained Dependency Details" >> "$REPORT_FILE"
            echo '```json' >> "$REPORT_FILE"
            jq '.warnings.list' "$REPORT_DIR/stale-$TIMESTAMP.json" >> "$REPORT_FILE" 2>/dev/null || echo "Error parsing unmaintained dependencies" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
        fi
    else
        print_warning "Failed to run unmaintained dependency check"
        echo "⚠️ **Unmaintained dependency check failed**" >> "$REPORT_FILE"
    fi
    
    echo "" >> "$REPORT_FILE"
}

# Function to run cargo deny checks
run_policy_checks() {
    print_status "Running security policy checks..."
    
    echo "## Security Policy Compliance" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # Check advisories
    echo "### Security Advisories" >> "$REPORT_FILE"
    if cargo deny check advisories > "$REPORT_DIR/deny-advisories-$TIMESTAMP.txt" 2>&1; then
        echo "✅ **Passed** - No security advisories violations" >> "$REPORT_FILE"
        print_success "Security advisories check passed"
    else
        echo "❌ **Failed** - Security advisories violations found" >> "$REPORT_FILE"
        print_error "Security advisories check failed"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        cat "$REPORT_DIR/deny-advisories-$TIMESTAMP.txt" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
    fi
    echo "" >> "$REPORT_FILE"
    
    # Check licenses
    echo "### License Compliance" >> "$REPORT_FILE"
    if cargo deny check licenses > "$REPORT_DIR/deny-licenses-$TIMESTAMP.txt" 2>&1; then
        echo "✅ **Passed** - All licenses are compliant" >> "$REPORT_FILE"
        print_success "License compliance check passed"
    else
        echo "❌ **Failed** - License compliance violations found" >> "$REPORT_FILE"
        print_error "License compliance check failed"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        cat "$REPORT_DIR/deny-licenses-$TIMESTAMP.txt" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
    fi
    echo "" >> "$REPORT_FILE"
    
    # Check bans
    echo "### Banned Dependencies" >> "$REPORT_FILE"
    if cargo deny check bans > "$REPORT_DIR/deny-bans-$TIMESTAMP.txt" 2>&1; then
        echo "✅ **Passed** - No banned dependencies found" >> "$REPORT_FILE"
        print_success "Banned dependencies check passed"
    else
        echo "❌ **Failed** - Banned dependencies found" >> "$REPORT_FILE"
        print_error "Banned dependencies check failed"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        cat "$REPORT_DIR/deny-bans-$TIMESTAMP.txt" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
    fi
    echo "" >> "$REPORT_FILE"
    
    # Check sources
    echo "### Source Verification" >> "$REPORT_FILE"
    if cargo deny check sources > "$REPORT_DIR/deny-sources-$TIMESTAMP.txt" 2>&1; then
        echo "✅ **Passed** - All sources are verified" >> "$REPORT_FILE"
        print_success "Source verification check passed"
    else
        echo "❌ **Failed** - Source verification issues found" >> "$REPORT_FILE"
        print_error "Source verification check failed"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        cat "$REPORT_DIR/deny-sources-$TIMESTAMP.txt" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
    fi
    echo "" >> "$REPORT_FILE"
}

# Function to check for outdated dependencies
check_outdated() {
    print_status "Checking for outdated dependencies..."
    
    echo "## Outdated Dependencies" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    if cargo outdated --format json > "$REPORT_DIR/outdated-$TIMESTAMP.json" 2>/dev/null; then
        local outdated_count=$(jq '.dependencies | length' "$REPORT_DIR/outdated-$TIMESTAMP.json" 2>/dev/null || echo "0")
        
        if [ "$outdated_count" -eq 0 ]; then
            echo "✅ **All dependencies are up to date**" >> "$REPORT_FILE"
            print_success "All dependencies are up to date"
        else
            echo "📦 **$outdated_count dependencies have updates available**" >> "$REPORT_FILE"
            print_warning "Found $outdated_count outdated dependencies"
            
            echo "" >> "$REPORT_FILE"
            echo "### Outdated Dependency Details" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
            cargo outdated --color never >> "$REPORT_FILE" 2>/dev/null || echo "Error getting outdated dependencies" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
        fi
    else
        print_warning "Failed to check for outdated dependencies"
        echo "⚠️ **Outdated dependency check failed**" >> "$REPORT_FILE"
    fi
    
    echo "" >> "$REPORT_FILE"
}

# Function to generate dependency tree analysis
analyze_dependency_tree() {
    print_status "Analyzing dependency tree..."
    
    echo "## Dependency Tree Analysis" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # Count total dependencies
    local total_deps=$(cargo tree --prefix none | wc -l)
    echo "**Total Dependencies:** $total_deps" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # Check for duplicate dependencies
    echo "### Duplicate Dependencies" >> "$REPORT_FILE"
    if cargo tree --duplicates > "$REPORT_DIR/duplicates-$TIMESTAMP.txt" 2>&1; then
        if [ -s "$REPORT_DIR/duplicates-$TIMESTAMP.txt" ]; then
            echo "⚠️ **Duplicate dependencies found**" >> "$REPORT_FILE"
            print_warning "Found duplicate dependencies"
            echo "" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
            cat "$REPORT_DIR/duplicates-$TIMESTAMP.txt" >> "$REPORT_FILE"
            echo '```' >> "$REPORT_FILE"
        else
            echo "✅ **No duplicate dependencies found**" >> "$REPORT_FILE"
            print_success "No duplicate dependencies detected"
        fi
    else
        echo "⚠️ **Failed to analyze duplicate dependencies**" >> "$REPORT_FILE"
    fi
    
    echo "" >> "$REPORT_FILE"
}

# Function to generate recommendations
generate_recommendations() {
    print_status "Generating security recommendations..."
    
    echo "## Security Recommendations" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # Check for deprecated packages
    if cargo tree --format "{p}" | grep -i "deprecated" > "$REPORT_DIR/deprecated-$TIMESTAMP.txt" 2>/dev/null; then
        echo "### Deprecated Dependencies" >> "$REPORT_FILE"
        echo "⚠️ **Action Required:** The following deprecated dependencies should be replaced:" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        cat "$REPORT_DIR/deprecated-$TIMESTAMP.txt" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        print_warning "Deprecated dependencies found - review required"
    fi
    
    echo "### General Recommendations" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "1. **Regular Updates**: Update dependencies monthly to get security patches" >> "$REPORT_FILE"
    echo "2. **Security Monitoring**: Enable GitHub Dependabot for automated security updates" >> "$REPORT_FILE"
    echo "3. **Audit Frequency**: Run security audits weekly during development" >> "$REPORT_FILE"
    echo "4. **Policy Enforcement**: Ensure \`cargo deny\` passes in CI/CD pipeline" >> "$REPORT_FILE"
    echo "5. **Documentation**: Keep security documentation updated with findings" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    echo "### Next Steps" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "- [ ] Review all security findings in this report" >> "$REPORT_FILE"
    echo "- [ ] Address any critical or high-severity vulnerabilities immediately" >> "$REPORT_FILE"
    echo "- [ ] Plan migration away from deprecated dependencies" >> "$REPORT_FILE"
    echo "- [ ] Update dependency versions where possible" >> "$REPORT_FILE"
    echo "- [ ] Schedule next security audit for $(date -d '+1 week' +%Y-%m-%d)" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
}

# Function to generate summary
generate_summary() {
    print_status "Generating audit summary..."
    
    echo "---" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "## Audit Complete" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "**Report Location:** \`$REPORT_FILE\`" >> "$REPORT_FILE"
    echo "**Detailed Logs:** \`$REPORT_DIR/*-$TIMESTAMP.*\`" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "For questions about this security audit, please contact the security team." >> "$REPORT_FILE"
}

# Main execution
main() {
    print_status "Starting comprehensive security audit..."
    echo "Report will be saved to: $REPORT_FILE"
    echo ""
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Install required tools
    install_tools
    
    # Generate report
    generate_report_header
    run_vulnerability_scan
    check_unmaintained
    run_policy_checks
    check_outdated
    analyze_dependency_tree
    generate_recommendations
    generate_summary
    
    print_success "Security audit completed successfully!"
    print_status "Report saved to: $REPORT_FILE"
    
    # Display quick summary
    echo ""
    echo "=== QUICK SUMMARY ==="
    if grep -q "❌" "$REPORT_FILE"; then
        print_error "Security issues found - review required"
        exit 1
    elif grep -q "⚠️" "$REPORT_FILE"; then
        print_warning "Warnings found - review recommended"
        exit 0
    else
        print_success "All security checks passed"
        exit 0
    fi
}

# Run main function
main "$@"