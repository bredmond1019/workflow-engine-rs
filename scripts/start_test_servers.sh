#!/bin/bash

# Script to start multiple MCP test servers for integration testing

echo "Starting MCP test servers for external client integration testing..."

# Function to kill background processes on script exit
cleanup() {
    echo "Stopping test servers..."
    kill $NOTION_PID $HELPSCOUT_PID $SLACK_PID $CUSTOMER_SUPPORT_PID 2>/dev/null
    wait $NOTION_PID $HELPSCOUT_PID $SLACK_PID $CUSTOMER_SUPPORT_PID 2>/dev/null
    echo "Test servers stopped."
}

# Set up trap to cleanup on script exit
trap cleanup EXIT

# Change to scripts directory to ensure uv environment is available
cd "$(dirname "$0")"

# Start Customer Support MCP server
echo "Starting Customer Support MCP server..."
uv run python customer_support_server.py &
CUSTOMER_SUPPORT_PID=$!

# Start Notion MCP server
echo "Starting Notion MCP server..."
uv run python multi_service_mcp_server.py --service notion &
NOTION_PID=$!

# Start HelpScout MCP server  
echo "Starting HelpScout MCP server..."
uv run python multi_service_mcp_server.py --service helpscout &
HELPSCOUT_PID=$!

# Start Slack MCP server
echo "Starting Slack MCP server..."
uv run python multi_service_mcp_server.py --service slack &
SLACK_PID=$!

# Wait a moment for servers to start
sleep 3

echo "Test servers started using MCP Python SDK:"
echo "  - Customer Support MCP server: Running"
echo "  - Notion MCP server: Running"
echo "  - HelpScout MCP server: Running"
echo "  - Slack MCP server: Running"
echo ""
echo "These servers are running using the MCP Python SDK and support"
echo "stdio communication protocol for MCP clients."
echo ""
echo "To run integration tests, execute:"
echo "  cargo test external_mcp_integration -- --ignored"
echo ""
echo "Press Ctrl+C to stop all servers..."

# Wait for user interrupt
wait $NOTION_PID $HELPSCOUT_PID $SLACK_PID $CUSTOMER_SUPPORT_PID