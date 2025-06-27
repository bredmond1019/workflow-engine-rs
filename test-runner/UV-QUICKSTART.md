# UV Quick Start Guide

UV is an extremely fast Python package manager written in Rust. It's designed to be a drop-in replacement for pip and pip-tools, but with significantly better performance.

## Why UV?

- **⚡ Speed**: 10-100x faster than pip
- **🔒 Reproducible**: Automatic lock file generation
- **💾 Space Efficient**: Global cache reduces disk usage
- **🎯 Accurate**: Better dependency resolution
- **🛠️ Modern**: Built with Rust for reliability and performance

## Installation

### macOS/Linux
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### Windows
```powershell
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"
```

### Via pip (fallback)
```bash
pip install uv
```

## Basic UV Commands

### Creating Virtual Environments
```bash
# Create a new virtual environment
uv venv

# Create with specific Python version
uv venv --python 3.11

# Activate the environment
source .venv/bin/activate  # Linux/macOS
.venv\Scripts\activate     # Windows
```

### Installing Packages
```bash
# Install a package
uv pip install requests

# Install from requirements.txt
uv pip install -r requirements.txt

# Install in editable mode
uv pip install -e .

# Install with extras
uv pip install -e ".[dev]"
```

### Managing Dependencies
```bash
# Generate requirements from pyproject.toml
uv pip compile pyproject.toml -o requirements.txt

# Freeze current environment
uv pip freeze > requirements.txt

# Sync environment with lock file
uv pip sync
```

### Uninstalling
```bash
# Uninstall a package
uv pip uninstall requests

# Uninstall all packages
uv pip uninstall --all
```

## UV vs pip Comparison

| Task | pip | UV |
|------|-----|-----|
| Install numpy | ~5s | ~0.5s |
| Install large project | ~60s | ~5s |
| Resolve complex deps | ~30s | ~2s |
| Create venv | ~3s | ~0.3s |

## Advanced Features

### Global Cache
UV maintains a global cache of packages, so installing the same package in multiple projects is nearly instant:

```bash
# Show cache info
uv cache info

# Clean cache
uv cache clean
```

### Resolution Strategy
UV uses a more sophisticated dependency resolver:

```bash
# Show resolution details
uv pip install --verbose requests

# Use specific resolution strategy
uv pip install --resolution=highest requests
```

### Compile Python Dependencies
UV can compile Python dependencies for faster installation:

```bash
# Enable compilation
uv pip install --compile numpy
```

## Integration with This Project

This test runner is designed to work seamlessly with UV:

1. **Fast Setup**: Use `./install.sh` for automatic UV installation
2. **Lock File**: UV automatically creates a lock file for reproducible installs
3. **Development**: Use `uv pip install -e ".[dev]"` for development dependencies
4. **CI/CD**: UV's speed makes it perfect for CI pipelines

## Troubleshooting

### UV Command Not Found
Add UV to your PATH:
```bash
# Linux/macOS
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Windows
# Add %USERPROFILE%\.cargo\bin to system PATH
```

### SSL Certificate Errors
```bash
# Trust PyPI certificates
uv pip install --trusted-host pypi.org requests
```

### Proxy Configuration
```bash
# Set proxy
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
```

## Best Practices

1. **Always use virtual environments**: UV makes this fast and easy
2. **Commit uv.lock**: Ensures reproducible builds
3. **Use `--compile`**: For production deployments
4. **Leverage caching**: UV's cache is shared across projects
5. **Update regularly**: `uv self update` to get latest features

## Learn More

- [UV Documentation](https://github.com/astral-sh/uv)
- [UV Benchmarks](https://github.com/astral-sh/uv#benchmarks)
- [Rye (UV's companion tool)](https://rye-up.com/)
- [Astral Blog](https://astral.sh/blog)

## Quick Reference Card

```bash
# Essential UV commands
uv venv                      # Create virtual environment
source .venv/bin/activate    # Activate environment
uv pip install package       # Install package
uv pip install -e .          # Install current project
uv pip sync                  # Sync with lock file
uv cache info               # Show cache info
uv self update              # Update UV itself
```