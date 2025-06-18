# Dgraph Integration Testing

This document describes the integration testing setup for the Knowledge Graph service with Dgraph.

## Overview

The integration tests verify end-to-end functionality against a real Dgraph instance, including:

- **Query Operations**: Complex queries, aggregations, traversals, search functionality
- **Mutation Operations**: Create, update, delete operations with relationships
- **Transaction Handling**: Transaction consistency, rollback scenarios, conflict resolution
- **Performance**: Benchmarks for large datasets and complex operations

## Test Structure

### Test Files

- `tests/knowledge_graph_integration_tests.rs` - Basic query and traversal tests
- `tests/knowledge_graph_mutation_tests.rs` - Mutation and relationship tests  
- `tests/knowledge_graph_transaction_tests.rs` - Transaction consistency tests

### Test Data

- `test-data/test-schema.graphql` - Simplified schema for testing
- `test-data/sample-data.json` - Sample concepts, resources, and relationships
- `docker-compose.test.yml` - Isolated Dgraph instance for testing

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust and Cargo
- curl (for health checks)

### Running Tests

1. **Run all integration tests:**
   ```bash
   cd services/knowledge_graph
   ./scripts/run-integration-tests.sh
   ```

2. **Run specific test categories:**
   ```bash
   ./scripts/run-integration-tests.sh --query-tests
   ./scripts/run-integration-tests.sh --mutation-tests
   ./scripts/run-integration-tests.sh --transaction-tests
   ```

3. **Keep environment running for debugging:**
   ```bash
   ./scripts/run-integration-tests.sh --keep-running
   ```

4. **Setup only (for manual testing):**
   ```bash
   ./scripts/run-integration-tests.sh --setup-only
   ```

### Manual Setup/Teardown

1. **Setup test environment:**
   ```bash
   ./scripts/test-dgraph-setup.sh
   ```

2. **Run tests manually:**
   ```bash
   cd ../../..  # Go to project root
   cargo test knowledge_graph_integration --ignored
   cargo test knowledge_graph_mutation --ignored  
   cargo test knowledge_graph_transaction --ignored
   ```

3. **Cleanup:**
   ```bash
   cd services/knowledge_graph
   ./scripts/test-dgraph-teardown.sh
   ```

## Test Environment Details

### Dgraph Configuration

The test environment uses different ports from development to avoid conflicts:

| Service | Development | Test | Purpose |
|---------|-------------|------|---------|
| Dgraph Alpha HTTP | 8080 | 18080 | GraphQL queries |
| Dgraph Alpha gRPC | 9080 | 19080 | Client connections |
| Dgraph Zero HTTP | 6080 | 16080 | Admin interface |
| Dgraph Zero gRPC | 5080 | 15080 | Cluster coordination |

### Test Data Schema

The test schema includes:

- **TestConcept**: Simplified concept type with relationships
- **TestResource**: Learning resources linked to concepts
- **TestLearningPath**: Ordered sequences of concepts
- **TestUserProgress**: Progress tracking for testing
- **TestSearchQuery**: Search analytics for testing

### Sample Data

The test environment includes sample data with:

- 8 programming concepts (Variables → Functions → Data Structures → Algorithms → OOP)
- 3 learning resources linked to concepts
- 1 learning path with ordered concepts
- User progress records for testing
- Search query analytics

## Test Categories

### Integration Tests (`knowledge_graph_integration_tests.rs`)

- **Connection and Health**: Basic connectivity and health checks
- **Schema Validation**: Verify schema loading and structure
- **Basic Queries**: Simple concept queries and filtering
- **Relationship Queries**: Prerequisites, enabledBy, relatedTo relationships
- **Search Functionality**: Fulltext search and exact matching
- **Aggregation Queries**: Count, avg, min, max operations
- **Complex Traversals**: Multi-level relationship traversals
- **Facet Queries**: Resource relationships with properties
- **Recursive Queries**: Deep prerequisite chains
- **Pagination**: Offset/limit and ordering
- **Query Builder Integration**: Generated query execution
- **Performance Tests**: Large query benchmarks

### Mutation Tests (`knowledge_graph_mutation_tests.rs`)

- **Create Operations**: Single and batch concept creation
- **Update Operations**: Field updates and version tracking
- **Delete Operations**: Concept deletion and relationship cleanup
- **Relationship Management**: Adding, updating, removing relationships
- **Resource Integration**: Creating concepts with linked resources
- **Batch Operations**: Multiple related mutations in one operation
- **Error Handling**: Invalid data and constraint violations

### Transaction Tests (`knowledge_graph_transaction_tests.rs`)

- **Simple Transactions**: Basic commit and rollback scenarios
- **Query + Mutation**: Mixed operations within transactions
- **Conflict Handling**: Concurrent transaction conflicts
- **Multi-Operation Transactions**: Complex multi-step operations
- **Error Handling**: Transaction failures and recovery
- **Read-Only Transactions**: Consistent read operations
- **Timeout Handling**: Long-running transaction behavior

## CI/CD Integration

### GitHub Actions

The `.github/workflows/dgraph-integration-tests.yml` workflow:

- Runs on push/PR to main branches affecting knowledge graph code
- Supports manual dispatch with test category selection
- Includes parallel test execution for faster feedback
- Collects logs and artifacts on failure
- Includes optional performance benchmarks

### Local CI Simulation

```bash
# Simulate CI environment locally
cd services/knowledge_graph
./scripts/run-integration-tests.sh --all-tests

# Test specific categories like CI
./scripts/run-integration-tests.sh --mutation-tests
```

## Debugging

### View Logs

```bash
# During test run
docker-compose -f docker-compose.test.yml logs -f

# After test completion (if kept running)
docker logs knowledge_graph_dgraph_alpha_test
docker logs knowledge_graph_dgraph_zero_test
```

### Access Dgraph UI

When keeping the environment running:

- Dgraph Ratel UI: http://localhost:8000 (if enabled)
- Alpha HTTP endpoint: http://localhost:18080
- Zero admin interface: http://localhost:16080

### Manual Queries

```bash
# Query concepts directly
curl -X POST http://localhost:18080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "{ concepts(func: type(TestConcept)) { uid name } }"}'

# Check schema
curl http://localhost:18080/admin/schema
```

### Common Issues

1. **Port conflicts**: Ensure development Dgraph isn't running
2. **Docker space**: Clean up with `docker system prune -f`
3. **Slow startup**: Wait for health checks to pass
4. **Test failures**: Check Dgraph logs for errors

## Performance Considerations

### Test Performance

- Tests run with `--test-threads=1` to avoid conflicts
- Each test category should complete within 5-10 minutes
- Large dataset tests are included for performance validation

### Resource Usage

- Test Dgraph uses smaller cache (512MB vs 2GB)
- Containers are configured for test efficiency
- Volumes are automatically cleaned up

### Optimization Tips

1. **Parallel Execution**: Use separate test categories in CI
2. **Data Cleanup**: Each test should clean up its data
3. **Connection Pooling**: Reuse clients within test suites
4. **Selective Testing**: Run specific categories during development

## Extending Tests

### Adding New Tests

1. Choose appropriate test file based on functionality
2. Follow existing patterns for setup/teardown
3. Use unique test data (UUIDs) to avoid conflicts
4. Include both positive and negative test cases

### Test Data Management

1. Create test-specific data with unique identifiers
2. Clean up data after tests (or use transactions)
3. Use meaningful names for debugging
4. Document complex test scenarios

### Best Practices

1. **Isolation**: Each test should be independent
2. **Deterministic**: Tests should produce consistent results
3. **Fast**: Optimize for quick feedback
4. **Comprehensive**: Cover edge cases and error conditions
5. **Maintainable**: Use helper functions and clear naming

## Troubleshooting

### Setup Issues

```bash
# Check Docker status
docker ps
docker-compose -f docker-compose.test.yml ps

# Check port availability
netstat -ln | grep ":18080\|:19080\|:15080\|:16080"

# Force cleanup
./scripts/test-dgraph-teardown.sh --images
docker system prune -af
```

### Test Failures

```bash
# Run with detailed output
RUST_LOG=debug cargo test knowledge_graph_integration --ignored -- --nocapture

# Check specific test
cargo test test_basic_query_operations --ignored -- --nocapture

# Run single test file
cargo test --test knowledge_graph_integration_tests --ignored
```

### Data Issues

```bash
# Check test data loading
curl http://localhost:18080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "{ concepts(func: type(TestConcept)) { count(uid) } }"}'

# Reload test data
cd services/knowledge_graph
docker-compose -f docker-compose.test.yml restart test-data-loader
```

## Resources

- [Dgraph Documentation](https://dgraph.io/docs/)
- [Dgraph GraphQL Schema](https://dgraph.io/docs/graphql/schema/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

For questions or issues, check the project's main documentation or create an issue in the repository.