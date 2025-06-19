//! Demonstration of the AI token counting and cost estimation system

use workflow_engine_core::ai::tokens::{
    Model, Provider, TokenUsage, CostBreakdown, UsageRecord,
    TokenCounterBuilder, PricingEngine, PricingConfig,
    UsageAnalytics, AnalyticsConfig, BudgetLimits, LimitConfig,
    TokenCounter,
};
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 AI Token Usage and Cost Estimation Demo");
    println!("==========================================\n");
    
    // 1. Token Counting Demo
    println!("1. Token Counting");
    println!("-----------------");
    
    let counter = TokenCounterBuilder::new()
        .build()
        .expect("Failed to build token counter");
    
    let sample_text = "Hello! This is a sample text to demonstrate our token counting system. It supports OpenAI, Anthropic, and AWS Bedrock models with accurate token estimation.";
    
    // OpenAI models
    let gpt4_tokens = counter.estimate_tokens(sample_text, &Model::Gpt4).await?;
    println!("GPT-4 estimated tokens: {}", gpt4_tokens);
    
    let gpt35_tokens = counter.estimate_tokens(sample_text, &Model::Gpt35Turbo).await?;
    println!("GPT-3.5 Turbo estimated tokens: {}", gpt35_tokens);
    
    // Anthropic models
    let claude_tokens = counter.estimate_tokens(sample_text, &Model::Claude3Sonnet).await?;
    println!("Claude-3 Sonnet estimated tokens: {}", claude_tokens);
    
    // Bedrock models
    let titan_tokens = counter.estimate_tokens(sample_text, &Model::TitanTextExpress).await?;
    println!("Titan Text Express estimated tokens: {}", titan_tokens);
    
    println!();
    
    // 2. Cost Estimation Demo
    println!("2. Cost Estimation");
    println!("------------------");
    
    let pricing_config = PricingConfig::default();
    let pricing_engine = PricingEngine::new(pricing_config);
    
    let usage = TokenUsage::new(1000, 500); // 1000 input, 500 output tokens
    
    // Calculate costs for different models
    let models = vec![
        Model::Gpt4,
        Model::Gpt35Turbo,
        Model::Claude3Opus,
        Model::Claude3Sonnet,
        Model::BedrockClaude3Haiku,
    ];
    
    for model in &models {
        let cost = pricing_engine.calculate_cost(&usage, model)?;
        println!("{:?}: ${:.6} (Input: ${:.6}, Output: ${:.6})", 
                 model, cost.total_cost, cost.input_cost, cost.output_cost);
    }
    
    println!();
    
    // 3. Cost Comparison Demo
    println!("3. Model Cost Comparison");
    println!("------------------------");
    
    let comparison = pricing_engine.compare_model_costs(
        &usage, 
        &Model::Gpt4, 
        &Model::Gpt35Turbo
    )?;
    
    println!("GPT-4 vs GPT-3.5 Turbo comparison:");
    println!("  GPT-4 cost: ${:.6}", comparison.cost_a.total_cost);
    println!("  GPT-3.5 cost: ${:.6}", comparison.cost_b.total_cost);
    println!("  Savings: ${:.6} ({:.1}%)", comparison.savings, comparison.percentage_difference);
    println!("  Cheaper model: {:?}", comparison.cheaper_model);
    
    println!();
    
    // 4. Usage Analytics Demo
    println!("4. Usage Analytics");
    println!("------------------");
    
    let analytics_config = AnalyticsConfig::default();
    let analytics = UsageAnalytics::new(analytics_config)?;
    
    // Simulate some usage records
    for i in 0..5 {
        let token_usage = TokenUsage::new(800 + i * 100, 400 + i * 50);
        let cost = pricing_engine.calculate_cost(&token_usage, &Model::Gpt4)?;
        
        let record = UsageRecord::new(
            Provider::OpenAI,
            Model::Gpt4,
            token_usage,
            cost,
        )
        .with_user_id(format!("user_{}", i % 3))
        .with_metadata("request_type".to_string(), serde_json::json!("demo"));
        
        analytics.record_usage(record).await?;
    }
    
    // Get usage statistics
    let stats = analytics.get_stats_for_period(
        Utc::now() - chrono::Duration::hours(1),
        Utc::now(),
    ).await?;
    
    println!("Usage Statistics:");
    println!("  Total Requests: {}", stats.total_requests);
    println!("  Total Input Tokens: {}", stats.total_input_tokens);
    println!("  Total Output Tokens: {}", stats.total_output_tokens);
    println!("  Total Cost: ${:.6}", stats.total_cost);
    
    // Get top models
    let top_models = analytics.get_top_models(3).await?;
    println!("\nTop Models by Usage:");
    for (model, model_stats) in &top_models {
        println!("  {:?}: {} requests, {} tokens, ${:.6}", 
                 model, model_stats.requests, 
                 model_stats.input_tokens + model_stats.output_tokens,
                 model_stats.cost);
    }
    
    println!();
    
    // 5. Budget Limits Demo
    println!("5. Budget Limits");
    println!("----------------");
    
    let mut limit_config = LimitConfig::default();
    // Set a very low daily limit for demonstration
    limit_config.global_limits.daily_cost_limit = Some(Decimal::from_f64(0.10).unwrap()); // $0.10
    
    let budget_limits = BudgetLimits::new(limit_config);
    
    let test_usage = TokenUsage::new(1000, 500);
    let test_cost = pricing_engine.calculate_cost(&test_usage, &Model::Gpt4)?;
    
    // Check if request would be allowed
    let allowed = budget_limits.check_request_allowed(
        &Provider::OpenAI,
        &Model::Gpt4,
        &test_usage,
        &test_cost,
        Some("demo_user"),
    ).await?;
    
    println!("Request cost: ${:.6}", test_cost.total_cost);
    println!("Daily limit: $0.10");
    println!("Request allowed: {}", allowed);
    
    if allowed {
        // Record the usage
        budget_limits.record_usage(
            &Provider::OpenAI,
            &Model::Gpt4,
            &test_usage,
            &test_cost,
            Some("demo_user"),
        ).await?;
        
        println!("Usage recorded successfully");
        
        // Check again - should now be blocked
        let allowed_again = budget_limits.check_request_allowed(
            &Provider::OpenAI,
            &Model::Gpt4,
            &test_usage,
            &test_cost,
            Some("demo_user"),
        ).await?;
        
        println!("Second request allowed: {}", allowed_again);
    }
    
    println!();
    
    // 6. Usage Forecasting Demo
    println!("6. Usage Forecasting");
    println!("--------------------");
    
    // Add more historical data for better forecasting
    for i in 0..14 {
        let days_ago = chrono::Duration::days(i);
        let timestamp = Utc::now() - days_ago;
        
        let daily_usage = TokenUsage::new((1000 + i * 50) as u32, (500 + i * 25) as u32);
        let daily_cost = pricing_engine.calculate_cost(&daily_usage, &Model::Gpt4)?;
        
        let mut record = UsageRecord::new(
            Provider::OpenAI,
            Model::Gpt4,
            daily_usage,
            daily_cost,
        );
        record.timestamp = timestamp;
        
        analytics.record_usage(record).await?;
    }
    
    // Generate forecast
    let forecast = analytics.forecast_usage(7).await?;
    
    println!("7-day forecast:");
    println!("  Predicted requests: {}", forecast.predicted_requests);
    println!("  Predicted tokens: {}", forecast.predicted_tokens);
    println!("  Predicted cost: ${:.6}", forecast.predicted_cost);
    println!("  Confidence level: {:.1}%", forecast.confidence_level * 100.0);
    
    println!("\n✅ Demo completed successfully!");
    
    Ok(())
}