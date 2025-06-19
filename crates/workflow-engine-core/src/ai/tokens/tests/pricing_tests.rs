//! Unit tests for token pricing calculations

#[cfg(test)]
mod tests {
    use super::super::*;
    use rust_decimal::Decimal;
    use crate::ai::tokens::{Model, Provider, TokenUsage, PricingEngine, VolumeTier};
    use crate::config::pricing::PricingEngineConfig;
    use crate::streaming::types::StreamMetadata;

    fn create_test_pricing_engine() -> PricingEngine {
        let config = PricingEngineConfig::default();
        PricingEngine::new(config)
    }

    #[test]
    fn test_basic_cost_calculation() {
        let engine = create_test_pricing_engine();
        let token_usage = TokenUsage::new(1000, 500);
        
        let result = engine.calculate_cost(&token_usage, &Model::Gpt35Turbo);
        assert!(result.is_ok());
        
        let cost = result.unwrap();
        assert!(cost.total_cost > Decimal::ZERO);
        assert_eq!(cost.total_cost, cost.input_cost + cost.output_cost);
    }

    #[test]
    fn test_streaming_cost_calculation() {
        let engine = create_test_pricing_engine();
        let metadata = StreamMetadata {
            model: "gpt-3.5-turbo".to_string(),
            provider: "openai".to_string(),
            token_count: Some(100),
            total_tokens: Some(500),
            processing_time_ms: Some(1000),
        };
        
        let result = engine.calculate_streaming_cost(&metadata);
        assert!(result.is_ok());
        
        let cost = result.unwrap();
        assert!(cost.total_cost > Decimal::ZERO);
    }

    #[test]
    fn test_cumulative_streaming_cost() {
        let engine = create_test_pricing_engine();
        let metadata = StreamMetadata {
            model: "claude-3-haiku".to_string(),
            provider: "anthropic".to_string(),
            token_count: Some(50),
            total_tokens: Some(1000),
            processing_time_ms: Some(500),
        };
        
        let result = engine.calculate_cumulative_streaming_cost(&metadata);
        assert!(result.is_ok());
        
        let cost = result.unwrap();
        assert!(cost.total_cost > Decimal::ZERO);
    }

    #[test]
    fn test_volume_discount_calculation() {
        let engine = create_test_pricing_engine();
        let token_usage = TokenUsage::new(10000, 5000);
        
        let standard_cost = engine.calculate_cost_with_volume_discount(
            &token_usage, 
            &Model::Gpt4, 
            &VolumeTier::Standard
        ).unwrap();
        
        let enterprise_cost = engine.calculate_cost_with_volume_discount(
            &token_usage, 
            &Model::Gpt4, 
            &VolumeTier::Enterprise
        ).unwrap();
        
        // Enterprise tier should be cheaper
        assert!(enterprise_cost.total_cost < standard_cost.total_cost);
    }

    #[test]
    fn test_cost_per_token() {
        let engine = create_test_pricing_engine();
        
        let input_cost = engine.get_cost_per_token(&Model::Gpt4, true);
        let output_cost = engine.get_cost_per_token(&Model::Gpt4, false);
        
        assert!(input_cost.is_ok());
        assert!(output_cost.is_ok());
        
        // Output tokens are typically more expensive than input tokens
        assert!(output_cost.unwrap() >= input_cost.unwrap());
    }

    #[test]
    fn test_model_comparison() {
        let engine = create_test_pricing_engine();
        let token_usage = TokenUsage::new(1000, 1000);
        
        let comparison = engine.compare_model_costs(
            &token_usage,
            &Model::Gpt4,
            &Model::Gpt35Turbo
        );
        
        assert!(comparison.is_ok());
        let comp = comparison.unwrap();
        
        // GPT-4 should be more expensive than GPT-3.5-turbo
        assert!(comp.cost_a.total_cost > comp.cost_b.total_cost);
        assert_eq!(comp.cheaper_model, Model::Gpt35Turbo);
    }

    #[test]
    fn test_invalid_model_string() {
        let engine = create_test_pricing_engine();
        let metadata = StreamMetadata {
            model: "invalid-model".to_string(),
            provider: "openai".to_string(),
            token_count: Some(100),
            total_tokens: Some(500),
            processing_time_ms: Some(1000),
        };
        
        let result = engine.calculate_streaming_cost(&metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_token_count() {
        let engine = create_test_pricing_engine();
        let metadata = StreamMetadata {
            model: "gpt-3.5-turbo".to_string(),
            provider: "openai".to_string(),
            token_count: None,
            total_tokens: Some(500),
            processing_time_ms: Some(1000),
        };
        
        let result = engine.calculate_streaming_cost(&metadata);
        assert!(result.is_err());
    }
}