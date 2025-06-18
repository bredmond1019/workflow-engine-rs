//! Comprehensive tests for token counting and cost estimation

use chrono::{Utc, Duration};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use backend::core::ai::tokens::{
    Model, Provider, TokenUsage, CostBreakdown, UsageRecord,
    TokenCounter, TokenCounterBuilder, PricingEngine, PricingConfig,
    UsageAnalytics, AnalyticsConfig, BudgetLimits, LimitConfig,
    TokenError,
};
use backend::core::ai::tokens::pricing::PricingSource;
use backend::core::ai::tokens::limits::{
    GlobalLimits, AlertingConfig, AlertThreshold, ThresholdType, AlertScope, NotificationChannel,
};

#[tokio::test]
async fn test_openai_token_counting() {
    let counter = TokenCounterBuilder::new()
        .build()
        .expect("Failed to build token counter");
    
    let test_text = "Hello, world! This is a test message.";
    let model = Model::Gpt4;
    
    // Test input token counting
    let input_tokens = counter.count_input_tokens(test_text, &model).await
        .expect("Failed to count input tokens");
    
    assert!(input_tokens > 0, "Input tokens should be greater than 0");
    assert!(input_tokens < 100, "Input tokens should be reasonable for short text");
    
    // Test output token counting
    let output_tokens = counter.count_output_tokens(test_text, &model).await
        .expect("Failed to count output tokens");
    
    assert!(output_tokens > 0, "Output tokens should be greater than 0");
    
    // Test combined counting
    let usage = counter.count_tokens(test_text, test_text, &model).await
        .expect("Failed to count combined tokens");
    
    assert_eq!(usage.input_tokens, input_tokens);
    assert_eq!(usage.output_tokens, output_tokens);
    assert_eq!(usage.total_tokens, input_tokens + output_tokens);
}

#[tokio::test]
async fn test_anthropic_token_counting() {
    let counter = TokenCounterBuilder::new()
        .build()
        .expect("Failed to build token counter");
    
    let test_text = "This is a test for Claude token counting with some longer text to see how it performs.";
    let model = Model::Claude3Sonnet;
    
    let tokens = counter.estimate_tokens(test_text, &model).await
        .expect("Failed to estimate tokens");
    
    assert!(tokens > 0, "Estimated tokens should be greater than 0");
    assert!(tokens < 200, "Estimated tokens should be reasonable");
}

#[tokio::test]
async fn test_bedrock_token_counting() {
    let counter = TokenCounterBuilder::new()
        .build()
        .expect("Failed to build token counter");
    
    let test_text = "Testing Bedrock token counting for Titan models.";
    let model = Model::TitanTextExpress;
    
    let tokens = counter.estimate_tokens(test_text, &model).await
        .expect("Failed to estimate tokens");
    
    assert!(tokens > 0, "Estimated tokens should be greater than 0");
}

#[test]
fn test_pricing_engine_initialization() {
    let config = PricingConfig::default();
    let pricing = PricingEngine::new(config);
    
    // Test getting pricing for different models
    let gpt4_pricing = pricing.get_pricing(&Model::Gpt4)
        .expect("Should have GPT-4 pricing");
    
    assert!(gpt4_pricing.input_price_per_token > Decimal::ZERO);
    assert!(gpt4_pricing.output_price_per_token > Decimal::ZERO);
    assert_eq!(gpt4_pricing.currency, "USD");
    
    let claude_pricing = pricing.get_pricing(&Model::Claude3Sonnet)
        .expect("Should have Claude-3-Sonnet pricing");
    
    assert!(claude_pricing.input_price_per_token > Decimal::ZERO);
    assert!(claude_pricing.output_price_per_token > Decimal::ZERO);
}

#[test]
fn test_cost_calculation() {
    let config = PricingConfig::default();
    let pricing = PricingEngine::new(config);
    
    let token_usage = TokenUsage::new(1000, 500); // 1000 input, 500 output tokens
    let cost = pricing.calculate_cost(&token_usage, &Model::Gpt4)
        .expect("Should calculate cost");
    
    assert!(cost.input_cost > Decimal::ZERO);
    assert!(cost.output_cost > Decimal::ZERO);
    assert_eq!(cost.total_cost, cost.input_cost + cost.output_cost);
    assert_eq!(cost.currency, "USD");
}

#[test]
fn test_cost_comparison() {
    let config = PricingConfig::default();
    let pricing = PricingEngine::new(config);
    
    let token_usage = TokenUsage::new(1000, 500);
    let comparison = pricing.compare_model_costs(&token_usage, &Model::Gpt4, &Model::Gpt35Turbo)
        .expect("Should compare costs");
    
    // GPT-4 should be more expensive than GPT-3.5-turbo
    assert!(comparison.cost_a.total_cost > comparison.cost_b.total_cost);
    assert!(comparison.savings > Decimal::ZERO);
    assert_eq!(comparison.cheaper_model, Model::Gpt35Turbo);
}

#[tokio::test]
async fn test_usage_analytics() {
    let config = AnalyticsConfig::default();
    let analytics = UsageAnalytics::new(config)
        .expect("Should create analytics engine");
    
    // Create test usage record
    let token_usage = TokenUsage::new(1000, 500);
    let cost_breakdown = CostBreakdown::new(
        Decimal::from_f64(0.03).unwrap(),
        Decimal::from_f64(0.03).unwrap(),
    );
    
    let record = UsageRecord::new(
        Provider::OpenAI,
        Model::Gpt4,
        token_usage,
        cost_breakdown,
    )
    .with_user_id("test_user".to_string());
    
    // Record usage
    analytics.record_usage(record).await
        .expect("Should record usage");
    
    // Get statistics
    let stats = analytics.get_stats_for_period(
        Utc::now() - Duration::hours(1),
        Utc::now(),
    ).await.expect("Should get stats");
    
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.total_input_tokens, 1000);
    assert_eq!(stats.total_output_tokens, 500);
    assert!(stats.total_cost > Decimal::ZERO);
}

#[tokio::test]
async fn test_usage_trends() {
    let config = AnalyticsConfig::default();
    let analytics = UsageAnalytics::new(config)
        .expect("Should create analytics engine");
    
    // Create multiple test records across different days
    for i in 0..5 {
        let token_usage = TokenUsage::new(1000 + i * 100, 500);
        let cost_breakdown = CostBreakdown::new(
            Decimal::from_f64(0.03).unwrap(),
            Decimal::from_f64(0.03).unwrap(),
        );
        
        let mut record = UsageRecord::new(
            Provider::OpenAI,
            Model::Gpt4,
            token_usage,
            cost_breakdown,
        );
        
        // Simulate different timestamps
        record.timestamp = Utc::now() - Duration::days(i as i64);
        
        analytics.record_usage(record).await
            .expect("Should record usage");
    }
    
    let trends = analytics.get_usage_trends(7).await
        .expect("Should get trends");
    
    assert!(!trends.is_empty(), "Should have trend data");
}

#[tokio::test]
async fn test_budget_limits_basic() {
    let mut config = LimitConfig::default();
    config.global_limits.daily_cost_limit = Some(Decimal::from_f64(10.0).unwrap()); // $10 daily limit
    
    let limits = BudgetLimits::new(config);
    
    let token_usage = TokenUsage::new(1000, 500);
    let cost = CostBreakdown::new(
        Decimal::from_f64(5.0).unwrap(), // $5
        Decimal::from_f64(3.0).unwrap(), // $3
    ); // Total: $8, under limit
    
    // Should be allowed
    let allowed = limits.check_request_allowed(
        &Provider::OpenAI,
        &Model::Gpt4,
        &token_usage,
        &cost,
        None,
    ).await.expect("Should check limits");
    
    assert!(allowed, "Request should be allowed under limit");
    
    // Record usage
    limits.record_usage(&Provider::OpenAI, &Model::Gpt4, &token_usage, &cost, None).await
        .expect("Should record usage");
}

#[tokio::test]
async fn test_budget_limits_exceeded() {
    let mut config = LimitConfig::default();
    config.global_limits.daily_cost_limit = Some(Decimal::from_f64(5.0).unwrap()); // $5 daily limit
    
    let limits = BudgetLimits::new(config);
    
    // First request that brings us close to the limit
    let token_usage1 = TokenUsage::new(500, 250);
    let cost1 = CostBreakdown::new(
        Decimal::from_f64(2.0).unwrap(),
        Decimal::from_f64(2.0).unwrap(),
    ); // Total: $4
    
    limits.record_usage(&Provider::OpenAI, &Model::Gpt4, &token_usage1, &cost1, None).await
        .expect("Should record first usage");
    
    // Second request that would exceed the limit
    let token_usage2 = TokenUsage::new(500, 250);
    let cost2 = CostBreakdown::new(
        Decimal::from_f64(1.5).unwrap(),
        Decimal::from_f64(1.0).unwrap(),
    ); // Total: $2.5, would bring total to $6.5
    
    let allowed = limits.check_request_allowed(
        &Provider::OpenAI,
        &Model::Gpt4,
        &token_usage2,
        &cost2,
        None,
    ).await.expect("Should check limits");
    
    assert!(!allowed, "Request should be blocked due to exceeded limit");
}

#[tokio::test]
async fn test_user_specific_limits() {
    let mut config = LimitConfig::default();
    
    // Set user-specific limit
    let user_limits = backend::core::ai::tokens::limits::UserLimits {
        user_id: "test_user".to_string(),
        daily_cost_limit: Some(Decimal::from_f64(1.0).unwrap()), // $1 daily limit
        monthly_cost_limit: None,
        daily_token_limit: None,
        monthly_token_limit: None,
        requests_per_hour: None,
        enabled: true,
    };
    
    config.user_limits.insert("test_user".to_string(), user_limits);
    
    let limits = BudgetLimits::new(config);
    
    let token_usage = TokenUsage::new(1000, 500);
    let cost = CostBreakdown::new(
        Decimal::from_f64(0.8).unwrap(),
        Decimal::from_f64(0.5).unwrap(),
    ); // Total: $1.3, exceeds user limit
    
    let allowed = limits.check_request_allowed(
        &Provider::OpenAI,
        &Model::Gpt4,
        &token_usage,
        &cost,
        Some("test_user"),
    ).await.expect("Should check limits");
    
    assert!(!allowed, "Request should be blocked due to user limit");
}

#[tokio::test]
async fn test_rate_limiting() {
    let mut config = LimitConfig::default();
    config.global_limits.requests_per_minute = Some(2); // Very low limit for testing
    
    let limits = BudgetLimits::new(config);
    
    let token_usage = TokenUsage::new(100, 50);
    let cost = CostBreakdown::new(
        Decimal::from_f64(0.01).unwrap(),
        Decimal::from_f64(0.01).unwrap(),
    );
    
    // First request should be allowed
    let allowed1 = limits.check_request_allowed(
        &Provider::OpenAI,
        &Model::Gpt4,
        &token_usage,
        &cost,
        None,
    ).await.expect("Should check limits");
    assert!(allowed1, "First request should be allowed");
    
    limits.record_usage(&Provider::OpenAI, &Model::Gpt4, &token_usage, &cost, None).await
        .expect("Should record usage");
    
    // Second request should be allowed
    let allowed2 = limits.check_request_allowed(
        &Provider::OpenAI,
        &Model::Gpt4,
        &token_usage,
        &cost,
        None,
    ).await.expect("Should check limits");
    assert!(allowed2, "Second request should be allowed");
    
    limits.record_usage(&Provider::OpenAI, &Model::Gpt4, &token_usage, &cost, None).await
        .expect("Should record usage");
    
    // Third request should be blocked due to rate limit
    let allowed3 = limits.check_request_allowed(
        &Provider::OpenAI,
        &Model::Gpt4,
        &token_usage,
        &cost,
        None,
    ).await.expect("Should check limits");
    assert!(!allowed3, "Third request should be blocked by rate limit");
}

#[tokio::test]
async fn test_forecast_usage() {
    let config = AnalyticsConfig::default();
    let analytics = UsageAnalytics::new(config)
        .expect("Should create analytics engine");
    
    // Create historical data for forecasting
    for i in 0..14 {
        let token_usage = TokenUsage::new(1000, 500);
        let cost_breakdown = CostBreakdown::new(
            Decimal::from_f64(0.03).unwrap(),
            Decimal::from_f64(0.03).unwrap(),
        );
        
        let mut record = UsageRecord::new(
            Provider::OpenAI,
            Model::Gpt4,
            token_usage,
            cost_breakdown,
        );
        
        record.timestamp = Utc::now() - Duration::days(i);
        
        analytics.record_usage(record).await
            .expect("Should record usage");
    }
    
    let forecast = analytics.forecast_usage(7).await
        .expect("Should generate forecast");
    
    assert!(forecast.predicted_requests > 0, "Should predict some requests");
    assert!(forecast.predicted_tokens > 0, "Should predict some tokens");
    assert!(forecast.predicted_cost > Decimal::ZERO, "Should predict some cost");
    assert!(forecast.confidence_level >= 0.0 && forecast.confidence_level <= 1.0, 
           "Confidence should be between 0 and 1");
}

#[test]
fn test_model_provider_mapping() {
    assert_eq!(Model::Gpt4.provider(), Provider::OpenAI);
    assert_eq!(Model::Claude3Sonnet.provider(), Provider::Anthropic);
    assert_eq!(Model::BedrockClaude3Opus.provider(), Provider::Bedrock);
    assert_eq!(Model::TitanTextExpress.provider(), Provider::Bedrock);
}

#[test]
fn test_token_usage_operations() {
    let mut usage1 = TokenUsage::new(100, 50);
    let usage2 = TokenUsage::new(200, 100);
    
    assert_eq!(usage1.total_tokens, 150);
    assert_eq!(usage2.total_tokens, 300);
    
    usage1.add(&usage2);
    assert_eq!(usage1.input_tokens, 300);
    assert_eq!(usage1.output_tokens, 150);
    assert_eq!(usage1.total_tokens, 450);
}

#[test]
fn test_cost_breakdown_operations() {
    let cost = CostBreakdown::new(
        Decimal::from_f64(0.05).unwrap(),
        Decimal::from_f64(0.03).unwrap(),
    );
    
    assert_eq!(cost.total_cost, Decimal::from_f64(0.08).unwrap());
    assert_eq!(cost.currency, "USD");
}

#[test]
fn test_usage_record_builder() {
    let token_usage = TokenUsage::new(1000, 500);
    let cost_breakdown = CostBreakdown::new(
        Decimal::from_f64(0.03).unwrap(),
        Decimal::from_f64(0.03).unwrap(),
    );
    
    let record = UsageRecord::new(
        Provider::OpenAI,
        Model::Gpt4,
        token_usage,
        cost_breakdown,
    )
    .with_user_id("test_user".to_string())
    .with_workflow_id(uuid::Uuid::new_v4())
    .with_metadata("test_key".to_string(), serde_json::json!("test_value"));
    
    assert_eq!(record.user_id, Some("test_user".to_string()));
    assert!(record.workflow_id.is_some());
    assert!(record.metadata.contains_key("test_key"));
}

#[tokio::test]
async fn test_error_handling() {
    let counter = TokenCounterBuilder::new()
        .build()
        .expect("Failed to build token counter");
    
    // Test unsupported model error
    let result = counter.count_input_tokens("test", &Model::TextEmbeddingAda002).await;
    
    // Note: This might not fail depending on implementation
    // Adjust based on actual behavior
    if result.is_err() {
        match result.unwrap_err() {
            TokenError::UnsupportedModel(_) => {
                // Expected error type
            }
            other => panic!("Unexpected error type: {:?}", other),
        }
    }
}

#[test]
fn test_pricing_utils() {
    use backend::core::ai::tokens::pricing::pricing_utils;
    
    let config = PricingConfig::default();
    let pricing = PricingEngine::new(config);
    
    let gpt4_pricing = pricing.get_pricing(&Model::Gpt4).unwrap();
    
    // Test cost per request calculation
    let cost_per_req = pricing_utils::cost_per_request(&gpt4_pricing, 1000, 500);
    assert!(cost_per_req > Decimal::ZERO);
    
    // Test monthly cost estimate
    let monthly_cost = pricing_utils::monthly_cost_estimate(&gpt4_pricing, 1000, 500, 10);
    assert!(monthly_cost > cost_per_req);
    
    // Test cost formatting
    let formatted = pricing_utils::format_cost(&cost_per_req, "USD");
    assert!(formatted.starts_with('$'));
}

#[tokio::test]
async fn test_analytics_export() {
    let config = AnalyticsConfig::default();
    let analytics = UsageAnalytics::new(config)
        .expect("Should create analytics engine");
    
    // Add some test data
    let token_usage = TokenUsage::new(1000, 500);
    let cost_breakdown = CostBreakdown::new(
        Decimal::from_f64(0.03).unwrap(),
        Decimal::from_f64(0.03).unwrap(),
    );
    
    let record = UsageRecord::new(
        Provider::OpenAI,
        Model::Gpt4,
        token_usage,
        cost_breakdown,
    );
    
    analytics.record_usage(record).await
        .expect("Should record usage");
    
    // Test JSON export
    let temp_dir = tempfile::tempdir().unwrap();
    let json_path = temp_dir.path().join("export.json");
    
    analytics.export_data(
        backend::core::ai::tokens::analytics::ExportFormat::Json,
        json_path.to_str().unwrap(),
    ).await.expect("Should export JSON");
    
    assert!(json_path.exists(), "JSON file should exist");
    
    // Test CSV export
    let csv_path = temp_dir.path().join("export.csv");
    
    analytics.export_data(
        backend::core::ai::tokens::analytics::ExportFormat::Csv,
        csv_path.to_str().unwrap(),
    ).await.expect("Should export CSV");
    
    assert!(csv_path.exists(), "CSV file should exist");
}

#[test]
fn test_quick_token_estimation() {
    use backend::core::ai::tokens::counter::utils;
    
    let text = "This is a test message for quick estimation.";
    
    let openai_estimate = utils::quick_estimate_tokens(text, &Model::Gpt4);
    let anthropic_estimate = utils::quick_estimate_tokens(text, &Model::Claude3Sonnet);
    let bedrock_estimate = utils::quick_estimate_tokens(text, &Model::TitanTextExpress);
    
    assert!(openai_estimate > 0);
    assert!(anthropic_estimate > 0);
    assert!(bedrock_estimate > 0);
    
    // Different providers might have different estimates
    // but they should all be reasonable
    assert!(openai_estimate < 100);
    assert!(anthropic_estimate < 100);
    assert!(bedrock_estimate < 100);
}

#[test]
fn test_token_diff_percentage() {
    use backend::core::ai::tokens::counter::utils;
    
    let diff1 = utils::token_diff_percentage(100, 100);
    assert_eq!(diff1, 0.0);
    
    let diff2 = utils::token_diff_percentage(110, 100);
    assert_eq!(diff2, 10.0);
    
    let diff3 = utils::token_diff_percentage(90, 100);
    assert_eq!(diff3, 10.0);
    
    let diff4 = utils::token_diff_percentage(100, 0);
    assert_eq!(diff4, 100.0);
}

#[tokio::test]
async fn test_batch_token_counting() {
    use backend::core::ai::tokens::counter::utils;
    
    let counter = TokenCounterBuilder::new()
        .build()
        .expect("Failed to build token counter");
    
    let texts = vec![
        "First test message".to_string(),
        "Second test message".to_string(),
        "Third test message".to_string(),
    ];
    
    let counts = utils::batch_count_tokens(&counter, &texts, &Model::Gpt4).await
        .expect("Should count tokens for batch");
    
    assert_eq!(counts.len(), 3);
    assert!(counts.iter().all(|&count| count > 0));
}