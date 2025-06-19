//! Full Workflow Execution Benchmarks
//!
//! This benchmark measures the performance of complete workflow executions
//! including multiple nodes, routing, parallel processing, and AI operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use workflow_engine_core::{
    workflow::builder::WorkflowBuilder,
    nodes::{Node, Router, ParallelNode},
    task::TaskContext,
    error::WorkflowError,
    ai::tokens::{TokenCounter, Model, PricingEngine},
    config::pricing::PricingEngineConfig,
};
use serde_json::{json, Value};
use std::time::Duration;
use uuid::Uuid;
use tokio::runtime::Runtime;

/// AI processing node that counts tokens
#[derive(Debug)]
struct AIProcessingNode {
    model: Model,
}

impl Node for AIProcessingNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let data: Value = context.get_event_data()?;
        let text = data.as_str().unwrap_or("Default text for token counting");
        
        // Count tokens
        let counter = TokenCounter::default();
        let token_count = counter.count_tokens(text, &self.model)
            .map_err(|e| WorkflowError::NodeError {
                node_id: "ai_node".to_string(),
                message: format!("Token counting failed: {}", e),
            })?;
        
        context.update_node("ai_result", json!({
            "model": format!("{:?}", self.model),
            "token_count": token_count,
            "estimated_cost": 0.001 * token_count as f64,
        }));
        
        Ok(context)
    }
}

/// Data validation node
#[derive(Debug)]
struct ValidationNode;

impl Node for ValidationNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let data: Value = context.get_event_data()?;
        
        // Perform various validations
        let is_valid = data.is_object() && 
                      data.get("required_field").is_some() &&
                      data.get("data").map(|d| d.is_string()).unwrap_or(false);
        
        if !is_valid {
            return Err(WorkflowError::NodeError {
                node_id: "validation".to_string(),
                message: "Data validation failed".to_string(),
            });
        }
        
        context.update_node("validation_passed", json!(true));
        Ok(context)
    }
}

/// Data transformation node
#[derive(Debug)]
struct TransformationNode;

impl Node for TransformationNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let data: Value = context.get_event_data()?;
        
        // Complex transformation logic
        let transformed = json!({
            "original": data.clone(),
            "transformed": {
                "fields": data.as_object().map(|obj| obj.keys().count()).unwrap_or(0),
                "size": serde_json::to_string(&data).unwrap().len(),
                "hash": format!("{:x}", md5::compute(serde_json::to_string(&data).unwrap().as_bytes())),
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        context.update_node("transformed_data", transformed);
        Ok(context)
    }
}

/// Storage simulation node
#[derive(Debug)]
struct StorageNode {
    latency_micros: u64,
}

impl Node for StorageNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Simulate storage latency
        std::thread::sleep(Duration::from_micros(self.latency_micros));
        
        let data: Value = context.get_event_data()?;
        context.update_node("stored", json!({
            "storage_id": Uuid::new_v4().to_string(),
            "size_bytes": serde_json::to_string(&data).unwrap().len(),
            "stored_at": chrono::Utc::now().to_rfc3339(),
        }));
        
        Ok(context)
    }
}

/// Create a simple workflow
fn create_simple_workflow() -> WorkflowBuilder {
    let mut builder = WorkflowBuilder::new("simple_workflow");
    
    builder.add_node("validate", Box::new(ValidationNode));
    builder.add_node("transform", Box::new(TransformationNode));
    builder.add_node("store", Box::new(StorageNode { latency_micros: 100 }));
    
    builder.add_edge("validate", "transform");
    builder.add_edge("transform", "store");
    
    builder
}

/// Create a complex workflow with routing
fn create_complex_workflow() -> WorkflowBuilder {
    let mut builder = WorkflowBuilder::new("complex_workflow");
    
    // Initial validation
    builder.add_node("validate", Box::new(ValidationNode));
    
    // Router for different processing paths
    let mut router = Router::new("process_router");
    router.add_branch("ai_path", Box::new(AIProcessingNode { model: Model::Gpt35Turbo }));
    router.add_branch("transform_path", Box::new(TransformationNode));
    builder.add_node("router", Box::new(router));
    
    // Parallel processing
    let mut parallel = ParallelNode::new("parallel_process");
    parallel.add_node("store1", Box::new(StorageNode { latency_micros: 50 }));
    parallel.add_node("store2", Box::new(StorageNode { latency_micros: 50 }));
    builder.add_node("parallel", Box::new(parallel));
    
    // Connect nodes
    builder.add_edge("validate", "router");
    builder.add_edge("router", "parallel");
    
    builder
}

/// Create test data for workflows
fn create_workflow_data(complexity: &str) -> Value {
    match complexity {
        "simple" => json!({
            "required_field": "present",
            "data": "Simple test data for workflow processing",
            "route": "transform_path"
        }),
        "medium" => json!({
            "required_field": "present",
            "data": "x".repeat(1000),
            "route": "ai_path",
            "metadata": {
                "source": "benchmark",
                "version": "1.0"
            }
        }),
        "complex" => {
            let mut obj = json!({
                "required_field": "present",
                "data": "x".repeat(5000),
                "route": "ai_path"
            });
            
            // Add nested data
            for i in 0..10 {
                obj[format!("field_{}", i)] = json!({
                    "index": i,
                    "data": "x".repeat(100),
                    "nested": {
                        "level": 2,
                        "content": "nested content"
                    }
                });
            }
            
            obj
        }
        _ => json!({"required_field": "present", "data": "default"})
    }
}

/// Benchmark simple workflow execution
fn benchmark_simple_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_workflow");
    group.measurement_time(Duration::from_secs(20));
    
    let workflow = create_simple_workflow().build().unwrap();
    let runtime = Runtime::new().unwrap();
    
    let data_complexities = vec![
        ("simple_data", create_workflow_data("simple")),
        ("medium_data", create_workflow_data("medium")),
        ("complex_data", create_workflow_data("complex")),
    ];
    
    for (name, data) in data_complexities {
        group.bench_function(name, |b| {
            b.iter(|| {
                let context = TaskContext::new(
                    Uuid::new_v4(),
                    "benchmark".to_string(),
                    data.clone(),
                );
                
                runtime.block_on(async {
                    black_box(workflow.execute(context).await)
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark complex workflow execution
fn benchmark_complex_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_workflow");
    group.measurement_time(Duration::from_secs(30));
    
    let workflow = create_complex_workflow().build().unwrap();
    let runtime = Runtime::new().unwrap();
    
    let scenarios = vec![
        ("transform_path", json!({
            "required_field": "present",
            "data": "Transform path data",
            "route": "transform_path"
        })),
        ("ai_path", json!({
            "required_field": "present",
            "data": "This is a longer text that will be processed by the AI node for token counting and cost estimation.",
            "route": "ai_path"
        })),
    ];
    
    for (name, data) in scenarios {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut context = TaskContext::new(
                    Uuid::new_v4(),
                    "benchmark".to_string(),
                    data.clone(),
                );
                context.update_node("route", data["route"].clone());
                
                runtime.block_on(async {
                    black_box(workflow.execute(context).await)
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark concurrent workflow execution
fn benchmark_concurrent_workflows(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_workflows");
    group.measurement_time(Duration::from_secs(30));
    
    let runtime = Runtime::new().unwrap();
    
    let concurrency_levels = vec![
        ("concurrent_10", 10),
        ("concurrent_50", 50),
        ("concurrent_100", 100),
        ("concurrent_500", 500),
    ];
    
    for (name, concurrent_count) in concurrency_levels {
        group.throughput(Throughput::Elements(concurrent_count));
        
        group.bench_function(name, |b| {
            let workflow = create_simple_workflow().build().unwrap();
            let data = create_workflow_data("simple");
            
            b.iter(|| {
                runtime.block_on(async {
                    let mut tasks = Vec::new();
                    
                    for _ in 0..concurrent_count {
                        let workflow = workflow.clone();
                        let data = data.clone();
                        
                        let task = tokio::spawn(async move {
                            let context = TaskContext::new(
                                Uuid::new_v4(),
                                "concurrent".to_string(),
                                data,
                            );
                            workflow.execute(context).await
                        });
                        
                        tasks.push(task);
                    }
                    
                    // Wait for all workflows to complete
                    for task in tasks {
                        let _ = task.await;
                    }
                });
            });
        });
    }
    
    group.finish();
}

/// Benchmark workflow with token counting and pricing
fn benchmark_ai_workflow_with_pricing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_workflow_pricing");
    
    let runtime = Runtime::new().unwrap();
    let pricing_config = PricingEngineConfig::default();
    let pricing_engine = PricingEngine::new(pricing_config);
    
    let text_lengths = vec![
        ("short_text", 100),
        ("medium_text", 1000),
        ("long_text", 5000),
    ];
    
    for (name, length) in text_lengths {
        group.bench_function(name, |b| {
            let text = "x".repeat(length);
            let data = json!({
                "required_field": "present",
                "data": text,
                "route": "ai_path"
            });
            
            b.iter(|| {
                let mut context = TaskContext::new(
                    Uuid::new_v4(),
                    "ai_benchmark".to_string(),
                    data.clone(),
                );
                
                runtime.block_on(async {
                    // Create workflow with AI processing
                    let ai_node = AIProcessingNode { model: Model::Gpt35Turbo };
                    let result = ai_node.process(context.clone());
                    
                    if let Ok(ctx) = result {
                        // Calculate pricing
                        if let Some(ai_result) = ctx.get_from_store_as::<Value>("ai_result") {
                            if let Some(token_count) = ai_result.get("token_count").and_then(|v| v.as_u64()) {
                                let usage = workflow_engine_core::ai::tokens::TokenUsage::new(
                                    token_count as u32,
                                    0
                                );
                                let _ = pricing_engine.calculate_cost(&usage, &Model::Gpt35Turbo);
                            }
                        }
                    }
                    
                    black_box(result)
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark error recovery in workflows
fn benchmark_error_recovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_recovery");
    
    #[derive(Debug)]
    struct FlakyNode {
        failure_rate: f32,
    }
    
    impl Node for FlakyNode {
        fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
            if rand::random::<f32>() < self.failure_rate {
                Err(WorkflowError::NodeError {
                    node_id: "flaky".to_string(),
                    message: "Random failure".to_string(),
                })
            } else {
                Ok(context)
            }
        }
    }
    
    let failure_rates = vec![
        ("no_failures", 0.0),
        ("10_percent_failures", 0.1),
        ("50_percent_failures", 0.5),
    ];
    
    for (name, rate) in failure_rates {
        group.bench_function(name, |b| {
            let mut builder = WorkflowBuilder::new("error_test");
            builder.add_node("flaky", Box::new(FlakyNode { failure_rate: rate }));
            builder.add_node("success", Box::new(ValidationNode));
            builder.add_edge("flaky", "success");
            
            let workflow = builder.build().unwrap();
            let runtime = Runtime::new().unwrap();
            
            b.iter(|| {
                let context = TaskContext::new(
                    Uuid::new_v4(),
                    "error_test".to_string(),
                    json!({"required_field": "present", "data": "test"}),
                );
                
                runtime.block_on(async {
                    black_box(workflow.execute(context).await)
                })
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_simple_workflow,
    benchmark_complex_workflow,
    benchmark_concurrent_workflows,
    benchmark_ai_workflow_with_pricing,
    benchmark_error_recovery
);

criterion_main!(benches);