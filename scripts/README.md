# AI System MCP Servers

This directory contains MCP (Model Context Protocol) servers for testing AI system integration.

## Setup

This is a Python project managed with `uv`. The project includes:

- **customer_support_server.py**: MCP server providing customer support tools
- **multi_service_mcp_server.py**: Multi-service MCP server supporting Notion, HelpScout, and Slack APIs
- **test_mcp_server.py**: Test server for basic MCP functionality
- **start_test_servers.sh**: Script to start all test servers

## Installation

The project dependencies are automatically managed by `uv`:

```bash
# Dependencies are installed automatically when running servers
uv run mcp dev customer_support_server.py
```

## Usage

### Individual Servers

Start individual servers in development mode:

```bash
# Customer support server
uv run mcp dev customer_support_server.py

# Multi-service servers
uv run mcp dev multi_service_mcp_server.py --service notion
uv run mcp dev multi_service_mcp_server.py --service helpscout
uv run mcp dev multi_service_mcp_server.py --service slack
```

### All Test Servers

Start all servers for integration testing:

```bash
./start_test_servers.sh
```

## Dependencies

- `mcp[cli]>=1.9.2`: MCP Python SDK with CLI tools

## Integration Testing

The servers are used for testing external MCP client integration in the main Rust application:

```bash
# From the main project directory
cargo test external_mcp_integration -- --ignored
```