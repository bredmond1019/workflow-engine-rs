# Knowledge Graph Troubleshooting Guide

## Overview

This guide provides solutions for common issues encountered when operating the Knowledge Graph Service, with a focus on Dgraph-specific problems, performance issues, and data consistency challenges.

## Common Dgraph Issues

### Connection Issues

#### Problem: "Failed to connect to Dgraph"

**Symptoms:**
- Error: `connection refused` or `timeout`
- Service unable to start
- Health checks failing

**Diagnosis:**
```bash
# Check if Dgraph is running
docker ps | grep dgraph

# Test Dgraph connectivity
curl http://localhost:8080/health

# Check Dgraph logs
docker logs dgraph-alpha

# Test gRPC port
nc -zv localhost 9080
```

**Solutions:**

1. **Dgraph not started:**
```bash
# Start Dgraph containers
docker-compose up -d dgraph-zero dgraph-alpha

# Wait for initialization
sleep 30

# Verify health
curl http://localhost:8080/health
```

2. **Network issues:**
```bash
# Check Docker network
docker network ls
docker network inspect knowledge_graph_network

# Recreate network if needed
docker-compose down
docker network prune
docker-compose up -d
```

3. **Port conflicts:**
```bash
# Check port usage
sudo lsof -i :9080
sudo lsof -i :8080

# Change ports in docker-compose.yml if needed
```

#### Problem: "Connection pool exhausted"

**Symptoms:**
- Intermittent failures
- Slow response times
- Error: `no connections available`

**Diagnosis:**
```bash
# Check pool metrics
curl http://localhost:3002/metrics | grep connection_pool

# Monitor active connections
watch -n 1 'curl -s http://localhost:3002/metrics | grep dgraph_connections'
```

**Solutions:**

1. **Increase pool size:**
```bash
# Update environment
export DGRAPH_MAX_CONNECTIONS=50
export DGRAPH_MIN_CONNECTIONS=10

# Restart service
docker-compose restart knowledge-graph
```

2. **Reduce connection lifetime:**
```bash
export DGRAPH_IDLE_TIMEOUT_SECS=120
export DGRAPH_HEALTH_CHECK_INTERVAL_SECS=20
```

3. **Fix connection leaks:**
```rust
// Ensure connections are properly released
let result = pool.with_connection(|conn| {
    // operations
})?; // Connection auto-released
```

### Query Performance Problems

#### Problem: "Query timeout"

**Symptoms:**
- Queries taking > 30 seconds
- Timeout errors
- High CPU usage on Dgraph

**Diagnosis:**
```bash
# Enable query tracing
curl -X POST http://localhost:8080/admin/schema \
  -d 'trace: true'

# Monitor slow queries
tail -f dgraph-alpha.log | grep "query_latency_ms"

# Check query complexity
curl http://localhost:8080/debug/prometheus | grep query_latency
```

**Solutions:**

1. **Optimize queries:**
```graphql
# Bad: Deep nesting without pagination
query {
  getConcepts {
    prerequisites {
      prerequisites {
        prerequisites {
          # Too deep!
        }
      }
    }
  }
}

# Good: Use pagination and limit depth
query {
  getConcepts(first: 20) {
    prerequisites(first: 10) {
      id
      name
    }
  }
}
```

2. **Add indexes:**
```bash
# Add missing indexes
curl -X POST http://localhost:8080/alter \
  -d 'name: string @index(fulltext, term) .'
```

3. **Increase timeouts:**
```bash
export DGRAPH_QUERY_TIMEOUT_MS=60000
```

#### Problem: "High memory usage"

**Symptoms:**
- Dgraph using excessive RAM
- OOM kills
- Slow garbage collection

**Diagnosis:**
```bash
# Check memory usage
docker stats dgraph-alpha

# View memory profile
curl http://localhost:8080/debug/pprof/heap > heap.prof
go tool pprof heap.prof
```

**Solutions:**

1. **Tune cache size:**
```bash
# Reduce cache
docker run dgraph/dgraph:latest \
  dgraph alpha --cache_mb=2048  # Default is 1/3 of RAM
```

2. **Enable memory limits:**
```yaml
# docker-compose.yml
services:
  dgraph-alpha:
    deploy:
      resources:
        limits:
          memory: 8G
```

### Graph Consistency Issues

#### Problem: "Circular dependencies detected"

**Symptoms:**
- Topological sort failing
- Learning path generation errors
- Infinite loops in traversal

**Diagnosis:**
```rust
// Check for cycles
let result = detect_cycles(&nodes, &edges);
if let Some(cycle) = result {
    println!("Cycle found: {:?}", cycle);
}
```

**Solutions:**

1. **Remove circular dependencies:**
```graphql
mutation {
  deleteEdge(
    from: "concept-1-id"
    to: "concept-2-id"
    predicate: "prerequisites"
  )
}
```

2. **Validate before insertion:**
```rust
fn add_prerequisite(from: Uuid, to: Uuid) -> Result<()> {
    // Check if adding edge creates cycle
    let mut temp_edges = edges.clone();
    temp_edges.push(GraphEdge { from: to, to: from, weight: 1.0 });
    
    if has_cycle(&nodes, &temp_edges) {
        return Err(anyhow!("Would create circular dependency"));
    }
    
    // Safe to add
    add_edge(from, to)
}
```

#### Problem: "Data inconsistency"

**Symptoms:**
- Missing relationships
- Duplicate nodes
- Orphaned data

**Diagnosis:**
```graphql
# Find orphaned concepts
query {
  queryConcept(filter: {
    and: [
      { not: { has: prerequisites } }
      { not: { has: enabledBy } }
      { not: { has: resources } }
    ]
  }) {
    id
    name
  }
}
```

**Solutions:**

1. **Run consistency checks:**
```rust
pub async fn check_graph_consistency() -> Result<ConsistencyReport> {
    let mut report = ConsistencyReport::default();
    
    // Check for orphaned nodes
    let orphans = find_orphaned_concepts().await?;
    report.orphaned_nodes = orphans.len();
    
    // Check for duplicate names
    let duplicates = find_duplicate_concepts().await?;
    report.duplicate_names = duplicates.len();
    
    // Check for missing inverses
    let missing = check_inverse_relationships().await?;
    report.missing_inverses = missing.len();
    
    Ok(report)
}
```

2. **Repair inconsistencies:**
```bash
# Backup first
curl -X POST http://localhost:8080/admin/backup

# Run repair script
./scripts/repair_graph_consistency.sh
```

## Performance Optimization

### Slow Response Times

#### Problem: "API responses taking > 1 second"

**Diagnosis:**
```bash
# Profile request
curl -w "@curl-format.txt" -o /dev/null -s \
  http://localhost:3002/api/v1/concept/123

# Check cache effectiveness
curl http://localhost:3002/metrics | grep cache_hit_ratio
```

**Solutions:**

1. **Enable caching:**
```rust
// Implement caching layer
#[cached(
    result = true,
    time = 3600,
    key = "String",
    convert = r#"{ format!("{}", concept_id) }"#
)]
async fn get_concept_cached(concept_id: Uuid) -> Result<Concept> {
    get_concept_from_dgraph(concept_id).await
}
```

2. **Optimize hot paths:**
```rust
// Pre-compute expensive operations
lazy_static! {
    static ref CONCEPT_RANKS: RwLock<HashMap<Uuid, f32>> = {
        RwLock::new(HashMap::new())
    };
}

// Update periodically
tokio::spawn(async {
    loop {
        update_concept_ranks().await;
        tokio::time::sleep(Duration::from_secs(300)).await;
    }
});
```

### Memory Leaks

#### Problem: "Service memory usage growing unbounded"

**Diagnosis:**
```bash
# Monitor memory over time
while true; do
    date
    ps aux | grep knowledge-graph | awk '{print $6}'
    sleep 60
done

# Heap dump
curl http://localhost:3002/debug/pprof/heap > heap.out
```

**Solutions:**

1. **Fix connection leaks:**
```rust
// Bad: Connection not released
let conn = pool.get_connection().await?;
let result = conn.query(query).await?;
// Connection leaked!

// Good: Ensure cleanup
let result = pool.with_connection(|conn| async {
    conn.query(query).await
}).await?;
```

2. **Clear caches periodically:**
```rust
// Implement cache eviction
tokio::spawn(async {
    loop {
        tokio::time::sleep(Duration::from_hours(1)).await;
        clear_expired_cache_entries().await;
    }
});
```

## Algorithm Failures

### Path Finding Issues

#### Problem: "No path found between concepts"

**Diagnosis:**
```rust
// Debug path finding
let debug_result = dijkstra_with_trace(start, goal, &edges);
println!("Visited nodes: {:?}", debug_result.visited);
println!("Unreachable from start: {:?}", 
    find_unreachable_nodes(start, &edges));
```

**Solutions:**

1. **Check graph connectivity:**
```rust
// Find connected components
let components = find_connected_components(&nodes, &edges);
println!("Number of components: {}", components.len());

// Check if nodes in same component
let start_component = find_component(start, &components);
let goal_component = find_component(goal, &components);
if start_component != goal_component {
    return Err(anyhow!("Nodes in different components"));
}
```

2. **Relax constraints:**
```rust
// Try with relaxed constraints
let constraints = PathConstraints {
    max_difficulty: None,  // Remove difficulty limit
    max_path_length: Some(50),  // Increase length
    forbidden_concepts: vec![],  // Clear forbidden list
};
```

### PageRank Convergence

#### Problem: "PageRank not converging"

**Diagnosis:**
```rust
// Track convergence
let mut iterations = 0;
let mut max_change = 1.0;
while max_change > tolerance && iterations < max_iterations {
    let old_ranks = ranks.clone();
    update_ranks(&mut ranks);
    max_change = calculate_max_change(&old_ranks, &ranks);
    println!("Iteration {}: max_change = {}", iterations, max_change);
    iterations += 1;
}
```

**Solutions:**

1. **Adjust parameters:**
```bash
export PAGERANK_DAMPING_FACTOR=0.85
export PAGERANK_MAX_ITERATIONS=200
export PAGERANK_TOLERANCE=0.001
```

2. **Handle special cases:**
```rust
// Handle dangling nodes
fn handle_dangling_nodes(ranks: &mut HashMap<Uuid, f32>) {
    let dangling_sum: f32 = dangling_nodes.iter()
        .map(|&node| ranks[&node])
        .sum();
    
    let distribution = dangling_sum / ranks.len() as f32;
    for rank in ranks.values_mut() {
        *rank += distribution;
    }
}
```

## Data Recovery

### Corrupted Database

#### Problem: "Database corruption detected"

**Steps:**

1. **Stop service:**
```bash
docker-compose stop knowledge-graph
```

2. **Backup corrupted data:**
```bash
cp -r /data/dgraph /data/dgraph.corrupted
```

3. **Restore from backup:**
```bash
# Find latest backup
aws s3 ls s3://kg-backups/dgraph/ --recursive

# Restore
dgraph restore -p s3://kg-backups/dgraph/20240120 \
  -z localhost:5080
```

4. **Verify integrity:**
```bash
curl http://localhost:8080/health
```

### Lost Connections

#### Problem: "Lost connection to Dgraph during operation"

**Recovery:**

1. **Implement retry logic:**
```rust
#[async_retry]
async fn execute_with_retry<T>(
    operation: impl Fn() -> Future<Output = Result<T>>
) -> Result<T> {
    let mut attempts = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < 3 => {
                attempts += 1;
                let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

2. **Handle partial updates:**
```rust
// Use transactions
let txn = client.new_transaction();
match txn.mutate(mutation).await {
    Ok(_) => txn.commit().await?,
    Err(e) => {
        txn.discard().await?;
        return Err(e);
    }
}
```

## Monitoring and Alerts

### Setting Up Alerts

```yaml
# prometheus-alerts.yml
groups:
  - name: knowledge_graph
    rules:
      - alert: HighQueryLatency
        expr: histogram_quantile(0.95, query_duration_seconds_bucket) > 1
        for: 5m
        annotations:
          summary: "High query latency detected"
          
      - alert: ConnectionPoolExhausted
        expr: dgraph_connection_pool_available == 0
        for: 1m
        annotations:
          summary: "Connection pool exhausted"
          
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        annotations:
          summary: "High error rate detected"
```

### Debug Endpoints

Enable debug endpoints for troubleshooting:

```rust
// Add debug routes
.route("/debug/config", web::get().to(show_config))
.route("/debug/connections", web::get().to(show_connections))
.route("/debug/cache", web::get().to(show_cache_stats))
.route("/debug/graph/stats", web::get().to(graph_statistics))
```

## Quick Reference

### Emergency Commands

```bash
# Restart everything
docker-compose down && docker-compose up -d

# Clear all caches
redis-cli FLUSHALL

# Force garbage collection
curl -X POST http://localhost:3002/admin/gc

# Emergency backup
./scripts/emergency_backup.sh

# Check all health endpoints
./scripts/health_check_all.sh
```

### Log Locations

- Service logs: `docker logs knowledge-graph`
- Dgraph logs: `/data/dgraph/zero.log`, `/data/dgraph/alpha.log`
- Application logs: `/var/log/knowledge-graph/app.log`
- Audit logs: `/var/log/knowledge-graph/audit.log`

### Key Metrics to Monitor

1. **Service Health**
   - Request rate and latency
   - Error rate
   - Active connections

2. **Dgraph Health**
   - Query latency
   - Memory usage
   - Disk I/O

3. **Algorithm Performance**
   - Execution time by algorithm
   - Cache hit rates
   - Queue depths

### Support Resources

- Dgraph Documentation: https://dgraph.io/docs/
- Dgraph Discuss Forum: https://discuss.dgraph.io/
- Service Issues: GitHub Issues
- Internal Wiki: Runbooks and procedures