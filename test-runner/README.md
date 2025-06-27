# AI Workflow Test Runner

A comprehensive test runner for monitoring and validating the AI workflow orchestration system components, built with UV for fast dependency management.

## Features

- 🚀 **Fast Installation**: Uses UV for blazing-fast dependency resolution
- 🎨 **Rich Terminal Output**: Beautiful, informative test results with progress bars
- ⚡ **Parallel Execution**: Run tests concurrently for faster results
- 🔄 **Retry Logic**: Automatic retry for failed tests with configurable delays
- 📊 **Comprehensive Reports**: JSON output with detailed test results
- 🔧 **Flexible Configuration**: YAML-based configuration with CLI overrides
- 🏗️ **Multiple Test Types**: Support for port checks, HTTP endpoints, and commands

## Installation with UV

UV is a fast Python package manager written in Rust. Install it first:

```bash
# Install UV (macOS/Linux)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install UV (Windows)
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"

# Or via pip
pip install uv
```

Then install the test runner:

```bash
# Clone the repository and navigate to test-runner
cd test-runner

# Create virtual environment and install dependencies
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install the package
uv pip install -e .

# Or install directly with all dependencies
uv pip sync
```

## Quick Start

```bash
# Run with default configuration
workflow-test

# Run with custom config file
workflow-test --config test-config.yaml

# Run only specific categories
workflow-test --category backend --category mcp

# Run with custom output directory
workflow-test --output-dir ./my-reports

# Run tests sequentially with no retry
workflow-test --sequential --no-retry

# Disable rich output
workflow-test --no-rich

# Use environment file
workflow-test --env-file .env.test
```

## Configuration

Create a `test-config.yaml` file to customize test definitions:

```yaml
test_categories:
  infrastructure:
    name: Infrastructure
    icon: 🏗️
    enabled: true
    tests:
      - name: PostgreSQL
        check: port
        host: localhost
        port: 5432
        timeout: 3.0
        retry_count: 3
        retry_delay: 1.0
      
      - name: Redis
        check: port
        host: localhost
        port: 6379

  backend:
    name: Backend Services
    icon: 🚀
    tests:
      - name: Main API
        check: http
        url: http://localhost:8080/health
        timeout: 10.0
      
      - name: GraphQL Gateway
        check: http
        url: http://localhost:4000/.well-known/apollo/server-health

  custom:
    name: Custom Checks
    icon: 🔧
    tests:
      - name: Database Migration
        check: command
        command: "psql -c 'SELECT 1' ai_workflow_db"
        timeout: 5.0

settings:
  output_dir: test-reports
  save_results: true
  rich_output: true
  parallel_execution: true
  max_workers: 10
  default_timeout: 5.0
  retry_failed: true
```

## Test Types

### Port Check
Verifies that a service is listening on a specific port:
```yaml
- name: PostgreSQL
  check: port
  host: localhost
  port: 5432
```

### HTTP Check
Verifies that an HTTP endpoint returns a 200 status:
```yaml
- name: API Health
  check: http
  url: http://localhost:8080/health
  timeout: 10.0
```

### Command Check
Executes a shell command and checks for exit code 0:
```yaml
- name: Database Check
  check: command
  command: "psql -c 'SELECT 1' mydb"
```

## Development

### Setup Development Environment

```bash
# Install with development dependencies
uv pip install -e ".[dev]"

# Run tests
pytest

# Run tests with coverage
pytest --cov

# Format code
black src tests
ruff check src tests

# Type checking
mypy src
```

### Project Structure

```
test-runner/
├── pyproject.toml          # UV project configuration
├── uv.lock                 # UV lock file (auto-generated)
├── README.md              # This file
├── src/
│   └── test_runner/
│       ├── __init__.py    # Package initialization
│       ├── config.py      # Configuration models
│       ├── runner.py      # Core test runner
│       └── cli.py         # CLI interface
└── tests/                 # Test suite
    ├── test_config.py
    ├── test_runner.py
    └── test_cli.py
```

## Output Format

Test results are saved as JSON files with the following structure:

```json
{
  "total": 15,
  "passed": 13,
  "failed": 2,
  "skipped": 0,
  "start_time": "2024-01-15T10:30:00",
  "end_time": "2024-01-15T10:30:45",
  "duration": 45.0,
  "services": {
    "infrastructure": [
      {
        "name": "PostgreSQL",
        "status": "passed",
        "duration": 0.15,
        "details": "localhost:5432",
        "error": null,
        "attempts": 1
      }
    ]
  }
}
```

## UV-Specific Features

### Fast Dependency Resolution

UV provides extremely fast dependency resolution compared to pip:

```bash
# Update dependencies
uv pip compile pyproject.toml -o requirements.txt

# Install from lock file
uv pip sync

# Add new dependency
uv pip install rich
```

### Lock File Management

UV automatically creates and maintains a lock file for reproducible installs:

```bash
# Generate lock file
uv pip freeze > uv.lock

# Install from lock file
uv pip sync --file uv.lock
```

### Caching

UV caches packages globally, making subsequent installs much faster:

```bash
# Clear UV cache
uv cache clean

# Show cache info
uv cache info
```

## Troubleshooting

### UV Not Found
If `uv` command is not found after installation:
```bash
# Add to PATH (macOS/Linux)
export PATH="$HOME/.cargo/bin:$PATH"

# Add to PATH (Windows)
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
```

### Permission Errors
If you encounter permission errors:
```bash
# Use user installation
uv pip install --user -e .
```

### Rich Output Not Working
If terminal doesn't support rich output:
```bash
# Disable rich output
workflow-test --no-rich

# Or set environment variable
export TERM=dumb
workflow-test
```

## License

MIT License - see LICENSE file for details.