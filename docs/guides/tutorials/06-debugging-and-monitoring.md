# Tutorial 6: Debugging and Monitoring - When Things Go Wrong

Your workflow is like a car - it runs great most of the time, but occasionally it makes strange noises, stalls unexpectedly, or just refuses to start. When that happens, you need to know how to diagnose and fix the problem. That's what debugging and monitoring are all about!

## Your Workflow Needs Regular Checkups

Just like you take your car for regular maintenance, your workflows need monitoring to catch problems before they become disasters. Let's learn how to be a workflow mechanic!

## Correlation IDs: Tracking Numbers for Your Packages

Imagine you ordered something online. The company gives you a tracking number so you can follow your package's journey. Correlation IDs work the same way for your workflow tasks.

### What's a Correlation ID?

```rust
// Without correlation ID - like shipping without tracking
async fn process_order(order: Order) -> Result<()> {
    log::info!("Processing order");  // Which order? Who knows!
    validate_payment(order).await?;
    log::info!("Payment validated"); // For which order? Still no idea!
    ship_product(order).await?;
    Ok(())
}

// With correlation ID - every step is trackable
async fn process_order_trackable(order: Order, correlation_id: &str) -> Result<()> {
    log::info!("Processing order {}", correlation_id);
    validate_payment(order, correlation_id).await?;
    log::info!("Payment validated for {}", correlation_id);
    ship_product(order, correlation_id).await?;
    Ok(())
}
```

### Following the Trail

Here's how correlation IDs help you debug:

```rust
use uuid::Uuid;
use tracing::{info, error, instrument};

pub struct WorkflowContext {
    correlation_id: String,
    started_at: Instant,
    metadata: HashMap<String, String>,
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            started_at: Instant::now(),
            metadata: HashMap::new(),
        }
    }
}

#[instrument(skip(data), fields(correlation_id = %ctx.correlation_id))]
pub async fn debug_friendly_workflow(data: Data, ctx: &WorkflowContext) -> Result<()> {
    info!("Starting workflow");
    
    // Every log now includes the correlation ID automatically!
    match process_step_1(data, ctx).await {
        Ok(result) => info!("Step 1 completed successfully"),
        Err(e) => {
            error!("Step 1 failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
```

When something goes wrong, you can search your logs for that correlation ID and see the complete journey:

```
[2024-01-10 10:15:23] INFO  correlation_id=abc-123 Starting workflow
[2024-01-10 10:15:24] INFO  correlation_id=abc-123 Step 1 completed
[2024-01-10 10:15:25] ERROR correlation_id=abc-123 Step 2 failed: Connection timeout
```

## Health Checks: Like a Doctor's Checkup

Your workflow needs regular health checks, just like you do. These checks tell you if everything is working properly before users notice problems.

### Basic Health Check

```rust
use actix_web::{HttpResponse, web};
use serde_json::json;

pub async fn health_check(app_state: web::Data<AppState>) -> HttpResponse {
    // Quick pulse check
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn detailed_health_check(app_state: web::Data<AppState>) -> HttpResponse {
    // Full physical examination
    let mut checks = HashMap::new();
    
    // Check database connection
    match app_state.db_pool.acquire().await {
        Ok(_) => checks.insert("database", "healthy"),
        Err(_) => checks.insert("database", "unhealthy"),
    };
    
    // Check external services
    match app_state.mcp_client.ping().await {
        Ok(_) => checks.insert("mcp_services", "healthy"),
        Err(_) => checks.insert("mcp_services", "unhealthy"),
    };
    
    // Check system resources
    let memory_usage = get_memory_usage();
    if memory_usage < 80.0 {
        checks.insert("memory", "healthy");
    } else {
        checks.insert("memory", "warning");
    }
    
    // Overall status
    let overall = if checks.values().all(|&v| v == "healthy") {
        "healthy"
    } else {
        "degraded"
    };
    
    HttpResponse::Ok().json(json!({
        "status": overall,
        "checks": checks,
        "timestamp": chrono::Utc::now()
    }))
}
```

### Setting Up Health Check Endpoints

```rust
pub fn configure_health_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/health")
            .route("/", web::get().to(health_check))
            .route("/detailed", web::get().to(detailed_health_check))
            .route("/ready", web::get().to(readiness_check))
            .route("/live", web::get().to(liveness_check))
    );
}
```

## The Dashboard in Your Car: Prometheus and Grafana

Prometheus and Grafana are like the dashboard in your car - they show you speed, fuel level, engine temperature, and warning lights. For workflows, they show request rates, error counts, response times, and system health.

### Setting Up Your Dashboard

```rust
use prometheus::{Encoder, TextEncoder, Counter, Histogram, Gauge};
use lazy_static::lazy_static;

lazy_static! {
    // Speedometer: How fast are we processing?
    static ref REQUEST_COUNTER: Counter = Counter::new(
        "workflow_requests_total", 
        "Total number of workflow requests"
    ).unwrap();
    
    // Fuel gauge: How much capacity is left?
    static ref QUEUE_SIZE: Gauge = Gauge::new(
        "workflow_queue_size",
        "Current number of tasks in queue"
    ).unwrap();
    
    // Temperature gauge: How long do things take?
    static ref RESPONSE_TIME: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "workflow_duration_seconds",
            "Workflow execution duration"
        ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0])
    ).unwrap();
    
    // Warning lights: What's going wrong?
    static ref ERROR_COUNTER: Counter = Counter::new(
        "workflow_errors_total",
        "Total number of workflow errors"
    ).unwrap();
}

pub fn record_workflow_metrics(duration: Duration, success: bool) {
    REQUEST_COUNTER.inc();
    RESPONSE_TIME.observe(duration.as_secs_f64());
    
    if !success {
        ERROR_COUNTER.inc();
    }
}
```

### Visualizing in Grafana

Your Grafana dashboard might look like this:

```
┌─────────────────────────────────────────────────┐
│            Workflow Health Dashboard            │
├─────────────────────────────────────────────────┤
│  Requests/min     Response Time    Error Rate   │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │    ╱╲    │    │  ___     │    │     _    │  │
│  │   ╱  ╲   │    │ ╱   ╲__  │    │    ╱ ╲   │  │
│  │  ╱    ╲  │    │╱        ╲ │    │   ╱   ╲  │  │
│  └──────────┘    └──────────┘    └──────────┘  │
│     1,250            1.2s           0.5%        │
├─────────────────────────────────────────────────┤
│  System Resources                               │
│  CPU: ████████░░ 78%   Memory: █████░░░░░ 52%  │
│  Disk: ███░░░░░░░ 31%  Network: ██████░░░ 65%  │
└─────────────────────────────────────────────────┘
```

## Walking Through a Failed Workflow

Let's debug a real problem together. Your workflow suddenly starts failing, and users are complaining. Here's your debugging process:

### Step 1: Check the Symptoms

```rust
// User complaint: "My review processing is failing!"
// First, check the health endpoint
curl http://localhost:8080/health/detailed

// Response shows:
{
  "status": "degraded",
  "checks": {
    "database": "healthy",
    "mcp_services": "unhealthy",
    "memory": "healthy"
  }
}
```

### Step 2: Find the Correlation ID

```rust
// Look up the user's failed request
let correlation_id = "550e8400-e29b-41d4-a716-446655440000";

// Search logs with this ID
grep "550e8400-e29b-41d4-a716-446655440000" /var/log/workflow.log
```

### Step 3: Follow the Trail

```
[10:15:00] INFO  correlation_id=550e8400... Starting review processing
[10:15:01] INFO  correlation_id=550e8400... Fetched review data
[10:15:02] INFO  correlation_id=550e8400... Starting sentiment analysis
[10:15:32] ERROR correlation_id=550e8400... MCP timeout: sentiment service not responding
[10:15:32] ERROR correlation_id=550e8400... Workflow failed: ServiceUnavailable
```

### Step 4: Implement a Fix

```rust
// Add retry logic for transient failures
pub async fn resilient_mcp_call<T>(
    client: &McpClient,
    request: Request,
    correlation_id: &str,
) -> Result<T> {
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        attempts += 1;
        
        match timeout(Duration::from_secs(5), client.call(request.clone())).await {
            Ok(Ok(response)) => return Ok(response),
            Ok(Err(e)) if attempts < max_attempts => {
                warn!(
                    "MCP call failed (attempt {}/{}): {}", 
                    attempts, max_attempts, e
                );
                tokio::time::sleep(Duration::from_secs(attempts as u64)).await;
            }
            Ok(Err(e)) => {
                error!("MCP call failed after {} attempts: {}", max_attempts, e);
                return Err(e);
            }
            Err(_) if attempts < max_attempts => {
                warn!("MCP call timed out (attempt {}/{})", attempts, max_attempts);
                tokio::time::sleep(Duration::from_secs(attempts as u64)).await;
            }
            Err(_) => {
                error!("MCP call timed out after {} attempts", max_attempts);
                return Err(WorkflowError::Timeout);
            }
        }
    }
}
```

## Common Error Patterns and Fixes

### Pattern 1: The Memory Leak

**Symptoms**: Workflow gets slower over time, eventually crashes

```rust
// Problem: Keeping all results in memory
pub struct LeakyWorkflow {
    all_results: Vec<BigData>,  // This grows forever!
}

// Solution: Process and release
pub struct EfficientWorkflow {
    result_writer: ResultWriter,  // Write to storage, not memory
}

impl EfficientWorkflow {
    pub async fn process(&mut self, data: BigData) -> Result<()> {
        let result = transform(data).await?;
        self.result_writer.write(result).await?;  // Don't keep in memory
        Ok(())
    }
}
```

### Pattern 2: The Cascade Failure

**Symptoms**: One service fails, everything fails

```rust
// Problem: No circuit breaker
async fn fragile_workflow(data: Data) -> Result<()> {
    let a = service_a(data).await?;  // If this fails...
    let b = service_b(a).await?;     // This never runs
    let c = service_c(b).await?;     // Neither does this
    Ok(c)
}

// Solution: Circuit breaker pattern
use circuit_breaker::CircuitBreaker;

pub struct ResilientWorkflow {
    service_a_breaker: CircuitBreaker,
    service_b_breaker: CircuitBreaker,
    service_c_breaker: CircuitBreaker,
}

impl ResilientWorkflow {
    pub async fn process(&self, data: Data) -> Result<()> {
        // Try service A with circuit breaker
        let a = match self.service_a_breaker.call(|| service_a(data)).await {
            Ok(result) => result,
            Err(_) => {
                // Fallback to cached or default value
                get_fallback_a(data)
            }
        };
        
        // Continue with degraded functionality
        Ok(())
    }
}
```

### Pattern 3: The Timeout Spiral

**Symptoms**: Timeouts cause retries, which cause more timeouts

```rust
// Problem: Fixed timeouts
const TIMEOUT: Duration = Duration::from_secs(5);

// Solution: Adaptive timeouts with backoff
pub struct AdaptiveTimeout {
    base_timeout: Duration,
    max_timeout: Duration,
    current_timeout: Duration,
}

impl AdaptiveTimeout {
    pub fn next_timeout(&mut self, success: bool) -> Duration {
        if success {
            // Gradually decrease timeout on success
            self.current_timeout = (self.current_timeout * 9) / 10;
            self.current_timeout.max(self.base_timeout)
        } else {
            // Increase timeout on failure
            self.current_timeout = (self.current_timeout * 12) / 10;
            self.current_timeout.min(self.max_timeout)
        }
    }
}
```

## Troubleshooting Flowchart

```
Is your workflow failing?
        │
        ▼
┌─────────────────┐
│ Check Health    │──── Healthy ──→ Check logs for errors
│ Endpoint        │
└────────┬────────┘
         │ Degraded
         ▼
┌─────────────────┐
│ Which component │
│ is unhealthy?   │
└────────┬────────┘
         │
    ┌────┴────┬──────────┬─────────┐
    ▼         ▼          ▼         ▼
Database   External   Memory    CPU
    │      Service      │        │
    ▼         ▼         ▼        ▼
Check      Check     Check    Check
connection  retry    leaks    hot
pool       config             spots
```

## Preventing Issues Before They Happen

### 1. Load Testing

```rust
// Test your workflow under stress
#[tokio::test]
async fn test_workflow_under_load() {
    let workflow = create_workflow().await;
    let tasks = (0..1000).map(|i| {
        tokio::spawn(async move {
            workflow.execute(generate_test_data(i)).await
        })
    });
    
    let results = join_all(tasks).await;
    let success_rate = results.iter()
        .filter(|r| r.is_ok())
        .count() as f64 / results.len() as f64;
    
    assert!(success_rate > 0.99, "Success rate too low: {}", success_rate);
}
```

### 2. Canary Deployments

```rust
// Route small percentage to new version
pub struct CanaryRouter {
    stable_workflow: Workflow,
    canary_workflow: Workflow,
    canary_percentage: f32,
}

impl CanaryRouter {
    pub async fn route(&self, request: Request) -> Result<Response> {
        let use_canary = rand::random::<f32>() < self.canary_percentage;
        
        if use_canary {
            info!("Routing to canary version");
            self.canary_workflow.execute(request).await
        } else {
            self.stable_workflow.execute(request).await
        }
    }
}
```

### 3. Automated Alerts

```rust
// Alert when things go wrong
pub struct AlertManager {
    thresholds: AlertThresholds,
}

impl AlertManager {
    pub async fn check_metrics(&self, metrics: &WorkflowMetrics) {
        if metrics.error_rate() > self.thresholds.max_error_rate {
            self.send_alert(
                AlertLevel::Critical,
                format!("Error rate {} exceeds threshold {}", 
                    metrics.error_rate(), 
                    self.thresholds.max_error_rate
                )
            ).await;
        }
        
        if metrics.p99_latency() > self.thresholds.max_latency {
            self.send_alert(
                AlertLevel::Warning,
                format!("P99 latency {} exceeds threshold {}", 
                    metrics.p99_latency().as_secs_f64(), 
                    self.thresholds.max_latency.as_secs_f64()
                )
            ).await;
        }
    }
}
```

## Debugging Toolkit

Here's your essential debugging toolkit:

```rust
// 1. Structured logging with context
use tracing::{info, error, warn, debug, instrument};

#[instrument(err, skip(sensitive_data))]
pub async fn debuggable_function(
    id: &str, 
    sensitive_data: SecretData
) -> Result<Output> {
    debug!("Starting processing");
    // Function automatically logs on error
    process(sensitive_data).await
}

// 2. Debug endpoints
pub async fn debug_state(app_state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "active_workflows": app_state.get_active_count(),
        "queue_depth": app_state.get_queue_size(),
        "connection_pool": app_state.get_pool_stats(),
        "cache_stats": app_state.get_cache_stats(),
    }))
}

// 3. Trace exports
pub fn export_trace(correlation_id: &str) -> Result<TraceData> {
    let spans = collect_spans(correlation_id);
    let trace = TraceData {
        correlation_id: correlation_id.to_string(),
        spans,
        total_duration: calculate_duration(&spans),
        bottlenecks: identify_slow_operations(&spans),
    };
    Ok(trace)
}
```

## Key Takeaways

1. **Correlation IDs**: Your GPS for tracking requests
2. **Health Checks**: Early warning system
3. **Metrics**: Can't fix what you can't measure
4. **Structured Logging**: Context is everything
5. **Circuit Breakers**: Prevent cascade failures
6. **Load Testing**: Find limits before production

⚠️ **Warning**: The best debugging is preventing bugs. Invest in monitoring and testing!

## Next Steps

Now that you can debug and monitor your workflows, let's learn how to make them production-ready in Tutorial 7: Best Practices!

Remember: Every expert was once a beginner who didn't give up when facing their first production bug. Happy debugging!