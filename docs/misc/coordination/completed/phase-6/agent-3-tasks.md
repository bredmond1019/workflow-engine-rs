# Agent Tasks: Core Features Agent

## Agent Role

**Primary Focus:** Complete critical core functionality by implementing the production pricing engine and validating performance benchmarks.

## Key Responsibilities

- Transform pricing engine from hardcoded values to live API integration
- Implement robust pricing data collection for OpenAI, Anthropic, and AWS Bedrock
- Create comprehensive benchmarking framework to validate performance claims
- Ensure pricing and performance features are production-ready for open source release
- Provide monitoring and fallback mechanisms for pricing data reliability

## Assigned Tasks

### From Original Task List

- [ ] **4.0 Complete Production Pricing Engine Implementation** - [Originally task 4.0 from main list]
  - [ ] **4.1 Implement live API pricing updates for OpenAI** - [Originally task 4.1 from main list]
    - [ ] 4.1.1 Create HTTP client for OpenAI pricing API endpoints
    - [ ] 4.1.2 Implement pricing data parsing and validation
    - [ ] 4.1.3 Add error handling and fallback to cached pricing
    - [ ] 4.1.4 Create scheduled update mechanism for pricing data
  - [ ] **4.2 Implement live API pricing updates for Anthropic** - [Originally task 4.2 from main list]
    - [ ] 4.2.1 Create HTTP client for Anthropic pricing API endpoints
    - [ ] 4.2.2 Implement pricing data parsing and validation for Claude models
    - [ ] 4.2.3 Add error handling and fallback mechanisms
    - [ ] 4.2.4 Integrate with existing token counting infrastructure
  - [ ] **4.3 Implement live API pricing updates for AWS Bedrock** - [Originally task 4.3 from main list]
    - [ ] 4.3.1 Create AWS SDK integration for Bedrock pricing
    - [ ] 4.3.2 Implement multi-region pricing data collection
    - [ ] 4.3.3 Add IAM role and credential management
    - [ ] 4.3.4 Create pricing aggregation for different Bedrock models
  - [ ] **4.4 Add configuration for pricing update frequency and fallback handling** - [Originally task 4.4 from main list]
    - [ ] 4.4.1 Create configuration structure for update intervals
    - [ ] 4.4.2 Implement background task scheduler for pricing updates
    - [ ] 4.4.3 Add monitoring and alerting for pricing update failures
    - [ ] 4.4.4 Create comprehensive fallback chain from API → cache → hardcoded

- [ ] **7.0 Implement Performance Benchmark Validation** - [Originally task 7.0 from main list]
  - [ ] **7.1 Create benchmarking framework to validate "15,000+ requests/second" claim** - [Originally task 7.1 from main list]
    - [ ] 7.1.1 Set up Criterion.rs benchmarking infrastructure
    - [ ] 7.1.2 Create HTTP API load testing scenarios
    - [ ] 7.1.3 Implement concurrent request handling benchmarks
    - [ ] 7.1.4 Document benchmark methodology and environment requirements
  - [ ] **7.2 Create benchmarks for "sub-millisecond node processing" claim** - [Originally task 7.2 from main list]
    - [ ] 7.2.1 Implement workflow node execution timing benchmarks
    - [ ] 7.2.2 Create micro-benchmarks for individual node types
    - [ ] 7.2.3 Add memory usage profiling for node operations
    - [ ] 7.2.4 Benchmark parallel vs sequential node execution performance
  - [ ] **7.3 Document benchmark setup and results in README** - [Originally task 7.3 from main list]
    - [ ] 7.3.1 Create benchmark execution documentation
    - [ ] 7.3.2 Add performance results to README with methodology notes
    - [ ] 7.3.3 Include hardware specifications for benchmark results
    - [ ] 7.3.4 Set up automated benchmark regression testing

## Relevant Files

### Pricing Engine Implementation
- `crates/workflow-engine-core/src/ai/tokens/pricing.rs` - Main pricing engine implementation requiring live API integration
- `crates/workflow-engine-core/src/ai/tokens/mod.rs` - Token management module exports
- `crates/workflow-engine-core/src/config/` - Configuration management for API keys and pricing settings
- `crates/workflow-engine-core/src/ai/` - AI provider integration points

### Benchmark and Performance
- `benches/` - Benchmark directory to create with Criterion.rs infrastructure
- `scripts/benchmark.sh` - Benchmark execution scripts
- `docs/performance.md` - Performance documentation
- `crates/workflow-engine-core/src/core/workflow/` - Workflow engine core for node benchmarking
- `crates/workflow-engine-api/src/` - API layer for HTTP request benchmarking

### Configuration and Monitoring
- `crates/workflow-engine-core/src/monitoring/` - Monitoring infrastructure for pricing alerts
- `.env.example` - Environment variable template for API keys
- `crates/workflow-engine-core/src/config/pricing.rs` - Pricing configuration structures

## Dependencies

### Prerequisites (What this agent needs before starting)
- **From Build & Infrastructure Agent:** Working compilation environment (Task 1.0 completion)
- **From Build & Infrastructure Agent:** Stable API layer compilation for benchmarking (Task 1.1 completion)
- **Optional coordination with Architecture Cleanup Agent:** AI provider decisions for pricing engine scope

### Provides to Others (What this agent delivers)
- **To Quality & Documentation Agent:** Benchmark results and methodology for documentation
- **To Quality & Documentation Agent:** Production-ready pricing engine for integration testing
- **To All Agents:** Performance validation data and monitoring infrastructure

## Handoff Points

- **After Task 4.1-4.3:** Notify Quality & Documentation Agent that pricing APIs are ready for testing
- **After Task 4.4:** Notify Quality & Documentation Agent that pricing configuration is documented
- **After Task 7.1-7.2:** Notify Quality & Documentation Agent that benchmark data is ready for README
- **Before Task 7.3:** Coordinate with Quality & Documentation Agent on benchmark documentation format

## Testing Responsibilities

- Unit tests for all pricing API integration code
- Integration tests for live API connections (with fallback testing)
- Benchmark validation and regression testing
- Performance profiling and memory usage analysis
- Configuration validation for all pricing providers

## Implementation Priority Order

1. **Start with Task 4.4** - Create configuration infrastructure (foundation for other pricing work)
2. **Continue with Task 4.1-4.3** - Implement live API integrations (core functionality)
3. **Follow with Task 7.1-7.2** - Create benchmark infrastructure (requires working codebase)
4. **Finish with Task 7.3** - Document results (requires completed benchmarks)

## Critical Success Criteria

- [ ] **Pricing engine uses live API data** instead of hardcoded values
- [ ] **All pricing providers work reliably** with proper error handling and fallbacks
- [ ] **Benchmark claims validated** with documented methodology and results
- [ ] **Performance regression testing** automated and documented
- [ ] **Configuration documented** for API keys and pricing update settings

## Technical Requirements

### Pricing Engine
- **HTTP client reliability:** Proper timeout, retry, and error handling
- **Data validation:** Schema validation for all API responses
- **Fallback chain:** API → cached → hardcoded pricing data
- **Configuration:** Environment-based API key and update frequency settings
- **Monitoring:** Alerts for pricing update failures

### Benchmark Framework
- **Methodology:** Consistent, repeatable benchmark conditions
- **Coverage:** API throughput, node processing speed, memory usage
- **Documentation:** Hardware specs, environment setup, interpretation guidelines
- **Automation:** CI integration for performance regression detection

## Coordination Notes

- **With Build & Infrastructure Agent:** Ensure compilation stability before starting development
- **With Architecture Cleanup Agent:** Coordinate on AI provider scope (Gemini/Ollama decisions)
- **With Quality & Documentation Agent:** Provide benchmark data and configuration docs

## Notes

- **Focus on production readiness** - this code will be used by open source users
- **Implement robust error handling** - live API integrations must handle failures gracefully
- **Document all configuration options** - users need clear setup instructions
- **Validate benchmark claims** - performance numbers in README must be accurate and reproducible
- **Consider rate limiting** - pricing API calls should be respectful of provider limits