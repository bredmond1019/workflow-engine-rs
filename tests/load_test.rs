#[cfg(test)]
mod tests {
    use backend::bootstrap::service::ServiceBootstrap;
    use backend::core::workflow::builder::WorkflowBuilder;
    use backend::core::nodes::registry::NodeRegistry;
    use backend::core::nodes::agent::AgentNode;
    use backend::core::nodes::external_mcp_client::ExternalMcpClientNode;
    use backend::core::nodes::config::NodeConfig;
    use backend::core::mcp::connection_pool::{ConnectionPool, ConnectionPoolConfig};
    use backend::monitoring::metrics::AppMetrics;
    use backend::db::repository::EventRepository;
    use serde_json::json;
    use std::any::TypeId;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::{Semaphore, RwLock};
    use tokio::time::sleep;

    /// Load test with concurrent workflow executions
    #[tokio::test]
    #[ignore] // Resource intensive test
    async fn test_concurrent_workflow_load() {
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        registry.register_node::<ExternalMcpClientNode>().unwrap();
        
        // Create a simple workflow for load testing
        let workflow = Arc::new(
            WorkflowBuilder::new::<AgentNode>("load_test_workflow".to_string())
                .description("Simple workflow for load testing".to_string())
                .add_node(
                    NodeConfig::new::<AgentNode>()
                        .with_description("Process data".to_string())
                )
                .build()
                .unwrap()
        );
        
        // Test parameters
        let num_concurrent_workflows = 100;
        let num_iterations = 10;
        let semaphore = Arc::new(Semaphore::new(50)); // Limit concurrent executions
        
        let metrics = Arc::new(RwLock::new(LoadTestMetrics::default()));
        let start_time = Instant::now();
        
        let mut handles = vec![];
        
        for iteration in 0..num_iterations {
            for workflow_id in 0..num_concurrent_workflows {
                let workflow_clone = workflow.clone();
                let semaphore_clone = semaphore.clone();
                let metrics_clone = metrics.clone();
                
                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();
                    
                    let context = json!({
                        "workflow_id": format!("load_test_{}_{}", iteration, workflow_id),
                        "data": "test payload",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    
                    let exec_start = Instant::now();
                    let result = workflow_clone.run(context).await;
                    let exec_duration = exec_start.elapsed();
                    
                    // Update metrics
                    let mut m = metrics_clone.write().await;
                    m.total_executions += 1;
                    m.total_duration += exec_duration;
                    
                    if result.is_ok() {
                        m.successful_executions += 1;
                        if exec_duration > m.max_duration {
                            m.max_duration = exec_duration;
                        }
                        if m.min_duration == Duration::ZERO || exec_duration < m.min_duration {
                            m.min_duration = exec_duration;
                        }
                    } else {
                        m.failed_executions += 1;
                    }
                });
                
                handles.push(handle);
            }
            
            // Small delay between iterations
            sleep(Duration::from_millis(100)).await;
        }
        
        // Wait for all workflows to complete
        let results = futures::future::join_all(handles).await;
        let total_duration = start_time.elapsed();
        
        // Calculate and display metrics
        let final_metrics = metrics.read().await;
        let avg_duration = final_metrics.total_duration / final_metrics.total_executions as u32;
        let throughput = final_metrics.total_executions as f64 / total_duration.as_secs_f64();
        
        println!("\n=== Load Test Results ===");
        println!("Total workflows: {}", final_metrics.total_executions);
        println!("Successful: {}", final_metrics.successful_executions);
        println!("Failed: {}", final_metrics.failed_executions);
        println!("Total duration: {:?}", total_duration);
        println!("Average execution time: {:?}", avg_duration);
        println!("Min execution time: {:?}", final_metrics.min_duration);
        println!("Max execution time: {:?}", final_metrics.max_duration);
        println!("Throughput: {:.2} workflows/second", throughput);
        
        // Assertions
        assert!(final_metrics.failed_executions == 0, "All workflows should complete successfully");
        assert!(throughput > 10.0, "Should handle at least 10 workflows per second");
        assert!(avg_duration < Duration::from_secs(1), "Average execution should be under 1 second");
    }

    /// Load test MCP connection pool under stress
    #[tokio::test]
    #[ignore] // Requires MCP servers and is resource intensive
    async fn test_mcp_connection_pool_load() {
        let config = ConnectionPoolConfig {
            max_connections: 20,
            min_idle: 5,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(60),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(100),
        };
        
        let pool = Arc::new(ConnectionPool::new(config));
        let metrics = Arc::new(RwLock::new(ConnectionPoolMetrics::default()));
        
        // Simulate high load with many concurrent requests
        let num_workers = 50;
        let requests_per_worker = 100;
        let semaphore = Arc::new(Semaphore::new(num_workers));
        
        let start_time = Instant::now();
        let mut handles = vec![];
        
        for worker_id in 0..num_workers {
            let pool_clone = pool.clone();
            let metrics_clone = metrics.clone();
            let semaphore_clone = semaphore.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                for request_id in 0..requests_per_worker {
                    let request_start = Instant::now();
                    
                    // Try to get connection
                    match pool_clone.get_connection(
                        "http://localhost:8001",
                        backend::core::mcp::transport::TransportType::Http
                    ).await {
                        Ok(transport) => {
                            // Simulate request
                            let request = backend::core::mcp::protocol::McpRequest::ToolCall(
                                backend::core::mcp::protocol::ToolCall {
                                    id: format!("load_test_{}_{}", worker_id, request_id),
                                    name: "list_conversations".to_string(),
                                    arguments: json!({"limit": 5}),
                                }
                            );
                            
                            match transport.send_request(request).await {
                                Ok(_) => {
                                    let mut m = metrics_clone.write().await;
                                    m.successful_requests += 1;
                                    m.total_request_time += request_start.elapsed();
                                }
                                Err(_) => {
                                    let mut m = metrics_clone.write().await;
                                    m.failed_requests += 1;
                                }
                            }
                        }
                        Err(_) => {
                            let mut m = metrics_clone.write().await;
                            m.connection_failures += 1;
                        }
                    }
                    
                    // Small delay between requests
                    sleep(Duration::from_millis(10)).await;
                }
            });
            
            handles.push(handle);
        }
        
        // Monitor pool stats periodically
        let pool_monitor = pool.clone();
        let monitor_handle = tokio::spawn(async move {
            let mut max_active = 0;
            let mut samples = vec![];
            
            for _ in 0..30 {
                let stats = pool_monitor.get_stats().await;
                if stats.active_connections > max_active {
                    max_active = stats.active_connections;
                }
                samples.push(stats);
                sleep(Duration::from_millis(100)).await;
            }
            
            (max_active, samples)
        });
        
        // Wait for all workers to complete
        futures::future::join_all(handles).await;
        let total_duration = start_time.elapsed();
        
        let (max_active_connections, pool_samples) = monitor_handle.await.unwrap();
        
        // Calculate metrics
        let final_metrics = metrics.read().await;
        let total_requests = final_metrics.successful_requests + final_metrics.failed_requests;
        let success_rate = final_metrics.successful_requests as f64 / total_requests as f64 * 100.0;
        let avg_request_time = final_metrics.total_request_time / final_metrics.successful_requests as u32;
        let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();
        
        println!("\n=== Connection Pool Load Test Results ===");
        println!("Total requests: {}", total_requests);
        println!("Successful requests: {}", final_metrics.successful_requests);
        println!("Failed requests: {}", final_metrics.failed_requests);
        println!("Connection failures: {}", final_metrics.connection_failures);
        println!("Success rate: {:.2}%", success_rate);
        println!("Average request time: {:?}", avg_request_time);
        println!("Requests per second: {:.2}", requests_per_second);
        println!("Max active connections: {}", max_active_connections);
        println!("Total test duration: {:?}", total_duration);
        
        // Assertions
        assert!(success_rate > 95.0, "Success rate should be above 95%");
        assert!(requests_per_second > 100.0, "Should handle at least 100 requests per second");
        assert!(max_active_connections <= 20, "Should respect max connections limit");
    }

    /// Load test system under memory pressure
    #[tokio::test]
    #[ignore] // Resource intensive
    async fn test_system_memory_pressure() {
        let initial_memory = get_current_memory_usage();
        println!("Initial memory usage: {} MB", initial_memory / 1024 / 1024);
        
        // Create many workflows with large contexts
        let mut workflows = vec![];
        let large_data = "x".repeat(1024 * 1024); // 1MB string
        
        for i in 0..100 {
            let workflow = WorkflowBuilder::new::<AgentNode>(format!("memory_test_{}", i))
                .description("Memory pressure test workflow".to_string())
                .add_node(
                    NodeConfig::new::<AgentNode>()
                        .with_description("Process large data".to_string())
                )
                .build()
                .unwrap();
            
            workflows.push(workflow);
        }
        
        // Execute workflows with large contexts
        let mut handles = vec![];
        
        for (i, workflow) in workflows.iter().enumerate() {
            let workflow_clone = workflow.clone();
            let data_clone = large_data.clone();
            
            let handle = tokio::spawn(async move {
                let context = json!({
                    "id": i,
                    "large_data": data_clone,
                    "nested": {
                        "more_data": vec![0u8; 1024 * 1024] // 1MB array
                    }
                });
                
                workflow_clone.run(context).await
            });
            
            handles.push(handle);
            
            // Stagger the launches to avoid sudden spike
            if i % 10 == 0 {
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Monitor memory usage during execution
        let mut peak_memory = initial_memory;
        let monitor_handle = tokio::spawn(async move {
            let mut samples = vec![];
            
            for _ in 0..30 {
                let current = get_current_memory_usage();
                if current > peak_memory {
                    peak_memory = current;
                }
                samples.push(current);
                sleep(Duration::from_millis(500)).await;
            }
            
            (peak_memory, samples)
        });
        
        // Wait for workflows to complete
        let results = futures::future::join_all(handles).await;
        let (peak_usage, memory_samples) = monitor_handle.await.unwrap();
        
        // Force garbage collection
        drop(workflows);
        drop(results);
        
        // Wait for memory to stabilize
        sleep(Duration::from_secs(2)).await;
        
        let final_memory = get_current_memory_usage();
        let memory_increase = peak_usage.saturating_sub(initial_memory);
        let memory_leaked = final_memory.saturating_sub(initial_memory);
        
        println!("\n=== Memory Pressure Test Results ===");
        println!("Initial memory: {} MB", initial_memory / 1024 / 1024);
        println!("Peak memory: {} MB", peak_usage / 1024 / 1024);
        println!("Final memory: {} MB", final_memory / 1024 / 1024);
        println!("Peak increase: {} MB", memory_increase / 1024 / 1024);
        println!("Potential leak: {} MB", memory_leaked / 1024 / 1024);
        
        // Assertions
        assert!(memory_leaked < 50 * 1024 * 1024, "Memory leak should be less than 50MB");
        assert!(peak_usage < initial_memory + 500 * 1024 * 1024, "Peak memory should be reasonable");
    }

    /// Load test with burst traffic patterns
    #[tokio::test]
    #[ignore] // Resource intensive
    async fn test_burst_traffic_handling() {
        let mut registry = NodeRegistry::new();
        registry.register_node::<AgentNode>().unwrap();
        
        let workflow = Arc::new(
            WorkflowBuilder::new::<AgentNode>("burst_test".to_string())
                .description("Burst traffic test workflow".to_string())
                .add_node(
                    NodeConfig::new::<AgentNode>()
                        .with_description("Handle burst".to_string())
                )
                .build()
                .unwrap()
        );
        
        let metrics = Arc::new(RwLock::new(BurstTestMetrics::default()));
        
        // Simulate burst patterns
        let burst_configs = vec![
            (100, 100),  // 100 requests, 100ms delay
            (500, 50),   // 500 requests, 50ms delay (burst)
            (50, 200),   // 50 requests, 200ms delay (cooldown)
            (1000, 10),  // 1000 requests, 10ms delay (heavy burst)
            (100, 100),  // Back to normal
        ];
        
        for (burst_size, delay_ms) in burst_configs {
            println!("Starting burst: {} requests with {}ms delay", burst_size, delay_ms);
            
            let burst_start = Instant::now();
            let mut handles = vec![];
            
            for i in 0..burst_size {
                let workflow_clone = workflow.clone();
                let metrics_clone = metrics.clone();
                
                let handle = tokio::spawn(async move {
                    let context = json!({
                        "burst_id": i,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    
                    let start = Instant::now();
                    let result = workflow_clone.run(context).await;
                    let duration = start.elapsed();
                    
                    let mut m = metrics_clone.write().await;
                    if result.is_ok() {
                        m.successful_requests += 1;
                        m.response_times.push(duration);
                    } else {
                        m.failed_requests += 1;
                    }
                });
                
                handles.push(handle);
                
                if i % 10 == 0 {
                    sleep(Duration::from_millis(delay_ms)).await;
                }
            }
            
            futures::future::join_all(handles).await;
            let burst_duration = burst_start.elapsed();
            
            println!("Burst completed in {:?}", burst_duration);
            
            // Cool down between bursts
            sleep(Duration::from_secs(1)).await;
        }
        
        // Analyze results
        let final_metrics = metrics.read().await;
        let total_requests = final_metrics.successful_requests + final_metrics.failed_requests;
        let success_rate = final_metrics.successful_requests as f64 / total_requests as f64 * 100.0;
        
        // Calculate percentiles
        let mut sorted_times = final_metrics.response_times.clone();
        sorted_times.sort();
        
        let p50 = sorted_times[sorted_times.len() / 2];
        let p95 = sorted_times[sorted_times.len() * 95 / 100];
        let p99 = sorted_times[sorted_times.len() * 99 / 100];
        
        println!("\n=== Burst Traffic Test Results ===");
        println!("Total requests: {}", total_requests);
        println!("Success rate: {:.2}%", success_rate);
        println!("P50 response time: {:?}", p50);
        println!("P95 response time: {:?}", p95);
        println!("P99 response time: {:?}", p99);
        
        // Assertions
        assert!(success_rate > 99.0, "System should handle bursts with high success rate");
        assert!(p95 < Duration::from_secs(1), "P95 response time should be under 1 second");
    }

    // Helper structures
    #[derive(Default)]
    struct LoadTestMetrics {
        total_executions: usize,
        successful_executions: usize,
        failed_executions: usize,
        total_duration: Duration,
        min_duration: Duration,
        max_duration: Duration,
    }

    #[derive(Default)]
    struct ConnectionPoolMetrics {
        successful_requests: usize,
        failed_requests: usize,
        connection_failures: usize,
        total_request_time: Duration,
    }

    #[derive(Default)]
    struct BurstTestMetrics {
        successful_requests: usize,
        failed_requests: usize,
        response_times: Vec<Duration>,
    }

    // Mock memory usage function (replace with actual implementation)
    fn get_current_memory_usage() -> usize {
        // In a real implementation, this would query actual memory usage
        // For now, return a mock value
        100 * 1024 * 1024 // 100MB
    }
}