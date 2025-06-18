//! Integration example demonstrating the AI token counting and cost estimation system

use crate::core::ai::tokens::{
    Model, Provider, TokenUsage, CostBreakdown, UsageRecord,
    TokenCounterBuilder, PricingEngine, PricingConfig,
    BudgetLimits, LimitConfig,
};
use crate::core::ai::tokens::limits::GlobalLimits;
use crate::core::ai::tokens::counter::TokenCounter;
use rust_decimal::{Decimal, prelude::FromPrimitive};

/// Example demonstrating basic token counting and cost estimation
pub async fn run_basic_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI Token Counting System - Basic Example");
    println!("============================================");
    
    // 1. Initialize token counter
    let counter = TokenCounterBuilder::new().build()?;
    
    // 2. Count tokens for different models
    let sample_text = "Hello! This is a sample AI request that we want to analyze for token usage and cost.";
    
    let gpt4_tokens = counter.estimate_tokens(sample_text, &Model::Gpt4).await?;
    let claude_tokens = counter.estimate_tokens(sample_text, &Model::Claude3Sonnet).await?;
    
    println!("Token Estimates:");
    println!("  GPT-4: {} tokens", gpt4_tokens);
    println!("  Claude-3 Sonnet: {} tokens", claude_tokens);
    
    // 3. Calculate costs
    let pricing = PricingEngine::new(PricingConfig::default());
    
    let usage = TokenUsage::new(gpt4_tokens, 0); // Only input tokens for estimation
    let gpt4_cost = pricing.calculate_cost(&usage, &Model::Gpt4)?;
    
    let usage = TokenUsage::new(claude_tokens, 0);
    let claude_cost = pricing.calculate_cost(&usage, &Model::Claude3Sonnet)?;
    
    println!("\nCost Estimates:");
    println!("  GPT-4: ${:.6}", gpt4_cost.total_cost);
    println!("  Claude-3 Sonnet: ${:.6}", claude_cost.total_cost);
    
    // 4. Demonstrate budget limits
    let mut limit_config = LimitConfig::default();
    limit_config.global_limits = GlobalLimits {
        daily_cost_limit: Some(Decimal::from_f64(1.0).unwrap()),
        monthly_cost_limit: None,
        daily_token_limit: Some(10000),
        monthly_token_limit: None,
        requests_per_minute: Some(60),
        requests_per_hour: Some(1000),
        enabled: true,
    };
    
    let budget_limits = BudgetLimits::new(limit_config);
    
    let test_usage = TokenUsage::new(1000, 500);
    let test_cost = pricing.calculate_cost(&test_usage, &Model::Gpt4)?;
    
    let allowed = budget_limits.check_request_allowed(
        &Provider::OpenAI,
        &Model::Gpt4,
        &test_usage,
        &test_cost,
        Some("example_user"),
    ).await?;
    
    println!("\nBudget Check:");
    println!("  Request cost: ${:.6}", test_cost.total_cost);
    println!("  Daily limit: $1.00");
    println!("  Request allowed: {}", allowed);
    
    if allowed {
        budget_limits.record_usage(
            &Provider::OpenAI,
            &Model::Gpt4,
            &test_usage,
            &test_cost,
            Some("example_user"),
        ).await?;
        println!("  Usage recorded successfully");
    }
    
    println!("\n✅ Basic example completed successfully!");
    Ok(())
}

/// Example demonstrating cost comparison between models
pub fn run_cost_comparison_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💰 Cost Comparison Example");
    println!("===========================");
    
    let pricing = PricingEngine::new(PricingConfig::default());
    let usage = TokenUsage::new(10000, 5000); // Large usage for comparison
    
    let models = vec![
        Model::Gpt4,
        Model::Gpt35Turbo,
        Model::Claude3Opus,
        Model::Claude3Sonnet,
        Model::Claude3Haiku,
    ];
    
    println!("Cost comparison for 10,000 input + 5,000 output tokens:");
    
    for model in &models {
        let cost = pricing.calculate_cost(&usage, model)?;
        println!("  {:?}: ${:.4}", model, cost.total_cost);
    }
    
    // Direct comparison between two models
    let comparison = pricing.compare_model_costs(
        &usage,
        &Model::Gpt4,
        &Model::Claude3Haiku,
    )?;
    
    println!("\nGPT-4 vs Claude-3 Haiku:");
    println!("  GPT-4: ${:.4}", comparison.cost_a.total_cost);
    println!("  Claude-3 Haiku: ${:.4}", comparison.cost_b.total_cost);
    println!("  Savings with Claude-3 Haiku: ${:.4} ({:.1}%)",
             comparison.savings, comparison.percentage_difference);
    println!("  Cheaper model: {:?}", comparison.cheaper_model);
    
    println!("\n✅ Cost comparison completed!");
    Ok(())
}

/// Example demonstrating usage record creation
pub fn run_usage_tracking_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Usage Tracking Example");
    println!("==========================");
    
    let usage = TokenUsage::new(1500, 800);
    let cost = CostBreakdown::new(
        Decimal::from_f64(0.045).unwrap(), // $0.045 input
        Decimal::from_f64(0.048).unwrap(), // $0.048 output
    );
    
    let record = UsageRecord::new(
        Provider::OpenAI,
        Model::Gpt4,
        usage,
        cost,
    )
    .with_user_id("demo_user_123".to_string())
    .with_workflow_id(uuid::Uuid::new_v4())
    .with_metadata("request_type".to_string(), serde_json::json!("completion"))
    .with_metadata("application".to_string(), serde_json::json!("ai-assistant"));
    
    println!("Usage Record Created:");
    println!("  ID: {}", record.id);
    println!("  Provider: {:?}", record.provider);
    println!("  Model: {:?}", record.model);
    println!("  Input Tokens: {}", record.token_usage.input_tokens);
    println!("  Output Tokens: {}", record.token_usage.output_tokens);
    println!("  Total Cost: ${:.6}", record.cost_breakdown.total_cost);
    println!("  User ID: {:?}", record.user_id);
    println!("  Workflow ID: {:?}", record.workflow_id);
    println!("  Metadata: {}", serde_json::to_string_pretty(&record.metadata)?);
    
    println!("\n✅ Usage tracking example completed!");
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_integration() {
        run_basic_example().await.expect("Basic example should succeed");
    }
    
    #[test]
    fn test_cost_comparison_integration() {
        run_cost_comparison_example().expect("Cost comparison should succeed");
    }
    
    #[test]
    fn test_usage_tracking_integration() {
        run_usage_tracking_example().expect("Usage tracking should succeed");
    }
}