# AI Workflow System - Examples & Tutorials

This directory contains comprehensive working examples and tutorials demonstrating the AI Workflow System capabilities, including GraphQL Federation, event sourcing, security features, and MCP integration.

## Quick Start Examples

### Core Examples Index

1. **[Basic Workflow](01_basic_workflow/)** - Simple workflow creation and execution
2. **[MCP Integration](02_mcp_integration/)** - Model Context Protocol usage patterns  
3. **[GraphQL Federation](03_graphql_federation/)** - Federation queries and subgraph interaction
4. **[Event Sourcing](04_event_sourcing/)** - Event-driven architecture patterns
5. **[Security Features](05_security_features/)** - JWT validation and input sanitization

### Legacy Python Examples (Maintained for Compatibility)

- **[Blog Content Pipeline](1_blog_content_pipeline.py)** - Complete content creation workflow
- **[Customer Support Automation](2_customer_support_automation.py)** - Automated support system
- **[Knowledge Base Search](3_knowledge_base_search.py)** - Multi-source knowledge search

## Architecture Overview

The examples demonstrate the full system architecture:

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Examples  │────▶│ GraphQL Gateway  │────▶│   Microservices │
│   (Clients) │     │   (Port 4000)    │     │  (8080-8084)   │
└─────────────┘     └──────────────────┘     └─────────────────┘
                            │
                    ┌───────┴────────┐
                    │                │
              Federation      Entity Resolution
```

## Key Features Demonstrated

### 🔧 **Core Functionality**
- Workflow creation and execution
- Node-based processing pipeline
- Type-safe error handling with boxed errors
- Event sourcing with CQRS patterns

### 🌐 **GraphQL Federation**
- Subgraph schema composition
- Cross-service entity resolution
- Query planning and execution
- Partial failure handling

### 🔒 **Security & Validation**
- JWT authentication patterns
- Input sanitization and validation
- Rate limiting examples
- Multi-tenant support

### 🔗 **Integration Patterns**
- MCP server communication
- External API integrations
- WebSocket real-time updates
- Database event storage

### 🏗️ **Architecture Patterns**
- Microservices coordination
- Event-driven communication
- Circuit breaker patterns
- Graceful degradation

## Prerequisites

### System Requirements

1. **Full System Stack** (recommended)
   ```bash
   # Quick start with Docker
   docker-compose up -d
   
   # Or start individual components
   cargo run --bin workflow-engine          # Main API (8080)
   cargo run --bin graphql-gateway          # Federation Gateway (4000) 
   cd frontend && npm run dev               # Frontend (5173)
   ```

2. **Development Environment**
   ```bash
   # Rust toolchain
   rustc --version  # 1.70+ required
   
   # Node.js for frontend examples
   node --version   # 18+ required
   
   # Environment variables
   export JWT_SECRET="your-secure-jwt-secret"  # Required - no default
   export DATABASE_URL="postgresql://user:pass@localhost/ai_workflow_db"
   ```

3. **Optional: MCP Test Servers**
   ```bash
   # Start Python MCP servers for integration examples
   ./scripts/start_test_servers.sh
   
   # HelpScout (8001), Notion (8002), Slack (8003)
   ```

## Running the Examples

### Core Examples (Structured Tutorials)

Navigate to each example directory for complete instructions:

```bash
# Basic workflow patterns
cd 01_basic_workflow && cargo run --example basic_workflow

# MCP integration patterns  
cd 02_mcp_integration && cargo run --example mcp_client

# GraphQL federation queries
cd 03_graphql_federation && npm run demo

# Event sourcing patterns
cd 04_event_sourcing && cargo run --example event_replay  

# Security feature demonstrations
cd 05_security_features && cargo run --example jwt_validation
```

### Legacy Python Examples

Run Python examples directly (maintained for compatibility):

```bash
# Set up environment
export API_BASE_URL="http://localhost:8080/api/v1" 
export WS_URL="ws://localhost:8080/ws"
export AUTH_TOKEN="your-jwt-token"

# Run individual examples
python 1_blog_content_pipeline.py
python 2_customer_support_automation.py  
python 3_knowledge_base_search.py
```

## Example Outputs

### Basic Workflow (01_basic_workflow)
```
$ cargo run --example basic_workflow

=== Basic Workflow Example ===
✅ Workflow created: simple_text_processor
✅ Node registered: text_input 
✅ Node registered: text_processor
✅ Node registered: text_output

Executing workflow...
Step 1: text_input - Processing input: "Hello, World!"
Step 2: text_processor - Transforming text to uppercase
Step 3: text_output - Result: "HELLO, WORLD!"
✅ Workflow completed successfully in 45ms
```

### MCP Integration (02_mcp_integration)
```
$ cargo run --example mcp_client

=== MCP Integration Example ===
🔗 Connecting to HelpScout MCP server (localhost:8001)...
✅ Connection established via stdio transport

📋 Available tools:
- search_tickets: Search support tickets
- create_ticket: Create new ticket
- update_ticket: Update existing ticket

🔧 Calling tool: search_tickets
Parameters: {"query": "login issues", "status": "open"}

📊 Results:
Found 3 tickets matching "login issues":
- #12345: User cannot access dashboard
- #12346: Login form validation error  
- #12347: SSO integration failing

✅ MCP operation completed in 234ms
```

### GraphQL Federation (03_graphql_federation)
```
$ npm run demo

=== GraphQL Federation Example ===
🌐 Querying federation gateway (localhost:4000)...

Query: {
  workflows {
    id
    name
    status
    events {
      id
      type
      timestamp
    }
  }
}

Response:
{
  "data": {
    "workflows": [
      {
        "id": "wf_001",
        "name": "customer_support_automation",
        "status": "running",
        "events": [
          {
            "id": "evt_001", 
            "type": "workflow_started",
            "timestamp": "2024-12-18T10:30:00Z"
          }
        ]
      }
    ]
  }
}
✅ Federated query resolved across 3 subgraphs
```

## Development Best Practices

### Error Handling Patterns

All examples demonstrate the new boxed error handling:

```rust
use workflow_engine_core::error::WorkflowError;

// Use specific error constructors
let validation_error = WorkflowError::validation_error(
    "Input must be positive",
    "amount", 
    "must be > 0",
    "in payment processing"
);

// Handle errors with proper categorization
match workflow.execute(input) {
    Ok(result) => println!("Success: {:?}", result),
    Err(WorkflowError::ValidationError(details)) => {
        eprintln!("Validation failed: {}", details);
    }
    Err(WorkflowError::ProcessingError(details)) => {
        eprintln!("Processing error in {}: {}", details.node_type, details.message);
    }
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

### Security Implementation

Examples demonstrate JWT authentication and input validation:

```rust
// JWT validation pattern
let token_data = auth::validate_jwt(&token)
    .map_err(|e| WorkflowError::validation_error(
        "Invalid JWT token",
        "authorization_header",
        "valid JWT required", 
        "in API authentication"
    ))?;

// Input sanitization
let sanitized_input = sanitize_input(&user_input)
    .map_err(|e| WorkflowError::validation_error(
        "Input contains invalid characters",
        "user_input",
        "alphanumeric only",
        "in user data processing"
    ))?;
```

## Testing the Examples

### Unit Tests
```bash
# Test individual examples
cargo test --package examples basic_workflow
cargo test --package examples mcp_integration
cargo test --package examples federation_queries
```

### Integration Tests
```bash
# Start test environment
./scripts/start_test_servers.sh

# Run integration tests
cargo test --test examples_integration -- --ignored
```

### End-to-End Tests
```bash
# Test complete workflows
cargo test --test examples_e2e -- --ignored --nocapture
```

## Troubleshooting

### Common Issues

1. **Connection Errors**
   - Verify GraphQL gateway is running on port 4000
   - Check main API server is running on port 8080
   - Ensure JWT_SECRET environment variable is set

2. **Authentication Failures**
   - Verify JWT token is not expired
   - Check token has required claims
   - Ensure token is passed in Authorization header

3. **MCP Integration Issues**
   - Start MCP test servers: `./scripts/start_test_servers.sh`
   - Check MCP server logs for connection errors
   - Verify stdio transport is working

4. **Federation Query Failures**
   - Check subgraph services are running
   - Verify schema composition is valid
   - Review gateway logs for resolution errors

### Debug Commands

```bash
# Check system health
curl http://localhost:4000/health/detailed

# Test individual services
curl http://localhost:8080/health
curl http://localhost:8081/health  # Realtime service
curl http://localhost:8082/health  # Content processing

# View federation introspection
curl -X POST http://localhost:4000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ __schema { types { name } } }"}'
```

## Contributing

### Adding New Examples

1. Create a new directory following the naming pattern
2. Include a comprehensive README.md
3. Add both Rust and Python examples where applicable
4. Ensure all examples use the new error handling patterns
5. Test thoroughly with the full system stack

### Example Structure

```
05_new_example/
├── README.md           # Comprehensive documentation
├── Cargo.toml         # Rust dependencies
├── src/
│   ├── main.rs        # Main Rust example
│   └── lib.rs         # Shared utilities
├── examples/
│   └── basic.rs       # Simple example
├── python/
│   ├── client.py      # Python client example
│   └── requirements.txt
└── tests/
    └── integration_test.rs
```

## Further Reading

- **[System Architecture](../docs/ARCHITECTURE.md)** - Complete system design
- **[GraphQL Federation Guide](../FEDERATION.md)** - Federation implementation details
- **[Security Documentation](../SECURITY.md)** - Security best practices
- **[API Reference](../docs/API_REFERENCE.md)** - Complete API documentation
- **[Testing Guide](../TEST_COVERAGE_REPORT.md)** - Testing methodology and coverage