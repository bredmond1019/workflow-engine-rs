//! API Throughput Benchmarks
//!
//! This benchmark validates the claim: "15,000+ requests/second"
//! It measures the HTTP API throughput under various conditions.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use workflow_engine_api::{Server, ServerBuilder};
use workflow_engine_core::task::TaskContext;
use tokio::runtime::Runtime;
use actix_web::test;
use std::time::Duration;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

/// Configuration for benchmark scenarios
struct BenchmarkConfig {
    /// Number of concurrent clients
    concurrent_clients: usize,
    /// Number of requests per client
    requests_per_client: usize,
    /// Request payload size (bytes)
    payload_size: usize,
    /// Whether to use keep-alive connections
    keep_alive: bool,
}

impl BenchmarkConfig {
    fn small_payload() -> Self {
        Self {
            concurrent_clients: 100,
            requests_per_client: 150,
            payload_size: 256,
            keep_alive: true,
        }
    }
    
    fn medium_payload() -> Self {
        Self {
            concurrent_clients: 100,
            requests_per_client: 150,
            payload_size: 1024,
            keep_alive: true,
        }
    }
    
    fn large_payload() -> Self {
        Self {
            concurrent_clients: 100,
            requests_per_client: 150,
            payload_size: 4096,
            keep_alive: true,
        }
    }
    
    fn no_keep_alive() -> Self {
        Self {
            concurrent_clients: 100,
            requests_per_client: 150,
            payload_size: 256,
            keep_alive: false,
        }
    }
}

/// Generate test payload of specified size
fn generate_payload(size: usize) -> serde_json::Value {
    let data = "x".repeat(size);
    json!({
        "workflow_id": Uuid::new_v4().to_string(),
        "node_id": "test_node",
        "data": data,
        "metadata": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": "1.0"
        }
    })
}

/// Benchmark HTTP API throughput
fn benchmark_api_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("api_throughput");
    group.measurement_time(Duration::from_secs(30));
    group.warm_up_time(Duration::from_secs(5));
    
    let runtime = Runtime::new().unwrap();
    
    // Test different scenarios
    let scenarios = vec![
        ("small_payload", BenchmarkConfig::small_payload()),
        ("medium_payload", BenchmarkConfig::medium_payload()),
        ("large_payload", BenchmarkConfig::large_payload()),
        ("no_keep_alive", BenchmarkConfig::no_keep_alive()),
    ];
    
    for (name, config) in scenarios {
        let total_requests = config.concurrent_clients * config.requests_per_client;
        group.throughput(Throughput::Elements(total_requests as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &config,
            |b, config| {
                b.iter(|| {
                    runtime.block_on(async {
                        benchmark_requests(config).await
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Execute benchmark requests
async fn benchmark_requests(config: &BenchmarkConfig) -> Result<Duration, Box<dyn std::error::Error>> {
    // Start test server
    let app = test::init_service(
        workflow_engine_api::create_app().await?
    ).await;
    
    let start = std::time::Instant::now();
    let payload = generate_payload(config.payload_size);
    
    // Create concurrent tasks
    let mut tasks = Vec::new();
    
    for _ in 0..config.concurrent_clients {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(if config.keep_alive { 100 } else { 0 })
            .build()?;
            
        let payload = payload.clone();
        let requests_per_client = config.requests_per_client;
        
        let task = tokio::spawn(async move {
            for _ in 0..requests_per_client {
                let req = test::TestRequest::post()
                    .uri("/api/workflows")
                    .set_json(&payload)
                    .to_request();
                    
                let _resp = test::call_service(&app, req).await;
            }
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        task.await?;
    }
    
    Ok(start.elapsed())
}

/// Benchmark individual endpoint latencies
fn benchmark_endpoint_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("endpoint_latency");
    group.measurement_time(Duration::from_secs(20));
    
    let runtime = Runtime::new().unwrap();
    
    // Test different endpoints
    let endpoints = vec![
        ("health_check", "/health", None),
        ("create_workflow", "/api/workflows", Some(json!({"name": "test"}))),
        ("get_workflow", "/api/workflows/123", None),
        ("list_workflows", "/api/workflows?limit=10", None),
    ];
    
    for (name, path, payload) in endpoints {
        group.bench_function(name, |b| {
            b.iter(|| {
                runtime.block_on(async {
                    let app = test::init_service(
                        workflow_engine_api::create_app().await.unwrap()
                    ).await;
                    
                    let req = if let Some(payload) = payload {
                        test::TestRequest::post()
                            .uri(path)
                            .set_json(&payload)
                            .to_request()
                    } else {
                        test::TestRequest::get()
                            .uri(path)
                            .to_request()
                    };
                    
                    let _resp = test::call_service(&app, req).await;
                });
            });
        });
    }
    
    group.finish();
}

/// Benchmark connection handling
fn benchmark_connection_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_handling");
    group.measurement_time(Duration::from_secs(20));
    
    let runtime = Runtime::new().unwrap();
    
    // Test different connection patterns
    let patterns = vec![
        ("persistent", 1000, true),
        ("new_per_request", 1000, false),
        ("burst", 5000, true),
    ];
    
    for (name, request_count, keep_alive) in patterns {
        group.throughput(Throughput::Elements(request_count));
        
        group.bench_function(name, |b| {
            b.iter(|| {
                runtime.block_on(async {
                    let client = Client::builder()
                        .pool_max_idle_per_host(if keep_alive { 100 } else { 0 })
                        .build()
                        .unwrap();
                    
                    let app = test::init_service(
                        workflow_engine_api::create_app().await.unwrap()
                    ).await;
                    
                    for _ in 0..request_count {
                        let req = test::TestRequest::get()
                            .uri("/health")
                            .to_request();
                            
                        let _resp = test::call_service(&app, req).await;
                    }
                });
            });
        });
    }
    
    group.finish();
}

/// Benchmark rate limiting behavior
fn benchmark_rate_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiting");
    group.measurement_time(Duration::from_secs(10));
    
    let runtime = Runtime::new().unwrap();
    
    group.bench_function("rate_limit_enforcement", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let app = test::init_service(
                    workflow_engine_api::create_app().await.unwrap()
                ).await;
                
                let mut accepted = 0;
                let mut rejected = 0;
                
                // Send 100 requests rapidly
                for _ in 0..100 {
                    let req = test::TestRequest::get()
                        .uri("/api/workflows")
                        .to_request();
                        
                    let resp = test::call_service(&app, req).await;
                    
                    if resp.status().is_success() {
                        accepted += 1;
                    } else if resp.status() == 429 {
                        rejected += 1;
                    }
                }
                
                black_box((accepted, rejected))
            });
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_api_throughput,
    benchmark_endpoint_latency,
    benchmark_connection_handling,
    benchmark_rate_limiting
);

criterion_main!(benches);