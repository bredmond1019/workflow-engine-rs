//! Usage analytics and tracking for AI token consumption

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration, Datelike, TimeZone, Timelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use uuid::Uuid;
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};
use crate::ai::tokens::{Model, Provider, UsageRecord, TokenError, TokenResult};

/// Configuration for usage analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enable_metrics: bool,
    pub retention_days: u32,
    pub aggregation_intervals: Vec<AggregationInterval>,
    pub export_config: ExportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AggregationInterval {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub enabled: bool,
    pub format: ExportFormat,
    pub destination: String, // file path or database connection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Database,
}

/// Aggregated usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub interval: AggregationInterval,
    pub total_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost: Decimal,
    pub provider_breakdown: HashMap<Provider, ProviderStats>,
    pub model_breakdown: HashMap<Model, ModelStats>,
    pub user_breakdown: HashMap<String, UserStats>,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: Decimal,
    pub avg_tokens_per_request: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: Decimal,
    pub avg_response_time_ms: f64,
    pub tokens_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub requests: u64,
    pub total_tokens: u64,
    pub total_cost: Decimal,
    pub favorite_model: Option<Model>,
    pub usage_trend: UsageTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Real-time usage analytics engine
pub struct UsageAnalytics {
    config: AnalyticsConfig,
    usage_records: Arc<RwLock<Vec<UsageRecord>>>,
    current_stats: Arc<RwLock<HashMap<AggregationInterval, UsageStats>>>,
    
    // Prometheus metrics
    request_counter: Counter,
    token_histogram: Histogram,
    cost_gauge: Gauge,
    error_counter: Counter,
}

impl UsageAnalytics {
    /// Create a new usage analytics engine
    pub fn new(config: AnalyticsConfig) -> TokenResult<Self> {
        let request_counter = register_counter!(
            "ai_requests_total",
            "Total number of AI requests"
        ).map_err(|e| TokenError::AnalyticsError(format!("Failed to register request counter: {}", e)))?;

        let token_histogram = register_histogram!(
            "ai_tokens_per_request",
            "Distribution of tokens per request"
        ).map_err(|e| TokenError::AnalyticsError(format!("Failed to register token histogram: {}", e)))?;

        let cost_gauge = register_gauge!(
            "ai_total_cost",
            "Total cost of AI requests"
        ).map_err(|e| TokenError::AnalyticsError(format!("Failed to register cost gauge: {}", e)))?;

        let error_counter = register_counter!(
            "ai_errors_total",
            "Total number of AI request errors"
        ).map_err(|e| TokenError::AnalyticsError(format!("Failed to register error counter: {}", e)))?;

        Ok(Self {
            config,
            usage_records: Arc::new(RwLock::new(Vec::new())),
            current_stats: Arc::new(RwLock::new(HashMap::new())),
            request_counter,
            token_histogram,
            cost_gauge,
            error_counter,
        })
    }

    /// Record a usage event
    pub async fn record_usage(&self, record: UsageRecord) -> TokenResult<()> {
        // Update Prometheus metrics
        if self.config.enable_metrics {
            self.request_counter.inc();
            self.token_histogram.observe(record.token_usage.total_tokens as f64);
            self.cost_gauge.set(record.cost_breakdown.total_cost.to_string().parse().unwrap_or(0.0));
        }

        // Store the record
        let mut records = self.usage_records.write().await;
        records.push(record);

        // Clean up old records based on retention policy
        self.cleanup_old_records(&mut records).await;

        // Update aggregated statistics
        self.update_aggregated_stats().await?;

        Ok(())
    }

    /// Record an error event
    pub async fn record_error(&self, provider: Provider, model: Model, error: &str) -> TokenResult<()> {
        if self.config.enable_metrics {
            self.error_counter.inc();
        }

        log::warn!("AI request error for {}:{} - {}", provider.to_string(), model.as_str(), error);
        Ok(())
    }

    /// Get usage statistics for a specific interval
    pub async fn get_stats(&self, interval: AggregationInterval) -> TokenResult<Option<UsageStats>> {
        let stats = self.current_stats.read().await;
        Ok(stats.get(&interval).cloned())
    }

    /// Get usage statistics for a custom time range
    pub async fn get_stats_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> TokenResult<UsageStats> {
        let records = self.usage_records.read().await;
        let filtered_records: Vec<&UsageRecord> = records
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect();

        self.calculate_stats(&filtered_records, start, end, AggregationInterval::Daily)
    }

    /// Get top models by usage
    pub async fn get_top_models(&self, limit: usize) -> TokenResult<Vec<(Model, ModelStats)>> {
        let records = self.usage_records.read().await;
        let mut model_usage: HashMap<Model, ModelStats> = HashMap::new();

        for record in records.iter() {
            let stats = model_usage.entry(record.model.clone()).or_insert(ModelStats {
                requests: 0,
                input_tokens: 0,
                output_tokens: 0,
                cost: Decimal::ZERO,
                avg_response_time_ms: 0.0,
                tokens_per_second: 0.0,
            });

            stats.requests += 1;
            stats.input_tokens += record.token_usage.input_tokens as u64;
            stats.output_tokens += record.token_usage.output_tokens as u64;
            stats.cost += record.cost_breakdown.total_cost;
        }

        let mut sorted_models: Vec<(Model, ModelStats)> = model_usage.into_iter().collect();
        sorted_models.sort_by(|a, b| b.1.requests.cmp(&a.1.requests));
        sorted_models.truncate(limit);

        Ok(sorted_models)
    }

    /// Get usage trends over time
    pub async fn get_usage_trends(&self, days: u32) -> TokenResult<Vec<DailyUsageTrend>> {
        let end_date = Utc::now();
        let start_date = end_date - Duration::days(days as i64);
        
        let records = self.usage_records.read().await;
        let mut daily_usage: HashMap<chrono::NaiveDate, DailyUsageTrend> = HashMap::new();

        for record in records.iter() {
            if record.timestamp >= start_date && record.timestamp <= end_date {
                let date = record.timestamp.date_naive();
                let trend = daily_usage.entry(date).or_insert(DailyUsageTrend {
                    date,
                    requests: 0,
                    total_tokens: 0,
                    total_cost: Decimal::ZERO,
                });

                trend.requests += 1;
                trend.total_tokens += record.token_usage.total_tokens as u64;
                trend.total_cost += record.cost_breakdown.total_cost;
            }
        }

        let mut trends: Vec<DailyUsageTrend> = daily_usage.into_values().collect();
        trends.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(trends)
    }

    /// Get cost breakdown by provider
    pub async fn get_cost_breakdown(&self) -> TokenResult<HashMap<Provider, Decimal>> {
        let records = self.usage_records.read().await;
        let mut provider_costs: HashMap<Provider, Decimal> = HashMap::new();

        for record in records.iter() {
            *provider_costs.entry(record.provider.clone()).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
        }

        Ok(provider_costs)
    }

    /// Export analytics data
    pub async fn export_data(&self, format: ExportFormat, destination: &str) -> TokenResult<()> {
        let records = self.usage_records.read().await;
        
        match format {
            ExportFormat::Json => {
                let json_data = serde_json::to_string_pretty(&*records)
                    .map_err(|e| TokenError::AnalyticsError(format!("JSON serialization failed: {}", e)))?;
                
                tokio::fs::write(destination, json_data).await
                    .map_err(|e| TokenError::AnalyticsError(format!("Failed to write JSON file: {}", e)))?;
            }
            ExportFormat::Csv => {
                self.export_to_csv(&records, destination).await?;
            }
            ExportFormat::Database => {
                // Database export would be implemented here
                return Err(TokenError::AnalyticsError("Database export not implemented".to_string()));
            }
        }

        Ok(())
    }

    /// Calculate forecasted usage based on historical trends
    pub async fn forecast_usage(&self, days_ahead: u32) -> TokenResult<UsageForecast> {
        let trends = self.get_usage_trends(30).await?; // Use last 30 days for forecasting
        
        if trends.len() < 7 {
            return Err(TokenError::AnalyticsError("Insufficient data for forecasting".to_string()));
        }

        // Simple linear regression for forecasting
        let avg_daily_requests: f64 = trends.iter().map(|t| t.requests as f64).sum::<f64>() / trends.len() as f64;
        let avg_daily_tokens: f64 = trends.iter().map(|t| t.total_tokens as f64).sum::<f64>() / trends.len() as f64;
        let avg_daily_cost: f64 = trends.iter().map(|t| t.total_cost.to_string().parse::<f64>().unwrap_or(0.0)).sum::<f64>() / trends.len() as f64;

        Ok(UsageForecast {
            forecast_date: Utc::now() + Duration::days(days_ahead as i64),
            predicted_requests: (avg_daily_requests * days_ahead as f64) as u64,
            predicted_tokens: (avg_daily_tokens * days_ahead as f64) as u64,
            predicted_cost: Decimal::from_f64(avg_daily_cost * days_ahead as f64).unwrap_or(Decimal::ZERO),
            confidence_level: self.calculate_confidence(&trends),
        })
    }

    /// Generate cost optimization recommendations based on usage patterns
    pub async fn get_cost_optimization_recommendations(&self) -> TokenResult<Vec<CostOptimizationRecommendation>> {
        let records = self.usage_records.read().await;
        let mut recommendations = Vec::new();

        // Analyze model usage patterns
        let model_stats = self.analyze_model_efficiency(&records).await?;
        
        // Find overused expensive models
        for (model, stats) in &model_stats {
            if stats.cost > Decimal::from(100) && stats.requests > 50 {
                let cheaper_alternative = self.find_cheaper_alternative(model).await?;
                if let Some((alt_model, potential_savings)) = cheaper_alternative {
                    recommendations.push(CostOptimizationRecommendation {
                        recommendation_type: OptimizationType::ModelSubstitution,
                        title: format!("Consider switching from {} to {}", model.as_str(), alt_model.as_str()),
                        description: format!("You could save approximately ${:.2} by using {} instead of {}", 
                                           potential_savings, alt_model.as_str(), model.as_str()),
                        potential_monthly_savings: potential_savings,
                        confidence_score: 0.8,
                        implementation_effort: ImplementationEffort::Low,
                        risk_level: RiskLevel::Low,
                    });
                }
            }
        }

        // Check for inefficient usage patterns
        let inefficiencies = self.detect_usage_inefficiencies(&records).await?;
        for inefficiency in inefficiencies {
            recommendations.push(inefficiency);
        }

        // Check for potential batch processing opportunities
        let batch_opportunities = self.identify_batch_opportunities(&records).await?;
        recommendations.extend(batch_opportunities);

        Ok(recommendations)
    }

    /// Get detailed cost breakdown analysis
    pub async fn get_detailed_cost_analysis(&self, period_days: u32) -> TokenResult<DetailedCostAnalysis> {
        let start_date = Utc::now() - Duration::days(period_days as i64);
        let records = self.usage_records.read().await;
        
        let period_records: Vec<&UsageRecord> = records
            .iter()
            .filter(|r| r.timestamp >= start_date)
            .collect();

        let total_cost = period_records.iter()
            .map(|r| r.cost_breakdown.total_cost)
            .sum();

        let provider_costs: HashMap<Provider, Decimal> = period_records
            .iter()
            .fold(HashMap::new(), |mut acc, record| {
                *acc.entry(record.provider.clone()).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
                acc
            });

        let model_costs: HashMap<Model, Decimal> = period_records
            .iter()
            .fold(HashMap::new(), |mut acc, record| {
                *acc.entry(record.model.clone()).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
                acc
            });

        let hourly_costs = self.calculate_hourly_cost_distribution(&period_records)?;
        let cost_trends = self.calculate_cost_trends(&period_records, period_days)?;

        Ok(DetailedCostAnalysis {
            period_start: start_date,
            period_end: Utc::now(),
            total_cost,
            average_daily_cost: total_cost / Decimal::from(period_days),
            provider_breakdown: provider_costs,
            model_breakdown: model_costs,
            hourly_distribution: hourly_costs,
            cost_trends,
            top_cost_drivers: self.identify_top_cost_drivers(&period_records)?,
        })
    }

    /// Generate monthly cost report with insights
    pub async fn generate_monthly_report(&self, month: u32, year: u32) -> TokenResult<MonthlyCostReport> {
        let start_date = chrono::Utc.with_ymd_and_hms(year as i32, month, 1, 0, 0, 0).unwrap();
        let end_date = if month == 12 {
            chrono::Utc.with_ymd_and_hms(year as i32 + 1, 1, 1, 0, 0, 0).unwrap()
        } else {
            chrono::Utc.with_ymd_and_hms(year as i32, month + 1, 1, 0, 0, 0).unwrap()
        };

        let records = self.usage_records.read().await;
        let month_records: Vec<&UsageRecord> = records
            .iter()
            .filter(|r| r.timestamp >= start_date && r.timestamp < end_date)
            .collect();

        let total_cost = month_records.iter()
            .map(|r| r.cost_breakdown.total_cost)
            .sum();

        let total_requests = month_records.len() as u64;
        let total_tokens = month_records.iter()
            .map(|r| r.token_usage.total_tokens as u64)
            .sum();

        // Compare with previous month
        let prev_month_start = if month == 1 {
            chrono::Utc.with_ymd_and_hms(year as i32 - 1, 12, 1, 0, 0, 0).unwrap()
        } else {
            chrono::Utc.with_ymd_and_hms(year as i32, month - 1, 1, 0, 0, 0).unwrap()
        };

        let prev_month_records: Vec<&UsageRecord> = records
            .iter()
            .filter(|r| r.timestamp >= prev_month_start && r.timestamp < start_date)
            .collect();

        let prev_month_cost: Decimal = prev_month_records.iter()
            .map(|r| r.cost_breakdown.total_cost)
            .sum();

        let cost_change_percentage = if prev_month_cost > Decimal::ZERO {
            let change: Decimal = (total_cost - prev_month_cost) / prev_month_cost * Decimal::from(100);
            change.to_f64().unwrap_or(0.0)
        } else {
            0.0
        };

        let insights = self.generate_monthly_insights(&month_records, &prev_month_records)?;

        Ok(MonthlyCostReport {
            month,
            year,
            period_start: start_date,
            period_end: end_date,
            total_cost,
            total_requests,
            total_tokens,
            cost_per_request: if total_requests > 0 { total_cost / Decimal::from(total_requests) } else { Decimal::ZERO },
            cost_per_token: if total_tokens > 0 { total_cost / Decimal::from(total_tokens) } else { Decimal::ZERO },
            cost_change_from_previous_month: cost_change_percentage,
            top_models_by_cost: self.get_top_models_by_cost(&month_records, 5)?,
            usage_by_provider: self.get_usage_by_provider(&month_records)?,
            daily_cost_breakdown: self.get_daily_cost_breakdown(&month_records)?,
            insights,
        })
    }

    // Helper methods for new functionality

    async fn analyze_model_efficiency(&self, records: &[UsageRecord]) -> TokenResult<HashMap<Model, ModelStats>> {
        let mut model_stats: HashMap<Model, ModelStats> = HashMap::new();

        for record in records {
            let stats = model_stats.entry(record.model.clone()).or_insert(ModelStats {
                requests: 0,
                input_tokens: 0,
                output_tokens: 0,
                cost: Decimal::ZERO,
                avg_response_time_ms: 0.0,
                tokens_per_second: 0.0,
            });

            stats.requests += 1;
            stats.input_tokens += record.token_usage.input_tokens as u64;
            stats.output_tokens += record.token_usage.output_tokens as u64;
            stats.cost += record.cost_breakdown.total_cost;
        }

        Ok(model_stats)
    }

    async fn find_cheaper_alternative(&self, model: &Model) -> TokenResult<Option<(Model, Decimal)>> {
        // Define model alternatives and their relative costs
        let alternatives = match model {
            Model::Gpt4 => vec![Model::Gpt4Turbo, Model::Claude3Sonnet],
            Model::Claude3Opus => vec![Model::Claude3Sonnet, Model::Claude3Haiku],
            Model::Gpt4Turbo => vec![Model::Gpt35Turbo, Model::Claude3Haiku],
            _ => vec![],
        };

        // For now, return a simplified calculation
        // In a real implementation, this would calculate based on actual usage patterns
        if !alternatives.is_empty() {
            Ok(Some((alternatives[0].clone(), Decimal::from(50))))
        } else {
            Ok(None)
        }
    }

    async fn detect_usage_inefficiencies(&self, records: &[UsageRecord]) -> TokenResult<Vec<CostOptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Check for frequent small requests that could be batched
        let small_request_count = records.iter()
            .filter(|r| r.token_usage.total_tokens < 100)
            .count();

        if small_request_count > records.len() / 2 {
            recommendations.push(CostOptimizationRecommendation {
                recommendation_type: OptimizationType::RequestBatching,
                title: "Consider batching small requests".to_string(),
                description: format!("{}% of your requests use fewer than 100 tokens. Batching these could reduce overhead costs.", 
                                   (small_request_count * 100) / records.len()),
                potential_monthly_savings: Decimal::from(25),
                confidence_score: 0.7,
                implementation_effort: ImplementationEffort::Medium,
                risk_level: RiskLevel::Low,
            });
        }

        Ok(recommendations)
    }

    async fn identify_batch_opportunities(&self, records: &[UsageRecord]) -> TokenResult<Vec<CostOptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Group requests by user and time proximity
        let mut user_request_patterns: HashMap<String, Vec<&UsageRecord>> = HashMap::new();
        
        for record in records {
            if let Some(user_id) = &record.user_id {
                user_request_patterns.entry(user_id.clone()).or_default().push(record);
            }
        }

        for (user_id, user_records) in user_request_patterns {
            if user_records.len() > 10 {
                // Check for requests within short time windows
                let mut close_requests = 0;
                for window in user_records.windows(2) {
                    if let [r1, r2] = window {
                        let time_diff = r2.timestamp - r1.timestamp;
                        if time_diff < Duration::minutes(5) {
                            close_requests += 1;
                        }
                    }
                }

                if close_requests > user_records.len() / 3 {
                    recommendations.push(CostOptimizationRecommendation {
                        recommendation_type: OptimizationType::RequestBatching,
                        title: format!("Batch optimization opportunity for user {}", user_id),
                        description: "This user makes many requests in short time windows. Consider batching these requests.".to_string(),
                        potential_monthly_savings: Decimal::from(15),
                        confidence_score: 0.6,
                        implementation_effort: ImplementationEffort::Medium,
                        risk_level: RiskLevel::Low,
                    });
                }
            }
        }

        Ok(recommendations)
    }

    fn calculate_hourly_cost_distribution(&self, records: &[&UsageRecord]) -> TokenResult<HashMap<u32, Decimal>> {
        let mut hourly_costs: HashMap<u32, Decimal> = HashMap::new();

        for record in records {
            let hour = record.timestamp.hour();
            *hourly_costs.entry(hour).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
        }

        Ok(hourly_costs)
    }

    fn calculate_cost_trends(&self, records: &[&UsageRecord], period_days: u32) -> TokenResult<CostTrendData> {
        let mut daily_costs: HashMap<chrono::NaiveDate, Decimal> = HashMap::new();

        for record in records {
            let date = record.timestamp.date_naive();
            *daily_costs.entry(date).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
        }

        let costs: Vec<f64> = daily_costs.values()
            .map(|d| d.to_string().parse().unwrap_or(0.0))
            .collect();

        let trend = if costs.len() >= 2 {
            let first_half = &costs[..costs.len()/2];
            let second_half = &costs[costs.len()/2..];
            
            let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
            let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;
            
            if second_avg > first_avg * 1.1 {
                TrendDirection::Increasing
            } else if second_avg < first_avg * 0.9 {
                TrendDirection::Decreasing
            } else {
                TrendDirection::Stable
            }
        } else {
            TrendDirection::Stable
        };

        Ok(CostTrendData {
            trend_direction: trend,
            daily_costs,
            average_daily_cost: costs.iter().sum::<f64>() / costs.len() as f64,
        })
    }

    fn identify_top_cost_drivers(&self, records: &[&UsageRecord]) -> TokenResult<Vec<CostDriver>> {
        let mut model_costs: HashMap<Model, Decimal> = HashMap::new();
        let mut provider_costs: HashMap<Provider, Decimal> = HashMap::new();

        for record in records {
            *model_costs.entry(record.model.clone()).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
            *provider_costs.entry(record.provider.clone()).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
        }

        let mut cost_drivers = Vec::new();

        // Top model by cost
        if let Some((model, cost)) = model_costs.iter().max_by_key(|(_, cost)| *cost) {
            cost_drivers.push(CostDriver {
                driver_type: CostDriverType::Model(model.clone()),
                cost: *cost,
                percentage_of_total: 0.0, // Would calculate from total
            });
        }

        // Top provider by cost
        if let Some((provider, cost)) = provider_costs.iter().max_by_key(|(_, cost)| *cost) {
            cost_drivers.push(CostDriver {
                driver_type: CostDriverType::Provider(provider.clone()),
                cost: *cost,
                percentage_of_total: 0.0,
            });
        }

        Ok(cost_drivers)
    }

    fn generate_monthly_insights(&self, current_records: &[&UsageRecord], prev_records: &[&UsageRecord]) -> TokenResult<Vec<String>> {
        let mut insights = Vec::new();

        let current_cost: Decimal = current_records.iter().map(|r| r.cost_breakdown.total_cost).sum();
        let prev_cost: Decimal = prev_records.iter().map(|r| r.cost_breakdown.total_cost).sum();

        if current_cost > prev_cost * Decimal::from_f64(1.2).unwrap() {
            insights.push("Costs increased by more than 20% compared to last month. Consider reviewing usage patterns.".to_string());
        }

        if current_records.len() > prev_records.len() * 2 {
            insights.push("Request volume doubled compared to last month. This may indicate growing adoption.".to_string());
        }

        // Check for model usage changes
        let current_models: std::collections::HashSet<_> = current_records.iter().map(|r| &r.model).collect();
        let prev_models: std::collections::HashSet<_> = prev_records.iter().map(|r| &r.model).collect();
        
        for model in current_models.difference(&prev_models) {
            insights.push(format!("New model adoption detected: {} started being used this month.", model.as_str()));
        }

        if insights.is_empty() {
            insights.push("Usage patterns are stable compared to the previous month.".to_string());
        }

        Ok(insights)
    }

    fn get_top_models_by_cost(&self, records: &[&UsageRecord], limit: usize) -> TokenResult<Vec<(Model, Decimal)>> {
        let mut model_costs: HashMap<Model, Decimal> = HashMap::new();

        for record in records {
            *model_costs.entry(record.model.clone()).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
        }

        let mut sorted_models: Vec<(Model, Decimal)> = model_costs.into_iter().collect();
        sorted_models.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_models.truncate(limit);

        Ok(sorted_models)
    }

    fn get_usage_by_provider(&self, records: &[&UsageRecord]) -> TokenResult<HashMap<Provider, ProviderUsageSummary>> {
        let mut provider_usage: HashMap<Provider, ProviderUsageSummary> = HashMap::new();

        for record in records {
            let summary = provider_usage.entry(record.provider.clone()).or_insert(ProviderUsageSummary {
                requests: 0,
                total_cost: Decimal::ZERO,
                total_tokens: 0,
            });

            summary.requests += 1;
            summary.total_cost += record.cost_breakdown.total_cost;
            summary.total_tokens += record.token_usage.total_tokens as u64;
        }

        Ok(provider_usage)
    }

    fn get_daily_cost_breakdown(&self, records: &[&UsageRecord]) -> TokenResult<HashMap<chrono::NaiveDate, Decimal>> {
        let mut daily_costs: HashMap<chrono::NaiveDate, Decimal> = HashMap::new();

        for record in records {
            let date = record.timestamp.date_naive();
            *daily_costs.entry(date).or_insert(Decimal::ZERO) += record.cost_breakdown.total_cost;
        }

        Ok(daily_costs)
    }

    // Helper methods

    async fn cleanup_old_records(&self, records: &mut Vec<UsageRecord>) {
        let cutoff_date = Utc::now() - Duration::days(self.config.retention_days as i64);
        records.retain(|record| record.timestamp >= cutoff_date);
    }

    async fn update_aggregated_stats(&self) -> TokenResult<()> {
        let records = self.usage_records.read().await;
        let mut stats = self.current_stats.write().await;

        for interval in &self.config.aggregation_intervals {
            let (start, end) = self.get_period_bounds(interval);
            let period_records: Vec<&UsageRecord> = records
                .iter()
                .filter(|r| r.timestamp >= start && r.timestamp <= end)
                .collect();

            let period_stats = self.calculate_stats(&period_records, start, end, interval.clone())?;
            stats.insert(interval.clone(), period_stats);
        }

        Ok(())
    }

    fn get_period_bounds(&self, interval: &AggregationInterval) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        match interval {
            AggregationInterval::Hourly => {
                let start = now.with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
                (start, now)
            }
            AggregationInterval::Daily => {
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                (start, now)
            }
            AggregationInterval::Weekly => {
                let days_since_monday = now.weekday().num_days_from_monday();
                let start = now.date_naive() - Duration::days(days_since_monday as i64);
                let start = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
                (start, now)
            }
            AggregationInterval::Monthly => {
                let start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc();
                (start, now)
            }
        }
    }

    fn calculate_stats(
        &self,
        records: &[&UsageRecord],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: AggregationInterval,
    ) -> TokenResult<UsageStats> {
        let mut provider_stats: HashMap<Provider, ProviderStats> = HashMap::new();
        let mut model_stats: HashMap<Model, ModelStats> = HashMap::new();
        let mut user_stats: HashMap<String, UserStats> = HashMap::new();

        let mut total_requests = 0u64;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;
        let mut total_cost = Decimal::ZERO;

        for record in records {
            total_requests += 1;
            total_input_tokens += record.token_usage.input_tokens as u64;
            total_output_tokens += record.token_usage.output_tokens as u64;
            total_cost += record.cost_breakdown.total_cost;

            // Update provider stats
            let provider_stat = provider_stats.entry(record.provider.clone()).or_insert(ProviderStats {
                requests: 0,
                input_tokens: 0,
                output_tokens: 0,
                cost: Decimal::ZERO,
                avg_tokens_per_request: 0.0,
                error_rate: 0.0,
            });
            provider_stat.requests += 1;
            provider_stat.input_tokens += record.token_usage.input_tokens as u64;
            provider_stat.output_tokens += record.token_usage.output_tokens as u64;
            provider_stat.cost += record.cost_breakdown.total_cost;

            // Update model stats
            let model_stat = model_stats.entry(record.model.clone()).or_insert(ModelStats {
                requests: 0,
                input_tokens: 0,
                output_tokens: 0,
                cost: Decimal::ZERO,
                avg_response_time_ms: 0.0,
                tokens_per_second: 0.0,
            });
            model_stat.requests += 1;
            model_stat.input_tokens += record.token_usage.input_tokens as u64;
            model_stat.output_tokens += record.token_usage.output_tokens as u64;
            model_stat.cost += record.cost_breakdown.total_cost;

            // Update user stats if user_id is present
            if let Some(user_id) = &record.user_id {
                let user_stat = user_stats.entry(user_id.clone()).or_insert(UserStats {
                    requests: 0,
                    total_tokens: 0,
                    total_cost: Decimal::ZERO,
                    favorite_model: None,
                    usage_trend: UsageTrend::Stable,
                });
                user_stat.requests += 1;
                user_stat.total_tokens += record.token_usage.total_tokens as u64;
                user_stat.total_cost += record.cost_breakdown.total_cost;
            }
        }

        // Calculate averages for provider stats
        for provider_stat in provider_stats.values_mut() {
            if provider_stat.requests > 0 {
                provider_stat.avg_tokens_per_request = 
                    (provider_stat.input_tokens + provider_stat.output_tokens) as f64 / provider_stat.requests as f64;
            }
        }

        Ok(UsageStats {
            period_start: start,
            period_end: end,
            interval,
            total_requests,
            total_input_tokens,
            total_output_tokens,
            total_cost,
            provider_breakdown: provider_stats,
            model_breakdown: model_stats,
            user_breakdown: user_stats,
            error_count: 0, // This would be tracked separately
        })
    }

    async fn export_to_csv(&self, records: &[UsageRecord], destination: &str) -> TokenResult<()> {
        let mut csv_content = String::new();
        csv_content.push_str("id,timestamp,provider,model,input_tokens,output_tokens,total_tokens,input_cost,output_cost,total_cost,user_id,workflow_id,session_id\n");

        for record in records {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                record.id,
                record.timestamp.to_rfc3339(),
                serde_json::to_string(&record.provider).unwrap_or_default().trim_matches('"'),
                serde_json::to_string(&record.model).unwrap_or_default().trim_matches('"'),
                record.token_usage.input_tokens,
                record.token_usage.output_tokens,
                record.token_usage.total_tokens,
                record.cost_breakdown.input_cost,
                record.cost_breakdown.output_cost,
                record.cost_breakdown.total_cost,
                record.user_id.as_ref().unwrap_or(&"".to_string()),
                record.workflow_id.map(|id| id.to_string()).unwrap_or_default(),
                record.session_id.map(|id| id.to_string()).unwrap_or_default()
            ));
        }

        tokio::fs::write(destination, csv_content).await
            .map_err(|e| TokenError::AnalyticsError(format!("Failed to write CSV file: {}", e)))?;

        Ok(())
    }

    fn calculate_confidence(&self, trends: &[DailyUsageTrend]) -> f64 {
        if trends.len() < 2 {
            return 0.0;
        }

        // Calculate variance in daily usage to determine confidence
        let mean_requests: f64 = trends.iter().map(|t| t.requests as f64).sum::<f64>() / trends.len() as f64;
        let variance: f64 = trends.iter()
            .map(|t| (t.requests as f64 - mean_requests).powi(2))
            .sum::<f64>() / trends.len() as f64;

        // Lower variance means higher confidence (simplified calculation)
        let coefficient_variation = if mean_requests > 0.0 { variance.sqrt() / mean_requests } else { 1.0 };
        (1.0 - coefficient_variation.min(1.0)).max(0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsageTrend {
    pub date: chrono::NaiveDate,
    pub requests: u64,
    pub total_tokens: u64,
    pub total_cost: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageForecast {
    pub forecast_date: DateTime<Utc>,
    pub predicted_requests: u64,
    pub predicted_tokens: u64,
    pub predicted_cost: Decimal,
    pub confidence_level: f64,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            retention_days: 90,
            aggregation_intervals: vec![
                AggregationInterval::Hourly,
                AggregationInterval::Daily,
                AggregationInterval::Weekly,
                AggregationInterval::Monthly,
            ],
            export_config: ExportConfig {
                enabled: false,
                format: ExportFormat::Json,
                destination: "./analytics_export.json".to_string(),
            },
        }
    }
}

/// Cost optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimizationRecommendation {
    pub recommendation_type: OptimizationType,
    pub title: String,
    pub description: String,
    pub potential_monthly_savings: Decimal,
    pub confidence_score: f64, // 0.0 to 1.0
    pub implementation_effort: ImplementationEffort,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    ModelSubstitution,
    RequestBatching,
    ParameterOptimization,
    UsagePattern,
    ProviderSwitching,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Detailed cost analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedCostAnalysis {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_cost: Decimal,
    pub average_daily_cost: Decimal,
    pub provider_breakdown: HashMap<Provider, Decimal>,
    pub model_breakdown: HashMap<Model, Decimal>,
    pub hourly_distribution: HashMap<u32, Decimal>,
    pub cost_trends: CostTrendData,
    pub top_cost_drivers: Vec<CostDriver>,
}

/// Cost trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrendData {
    pub trend_direction: TrendDirection,
    pub daily_costs: HashMap<chrono::NaiveDate, Decimal>,
    pub average_daily_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Cost driver identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDriver {
    pub driver_type: CostDriverType,
    pub cost: Decimal,
    pub percentage_of_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostDriverType {
    Model(Model),
    Provider(Provider),
    User(String),
    TimeOfDay(u32), // Hour
}

/// Monthly cost report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyCostReport {
    pub month: u32,
    pub year: u32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_cost: Decimal,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub cost_per_request: Decimal,
    pub cost_per_token: Decimal,
    pub cost_change_from_previous_month: f64,
    pub top_models_by_cost: Vec<(Model, Decimal)>,
    pub usage_by_provider: HashMap<Provider, ProviderUsageSummary>,
    pub daily_cost_breakdown: HashMap<chrono::NaiveDate, Decimal>,
    pub insights: Vec<String>,
}

/// Provider usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsageSummary {
    pub requests: u64,
    pub total_cost: Decimal,
    pub total_tokens: u64,
}

impl ToString for Provider {
    fn to_string(&self) -> String {
        match self {
            Provider::OpenAI => "OpenAI".to_string(),
            Provider::Anthropic => "Anthropic".to_string(),
            Provider::Bedrock => "Bedrock".to_string(),
        }
    }
}