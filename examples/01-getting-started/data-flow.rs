//! # Data Flow - Understanding How Data Moves Through Workflows
//!
//! This example demonstrates how data flows through a workflow, including:
//! - Type-safe data extraction and storage
//! - Working with different data types
//! - Data transformation between nodes
//! - Understanding the data lifecycle
//!
//! ## What You'll Learn
//! - Type-safe serialization/deserialization
//! - Complex data structures in workflows
//! - Data validation and transformation
//! - Accessing data from multiple nodes
//! - Error handling for data operations
//!
//! ## Usage
//! ```bash
//! cargo run --bin data-flow
//! ```

use workflow_engine_core::prelude::*;
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User profile input structure
#[derive(Debug, Deserialize, Serialize, Clone)]
struct UserProfile {
    id: u32,
    name: String,
    email: String,
    age: u32,
    interests: Vec<String>,
    metadata: HashMap<String, serde_json::Value>,
}

/// Enriched user data with additional computed fields
#[derive(Debug, Deserialize, Serialize)]
struct EnrichedUserData {
    profile: UserProfile,
    age_category: String,
    interests_count: usize,
    email_domain: String,
    risk_score: f64,
    recommendations: Vec<String>,
    enriched_at: String,
}

/// Analytics summary for reporting
#[derive(Debug, Deserialize, Serialize)]
struct UserAnalytics {
    user_id: u32,
    profile_completeness: f64,
    engagement_score: f64,
    category_scores: HashMap<String, f64>,
    insights: Vec<String>,
    computed_at: String,
}

/// A node that enriches user profile data with computed fields
#[derive(Debug)]
struct UserEnrichmentNode;

impl UserEnrichmentNode {
    fn categorize_age(&self, age: u32) -> String {
        match age {
            0..=17 => "Minor".to_string(),
            18..=25 => "Young Adult".to_string(),
            26..=40 => "Adult".to_string(),
            41..=65 => "Middle-aged".to_string(),
            _ => "Senior".to_string(),
        }
    }
    
    fn extract_email_domain(&self, email: &str) -> String {
        email.split('@')
            .nth(1)
            .unwrap_or("unknown")
            .to_string()
    }
    
    fn calculate_risk_score(&self, profile: &UserProfile) -> f64 {
        let mut score = 0.0;
        
        // Age factor
        if profile.age < 18 {
            score += 0.3;
        } else if profile.age > 65 {
            score += 0.2;
        }
        
        // Email domain factor
        let domain = self.extract_email_domain(&profile.email);
        if domain.ends_with(".temp") || domain.ends_with(".fake") {
            score += 0.5;
        }
        
        // Interest diversity factor
        if profile.interests.len() < 2 {
            score += 0.1;
        }
        
        score.min(1.0)
    }
    
    fn generate_recommendations(&self, profile: &UserProfile) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Interest-based recommendations
        if profile.interests.contains(&"technology".to_string()) {
            recommendations.push("Check out our latest tech articles".to_string());
        }
        if profile.interests.contains(&"sports".to_string()) {
            recommendations.push("Join our sports community".to_string());
        }
        if profile.interests.contains(&"music".to_string()) {
            recommendations.push("Discover new music recommendations".to_string());
        }
        
        // Age-based recommendations
        if profile.age >= 18 && profile.age <= 25 {
            recommendations.push("Explore student discounts".to_string());
        }
        
        // Default recommendation if none match
        if recommendations.is_empty() {
            recommendations.push("Complete your profile for personalized recommendations".to_string());
        }
        
        recommendations
    }
}

impl Node for UserEnrichmentNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🔄 UserEnrichmentNode: Enriching user profile...");
        
        // Extract the user profile from input data
        let profile: UserProfile = context.get_event_data()?;
        
        println!("   👤 Processing user: {} (ID: {})", profile.name, profile.id);
        
        // Enrich the profile with computed fields
        let enriched_data = EnrichedUserData {
            age_category: self.categorize_age(profile.age),
            interests_count: profile.interests.len(),
            email_domain: self.extract_email_domain(&profile.email),
            risk_score: self.calculate_risk_score(&profile),
            recommendations: self.generate_recommendations(&profile),
            enriched_at: chrono::Utc::now().to_rfc3339(),
            profile: profile.clone(),
        };
        
        println!("   📊 Age category: {}", enriched_data.age_category);
        println!("   📧 Email domain: {}", enriched_data.email_domain);
        println!("   ⚠️  Risk score: {:.2}", enriched_data.risk_score);
        println!("   💡 Recommendations: {}", enriched_data.recommendations.len());
        
        // Store the enriched data for other nodes to use
        context.update_node("enriched_user", enriched_data);
        
        // Also store some metadata for debugging
        context.set_metadata("original_user_id", profile.id)?;
        context.set_metadata("enrichment_version", "2.1")?;
        context.set_metadata("processing_node", "enrichment")?;
        
        println!("   ✅ User profile enriched successfully");
        
        Ok(context)
    }
}

/// A node that performs analytics on enriched user data
#[derive(Debug)]
struct UserAnalyticsNode;

impl UserAnalyticsNode {
    fn calculate_profile_completeness(&self, profile: &UserProfile) -> f64 {
        let mut score = 0.0;
        let total_fields = 6.0; // id, name, email, age, interests, metadata
        
        // Basic fields (always present due to struct requirements)
        score += 4.0; // id, name, email, age
        
        // Interests
        if !profile.interests.is_empty() {
            score += 1.0;
        }
        
        // Metadata
        if !profile.metadata.is_empty() {
            score += 1.0;
        }
        
        score / total_fields
    }
    
    fn calculate_engagement_score(&self, profile: &UserProfile) -> f64 {
        let mut score = 0.5; // Base score
        
        // Interest diversity
        let unique_interests = profile.interests.len();
        score += (unique_interests as f64 * 0.1).min(0.3);
        
        // Metadata richness
        score += (profile.metadata.len() as f64 * 0.05).min(0.2);
        
        score.min(1.0)
    }
    
    fn calculate_category_scores(&self, profile: &UserProfile, enriched: &EnrichedUserData) -> HashMap<String, f64> {
        let mut scores = HashMap::new();
        
        // Trust score based on risk
        scores.insert("trust".to_string(), 1.0 - enriched.risk_score);
        
        // Activity score based on interests
        let activity_score = (profile.interests.len() as f64 / 5.0).min(1.0);
        scores.insert("activity".to_string(), activity_score);
        
        // Age appropriateness score
        let age_score = match profile.age {
            18..=65 => 1.0,
            66..=80 => 0.8,
            _ => 0.6,
        };
        scores.insert("age_appropriate".to_string(), age_score);
        
        scores
    }
    
    fn generate_insights(&self, profile: &UserProfile, enriched: &EnrichedUserData, analytics: &UserAnalytics) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Profile completeness insights
        if analytics.profile_completeness < 0.7 {
            insights.push("Profile could be more complete for better personalization".to_string());
        }
        
        // Risk insights
        if enriched.risk_score > 0.5 {
            insights.push("High-risk profile detected - review required".to_string());
        }
        
        // Engagement insights
        if analytics.engagement_score > 0.8 {
            insights.push("Highly engaged user - great for targeted campaigns".to_string());
        } else if analytics.engagement_score < 0.4 {
            insights.push("Low engagement - consider re-engagement strategies".to_string());
        }
        
        // Interest insights
        if profile.interests.is_empty() {
            insights.push("No interests specified - prompt for interest selection".to_string());
        } else if profile.interests.len() > 5 {
            insights.push("Diverse interests - good candidate for varied content".to_string());
        }
        
        // Age category insights
        match enriched.age_category.as_str() {
            "Young Adult" => insights.push("Target with youth-oriented content and offers".to_string()),
            "Senior" => insights.push("Focus on accessibility and clear communication".to_string()),
            _ => {}
        }
        
        insights
    }
}

impl Node for UserAnalyticsNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📊 UserAnalyticsNode: Analyzing user data...");
        
        // Get the enriched user data from the previous node
        let enriched_user: EnrichedUserData = context
            .get_node_data("enriched_user")?
            .ok_or_else(|| WorkflowError::validation_error(
                "Missing enriched user data from previous node"
            ))?;
        
        let profile = &enriched_user.profile;
        
        println!("   🔍 Analyzing user: {} (ID: {})", profile.name, profile.id);
        
        // Calculate various analytics metrics
        let profile_completeness = self.calculate_profile_completeness(profile);
        let engagement_score = self.calculate_engagement_score(profile);
        let category_scores = self.calculate_category_scores(profile, &enriched_user);
        
        // Create initial analytics structure (without insights to avoid borrow issues)
        let mut analytics = UserAnalytics {
            user_id: profile.id,
            profile_completeness,
            engagement_score,
            category_scores: category_scores.clone(),
            insights: Vec::new(), // Will be filled below
            computed_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // Generate insights based on the analytics
        analytics.insights = self.generate_insights(profile, &enriched_user, &analytics);
        
        println!("   📈 Profile completeness: {:.1}%", profile_completeness * 100.0);
        println!("   🎯 Engagement score: {:.2}", engagement_score);
        println!("   💭 Insights generated: {}", analytics.insights.len());
        
        // Store analytics results
        context.update_node("user_analytics", &analytics);
        
        // Store summary metadata
        context.set_metadata("analytics_version", "3.0")?;
        context.set_metadata("completeness_score", profile_completeness)?;
        context.set_metadata("engagement_score", engagement_score)?;
        context.set_metadata("insights_count", analytics.insights.len())?;
        
        println!("   ✅ User analytics computed successfully");
        
        Ok(context)
    }
}

/// A node that generates a final report combining all data
#[derive(Debug)]
struct ReportGeneratorNode;

impl Node for ReportGeneratorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📄 ReportGeneratorNode: Generating comprehensive report...");
        
        // Collect data from all previous nodes
        let original_profile: UserProfile = context.get_event_data()?;
        let enriched_user: EnrichedUserData = context
            .get_node_data("enriched_user")?
            .ok_or_else(|| WorkflowError::validation_error("Missing enriched user data"))?;
        let analytics: UserAnalytics = context
            .get_node_data("user_analytics")?
            .ok_or_else(|| WorkflowError::validation_error("Missing analytics data"))?;
        
        // Generate comprehensive report
        let report = json!({
            "report_id": uuid::Uuid::new_v4(),
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "user_summary": {
                "id": original_profile.id,
                "name": original_profile.name,
                "email": original_profile.email,
                "age": original_profile.age,
                "age_category": enriched_user.age_category,
                "interests_count": enriched_user.interests_count,
                "email_domain": enriched_user.email_domain
            },
            "risk_assessment": {
                "risk_score": enriched_user.risk_score,
                "risk_level": if enriched_user.risk_score > 0.7 { "HIGH" } 
                            else if enriched_user.risk_score > 0.4 { "MEDIUM" } 
                            else { "LOW" },
                "recommendations_count": enriched_user.recommendations.len()
            },
            "analytics_summary": {
                "profile_completeness": analytics.profile_completeness,
                "engagement_score": analytics.engagement_score,
                "category_scores": analytics.category_scores,
                "insights_count": analytics.insights.len()
            },
            "recommendations": enriched_user.recommendations,
            "insights": analytics.insights,
            "data_sources": {
                "original_profile": "user_input",
                "enrichment": "computed_fields",
                "analytics": "behavioral_analysis"
            },
            "processing_metadata": {
                "total_processing_time_estimate": "250ms",
                "nodes_executed": ["enrichment", "analytics", "report_generation"],
                "data_quality": "high"
            }
        });
        
        // Store the final report
        context.update_node("final_report", report);
        
        // Add final metadata
        context.set_metadata("report_generated", true)?;
        context.set_metadata("report_quality", "comprehensive")?;
        context.set_metadata("workflow_complete", true)?;
        
        println!("   📊 Report includes:");
        println!("      • User summary and categorization");
        println!("      • Risk assessment and scoring");
        println!("      • Analytics and insights");
        println!("      • Personalized recommendations");
        println!("   ✅ Comprehensive report generated");
        
        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Data Flow Example - Understanding Data Movement");
    println!("=".repeat(60));
    println!("This example shows how data flows and transforms through nodes.\n");
    
    // Create processing nodes
    println!("📦 Creating data processing pipeline...");
    let enrichment_node = UserEnrichmentNode;
    let analytics_node = UserAnalyticsNode;
    let report_node = ReportGeneratorNode;
    println!("   ✅ Created 3-stage processing pipeline\n");
    
    // Create test users with different characteristics
    let test_users = vec![
        UserProfile {
            id: 1001,
            name: "Alice Johnson".to_string(),
            email: "alice@techcorp.com".to_string(),
            age: 28,
            interests: vec!["technology".to_string(), "programming".to_string(), "music".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("signup_source".to_string(), json!("organic"));
                map.insert("preferred_language".to_string(), json!("english"));
                map
            },
        },
        UserProfile {
            id: 1002,
            name: "Bob Smith".to_string(),
            email: "bob@example.temp".to_string(),
            age: 17,
            interests: vec!["gaming".to_string()],
            metadata: HashMap::new(),
        },
        UserProfile {
            id: 1003,
            name: "Carol Williams".to_string(),
            email: "carol@university.edu".to_string(),
            age: 45,
            interests: vec!["sports".to_string(), "cooking".to_string(), "travel".to_string(), "photography".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("subscription_tier".to_string(), json!("premium"));
                map.insert("last_login".to_string(), json!("2024-01-15"));
                map.insert("timezone".to_string(), json!("PST"));
                map
            },
        },
    ];
    
    // Process each user through the complete pipeline
    for (i, user) in test_users.into_iter().enumerate() {
        println!("🔄 Processing User {} of 3", i + 1);
        println!("   👤 User: {} (age {}, {} interests)", 
            user.name, user.age, user.interests.len());
        
        // Create workflow context
        let mut context = TaskContext::new(
            "user_data_flow_workflow".to_string(),
            serde_json::to_value(&user)?
        );
        
        // Execute the complete data processing pipeline
        println!("   🔄 Stage 1: User Enrichment");
        context = enrichment_node.process(context)?;
        
        println!("   🔄 Stage 2: Analytics Computation");
        context = analytics_node.process(context)?;
        
        println!("   🔄 Stage 3: Report Generation");
        context = report_node.process(context)?;
        
        // Display final results
        if let Some(report) = context.get_node_data::<serde_json::Value>("final_report")? {
            println!("   📋 FINAL REPORT SUMMARY:");
            
            if let Some(user_summary) = report.get("user_summary") {
                println!("      👤 User: {} ({})", 
                    user_summary["name"].as_str().unwrap_or("Unknown"),
                    user_summary["age_category"].as_str().unwrap_or("Unknown")
                );
            }
            
            if let Some(risk) = report.get("risk_assessment") {
                println!("      ⚠️  Risk Level: {} (Score: {:.2})", 
                    risk["risk_level"].as_str().unwrap_or("Unknown"),
                    risk["risk_score"].as_f64().unwrap_or(0.0)
                );
            }
            
            if let Some(analytics) = report.get("analytics_summary") {
                println!("      📊 Profile Completeness: {:.1}%", 
                    analytics["profile_completeness"].as_f64().unwrap_or(0.0) * 100.0
                );
                println!("      🎯 Engagement Score: {:.2}", 
                    analytics["engagement_score"].as_f64().unwrap_or(0.0)
                );
            }
            
            if let Some(recommendations) = report.get("recommendations").and_then(|r| r.as_array()) {
                println!("      💡 Recommendations: {}", recommendations.len());
                for (i, rec) in recommendations.iter().take(2).enumerate() {
                    if let Some(text) = rec.as_str() {
                        println!("         {}. {}", i + 1, text);
                    }
                }
            }
            
            if let Some(insights) = report.get("insights").and_then(|i| i.as_array()) {
                if !insights.is_empty() {
                    println!("      🔍 Key Insight: {}", 
                        insights[0].as_str().unwrap_or("No insights available")
                    );
                }
            }
        }
        
        // Show data flow through the pipeline
        println!("   📊 DATA FLOW SUMMARY:");
        println!("      Input: UserProfile -> TaskContext");
        println!("      Stage 1: UserProfile -> EnrichedUserData");
        println!("      Stage 2: EnrichedUserData -> UserAnalytics");
        println!("      Stage 3: All Data -> ComprehensiveReport");
        
        // Show metadata accumulated through the pipeline
        println!("   🔍 Accumulated Metadata:");
        for (key, value) in context.get_all_metadata() {
            println!("      {}: {}", key, value);
        }
        
        println!("   ✅ User processing completed\n");
    }
    
    println!("🎉 Data Flow Example Complete!");
    println!("=".repeat(60));
    println!("🎓 What you learned:");
    println!("   • Type-safe data serialization and deserialization");
    println!("   • Complex data structures in workflow contexts");
    println!("   • Data transformation through multiple stages");
    println!("   • Accessing data from previous nodes");
    println!("   • Building comprehensive data processing pipelines");
    println!("   • Using metadata to track processing state");
    println!();
    println!("📊 Data Flow Patterns Demonstrated:");
    println!("   • Input validation and parsing");
    println!("   • Data enrichment and computed fields");
    println!("   • Analytics and scoring algorithms");
    println!("   • Report generation and data aggregation");
    println!();
    println!("➡️  Next steps:");
    println!("   • Try adding your own computed fields");
    println!("   • Experiment with different data types");
    println!("   • Add error handling for missing data");
    println!("   • Move on to the simple-pipeline.rs example");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_user() -> UserProfile {
        UserProfile {
            id: 999,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: 30,
            interests: vec!["technology".to_string()],
            metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn test_user_enrichment_node() {
        let node = UserEnrichmentNode;
        let user = create_test_user();
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(user).unwrap()
        );
        
        let result = node.process(context).unwrap();
        let enriched: EnrichedUserData = result.get_node_data("enriched_user").unwrap().unwrap();
        
        assert_eq!(enriched.profile.id, 999);
        assert_eq!(enriched.age_category, "Adult");
        assert_eq!(enriched.email_domain, "example.com");
        assert!(enriched.risk_score >= 0.0 && enriched.risk_score <= 1.0);
    }
    
    #[test]
    fn test_user_analytics_node() {
        let enrichment_node = UserEnrichmentNode;
        let analytics_node = UserAnalyticsNode;
        let user = create_test_user();
        
        let mut context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(user).unwrap()
        );
        
        // First enrich the data
        context = enrichment_node.process(context).unwrap();
        
        // Then run analytics
        context = analytics_node.process(context).unwrap();
        
        let analytics: UserAnalytics = context.get_node_data("user_analytics").unwrap().unwrap();
        
        assert_eq!(analytics.user_id, 999);
        assert!(analytics.profile_completeness > 0.0);
        assert!(analytics.engagement_score > 0.0);
        assert!(!analytics.category_scores.is_empty());
    }
    
    #[test]
    fn test_complete_data_flow() {
        let enrichment_node = UserEnrichmentNode;
        let analytics_node = UserAnalyticsNode;
        let report_node = ReportGeneratorNode;
        let user = create_test_user();
        
        let mut context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(user).unwrap()
        );
        
        // Execute complete pipeline
        context = enrichment_node.process(context).unwrap();
        context = analytics_node.process(context).unwrap();
        context = report_node.process(context).unwrap();
        
        // Verify final report exists and has expected structure
        let report: serde_json::Value = context.get_node_data("final_report").unwrap().unwrap();
        
        assert!(report.get("report_id").is_some());
        assert!(report.get("user_summary").is_some());
        assert!(report.get("risk_assessment").is_some());
        assert!(report.get("analytics_summary").is_some());
        assert!(report.get("recommendations").is_some());
        
        // Verify workflow completion metadata
        assert_eq!(
            context.get_metadata::<bool>("workflow_complete").unwrap().unwrap(),
            true
        );
    }
}