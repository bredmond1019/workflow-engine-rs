#!/bin/bash

# AI Workflow System - Database Setup Script
# This script creates the necessary PostgreSQL user and database for development

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration (matches .env.example)
DB_USER=${DB_USER:-"aiworkflow"}
DB_PASSWORD=${DB_PASSWORD:-"aiworkflow123"}
DB_NAME=${DB_NAME:-"ai_workflow"}
DB_HOST=${DB_HOST:-"localhost"}
DB_PORT=${DB_PORT:-"5432"}

echo -e "${BLUE}AI Workflow System - Database Setup${NC}"
echo "======================================"

# Function to print colored output
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

# Check if PostgreSQL is available
if ! command -v psql &> /dev/null; then
    print_error "PostgreSQL is not installed or not in PATH"
    print_info "Please install PostgreSQL first:"
    print_info "  macOS: brew install postgresql"
    print_info "  Ubuntu: sudo apt-get install postgresql postgresql-contrib"
    print_info "  Windows: Download from https://www.postgresql.org/download/windows/"
    exit 1
fi

print_status "PostgreSQL found: $(psql --version | cut -d' ' -f3)"

# Check if PostgreSQL service is running
if ! pg_isready -h $DB_HOST -p $DB_PORT &> /dev/null; then
    print_warning "PostgreSQL service is not running"
    print_info "Starting PostgreSQL service..."
    
    # Try to start PostgreSQL based on the system
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS with Homebrew
        if command -v brew &> /dev/null && brew services list | grep -q postgresql; then
            brew services start postgresql
        else
            print_error "Could not start PostgreSQL automatically"
            print_info "Please start PostgreSQL manually:"
            print_info "  macOS: brew services start postgresql"
            exit 1
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        if command -v systemctl &> /dev/null; then
            sudo systemctl start postgresql
        elif command -v service &> /dev/null; then
            sudo service postgresql start
        else
            print_error "Could not start PostgreSQL automatically"
            print_info "Please start PostgreSQL manually"
            exit 1
        fi
    else
        print_error "Unsupported operating system for automatic PostgreSQL startup"
        print_info "Please start PostgreSQL manually and re-run this script"
        exit 1
    fi
    
    # Wait a moment for the service to start
    sleep 2
fi

print_status "PostgreSQL service is running"

# Function to run PostgreSQL commands with error handling
run_psql() {
    local command="$1"
    local description="$2"
    
    print_info "Executing: $description"
    if psql -h $DB_HOST -p $DB_PORT -U postgres -c "$command" &> /dev/null; then
        print_status "$description completed"
        return 0
    else
        # Try with current user if postgres user doesn't work
        if psql -h $DB_HOST -p $DB_PORT -c "$command" &> /dev/null; then
            print_status "$description completed"
            return 0
        else
            print_error "$description failed"
            return 1
        fi
    fi
}

# Check if user already exists
print_info "Checking if user '$DB_USER' exists..."
if psql -h $DB_HOST -p $DB_PORT -U postgres -t -c "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" 2>/dev/null | grep -q 1; then
    print_warning "User '$DB_USER' already exists"
else
    # Create the database user
    print_info "Creating database user '$DB_USER'..."
    if run_psql "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" "Create user $DB_USER"; then
        print_status "User '$DB_USER' created successfully"
    else
        print_error "Failed to create user '$DB_USER'"
        print_info "You may need to run this script with appropriate PostgreSQL privileges"
        print_info "Or create the user manually: createuser -s $DB_USER"
        exit 1
    fi
fi

# Grant superuser privileges (for development)
print_info "Granting privileges to user '$DB_USER'..."
run_psql "ALTER USER $DB_USER CREATEDB;" "Grant CREATEDB privilege"
run_psql "ALTER USER $DB_USER WITH SUPERUSER;" "Grant SUPERUSER privilege (development only)"

# Check if database already exists
print_info "Checking if database '$DB_NAME' exists..."
if psql -h $DB_HOST -p $DB_PORT -U postgres -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw $DB_NAME; then
    print_warning "Database '$DB_NAME' already exists"
else
    # Create the database
    print_info "Creating database '$DB_NAME'..."
    if run_psql "CREATE DATABASE $DB_NAME OWNER $DB_USER;" "Create database $DB_NAME"; then
        print_status "Database '$DB_NAME' created successfully"
    else
        print_error "Failed to create database '$DB_NAME'"
        print_info "You may need to create it manually: createdb -O $DB_USER $DB_NAME"
        exit 1
    fi
fi

# Test the connection with the new user
print_info "Testing connection with user '$DB_USER'..."
export PGPASSWORD="$DB_PASSWORD"
if psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT version();" &> /dev/null; then
    print_status "Connection test successful"
else
    print_error "Connection test failed"
    print_info "Please check your credentials and try again"
    exit 1
fi

# Initialize the database schema
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INIT_SQL_FILE="$SCRIPT_DIR/init-db.sql"

if [ ! -f "$INIT_SQL_FILE" ]; then
    print_error "Database initialization script not found: $INIT_SQL_FILE"
    exit 1
fi

print_info "Initializing database schema..."
if psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f "$INIT_SQL_FILE" &> /dev/null; then
    print_status "Database schema initialized successfully"
else
    print_error "Failed to initialize database schema"
    print_info "Please check the init-db.sql file for errors"
    exit 1
fi

# Construct the DATABASE_URL
DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"

echo ""
echo -e "${GREEN}✅ Database setup completed successfully!${NC}"
echo ""
echo "Database connection details:"
echo "  Host: $DB_HOST"
echo "  Port: $DB_PORT"
echo "  Database: $DB_NAME"
echo "  User: $DB_USER"
echo "  DATABASE_URL: $DATABASE_URL"
echo ""
echo "Next steps:"
echo "1. Ensure your .env file contains:"
echo "   DATABASE_URL=$DATABASE_URL"
echo "2. Test the application: cargo run"
echo "3. Access health check: curl http://localhost:8080/api/v1/health"
echo ""

unset PGPASSWORD