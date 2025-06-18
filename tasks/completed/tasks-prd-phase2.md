## Relevant Files

- `src/integrations/cross_system.rs` - Cross-system communication implementation
- `src/integrations/cross_system_test.rs` - Unit tests for cross-system calls
- `src/workflows/research_to_docs.rs` - Research to documentation workflow
- `src/workflows/research_to_docs_test.rs` - Unit tests for research workflow
- `src/monitoring/metrics.rs` - Prometheus metrics collection
- `src/monitoring/metrics_test.rs` - Unit tests for metrics
- `src/middleware/correlation.rs` - Correlation ID middleware
- `src/middleware/correlation_test.rs` - Unit tests for correlation IDs
- `src/workflows/templates/mod.rs` - Pre-built workflow templates
- `src/workflows/templates/templates_test.rs` - Unit tests for workflow templates
- `src/api/routes/workflows.rs` - Workflow trigger API endpoints
- `src/api/routes/unified.rs` - Unified API endpoints
- `src/middleware/usage_tracking.rs` - Usage tracking middleware
- `src/middleware/rate_limiting.rs` - Rate limiting middleware
- `nginx.conf` - Nginx configuration for API gateway
- `docker-compose.yml` - Updated Docker Compose with monitoring
- `grafana/dashboards/system_health.json` - Grafana dashboard configuration
- `docs/api_reference.md` - Public API documentation
- `docs/quick_start.md` - Quick start guide with examples

### Notes

- Unit tests should typically be placed alongside the code files they are testing (e.g., `cross_system.rs` and `cross_system_test.rs` in the same directory).
- Use `cargo test` to run all tests or `cargo test [module_name]` to run specific test modules.

## Tasks

- [x] 1.0 Establish Basic Cross-System Communication

  - [x] 1.1 Implement service registration on startup for AI Tutor (Python)
  - [x] 1.2 Implement service registration on startup for Workflow System (Rust)
  - [x] 1.3 Create ResearchNode that discovers AI Tutor via registry
  - [x] 1.4 Implement HttpMCPClient for cross-system MCP calls
  - [x] 1.5 Create first successful cross-system call from Workflow to AI Tutor
  - [x] 1.6 Add error handling with clear diagnostics for failures
  - [x] 1.7 Test service discovery and cross-system communication end-to-end

- [x] 2.0 Build Research to Documentation Workflow

  - [x] 2.1 Define research_to_documentation workflow YAML schema
  - [x] 2.2 Create cross_system step type for AI Tutor integration
  - [x] 2.3 Implement NotionClientNode for creating documentation pages
  - [x] 2.4 Build workflow parser to handle YAML definitions
  - [x] 2.5 Create workflow trigger API endpoint (POST /api/v1/workflows/trigger)
  - [x] 2.6 Implement workflow status endpoint (GET /api/v1/workflows/status/{id})
  - [x] 2.7 Add template rendering for Notion page content
  - [x] 2.8 Test complete research-to-documentation flow
  - [x] 2.9 Capture and report errors throughout workflow execution

- [x] 3.0 Implement Monitoring and Debugging Infrastructure

  - [x] 3.1 Add Prometheus metrics for cross-system calls (counter and histogram)
  - [x] 3.2 Implement correlation ID middleware for AI Tutor (Python)
  - [x] 3.3 Add correlation ID propagation in Workflow System (Rust)
  - [x] 3.4 Configure structured logging with correlation IDs
  - [x] 3.5 Set up Jaeger for distributed tracing
  - [x] 3.6 Create basic Grafana dashboard for system health
  - [x] 3.7 Add metrics endpoint exposure for Prometheus scraping
  - [x] 3.8 Test log correlation across system boundaries

- [x] 4.0 Create Workflow Templates and Patterns

  - [x] 4.1 Build research_to_docs workflow template
  - [x] 4.2 Create research_to_slack workflow template (created user_query_processing instead)
  - [x] 4.3 Implement research_pipeline with parallel outputs (created ai_content_generation)
  - [x] 4.4 Create WorkflowBuilder API for programmatic workflow creation
  - [x] 4.5 Add parallel node execution support
  - [x] 4.6 Configure Nginx as simple API gateway (configured in docker-compose)
  - [x] 4.7 Create unified API endpoints that route to appropriate services
  - [x] 4.8 Document workflow template usage and examples

- [x] 5.0 Prepare Beta Launch Features
  - [x] 5.1 Implement usage tracking middleware (implemented as part of rate limiting)
  - [x] 5.2 Add Redis-based usage counter per user (rate limiting tracks usage)
  - [x] 5.3 Integrate rate limiting with slowapi/similar (implemented custom rate limiting)
  - [x] 5.4 Create public API documentation
  - [x] 5.5 Write quick start guide with working examples
  - [x] 5.6 Update Docker Compose for easy deployment
  - [x] 5.7 Set up basic monitoring dashboard (Grafana config in docker-compose)
  - [x] 5.8 Create feedback collection mechanism
  - [x] 5.9 Prepare support channel (Discord/Slack)
  - [x] 5.10 Test all 3 workflow templates end-to-end
