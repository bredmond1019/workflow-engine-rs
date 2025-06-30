# Agent Tasks: Microservices Business Logic

## Agent Role

**Primary Focus:** Implement core microservice APIs and business logic to provide functional service endpoints

## Key Responsibilities

- Replace stubbed API endpoints with real business logic
- Implement content processing algorithms and AI integration
- Develop knowledge graph query execution and graph operations
- Complete realtime communication message handling

## Assigned Tasks

### From Original Task List

- [x] **2.0 Complete Core Microservice Business Logic Implementation** - (Originally task 2.0 from main list) ✅ **COMPLETE**
  - [x] **2.1 Implement Content Processing Service API** ✅ **COMPLETE**
    - [x] 2.1.1 Replace hardcoded response in `services/content_processing/src/api.rs`
    - [x] 2.1.2 Implement content analysis algorithms in `src/processor.rs`
    - [x] 2.1.3 Add AI model integration for content understanding
    - [x] 2.1.4 Implement quality assessment and scoring logic
    - [x] 2.1.5 Add content categorization and metadata extraction
    - [x] 2.1.6 Implement proper error handling and validation
  - [x] **2.2 Implement Knowledge Graph Service API** ✅ **COMPLETE**
    - [x] 2.2.1 Replace placeholder response in `services/knowledge_graph/src/api.rs`
    - [x] 2.2.2 Implement Dgraph connection and query execution
    - [x] 2.2.3 Add graph traversal algorithms in `src/graph.rs`
    - [x] 2.2.4 Implement relationship discovery and path finding
    - [x] 2.2.5 Add node and edge creation functionality
    - [x] 2.2.6 Implement query result formatting and pagination
  - [x] **2.3 Complete Realtime Communication Service** ✅ **COMPLETE**
    - [x] 2.3.1 Implement message routing logic in `src/handlers.rs`
    - [x] 2.3.2 Add WebSocket session management
    - [x] 2.3.3 Implement message persistence and history
    - [x] 2.3.4 Add real-time notification delivery
    - [x] 2.3.5 Implement user presence and status tracking

## Relevant Files

- `services/content_processing/src/api.rs` - Main API endpoints (currently stubbed with hardcoded responses)
- `services/content_processing/src/processor.rs` - Content processing business logic implementation
- `services/content_processing/src/lib.rs` - Content types and processing options definitions
- `services/content_processing/tests/integration_tests.rs` - Integration tests for content processing
- `services/knowledge_graph/src/api.rs` - Graph query API endpoints (currently placeholder responses)
- `services/knowledge_graph/src/graph.rs` - Graph operations and query logic
- `services/knowledge_graph/src/client/` - Dgraph connection and client management
- `services/knowledge_graph/tests/graph_tests.rs` - Unit tests for graph operations
- `services/realtime_communication/src/handlers.rs` - Message routing and WebSocket handlers
- `services/realtime_communication/src/websocket.rs` - WebSocket session management
- `services/realtime_communication/tests/websocket_tests.rs` - WebSocket functionality tests

## Dependencies

### Prerequisites (What this agent needs before starting)

- **From Agent 1:** Working compilation environment and functional MCP clients
- **From Agent 4:** Basic event store functionality for message persistence
- **External Dependencies:** Dgraph instance for knowledge graph service

### Provides to Others (What this agent delivers)

- **To Agent 3:** Functional service APIs for AI integration testing
- **To Agent 4:** Service endpoints that generate events for event sourcing
- **To Agent 5:** Complete microservice implementations for production deployment

## Handoff Points

- **After Task 2.1:** Notify Agent 3 that Content Processing API is ready for AI integration
- **After Task 2.2:** Confirm Knowledge Graph API is functional for Agent 3's MCP testing
- **After Task 2.3:** Signal that Realtime Communication is ready for production integration
- **Before Task 2.2.2:** Wait for confirmation from Agent 4 that database connections are stable

## Testing Responsibilities

- Unit tests for all business logic implementations
- Integration testing with Dgraph for knowledge graph functionality
- WebSocket connection testing for realtime communication
- API endpoint testing with realistic payloads
- Performance testing for content processing algorithms

## Implementation Priorities

### Phase 1: Critical API Stubs (Week 1)
1. **Content Processing API** (Task 2.1.1)
   - Replace hardcoded response with basic processing logic
   - Implement content type detection and validation
   - Add structured error responses

2. **Knowledge Graph API** (Task 2.2.1)
   - Replace placeholder with basic Dgraph query execution
   - Implement query parameter validation
   - Add proper error handling for graph operations

### Phase 2: Core Business Logic (Week 2)
1. **Content Analysis** (Tasks 2.1.2-2.1.5)
   - Text classification and categorization
   - Quality assessment scoring algorithms
   - Metadata extraction pipelines

2. **Graph Operations** (Tasks 2.2.2-2.2.5)
   - Graph traversal and path finding
   - Relationship discovery algorithms
   - Node/edge creation and management

### Phase 3: Advanced Features (Week 3)
1. **Realtime Communication** (Tasks 2.3.1-2.3.5)
   - Message routing and session management
   - Persistence and notification systems
   - User presence tracking

## Technical Implementation Notes

### Content Processing Service
- **AI Integration:** Use existing AI provider clients in `src/core/ai/`
- **Content Types:** Support HTML, Markdown, PDF, PlainText, JSON, XML
- **Processing Pipeline:** Validation → Analysis → AI Enhancement → Storage
- **Quality Metrics:** Readability scores, sentiment analysis, complexity assessment

### Knowledge Graph Service
- **Dgraph Integration:** Use existing connection pool patterns
- **Query Types:** Node lookup, relationship traversal, pattern matching
- **Performance:** Implement query caching and result pagination
- **Schema:** Support dynamic schema evolution for graph data

### Realtime Communication Service
- **WebSocket Management:** Connection pooling and lifecycle management
- **Message Types:** Direct messages, broadcast, presence updates
- **Persistence:** Store message history and delivery status
- **Scaling:** Prepare for horizontal scaling with Redis pub/sub

## Critical Success Criteria

1. **All API endpoints return real data instead of hardcoded responses**
2. **Content processing produces accurate analysis results**
3. **Knowledge graph queries execute against real Dgraph instance**
4. **Realtime communication handles concurrent WebSocket connections**
5. **Integration tests pass with external dependencies**

## Notes

- Follow existing code conventions in microservice implementations
- Use dependency injection patterns from `src/bootstrap/service.rs`
- Coordinate with Agent 3 for AI provider integration testing
- Ensure all services are stateless for horizontal scaling
- Document API changes and new endpoints for Agent 5's deployment work