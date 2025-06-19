//! # Async External API Workflow Example
//!
//! This example demonstrates an asynchronous workflow that integrates with external APIs.
//! It shows how to:
//!
//! - Create async nodes using the AsyncNode trait
//! - Make HTTP requests to external APIs
//! - Handle network errors and retries
//! - Process and transform API responses
//! - Use parallel processing for multiple API calls
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example 02_async_external_api_workflow
//! ```

use workflow_engine_core::prelude::*;
use serde_json::json;
use tokio::time::{sleep, Duration};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Mock weather API response structure
#[derive(Debug, Deserialize, Serialize)]
struct WeatherData {
    temperature: f64,
    humidity: f64,
    condition: String,
    location: String,
}

/// Mock news API response structure
#[derive(Debug, Deserialize, Serialize)]
struct NewsData {
    headlines: Vec<String>,
    total_articles: u32,
    category: String,
}

/// Async node that fetches weather data from a mock external API
#[derive(Debug)]
struct WeatherApiNode {
    api_url: String,
}

impl WeatherApiNode {
    fn new(api_url: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
        }
    }
    
    /// Mock external weather API call
    async fn fetch_weather(&self, location: &str) -> Result<WeatherData, WorkflowError> {
        println!("🌤️  Calling weather API for location: {}", location);
        
        // Simulate network delay
        sleep(Duration::from_millis(500)).await;
        
        // Mock response based on location
        let weather = match location.to_lowercase().as_str() {
            "london" => WeatherData {
                temperature: 15.5,
                humidity: 78.0,
                condition: "Cloudy".to_string(),
                location: location.to_string(),
            },
            "tokyo" => WeatherData {
                temperature: 22.3,
                humidity: 65.0,
                condition: "Sunny".to_string(),
                location: location.to_string(),
            },
            "new york" => WeatherData {
                temperature: 18.1,
                humidity: 72.0,
                condition: "Partly Cloudy".to_string(),
                location: location.to_string(),
            },
            _ => WeatherData {
                temperature: 20.0,
                humidity: 50.0,
                condition: "Unknown".to_string(),
                location: location.to_string(),
            }
        };
        
        println!("✅ Weather data received for {}: {}°C, {}", 
                location, weather.temperature, weather.condition);
        
        Ok(weather)
    }
}

#[async_trait]
impl AsyncNode for WeatherApiNode {
    async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract input data
        let input: serde_json::Value = context.get_event_data()?;
        
        let location = input
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("London");
        
        // Fetch weather data with error handling
        let weather_result = match self.fetch_weather(location).await {
            Ok(weather) => {
                context.set_metadata("weather_api_status", "success")?;
                weather
            }
            Err(e) => {
                context.set_metadata("weather_api_status", "failed")?;
                context.set_metadata("weather_api_error", e.to_string())?;
                
                // Fallback data
                WeatherData {
                    temperature: 0.0,
                    humidity: 0.0,
                    condition: "API Error".to_string(),
                    location: location.to_string(),
                }
            }
        };
        
        // Store the weather data
        context.update_node("weather", weather_result);
        
        Ok(context)
    }
}

/// Async node that fetches news data from a mock external API
#[derive(Debug)]
struct NewsApiNode {
    api_key: String,
}

impl NewsApiNode {
    fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
    
    /// Mock external news API call
    async fn fetch_news(&self, category: &str) -> Result<NewsData, WorkflowError> {
        println!("📰 Calling news API for category: {}", category);
        
        // Simulate network delay
        sleep(Duration::from_millis(300)).await;
        
        // Mock response based on category
        let news = match category.to_lowercase().as_str() {
            "technology" => NewsData {
                headlines: vec![
                    "AI Breakthrough in Workflow Automation".to_string(),
                    "New Rust Framework Revolutionizes Backend Development".to_string(),
                    "Cloud Computing Trends for 2024".to_string(),
                ],
                total_articles: 150,
                category: category.to_string(),
            },
            "business" => NewsData {
                headlines: vec![
                    "Stock Market Reaches New Heights".to_string(),
                    "Startup Funding Increases by 25%".to_string(),
                    "Remote Work Policies Shape Future".to_string(),
                ],
                total_articles: 89,
                category: category.to_string(),
            },
            _ => NewsData {
                headlines: vec!["General News Update".to_string()],
                total_articles: 1,
                category: category.to_string(),
            }
        };
        
        println!("✅ News data received for {}: {} articles", 
                category, news.total_articles);
        
        Ok(news)
    }
}

#[async_trait]
impl AsyncNode for NewsApiNode {
    async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Extract input data
        let input: serde_json::Value = context.get_event_data()?;
        
        let category = input
            .get("news_category")
            .and_then(|v| v.as_str())
            .unwrap_or("technology");
        
        // Fetch news data with retry logic
        let mut retries = 3;
        let news_result = loop {
            match self.fetch_news(category).await {
                Ok(news) => break Ok(news),
                Err(e) if retries > 0 => {
                    retries -= 1;
                    println!("⚠️  API call failed, retrying... ({} attempts left)", retries);
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
                Err(e) => break Err(e),
            }
        };
        
        let news = match news_result {
            Ok(news) => {
                context.set_metadata("news_api_status", "success")?;
                news
            }
            Err(e) => {
                context.set_metadata("news_api_status", "failed")?;
                context.set_metadata("news_api_error", e.to_string())?;
                
                // Fallback data
                NewsData {
                    headlines: vec!["News service temporarily unavailable".to_string()],
                    total_articles: 0,
                    category: category.to_string(),
                }
            }
        };
        
        // Store the news data
        context.update_node("news", news);
        
        Ok(context)
    }
}

/// Aggregator node that combines data from multiple sources
#[derive(Debug)]
struct DataAggregatorNode;

#[async_trait]
impl AsyncNode for DataAggregatorNode {
    async fn process_async(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔄 Aggregating data from multiple sources...");
        
        // Extract weather data
        let weather: Option<WeatherData> = context.get_node_data("weather")?;
        
        // Extract news data
        let news: Option<NewsData> = context.get_node_data("news")?;
        
        // Create summary
        let summary = json!({
            "timestamp": chrono::Utc::now(),
            "weather_available": weather.is_some(),
            "news_available": news.is_some(),
            "weather_data": weather,
            "news_data": news,
            "aggregation_status": "completed"
        });
        
        // Generate insights
        let mut insights = Vec::new();
        
        if let Some(ref w) = weather {
            if w.temperature > 25.0 {
                insights.push("It's a hot day! Consider staying hydrated.".to_string());
            } else if w.temperature < 10.0 {
                insights.push("Cold weather today. Dress warmly!".to_string());
            }
            
            if w.humidity > 80.0 {
                insights.push("High humidity levels detected.".to_string());
            }
        }
        
        if let Some(ref n) = news {
            if n.category == "technology" && n.total_articles > 100 {
                insights.push("High technology news activity today.".to_string());
            }
        }
        
        let aggregated_data = json!({
            "summary": summary,
            "insights": insights,
            "data_sources": ["weather_api", "news_api"],
            "processing_time_ms": 800 // Mock processing time
        });
        
        context.update_node("aggregated_data", aggregated_data);
        context.set_metadata("aggregation_complete", true)?;
        
        println!("✅ Data aggregation completed successfully");
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Starting Async External API Workflow Example");
    println!("================================================");
    
    // Build an async workflow with external API integration
    let workflow = TypedWorkflowBuilder::new("external_api_workflow")
        .description("Demonstrates async API calls and data aggregation")
        .start_with_node(NodeId::new("weather"))
        .parallel_nodes(vec![
            NodeId::new("weather"),
            NodeId::new("news"),
        ])
        .then_node(NodeId::new("aggregator"))
        .build()?;
    
    // Register async nodes
    workflow.register_async_node(
        NodeId::new("weather"), 
        WeatherApiNode::new("https://api.weather.example.com")
    );
    
    workflow.register_async_node(
        NodeId::new("news"), 
        NewsApiNode::new("mock-api-key-12345")
    );
    
    workflow.register_async_node(
        NodeId::new("aggregator"), 
        DataAggregatorNode
    );
    
    println!("📋 Async workflow built with 3 nodes:");
    println!("   1. WeatherApiNode - Fetches weather data");
    println!("   2. NewsApiNode - Fetches news data (parallel)");
    println!("   3. DataAggregatorNode - Combines and analyzes data");
    println!();
    
    // Test different scenarios
    let test_cases = vec![
        json!({
            "location": "Tokyo",
            "news_category": "technology"
        }),
        json!({
            "location": "London",
            "news_category": "business"
        }),
        json!({
            "location": "New York",
            "news_category": "general"
        }),
    ];
    
    for (i, input_data) in test_cases.into_iter().enumerate() {
        println!("🔄 Test Case {} - Input: {}", i + 1, input_data);
        
        let start_time = std::time::Instant::now();
        
        // Run the async workflow
        let result = workflow.run_async(input_data).await?;
        
        let execution_time = start_time.elapsed();
        
        // Extract final results
        if let Some(aggregated) = result.get_node_data::<serde_json::Value>("aggregated_data")? {
            println!("   📊 Aggregated Results:");
            if let Some(insights) = aggregated["insights"].as_array() {
                for insight in insights {
                    println!("      💡 {}", insight.as_str().unwrap_or(""));
                }
            }
            
            if let Some(summary) = aggregated.get("summary") {
                println!("   📈 Summary: Weather Available: {}, News Available: {}",
                    summary["weather_available"],
                    summary["news_available"]
                );
            }
        }
        
        println!("   ⏱️  Execution time: {:?}", execution_time);
        println!("   ✅ Test case completed");
        println!();
    }
    
    println!("🎉 Async External API Workflow Example completed!");
    println!("==================================================");
    println!();
    println!("Key features demonstrated:");
    println!("• AsyncNode trait for non-blocking operations");
    println!("• Parallel execution of independent API calls");
    println!("• Error handling and retry logic");
    println!("• Data aggregation from multiple sources");
    println!("• Real-world async patterns with external services");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_weather_api_node() {
        let node = WeatherApiNode::new("test-url");
        let context = TaskContext::new(
            "test".to_string(),
            json!({"location": "Tokyo"})
        );
        
        let result = node.process_async(context).await.unwrap();
        let weather: WeatherData = result.get_node_data("weather").unwrap().unwrap();
        
        assert_eq!(weather.location, "Tokyo");
        assert_eq!(weather.condition, "Sunny");
        assert!(weather.temperature > 0.0);
    }
    
    #[tokio::test]
    async fn test_news_api_node() {
        let node = NewsApiNode::new("test-key");
        let context = TaskContext::new(
            "test".to_string(),
            json!({"news_category": "technology"})
        );
        
        let result = node.process_async(context).await.unwrap();
        let news: NewsData = result.get_node_data("news").unwrap().unwrap();
        
        assert_eq!(news.category, "technology");
        assert!(news.total_articles > 0);
        assert!(!news.headlines.is_empty());
    }
    
    #[tokio::test]
    async fn test_data_aggregator_node() {
        let node = DataAggregatorNode;
        let mut context = TaskContext::new(
            "test".to_string(),
            json!({})
        );
        
        // Add mock data from previous nodes
        context.update_node("weather", WeatherData {
            temperature: 25.5,
            humidity: 60.0,
            condition: "Sunny".to_string(),
            location: "Test City".to_string(),
        });
        
        context.update_node("news", NewsData {
            headlines: vec!["Test headline".to_string()],
            total_articles: 1,
            category: "test".to_string(),
        });
        
        let result = node.process_async(context).await.unwrap();
        let aggregated: serde_json::Value = result.get_node_data("aggregated_data").unwrap().unwrap();
        
        assert!(aggregated["summary"]["weather_available"].as_bool().unwrap());
        assert!(aggregated["summary"]["news_available"].as_bool().unwrap());
        assert!(aggregated["insights"].is_array());
    }
}