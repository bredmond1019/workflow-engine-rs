#!/usr/bin/env python3
"""
AI Workflow System - Setup Validation Script
This script validates that the development environment is correctly configured
"""

import sys
import os
import subprocess
import json
import time
from pathlib import Path

# Colors for terminal output
class Colors:
    GREEN = '\033[0;32m'
    RED = '\033[0;31m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color

def print_status(message):
    print(f"{Colors.GREEN}✓{Colors.NC} {message}")

def print_error(message):
    print(f"{Colors.RED}✗{Colors.NC} {message}")

def print_warning(message):
    print(f"{Colors.YELLOW}⚠{Colors.NC} {message}")

def print_info(message):
    print(f"{Colors.BLUE}ℹ{Colors.NC} {message}")

def print_section(title):
    print(f"\n{Colors.BLUE}=== {title} ==={Colors.NC}\n")

def check_command(command, min_version=None):
    """Check if a command exists and optionally check its version"""
    try:
        result = subprocess.run([command, "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            version_output = result.stdout.strip()
            if min_version:
                # Extract version number (basic implementation)
                import re
                version_match = re.search(r'(\d+\.\d+(?:\.\d+)?)', version_output)
                if version_match:
                    version = version_match.group(1)
                    return True, version
            return True, version_output
        return False, None
    except FileNotFoundError:
        return False, None

def check_rust_setup():
    """Validate Rust installation and project compilation"""
    print_section("Rust Environment")
    
    # Check Rust installation
    rust_installed, rust_version = check_command("rustc")
    if rust_installed:
        print_status(f"Rust installed: {rust_version}")
        
        # Check minimum version (1.75+)
        import re
        version_match = re.search(r'(\d+)\.(\d+)', rust_version)
        if version_match:
            major = int(version_match.group(1))
            minor = int(version_match.group(2))
            if major > 1 or (major == 1 and minor >= 75):
                print_status("Rust version meets requirements (1.75+)")
            else:
                print_error(f"Rust version too old. Found {major}.{minor}, need 1.75+")
                return False
    else:
        print_error("Rust not installed")
        return False
    
    # Check Cargo
    cargo_installed, cargo_version = check_command("cargo")
    if cargo_installed:
        print_status(f"Cargo installed: {cargo_version}")
    else:
        print_error("Cargo not installed")
        return False
    
    # Check if project compiles
    print_info("Checking if project compiles...")
    compile_result = subprocess.run(["cargo", "check", "--quiet"], capture_output=True)
    if compile_result.returncode == 0:
        print_status("Project compiles successfully")
    else:
        print_error("Project compilation failed")
        return False
    
    return True

def check_database_setup():
    """Validate PostgreSQL installation and database connection"""
    print_section("Database Setup")
    
    # Check PostgreSQL installation
    pg_installed, pg_version = check_command("psql")
    if pg_installed:
        print_status(f"PostgreSQL installed: {pg_version}")
    else:
        print_error("PostgreSQL not installed")
        return False
    
    # Check if PostgreSQL is running
    pg_ready = subprocess.run(["pg_isready", "-q"], capture_output=True)
    if pg_ready.returncode == 0:
        print_status("PostgreSQL service is running")
    else:
        print_error("PostgreSQL service is not running")
        return False
    
    # Check database connection
    if os.path.exists(".env"):
        print_info("Testing database connection...")
        # Load DATABASE_URL from .env
        with open(".env", "r") as f:
            for line in f:
                if line.startswith("DATABASE_URL="):
                    db_url = line.strip().split("=", 1)[1]
                    # Test connection
                    test_query = subprocess.run(
                        ["psql", db_url, "-c", "SELECT 1;"],
                        capture_output=True
                    )
                    if test_query.returncode == 0:
                        print_status("Database connection successful")
                        
                        # Check if tables exist
                        check_tables = subprocess.run(
                            ["psql", db_url, "-c", "\\dt"],
                            capture_output=True,
                            text=True
                        )
                        if "agents" in check_tables.stdout and "events" in check_tables.stdout:
                            print_status("Required database tables exist")
                        else:
                            print_warning("Some database tables may be missing")
                            print_info("Run: ./scripts/database-setup.sh")
                    else:
                        print_error("Database connection failed")
                        return False
                    break
    else:
        print_error(".env file not found")
        return False
    
    return True

def check_python_setup():
    """Validate Python and MCP server setup"""
    print_section("Python Environment")
    
    # Check Python installation
    python_installed, python_version = check_command("python3")
    if python_installed:
        print_status(f"Python installed: {python_version}")
        
        # Check minimum version (3.11+)
        import re
        version_match = re.search(r'(\d+)\.(\d+)', python_version)
        if version_match:
            major = int(version_match.group(1))
            minor = int(version_match.group(2))
            if major > 3 or (major == 3 and minor >= 11):
                print_status("Python version meets requirements (3.11+)")
            else:
                print_warning(f"Python version {major}.{minor} is old. Recommend 3.11+")
    else:
        print_error("Python not installed")
        return False
    
    # Check uv installation
    uv_installed, uv_version = check_command("uv")
    if uv_installed:
        print_status(f"uv installed: {uv_version}")
    else:
        print_warning("uv not installed. Install with: curl -LsSf https://astral.sh/uv/install.sh | sh")
    
    # Check MCP server dependencies
    if os.path.exists("mcp-servers/pyproject.toml"):
        print_status("MCP servers directory found")
        if uv_installed:
            # Check if dependencies are installed
            if os.path.exists("mcp-servers/.venv"):
                print_status("MCP server virtual environment exists")
            else:
                print_warning("MCP server dependencies not installed")
                print_info("Run: cd mcp-servers && uv sync")
    else:
        print_warning("MCP servers not found")
    
    return True

def check_running_services():
    """Check if any services are currently running"""
    print_section("Running Services")
    
    services_running = False
    
    # Check main API server
    try:
        import requests
        response = requests.get("http://localhost:8080/api/v1/health", timeout=1)
        if response.status_code == 200:
            print_status("AI Workflow API is running on port 8080")
            services_running = True
    except:
        print_info("AI Workflow API is not running")
    
    # Check MCP servers
    mcp_servers = [
        ("HelpScout MCP", 8001),
        ("Notion MCP", 8002),
        ("Slack MCP", 8003)
    ]
    
    for server_name, port in mcp_servers:
        try:
            import socket
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(1)
            result = sock.connect_ex(('localhost', port))
            sock.close()
            if result == 0:
                print_status(f"{server_name} is running on port {port}")
                services_running = True
            else:
                print_info(f"{server_name} is not running")
        except:
            print_info(f"{server_name} is not running")
    
    if not services_running:
        print_info("No services are currently running")
        print_info("Start services with: ./dev.sh start")
    
    return True

def run_validation():
    """Run all validation checks"""
    print(f"{Colors.BLUE}AI Workflow System - Setup Validation{Colors.NC}")
    print("=" * 50)
    
    all_checks_passed = True
    
    # Run all checks
    if not check_rust_setup():
        all_checks_passed = False
    
    if not check_database_setup():
        all_checks_passed = False
    
    if not check_python_setup():
        all_checks_passed = False
    
    check_running_services()
    
    # Final summary
    print_section("Validation Summary")
    
    if all_checks_passed:
        print_status("All critical checks passed! ✨")
        print("\nYour development environment is ready to use.")
        print("\nQuick start commands:")
        print(f"  {Colors.GREEN}cargo run{Colors.NC}              - Start the API server")
        print(f"  {Colors.GREEN}cargo test{Colors.NC}             - Run tests")
        print(f"  {Colors.GREEN}./dev.sh start{Colors.NC}         - Start all services")
        print(f"  {Colors.GREEN}./dev.sh mcp{Colors.NC}           - Start MCP servers only")
    else:
        print_error("Some checks failed!")
        print("\nPlease fix the issues above and run validation again.")
        print(f"\nFor automatic setup, run: {Colors.BLUE}./scripts/setup.sh{Colors.NC}")
        sys.exit(1)

if __name__ == "__main__":
    # Change to project root
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    os.chdir(project_root)
    
    run_validation()