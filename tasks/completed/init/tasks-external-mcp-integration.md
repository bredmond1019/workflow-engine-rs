## Relevant Files

- `src/core/nodes/external_mcp_client.rs` - Shared external MCP client functionality, traits, and configuration structures
- `src/core/mcp/clients/notion_client.rs` - NotionClientNode implementation for connecting to external Notion MCP server
- `src/core/mcp/clients/helpscout_client.rs` - HelpscoutClientNode implementation for connecting to external HelpScout MCP server (created)
- `src/core/mcp/clients/slack_client.rs` - SlackClientNode implementation for connecting to external Slack MCP server (created)
- `src/core/nodes/mod.rs` - Updated module exports for new client nodes
- `src/core/workflow/builder.rs` - Enhanced workflow builder with external MCP client support
- `src/core/mcp/clients/notion_client_test.rs` - Unit tests for NotionClientNode
- `src/core/mcp/clients/helpscout_client_test.rs` - Unit tests for HelpscoutClientNode (created)
- `src/core/mcp/clients/slack_client_test.rs` - Unit tests for SlackClientNode (created)
- `tests/external_mcp_integration_test.rs` - Integration tests for external MCP client connections
- `scripts/multi_service_mcp_server.py` - Multi-service MCP server for testing all three clients
- `scripts/start_test_servers.sh` - Script to start all test servers for integration testing
- `docs/external-mcp-clients.md` - Comprehensive documentation for external MCP client usage

### Notes

- Unit tests should be placed alongside the code files they are testing
- Integration tests should verify actual connections to running MCP servers
- Use existing transport and client implementations from `src/core/mcp/` as foundation
- Support HTTP, WebSocket, and stdio transport options

## Tasks

- [x] 1.0 Create base infrastructure for external MCP client nodes
  - [x] 1.1 Create `external_mcp_client.rs` with base traits and shared functionality
  - [x] 1.2 Define `ExternalMCPClientNode` trait with required methods (connect, execute_tool, disconnect)
  - [x] 1.3 Implement connection pooling for external MCP connections
  - [x] 1.4 Add support for HTTP, WebSocket, and stdio transport types
  - [x] 1.5 Create error handling specific to external MCP connections
  - [x] 1.6 Implement retry logic with exponential backoff for failed connections

- [x] 2.0 Implement NotionClientNode with external MCP server connection
  - [x] 2.1 Create `notion_client.rs` implementing the `ExternalMCPClientNode` trait
  - [x] 2.2 Configure Notion-specific MCP tools (search_pages, create_page, update_page, etc.)
  - [x] 2.3 Implement connection initialization with configurable transport (HTTP/WebSocket/stdio)
  - [x] 2.4 Add Notion-specific error handling and response parsing
  - [x] 2.5 Create unit tests for NotionClientNode functionality
  - [x] 2.6 Add configuration options for Notion server URL and authentication

- [x] 3.0 Implement HelpscoutClientNode with external MCP server connection  
  - [x] 3.1 Create `helpscout_client.rs` implementing the `ExternalMCPClientNode` trait
  - [x] 3.2 Configure HelpScout-specific MCP tools (search_articles, get_article, list_collections, etc.)
  - [x] 3.3 Implement connection initialization with configurable transport
  - [x] 3.4 Add HelpScout-specific error handling and response parsing
  - [x] 3.5 Create unit tests for HelpscoutClientNode functionality
  - [x] 3.6 Add configuration options for HelpScout server URL and authentication

- [x] 4.0 Implement SlackClientNode with external MCP server connection
  - [x] 4.1 Create `slack_client.rs` implementing the `ExternalMCPClientNode` trait
  - [x] 4.2 Configure Slack-specific MCP tools (send_message, list_channels, get_user_info, etc.)
  - [x] 4.3 Implement connection initialization with configurable transport
  - [x] 4.4 Add Slack-specific error handling and response parsing
  - [x] 4.5 Create unit tests for SlackClientNode functionality
  - [x] 4.6 Add configuration options for Slack server URL and authentication

- [x] 5.0 Add configuration and testing infrastructure
  - [x] 5.1 Create `external_config.rs` with configuration structures for external MCP servers
  - [x] 5.2 Add environment variable support for MCP server URLs (NOTION_MCP_URL, HELPSCOUT_MCP_URL, SLACK_MCP_URL)
  - [x] 5.3 Update `src/core/nodes/mod.rs` to export new client nodes
  - [x] 5.4 Create integration tests that verify connections to actual running MCP servers
  - [x] 5.5 Add example Python MCP servers for testing (if not already provided)
  - [x] 5.6 Update workflow builder to support external MCP client nodes
  - [x] 5.7 Add documentation for configuring and using external MCP client nodes