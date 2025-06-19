//! Node Processing Benchmarks
//!
//! This benchmark validates the claim: "sub-millisecond node processing"
//! It measures the execution time of various workflow nodes under different conditions.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use workflow_engine_core::{
    nodes::{Node, Router, ParallelNode, AsyncNode, AsyncNodeAdapter},
    task::TaskContext,
    error::WorkflowError,
};
use serde_json::{json, Value};
use std::time::Duration;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Simple compute node for benchmarking
#[derive(Debug)]
struct ComputeNode {
    complexity: usize,
}

impl Node for ComputeNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let data: Value = context.get_event_data()?;
        
        // Simulate computation based on complexity
        let mut result = 0;
        for i in 0..self.complexity {
            result = black_box(result ^ i);
        }
        
        context.update_node("result", json!({
            "computed": result,
            "input": data,
            "node_id": context.get_current_node()
        }));
        
        Ok(context)
    }
}

/// I/O simulation node
#[derive(Debug)]
struct IONode {
    operations: usize,
}

impl Node for IONode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let mut data = Vec::new();
        
        // Simulate I/O operations
        for i in 0..self.operations {
            data.push(json!({
                "index": i,
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "data": format!("Operation {}", i)
            }));
        }
        
        context.update_node("io_results", json!(data));
        Ok(context)
    }
}

/// JSON transformation node
#[derive(Debug)]
struct TransformNode;

impl Node for TransformNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let data: Value = context.get_event_data()?;
        
        // Perform various JSON transformations
        let transformed = json!({
            "original": data,
            "transformed": {
                "uppercase": data.as_str().map(|s| s.to_uppercase()),
                "length": data.as_str().map(|s| s.len()),
                "reversed": data.as_str().map(|s| s.chars().rev().collect::<String>()),
            },
            "metadata": {
                "processed_at": chrono::Utc::now().to_rfc3339(),
                "node_version": "1.0"
            }
        });
        
        context.update_node("transformed", transformed);
        Ok(context)
    }
}

/// Async computation node
#[derive(Debug)]
struct AsyncComputeNode {
    delay_micros: u64,
}

#[async_trait::async_trait]
impl AsyncNode for AsyncComputeNode {
    async fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Simulate async operation
        tokio::time::sleep(Duration::from_micros(self.delay_micros)).await;
        
        let data: Value = context.get_event_data()?;
        context.update_node("async_result", json!({
            "processed": true,
            "delay_micros": self.delay_micros,
            "input": data
        }));
        
        Ok(context)
    }
}

/// Create test contexts with different data sizes
fn create_test_context(data_size: usize) -> TaskContext {
    let data = match data_size {
        256 => json!({"small": "x".repeat(200)}),
        1024 => json!({"medium": "x".repeat(900)}),
        4096 => json!({"large": "x".repeat(4000)}),
        _ => json!({"test": "data"}),
    };
    
    TaskContext::new(
        Uuid::new_v4(),
        "test_workflow".to_string(),
        data,
    )
}

/// Benchmark single node execution
fn benchmark_single_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_node_processing");
    group.measurement_time(Duration::from_secs(20));
    
    // Test different node types and complexities
    let scenarios = vec![
        ("simple_compute_low", Box::new(ComputeNode { complexity: 10 }) as Box<dyn Node>),
        ("simple_compute_medium", Box::new(ComputeNode { complexity: 100 }) as Box<dyn Node>),
        ("simple_compute_high", Box::new(ComputeNode { complexity: 1000 }) as Box<dyn Node>),
        ("io_operations_10", Box::new(IONode { operations: 10 }) as Box<dyn Node>),
        ("io_operations_100", Box::new(IONode { operations: 100 }) as Box<dyn Node>),
        ("json_transform", Box::new(TransformNode) as Box<dyn Node>),
    ];
    
    for (name, node) in scenarios {
        group.bench_function(name, |b| {
            let context = create_test_context(256);
            b.iter(|| {
                let ctx = context.clone();
                black_box(node.process(ctx))
            });
        });
    }
    
    group.finish();
}

/// Benchmark router node execution
fn benchmark_router_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("router_node_processing");
    
    // Create router with multiple branches
    let mut router = Router::new("test_router");
    router.add_branch("small", Box::new(ComputeNode { complexity: 10 }));
    router.add_branch("medium", Box::new(ComputeNode { complexity: 100 }));
    router.add_branch("large", Box::new(ComputeNode { complexity: 1000 }));
    
    let conditions = vec![
        ("route_small", json!({"route": "small"})),
        ("route_medium", json!({"route": "medium"})),
        ("route_large", json!({"route": "large"})),
    ];
    
    for (name, data) in conditions {
        group.bench_function(name, |b| {
            let mut context = TaskContext::new(
                Uuid::new_v4(),
                "router_test".to_string(),
                data.clone(),
            );
            context.update_node("route", data["route"].clone());
            
            b.iter(|| {
                let ctx = context.clone();
                black_box(router.process(ctx))
            });
        });
    }
    
    group.finish();
}

/// Benchmark parallel node execution
fn benchmark_parallel_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_node_processing");
    group.measurement_time(Duration::from_secs(20));
    
    let runtime = Runtime::new().unwrap();
    
    // Test different parallel configurations
    let configs = vec![
        ("parallel_2_nodes", 2),
        ("parallel_4_nodes", 4),
        ("parallel_8_nodes", 8),
        ("parallel_16_nodes", 16),
    ];
    
    for (name, node_count) in configs {
        group.throughput(Throughput::Elements(node_count));
        
        let mut parallel_node = ParallelNode::new("parallel_test");
        for i in 0..node_count {
            parallel_node.add_node(
                format!("node_{}", i),
                Box::new(ComputeNode { complexity: 100 })
            );
        }
        
        group.bench_function(name, |b| {
            let context = create_test_context(256);
            b.iter(|| {
                let ctx = context.clone();
                runtime.block_on(async {
                    black_box(parallel_node.process_async(ctx).await)
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark async node execution
fn benchmark_async_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_node_processing");
    
    let runtime = Runtime::new().unwrap();
    
    // Test different async delays
    let delays = vec![
        ("async_10us", 10),
        ("async_100us", 100),
        ("async_500us", 500),
        ("async_1000us", 1000),
    ];
    
    for (name, delay_micros) in delays {
        let node = AsyncNodeAdapter::new(
            Arc::new(AsyncComputeNode { delay_micros })
        );
        
        group.bench_function(name, |b| {
            let context = create_test_context(256);
            b.iter(|| {
                let ctx = context.clone();
                runtime.block_on(async {
                    black_box(node.process_async(ctx).await)
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark workflow with different data sizes
fn benchmark_data_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_size_impact");
    
    let node = ComputeNode { complexity: 100 };
    let data_sizes = vec![
        ("256_bytes", 256),
        ("1_kb", 1024),
        ("4_kb", 4096),
    ];
    
    for (name, size) in data_sizes {
        group.throughput(Throughput::Bytes(size));
        
        group.bench_function(name, |b| {
            let context = create_test_context(size as usize);
            b.iter(|| {
                let ctx = context.clone();
                black_box(node.process(ctx))
            });
        });
    }
    
    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    // Test nodes that allocate different amounts of memory
    let scenarios = vec![
        ("no_allocation", 0),
        ("small_allocation", 1024),
        ("medium_allocation", 1024 * 10),
        ("large_allocation", 1024 * 100),
    ];
    
    for (name, allocation_size) in scenarios {
        group.bench_function(name, |b| {
            let context = create_test_context(256);
            b.iter(|| {
                let mut ctx = context.clone();
                
                // Simulate memory allocation
                if allocation_size > 0 {
                    let data: Vec<u8> = vec![0; allocation_size];
                    ctx.update_node("allocated", json!({
                        "size": allocation_size,
                        "checksum": data.iter().sum::<u8>()
                    }));
                }
                
                black_box(ctx)
            });
        });
    }
    
    group.finish();
}

/// Benchmark error handling overhead
fn benchmark_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    #[derive(Debug)]
    struct ErrorNode {
        should_error: bool,
    }
    
    impl Node for ErrorNode {
        fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
            if self.should_error {
                Err(WorkflowError::NodeError {
                    node_id: "error_node".to_string(),
                    message: "Simulated error".to_string(),
                })
            } else {
                Ok(context)
            }
        }
    }
    
    group.bench_function("success_path", |b| {
        let node = ErrorNode { should_error: false };
        let context = create_test_context(256);
        
        b.iter(|| {
            let ctx = context.clone();
            black_box(node.process(ctx))
        });
    });
    
    group.bench_function("error_path", |b| {
        let node = ErrorNode { should_error: true };
        let context = create_test_context(256);
        
        b.iter(|| {
            let ctx = context.clone();
            black_box(node.process(ctx))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_node,
    benchmark_router_node,
    benchmark_parallel_node,
    benchmark_async_node,
    benchmark_data_size_impact,
    benchmark_memory_patterns,
    benchmark_error_handling
);

criterion_main!(benches);