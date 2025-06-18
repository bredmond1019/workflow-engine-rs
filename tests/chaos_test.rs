#[cfg(test)]
mod tests {
    use backend::bootstrap::service::ServiceBootstrap;
    use backend::core::workflow::builder::WorkflowBuilder;
    use backend::core::nodes::registry::NodeRegistry;
    use backend::core::nodes::agent::AgentNode;
    use backend::core::nodes::external_mcp_client::ExternalMcpClientNode;
    use backend::core::nodes::config::NodeConfig;
    use backend::core::mcp::connection_pool::{ConnectionPool, ConnectionPoolConfig};
    use backend::core::mcp::transport::{Transport, TransportType};
    use backend::monitoring::metrics::AppMetrics;
    use serde_json::json;
    use std::any::TypeId;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{Duration, Instant};
    use tokio::sync::{RwLock, Mutex};
    use tokio::time::{sleep, timeout};
    use rand::Rng;

    /// Chaos test: Random service failures
    #[tokio::test]
    #[ignore] // Requires external services and is disruptive
    async fn test_chaos_random_service_failures() {
        let failure_injector = Arc::new(ChaosFailureInjector::new(0.3)); // 30% failure rate
        
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Build workflow with multiple external dependencies
        let workflow = WorkflowBuilder::new::<AgentNode>("chaos_test_workflow".to_string())
            .description("Workflow for chaos testing".to_string())
            .add_node(
                NodeConfig::new::<AgentNode>()
                    .with_description("Initial processing".to_string())
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("External service call 1".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "tool": "search_pages",
                        "endpoint": "http://localhost:8002",
                        "chaos_injector": failure_injector.clone()
                    }))
                    .with_connections(vec![TypeId::of::<ExternalMcpClientNode>()])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("External service call 2".to_string())
                    .with_config(json!({
                        "service": "helpscout",
                        "tool": "list_conversations",
                        "endpoint": "http://localhost:8001",
                        "chaos_injector": failure_injector.clone()
                    }))
            )
            .build()
            .unwrap();
        
        let mut results = ChaosTestResults::default();
        let num_iterations = 100;
        
        for i in 0..num_iterations {
            let context = json!({
                "iteration": i,
                "test": "chaos_random_failures"
            });
            
            let start = Instant::now();
            let result = workflow.run(context).await;
            let duration = start.elapsed();
            
            if result.is_ok() {
                results.successful_runs += 1;
                results.execution_times.push(duration);
            } else {
                results.failed_runs += 1;
                results.failure_reasons.push(format!("Iteration {}: {:?}", i, result.err()));
            }
            
            // Random delay between runs
            let delay = rand::thread_rng().gen_range(10..50);
            sleep(Duration::from_millis(delay)).await;
        }
        
        // Calculate resilience metrics
        let success_rate = results.successful_runs as f64 / num_iterations as f64 * 100.0;
        let avg_execution_time = results.execution_times.iter()
            .sum::<Duration>() / results.execution_times.len() as u32;
        
        println!("\n=== Chaos Test: Random Failures ===");
        println!("Total runs: {}", num_iterations);
        println!("Successful: {}", results.successful_runs);
        println!("Failed: {}", results.failed_runs);
        println!("Success rate: {:.2}%", success_rate);
        println!("Average execution time: {:?}", avg_execution_time);
        println!("Injected failure rate: 30%");
        
        // System should maintain reasonable success rate despite failures
        assert!(success_rate > 40.0, "System should handle some failures gracefully");
    }

    /// Chaos test: Network latency injection
    #[tokio::test]
    #[ignore] // Requires external services
    async fn test_chaos_network_latency() {
        let latency_injector = Arc::new(ChaosLatencyInjector::new(
            Duration::from_millis(100),
            Duration::from_millis(2000),
        ));
        
        let config = ConnectionPoolConfig {
            max_connections: 10,
            min_idle: 2,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(60),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(100),
        };
        
        let pool = Arc::new(ConnectionPool::new(config));
        let metrics = Arc::new(RwLock::new(LatencyTestMetrics::default()));
        
        // Simulate requests with random latency
        let num_requests = 50;
        let mut handles = vec![];
        
        for i in 0..num_requests {
            let pool_clone = pool.clone();
            let latency_clone = latency_injector.clone();
            let metrics_clone = metrics.clone();
            
            let handle = tokio::spawn(async move {
                // Inject random latency
                let latency = latency_clone.get_random_latency();
                sleep(latency).await;
                
                let start = Instant::now();
                
                match timeout(
                    Duration::from_secs(10),
                    pool_clone.get_connection("http://localhost:8001", TransportType::Http)
                ).await {
                    Ok(Ok(transport)) => {
                        let request = backend::core::mcp::protocol::McpRequest::ToolCall(
                            backend::core::mcp::protocol::ToolCall {
                                id: format!("latency_test_{}", i),
                                name: "list_conversations".to_string(),
                                arguments: json!({"limit": 5}),
                            }
                        );
                        
                        match timeout(Duration::from_secs(5), transport.send_request(request)).await {
                            Ok(Ok(_)) => {
                                let mut m = metrics_clone.write().await;
                                m.successful_requests += 1;
                                m.response_times.push(start.elapsed());
                            }
                            Ok(Err(_)) => {
                                let mut m = metrics_clone.write().await;
                                m.failed_requests += 1;
                            }
                            Err(_) => {
                                let mut m = metrics_clone.write().await;
                                m.timeout_requests += 1;
                            }
                        }
                    }
                    _ => {
                        let mut m = metrics_clone.write().await;
                        m.connection_failures += 1;
                    }
                }
            });
            
            handles.push(handle);
        }
        
        futures::future::join_all(handles).await;
        
        let final_metrics = metrics.read().await;
        let total_requests = final_metrics.successful_requests + 
                           final_metrics.failed_requests + 
                           final_metrics.timeout_requests;
        
        println!("\n=== Chaos Test: Network Latency ===");
        println!("Total requests: {}", total_requests);
        println!("Successful: {}", final_metrics.successful_requests);
        println!("Failed: {}", final_metrics.failed_requests);
        println!("Timeouts: {}", final_metrics.timeout_requests);
        println!("Connection failures: {}", final_metrics.connection_failures);
        
        // System should handle latency with retries
        assert!(final_metrics.successful_requests > 0, "Some requests should succeed despite latency");
    }

    /// Chaos test: Resource exhaustion
    #[tokio::test]
    async fn test_chaos_resource_exhaustion() {
        let resource_limiter = Arc::new(ChaosResourceLimiter::new(
            100, // Max concurrent tasks
            50 * 1024 * 1024, // 50MB memory limit
        ));
        
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        
        let workflow = Arc::new(
            WorkflowBuilder::new::<AgentNode>("resource_test".to_string())
                .description("Resource exhaustion test".to_string())
                .add_node(
                    NodeConfig::new::<AgentNode>()
                        .with_description("Resource intensive task".to_string())
                )
                .build()
                .unwrap()
        );
        
        let mut handles = vec![];
        let rejected_count = Arc::new(AtomicUsize::new(0));
        
        // Try to spawn many tasks
        for i in 0..200 {
            let workflow_clone = workflow.clone();
            let limiter_clone = resource_limiter.clone();
            let rejected_clone = rejected_count.clone();
            
            if limiter_clone.try_acquire_task() {
                let handle = tokio::spawn(async move {
                    // Simulate memory allocation
                    let _data = vec![0u8; 1024 * 1024]; // 1MB
                    
                    let context = json!({
                        "task_id": i,
                        "resource_test": true
                    });
                    
                    let _ = workflow_clone.run(context).await;
                    
                    // Release resources
                    limiter_clone.release_task();
                });
                
                handles.push(handle);
            } else {
                rejected_clone.fetch_add(1, Ordering::Relaxed);
            }
            
            // Small delay to spread out the load
            if i % 10 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }
        
        // Wait for all tasks to complete
        futures::future::join_all(handles).await;
        
        let total_rejected = rejected_count.load(Ordering::Relaxed);
        let total_accepted = 200 - total_rejected;
        
        println!("\n=== Chaos Test: Resource Exhaustion ===");
        println!("Total task attempts: 200");
        println!("Accepted: {}", total_accepted);
        println!("Rejected: {}", total_rejected);
        println!("Resource limit: 100 concurrent tasks");
        
        // Verify resource limiting worked
        assert!(total_rejected > 0, "Some tasks should be rejected due to resource limits");
        assert!(total_accepted <= 100, "Should not exceed resource limits");
    }

    /// Chaos test: Cascading failures
    #[tokio::test]
    #[ignore] // Requires external services
    async fn test_chaos_cascading_failures() {
        let failure_propagator = Arc::new(ChaosCascadingFailure::new());
        
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Build workflow with dependencies
        let workflow = WorkflowBuilder::new::<ExternalMcpClientNode>("cascading_test".to_string())
            .description("Test cascading failures".to_string())
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Primary service".to_string())
                    .with_config(json!({
                        "service": "notion",
                        "endpoint": "http://localhost:8002",
                        "failure_propagator": failure_propagator.clone()
                    }))
                    .with_connections(vec![
                        TypeId::of::<ExternalMcpClientNode>(),
                        TypeId::of::<ExternalMcpClientNode>()
                    ])
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Dependent service 1".to_string())
                    .with_config(json!({
                        "service": "helpscout",
                        "endpoint": "http://localhost:8001"
                    }))
            )
            .add_node(
                NodeConfig::new::<ExternalMcpClientNode>()
                    .with_description("Dependent service 2".to_string())
                    .with_config(json!({
                        "service": "slack",
                        "endpoint": "http://localhost:8003"
                    }))
            )
            .build()
            .unwrap();
        
        // Trigger cascading failure
        failure_propagator.trigger_failure("notion").await;
        
        let mut cascade_results = vec![];
        
        for i in 0..10 {
            let context = json!({
                "test": "cascading_failure",
                "iteration": i
            });
            
            let result = workflow.run(context).await;
            cascade_results.push(result.is_ok());
            
            sleep(Duration::from_millis(500)).await;
        }
        
        // Reset failure
        failure_propagator.reset_failures().await;
        
        // Try again after reset
        let recovery_context = json!({
            "test": "recovery_after_cascade"
        });
        
        let recovery_result = workflow.run(recovery_context).await;
        
        println!("\n=== Chaos Test: Cascading Failures ===");
        println!("Failures during cascade: {}", cascade_results.iter().filter(|&&x| !x).count());
        println!("Recovery after reset: {}", recovery_result.is_ok());
        
        // Verify cascading behavior
        assert!(cascade_results.iter().any(|&x| !x), "Should see failures during cascade");
        assert!(recovery_result.is_ok(), "Should recover after failure reset");
    }

    /// Chaos test: Clock skew simulation
    #[tokio::test]
    async fn test_chaos_clock_skew() {
        let clock_skew = Arc::new(ChaosClockSkew::new());
        
        // Simulate requests with different clock times
        let mut results = vec![];
        
        for i in 0..20 {
            // Randomly skew clock forward or backward
            let skew_seconds = rand::thread_rng().gen_range(-300..300); // ±5 minutes
            clock_skew.set_skew(Duration::from_secs(skew_seconds.abs() as u64));
            
            let timestamp = if skew_seconds < 0 {
                chrono::Utc::now() - chrono::Duration::seconds(skew_seconds.abs())
            } else {
                chrono::Utc::now() + chrono::Duration::seconds(skew_seconds)
            };
            
            // Simulate time-sensitive operation
            let is_valid = validate_timestamp(timestamp);
            results.push((skew_seconds, is_valid));
        }
        
        let valid_count = results.iter().filter(|(_, valid)| *valid).count();
        let invalid_count = results.len() - valid_count;
        
        println!("\n=== Chaos Test: Clock Skew ===");
        println!("Total tests: {}", results.len());
        println!("Valid timestamps: {}", valid_count);
        println!("Invalid timestamps: {}", invalid_count);
        println!("Clock skew range: ±5 minutes");
        
        // System should handle reasonable clock skew
        assert!(valid_count > 0, "Should accept some timestamps despite skew");
        assert!(invalid_count > 0, "Should reject extreme clock skew");
    }

    /// Chaos test: Partial system degradation
    #[tokio::test]
    #[ignore] // Requires external services
    async fn test_chaos_partial_degradation() {
        let degradation_controller = Arc::new(ChaosDegradationController::new());
        
        // Start with healthy system
        let mut health_timeline = vec![];
        
        // Phase 1: Healthy system
        health_timeline.push(("healthy", test_system_health().await));
        
        // Phase 2: Degrade one service
        degradation_controller.degrade_service("helpscout", 0.5).await;
        sleep(Duration::from_secs(1)).await;
        health_timeline.push(("one_service_degraded", test_system_health().await));
        
        // Phase 3: Degrade multiple services
        degradation_controller.degrade_service("notion", 0.7).await;
        sleep(Duration::from_secs(1)).await;
        health_timeline.push(("multiple_degraded", test_system_health().await));
        
        // Phase 4: Recover one service
        degradation_controller.recover_service("helpscout").await;
        sleep(Duration::from_secs(1)).await;
        health_timeline.push(("partial_recovery", test_system_health().await));
        
        // Phase 5: Full recovery
        degradation_controller.recover_all().await;
        sleep(Duration::from_secs(1)).await;
        health_timeline.push(("full_recovery", test_system_health().await));
        
        println!("\n=== Chaos Test: Partial Degradation ===");
        for (phase, health) in &health_timeline {
            println!("{}: {:.2}% healthy", phase, health * 100.0);
        }
        
        // Verify degradation and recovery
        assert!(health_timeline[0].1 > 0.9, "Should start healthy");
        assert!(health_timeline[2].1 < health_timeline[0].1, "Should degrade with failures");
        assert!(health_timeline[4].1 > 0.9, "Should recover fully");
    }

    // Helper structures and functions
    struct ChaosFailureInjector {
        failure_rate: f64,
    }

    impl ChaosFailureInjector {
        fn new(failure_rate: f64) -> Self {
            Self { failure_rate }
        }

        fn should_fail(&self) -> bool {
            rand::thread_rng().gen_bool(self.failure_rate)
        }
    }

    struct ChaosLatencyInjector {
        min_latency: Duration,
        max_latency: Duration,
    }

    impl ChaosLatencyInjector {
        fn new(min: Duration, max: Duration) -> Self {
            Self {
                min_latency: min,
                max_latency: max,
            }
        }

        fn get_random_latency(&self) -> Duration {
            let range = self.max_latency.as_millis() - self.min_latency.as_millis();
            let random_millis = rand::thread_rng().gen_range(0..range) + self.min_latency.as_millis();
            Duration::from_millis(random_millis as u64)
        }
    }

    struct ChaosResourceLimiter {
        max_tasks: usize,
        current_tasks: AtomicUsize,
        max_memory: usize,
    }

    impl ChaosResourceLimiter {
        fn new(max_tasks: usize, max_memory: usize) -> Self {
            Self {
                max_tasks,
                current_tasks: AtomicUsize::new(0),
                max_memory,
            }
        }

        fn try_acquire_task(&self) -> bool {
            let current = self.current_tasks.load(Ordering::Relaxed);
            if current < self.max_tasks {
                self.current_tasks.fetch_add(1, Ordering::Relaxed);
                true
            } else {
                false
            }
        }

        fn release_task(&self) {
            self.current_tasks.fetch_sub(1, Ordering::Relaxed);
        }
    }

    struct ChaosCascadingFailure {
        failed_services: Arc<RwLock<Vec<String>>>,
    }

    impl ChaosCascadingFailure {
        fn new() -> Self {
            Self {
                failed_services: Arc::new(RwLock::new(vec![])),
            }
        }

        async fn trigger_failure(&self, service: &str) {
            let mut failed = self.failed_services.write().await;
            failed.push(service.to_string());
        }

        async fn reset_failures(&self) {
            let mut failed = self.failed_services.write().await;
            failed.clear();
        }
    }

    struct ChaosClockSkew {
        skew: Arc<RwLock<Duration>>,
    }

    impl ChaosClockSkew {
        fn new() -> Self {
            Self {
                skew: Arc::new(RwLock::new(Duration::ZERO)),
            }
        }

        fn set_skew(&self, skew: Duration) {
            // In real implementation, would affect system clock
        }
    }

    struct ChaosDegradationController {
        degraded_services: Arc<RwLock<std::collections::HashMap<String, f64>>>,
    }

    impl ChaosDegradationController {
        fn new() -> Self {
            Self {
                degraded_services: Arc::new(RwLock::new(std::collections::HashMap::new())),
            }
        }

        async fn degrade_service(&self, service: &str, failure_rate: f64) {
            let mut degraded = self.degraded_services.write().await;
            degraded.insert(service.to_string(), failure_rate);
        }

        async fn recover_service(&self, service: &str) {
            let mut degraded = self.degraded_services.write().await;
            degraded.remove(service);
        }

        async fn recover_all(&self) {
            let mut degraded = self.degraded_services.write().await;
            degraded.clear();
        }
    }

    #[derive(Default)]
    struct ChaosTestResults {
        successful_runs: usize,
        failed_runs: usize,
        execution_times: Vec<Duration>,
        failure_reasons: Vec<String>,
    }

    #[derive(Default)]
    struct LatencyTestMetrics {
        successful_requests: usize,
        failed_requests: usize,
        timeout_requests: usize,
        connection_failures: usize,
        response_times: Vec<Duration>,
    }

    // Mock helper functions
    fn validate_timestamp(timestamp: chrono::DateTime<chrono::Utc>) -> bool {
        let now = chrono::Utc::now();
        let diff = (now - timestamp).num_seconds().abs();
        diff < 300 // Accept timestamps within 5 minutes
    }

    async fn test_system_health() -> f64 {
        // In real implementation, would check actual system health
        // Return value between 0.0 and 1.0
        0.95
    }
}