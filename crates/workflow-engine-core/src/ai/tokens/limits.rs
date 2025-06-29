//! Budget limits and alerting system for AI usage control

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration, Timelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;
use crate::ai::tokens::{Model, Provider, TokenUsage, CostBreakdown, TokenError, TokenResult};
#[cfg(feature = "streaming")]
use crate::streaming::types::StreamMetadata;

/// Budget limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitConfig {
    pub global_limits: GlobalLimits,
    pub provider_limits: HashMap<Provider, ProviderLimits>,
    pub model_limits: HashMap<Model, ModelLimits>,
    pub user_limits: HashMap<String, UserLimits>,
    pub alerting: AlertingConfig,
}

/// Global budget limits across all providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLimits {
    pub daily_cost_limit: Option<Decimal>,
    pub monthly_cost_limit: Option<Decimal>,
    pub daily_token_limit: Option<u64>,
    pub monthly_token_limit: Option<u64>,
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>,
    pub enabled: bool,
}

/// Provider-specific limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderLimits {
    pub provider: Provider,
    pub daily_cost_limit: Option<Decimal>,
    pub monthly_cost_limit: Option<Decimal>,
    pub daily_token_limit: Option<u64>,
    pub monthly_token_limit: Option<u64>,
    pub requests_per_minute: Option<u32>,
    pub enabled: bool,
}

/// Model-specific limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimits {
    pub model: Model,
    pub daily_cost_limit: Option<Decimal>,
    pub monthly_cost_limit: Option<Decimal>,
    pub daily_token_limit: Option<u64>,
    pub monthly_token_limit: Option<u64>,
    pub max_tokens_per_request: Option<u32>,
    pub enabled: bool,
}

/// User-specific limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLimits {
    pub user_id: String,
    pub daily_cost_limit: Option<Decimal>,
    pub monthly_cost_limit: Option<Decimal>,
    pub daily_token_limit: Option<u64>,
    pub monthly_token_limit: Option<u64>,
    pub requests_per_hour: Option<u32>,
    pub enabled: bool,
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub enabled: bool,
    pub warning_thresholds: Vec<AlertThreshold>,
    pub notification_channels: Vec<NotificationChannel>,
    pub cooldown_minutes: u32,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    pub name: String,
    pub threshold_type: ThresholdType,
    pub percentage: f64, // e.g., 80.0 for 80% of limit
    pub scope: AlertScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdType {
    Cost,
    Tokens,
    Requests,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertScope {
    Global,
    Provider(Provider),
    Model(Model),
    User(String),
}

/// Notification channels for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Log,
    Email(String),
    Webhook(String),
    Slack(SlackConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
}

/// Current usage tracking for limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindow {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub requests: u32,
    pub total_tokens: u64,
    pub total_cost: Decimal,
}

/// Limit violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitViolation {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub limit_type: LimitType,
    pub scope: AlertScope,
    pub current_value: Decimal,
    pub limit_value: Decimal,
    pub percentage_used: f64,
    pub action_taken: LimitAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LimitType {
    DailyCost,
    MonthlyCost,
    DailyTokens,
    MonthlyTokens,
    RequestsPerMinute,
    RequestsPerHour,
    TokensPerRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LimitAction {
    Warning,
    Block,
    Throttle,
}

/// Main budget limits engine
pub struct BudgetLimits {
    config: Arc<RwLock<LimitConfig>>,
    current_usage: Arc<RwLock<HashMap<String, UsageWindow>>>,
    violations: Arc<RwLock<Vec<LimitViolation>>>,
    last_alert_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl BudgetLimits {
    /// Create a new budget limits engine
    pub fn new(config: LimitConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            current_usage: Arc::new(RwLock::new(HashMap::new())),
            violations: Arc::new(RwLock::new(Vec::new())),
            last_alert_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request is allowed given current usage and limits
    pub async fn check_request_allowed(
        &self,
        provider: &Provider,
        model: &Model,
        token_usage: &TokenUsage,
        cost: &CostBreakdown,
        user_id: Option<&str>,
    ) -> TokenResult<bool> {
        let config = self.config.read().await;

        // Check global limits
        if config.global_limits.enabled && !self.check_global_limits(&config.global_limits, token_usage, cost).await? {
            return Ok(false);
        }

        // Check provider limits
        if let Some(provider_limits) = config.provider_limits.get(provider) {
            if provider_limits.enabled && !self.check_provider_limits(provider_limits, token_usage, cost).await? {
                return Ok(false);
            }
        }

        // Check model limits
        if let Some(model_limits) = config.model_limits.get(model) {
            if model_limits.enabled && !self.check_model_limits(model_limits, token_usage, cost).await? {
                return Ok(false);
            }
        }

        // Check user limits
        if let Some(user_id) = user_id {
            if let Some(user_limits) = config.user_limits.get(user_id) {
                if user_limits.enabled && !self.check_user_limits(user_limits, token_usage, cost).await? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Record usage for limit tracking
    pub async fn record_usage(
        &self,
        provider: &Provider,
        model: &Model,
        token_usage: &TokenUsage,
        cost: &CostBreakdown,
        user_id: Option<&str>,
    ) -> TokenResult<()> {
        let now = Utc::now();

        // Update global usage
        self.update_usage_window("global", token_usage, cost, now).await;

        // Update provider usage
        let provider_key = format!("provider:{}", provider.to_string());
        self.update_usage_window(&provider_key, token_usage, cost, now).await;

        // Update model usage
        let model_key = format!("model:{}", model.as_str());
        self.update_usage_window(&model_key, token_usage, cost, now).await;

        // Update user usage
        if let Some(user_id) = user_id {
            let user_key = format!("user:{}", user_id);
            self.update_usage_window(&user_key, token_usage, cost, now).await;
        }

        // Check for threshold violations and send alerts
        self.check_and_send_alerts(provider, model, user_id).await?;

        Ok(())
    }

    /// Get current usage for a specific scope
    pub async fn get_current_usage(&self, scope: &AlertScope) -> TokenResult<Option<UsageWindow>> {
        let usage = self.current_usage.read().await;
        let key = self.scope_to_key(scope);
        Ok(usage.get(&key).cloned())
    }

    /// Get all recent violations
    pub async fn get_violations(&self, since: Option<DateTime<Utc>>) -> TokenResult<Vec<LimitViolation>> {
        let violations = self.violations.read().await;
        let cutoff = since.unwrap_or_else(|| Utc::now() - Duration::days(7));
        
        Ok(violations.iter()
            .filter(|v| v.timestamp >= cutoff)
            .cloned()
            .collect())
    }

    /// Update limit configuration
    pub async fn update_config(&self, new_config: LimitConfig) -> TokenResult<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }

    /// Reset usage counters (useful for testing or manual reset)
    pub async fn reset_usage(&self, scope: Option<AlertScope>) -> TokenResult<()> {
        let mut usage = self.current_usage.write().await;
        
        if let Some(scope) = scope {
            let key = self.scope_to_key(&scope);
            usage.remove(&key);
        } else {
            usage.clear();
        }
        
        Ok(())
    }

    /// Check throttling status for streaming requests
    #[cfg(feature = "streaming")]
    pub async fn check_streaming_throttle(
        &self,
        metadata: &StreamMetadata,
        user_id: Option<&str>,
    ) -> TokenResult<ThrottleDecision> {
        let provider = self.parse_provider_from_string(&metadata.provider)?;
        let _model = self.parse_model_from_string(&metadata.model)?;
        
        // Check if we're approaching limits
        let config = self.config.read().await;
        
        // Check global throttling
        if let Some(global_throttle) = self.check_global_throttling(&config.global_limits).await? {
            return Ok(global_throttle);
        }

        // Check provider throttling
        if let Some(provider_limits) = config.provider_limits.get(&provider) {
            if let Some(provider_throttle) = self.check_provider_throttling(provider_limits).await? {
                return Ok(provider_throttle);
            }
        }

        // Check user throttling
        if let Some(user_id) = user_id {
            if let Some(user_limits) = config.user_limits.get(user_id) {
                if let Some(user_throttle) = self.check_user_throttling(user_limits).await? {
                    return Ok(user_throttle);
                }
            }
        }

        Ok(ThrottleDecision::Allow)
    }

    /// Apply dynamic throttling based on cost trajectory
    pub async fn calculate_dynamic_throttle(
        &self,
        _provider: &Provider,
        current_cost_rate: Decimal, // cost per minute
        budget_limit: Decimal,
        time_remaining_minutes: u32,
    ) -> TokenResult<ThrottleDecision> {
        let projected_total = current_cost_rate * Decimal::from(time_remaining_minutes);
        
        if projected_total > budget_limit {
            // Calculate required throttling percentage
            let throttle_factor = budget_limit / projected_total;
            let throttle_percentage = (Decimal::from(1) - throttle_factor) * Decimal::from(100);
            
            if throttle_percentage > Decimal::from(50) {
                Ok(ThrottleDecision::Block {
                    reason: "Projected cost exceeds budget limit significantly".to_string(),
                    retry_after_seconds: 300, // 5 minutes
                })
            } else {
                Ok(ThrottleDecision::Throttle {
                    delay_ms: (throttle_percentage.to_u64().unwrap_or(10) * 10) as u32,
                    throttle_percentage: throttle_percentage.to_f64().unwrap_or(10.0),
                    reason: format!("Reducing request rate by {:.1}% to stay within budget", throttle_percentage),
                })
            }
        } else {
            Ok(ThrottleDecision::Allow)
        }
    }

    /// Get current throttling status
    pub async fn get_throttling_status(&self) -> TokenResult<ThrottlingStatus> {
        let config = self.config.read().await;
        let usage = self.current_usage.read().await;
        
        let mut active_throttles = Vec::new();
        
        // Check global throttling
        if let Some(global_usage) = usage.get("global") {
            if let Some(throttle) = self.evaluate_throttle_need(&config.global_limits, global_usage)? {
                active_throttles.push(ActiveThrottle {
                    scope: AlertScope::Global,
                    throttle_type: throttle.clone(),
                    start_time: Utc::now(), // In real implementation, track actual start time
                    expected_duration_seconds: 300,
                });
            }
        }

        // Check provider throttling
        for (provider, limits) in &config.provider_limits {
            let provider_key = format!("provider:{}", provider.to_string());
            if let Some(provider_usage) = usage.get(&provider_key) {
                if let Some(throttle) = self.evaluate_provider_throttle_need(limits, provider_usage)? {
                    active_throttles.push(ActiveThrottle {
                        scope: AlertScope::Provider(provider.clone()),
                        throttle_type: throttle.clone(),
                        start_time: Utc::now(),
                        expected_duration_seconds: 300,
                    });
                }
            }
        }

        Ok(ThrottlingStatus {
            is_active: !active_throttles.is_empty(),
            active_throttles,
            system_load: self.calculate_system_load().await?,
        })
    }

    /// Create throttling override for emergency situations
    pub async fn create_throttling_override(
        &self,
        scope: AlertScope,
        duration_minutes: u32,
        reason: String,
        authorized_by: String,
    ) -> TokenResult<ThrottlingOverride> {
        let override_id = Uuid::new_v4();
        let override_record = ThrottlingOverride {
            id: override_id,
            scope,
            start_time: Utc::now(),
            end_time: Utc::now() + Duration::minutes(duration_minutes as i64),
            reason,
            authorized_by,
            is_active: true,
        };

        // In a real implementation, store this override in persistent storage
        log::warn!("Throttling override created: {:?}", override_record);

        Ok(override_record)
    }

    // Helper methods for throttling

    async fn check_global_throttling(&self, limits: &GlobalLimits) -> TokenResult<Option<ThrottleDecision>> {
        let usage = self.get_current_usage(&AlertScope::Global).await?;
        
        if let Some(usage) = usage {
            if let Some(daily_limit) = limits.daily_cost_limit {
                let usage_percentage = (usage.total_cost / daily_limit * Decimal::from(100))
                    .to_f64()
                    .unwrap_or(0.0);
                
                if usage_percentage > 95.0 {
                    return Ok(Some(ThrottleDecision::Block {
                        reason: "Daily cost limit nearly exceeded".to_string(),
                        retry_after_seconds: 3600, // 1 hour
                    }));
                } else if usage_percentage > 80.0 {
                    return Ok(Some(ThrottleDecision::Throttle {
                        delay_ms: ((usage_percentage - 80.0) * 50.0) as u32,
                        throttle_percentage: usage_percentage - 80.0,
                        reason: format!("Throttling due to {}% of daily budget used", usage_percentage),
                    }));
                }
            }

            // Check rate limits
            if let Some(rpm_limit) = limits.requests_per_minute {
                if self.is_same_minute(usage.window_start) && usage.requests >= rpm_limit {
                    return Ok(Some(ThrottleDecision::Block {
                        reason: "Request rate limit exceeded".to_string(),
                        retry_after_seconds: 60,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn check_provider_throttling(&self, limits: &ProviderLimits) -> TokenResult<Option<ThrottleDecision>> {
        let scope = AlertScope::Provider(limits.provider.clone());
        let usage = self.get_current_usage(&scope).await?;
        
        if let Some(usage) = usage {
            if let Some(daily_limit) = limits.daily_cost_limit {
                let usage_percentage = (usage.total_cost / daily_limit * Decimal::from(100))
                    .to_f64()
                    .unwrap_or(0.0);
                
                if usage_percentage > 90.0 {
                    return Ok(Some(ThrottleDecision::Block {
                        reason: format!("Provider {} daily limit nearly exceeded", limits.provider.to_string()),
                        retry_after_seconds: 1800, // 30 minutes
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn check_user_throttling(&self, limits: &UserLimits) -> TokenResult<Option<ThrottleDecision>> {
        let scope = AlertScope::User(limits.user_id.clone());
        let usage = self.get_current_usage(&scope).await?;
        
        if let Some(usage) = usage {
            if let Some(hourly_limit) = limits.requests_per_hour {
                if self.is_same_hour(usage.window_start) && usage.requests >= hourly_limit {
                    return Ok(Some(ThrottleDecision::Block {
                        reason: format!("User {} hourly request limit exceeded", limits.user_id),
                        retry_after_seconds: 300, // 5 minutes
                    }));
                }
            }
        }

        Ok(None)
    }

    fn evaluate_throttle_need(&self, limits: &GlobalLimits, usage: &UsageWindow) -> TokenResult<Option<ThrottleType>> {
        if let Some(daily_limit) = limits.daily_cost_limit {
            let usage_percentage = (usage.total_cost / daily_limit * Decimal::from(100))
                .to_f64()
                .unwrap_or(0.0);
            
            if usage_percentage > 95.0 {
                return Ok(Some(ThrottleType::Block));
            } else if usage_percentage > 80.0 {
                return Ok(Some(ThrottleType::RateLimit {
                    delay_ms: ((usage_percentage - 80.0) * 50.0) as u32,
                }));
            }
        }

        Ok(None)
    }

    fn evaluate_provider_throttle_need(&self, limits: &ProviderLimits, usage: &UsageWindow) -> TokenResult<Option<ThrottleType>> {
        if let Some(daily_limit) = limits.daily_cost_limit {
            let usage_percentage = (usage.total_cost / daily_limit * Decimal::from(100))
                .to_f64()
                .unwrap_or(0.0);
            
            if usage_percentage > 90.0 {
                return Ok(Some(ThrottleType::Block));
            }
        }

        Ok(None)
    }

    async fn calculate_system_load(&self) -> TokenResult<SystemLoadMetrics> {
        let usage = self.current_usage.read().await;
        
        let total_requests: u32 = usage.values().map(|u| u.requests).sum();
        let total_cost: Decimal = usage.values().map(|u| u.total_cost).sum();
        
        Ok(SystemLoadMetrics {
            current_requests_per_minute: total_requests,
            current_cost_per_minute: total_cost,
            active_throttles: 0, // Would calculate from actual throttles
        })
    }

    fn parse_provider_from_string(&self, provider_str: &str) -> TokenResult<Provider> {
        match provider_str.to_lowercase().as_str() {
            "openai" => Ok(Provider::OpenAI),
            "anthropic" => Ok(Provider::Anthropic),
            "bedrock" => Ok(Provider::Bedrock),
            _ => Err(TokenError::UnsupportedModel(format!("Unknown provider: {}", provider_str))),
        }
    }

    fn parse_model_from_string(&self, model_str: &str) -> TokenResult<Model> {
        match model_str {
            "gpt-4" => Ok(Model::Gpt4),
            "gpt-4-turbo" => Ok(Model::Gpt4Turbo),
            "gpt-3.5-turbo" => Ok(Model::Gpt35Turbo),
            "claude-3-opus" => Ok(Model::Claude3Opus),
            "claude-3-sonnet" => Ok(Model::Claude3Sonnet),
            "claude-3-haiku" => Ok(Model::Claude3Haiku),
            _ => Err(TokenError::UnsupportedModel(model_str.to_string())),
        }
    }

    // Helper methods

    async fn check_global_limits(
        &self,
        limits: &GlobalLimits,
        token_usage: &TokenUsage,
        cost: &CostBreakdown,
    ) -> TokenResult<bool> {
        let usage = self.get_current_usage(&AlertScope::Global).await?;
        
        if let Some(usage) = usage {
            // Check daily cost limit
            if let Some(daily_limit) = limits.daily_cost_limit {
                if self.is_same_day(usage.window_start) && usage.total_cost + cost.total_cost > daily_limit {
                    self.record_violation(
                        LimitType::DailyCost,
                        AlertScope::Global,
                        usage.total_cost + cost.total_cost,
                        daily_limit,
                        LimitAction::Block,
                    ).await;
                    return Ok(false);
                }
            }

            // Check daily token limit
            if let Some(daily_token_limit) = limits.daily_token_limit {
                if self.is_same_day(usage.window_start) && usage.total_tokens + token_usage.total_tokens as u64 > daily_token_limit {
                    self.record_violation(
                        LimitType::DailyTokens,
                        AlertScope::Global,
                        Decimal::from(usage.total_tokens + token_usage.total_tokens as u64),
                        Decimal::from(daily_token_limit),
                        LimitAction::Block,
                    ).await;
                    return Ok(false);
                }
            }

            // Check rate limits (requests per minute)
            if let Some(rpm_limit) = limits.requests_per_minute {
                if self.is_same_minute(usage.window_start) && usage.requests + 1 > rpm_limit {
                    self.record_violation(
                        LimitType::RequestsPerMinute,
                        AlertScope::Global,
                        Decimal::from(usage.requests + 1),
                        Decimal::from(rpm_limit),
                        LimitAction::Block,
                    ).await;
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    async fn check_provider_limits(
        &self,
        limits: &ProviderLimits,
        token_usage: &TokenUsage,
        cost: &CostBreakdown,
    ) -> TokenResult<bool> {
        let scope = AlertScope::Provider(limits.provider.clone());
        let usage = self.get_current_usage(&scope).await?;
        
        if let Some(usage) = usage {
            if let Some(daily_limit) = limits.daily_cost_limit {
                if self.is_same_day(usage.window_start) && usage.total_cost + cost.total_cost > daily_limit {
                    self.record_violation(
                        LimitType::DailyCost,
                        scope,
                        usage.total_cost + cost.total_cost,
                        daily_limit,
                        LimitAction::Block,
                    ).await;
                    return Ok(false);
                }
            }

            if let Some(daily_token_limit) = limits.daily_token_limit {
                if self.is_same_day(usage.window_start) && usage.total_tokens + token_usage.total_tokens as u64 > daily_token_limit {
                    self.record_violation(
                        LimitType::DailyTokens,
                        scope,
                        Decimal::from(usage.total_tokens + token_usage.total_tokens as u64),
                        Decimal::from(daily_token_limit),
                        LimitAction::Block,
                    ).await;
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    async fn check_model_limits(
        &self,
        limits: &ModelLimits,
        token_usage: &TokenUsage,
        cost: &CostBreakdown,
    ) -> TokenResult<bool> {
        // Check max tokens per request
        if let Some(max_tokens) = limits.max_tokens_per_request {
            if token_usage.total_tokens > max_tokens {
                let scope = AlertScope::Model(limits.model.clone());
                self.record_violation(
                    LimitType::TokensPerRequest,
                    scope,
                    Decimal::from(token_usage.total_tokens),
                    Decimal::from(max_tokens),
                    LimitAction::Block,
                ).await;
                return Ok(false);
            }
        }

        // Check other model limits similar to provider limits
        let scope = AlertScope::Model(limits.model.clone());
        let usage = self.get_current_usage(&scope).await?;
        
        if let Some(usage) = usage {
            if let Some(daily_limit) = limits.daily_cost_limit {
                if self.is_same_day(usage.window_start) && usage.total_cost + cost.total_cost > daily_limit {
                    self.record_violation(
                        LimitType::DailyCost,
                        scope,
                        usage.total_cost + cost.total_cost,
                        daily_limit,
                        LimitAction::Block,
                    ).await;
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    async fn check_user_limits(
        &self,
        limits: &UserLimits,
        _token_usage: &TokenUsage,
        cost: &CostBreakdown,
    ) -> TokenResult<bool> {
        let scope = AlertScope::User(limits.user_id.clone());
        let usage = self.get_current_usage(&scope).await?;
        
        if let Some(usage) = usage {
            if let Some(daily_limit) = limits.daily_cost_limit {
                if self.is_same_day(usage.window_start) && usage.total_cost + cost.total_cost > daily_limit {
                    self.record_violation(
                        LimitType::DailyCost,
                        scope,
                        usage.total_cost + cost.total_cost,
                        daily_limit,
                        LimitAction::Block,
                    ).await;
                    return Ok(false);
                }
            }

            if let Some(hourly_limit) = limits.requests_per_hour {
                if self.is_same_hour(usage.window_start) && usage.requests + 1 > hourly_limit {
                    self.record_violation(
                        LimitType::RequestsPerHour,
                        scope,
                        Decimal::from(usage.requests + 1),
                        Decimal::from(hourly_limit),
                        LimitAction::Block,
                    ).await;
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    async fn update_usage_window(
        &self,
        key: &str,
        token_usage: &TokenUsage,
        cost: &CostBreakdown,
        timestamp: DateTime<Utc>,
    ) {
        let mut usage = self.current_usage.write().await;
        
        let window = usage.entry(key.to_string()).or_insert_with(|| UsageWindow {
            window_start: timestamp,
            window_end: timestamp,
            requests: 0,
            total_tokens: 0,
            total_cost: Decimal::ZERO,
        });

        // Reset window if it's a new day
        if !self.is_same_day(window.window_start) {
            window.window_start = timestamp.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
            window.requests = 0;
            window.total_tokens = 0;
            window.total_cost = Decimal::ZERO;
        }

        window.window_end = timestamp;
        window.requests += 1;
        window.total_tokens += token_usage.total_tokens as u64;
        window.total_cost += cost.total_cost;
    }

    async fn record_violation(
        &self,
        limit_type: LimitType,
        scope: AlertScope,
        current_value: Decimal,
        limit_value: Decimal,
        action: LimitAction,
    ) {
        let violation = LimitViolation {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            limit_type,
            scope,
            current_value,
            limit_value,
            percentage_used: if limit_value > Decimal::ZERO {
                (current_value / limit_value * Decimal::from(100)).to_f64().unwrap_or(0.0)
            } else {
                100.0
            },
            action_taken: action,
        };

        let mut violations = self.violations.write().await;
        violations.push(violation.clone());

        log::warn!("Budget limit violation: {:?}", violation);
    }

    async fn check_and_send_alerts(
        &self,
        provider: &Provider,
        model: &Model,
        user_id: Option<&str>,
    ) -> TokenResult<()> {
        let config = self.config.read().await;
        
        if !config.alerting.enabled {
            return Ok(());
        }

        // Check thresholds for each scope
        for threshold in &config.alerting.warning_thresholds {
            let should_alert = match &threshold.scope {
                AlertScope::Global => self.check_threshold_violation(threshold, &AlertScope::Global).await?,
                AlertScope::Provider(p) if p == provider => self.check_threshold_violation(threshold, &AlertScope::Provider(provider.clone())).await?,
                AlertScope::Model(m) if m == model => self.check_threshold_violation(threshold, &AlertScope::Model(model.clone())).await?,
                AlertScope::User(u) if user_id.map(|id| id == u).unwrap_or(false) => {
                    self.check_threshold_violation(threshold, &AlertScope::User(u.clone())).await?
                },
                _ => false,
            };

            if should_alert {
                self.send_alert(threshold, &config.alerting).await?;
            }
        }

        Ok(())
    }

    async fn check_threshold_violation(&self, threshold: &AlertThreshold, scope: &AlertScope) -> TokenResult<bool> {
        let usage = self.get_current_usage(scope).await?;
        
        if let Some(usage) = usage {
            let config = self.config.read().await;
            
            let (current_value, limit_value) = match &threshold.threshold_type {
                ThresholdType::Cost => {
                    let limit = self.get_cost_limit_for_scope(scope, &config);
                    (usage.total_cost, limit)
                },
                ThresholdType::Tokens => {
                    let limit = self.get_token_limit_for_scope(scope, &config);
                    (Decimal::from(usage.total_tokens), limit)
                },
                ThresholdType::Requests => {
                    let limit = self.get_request_limit_for_scope(scope, &config);
                    (Decimal::from(usage.requests), limit)
                },
            };

            if limit_value > Decimal::ZERO {
                let percentage_used = (current_value / limit_value * Decimal::from(100)).to_f64().unwrap_or(0.0);
                return Ok(percentage_used >= threshold.percentage);
            }
        }

        Ok(false)
    }

    async fn send_alert(&self, threshold: &AlertThreshold, config: &AlertingConfig) -> TokenResult<()> {
        let alert_key = format!("{}:{}", threshold.name, self.scope_to_key(&threshold.scope));
        
        // Check cooldown
        {
            let last_alerts = self.last_alert_times.read().await;
            if let Some(last_alert) = last_alerts.get(&alert_key) {
                let cooldown_duration = Duration::minutes(config.cooldown_minutes as i64);
                if Utc::now() - *last_alert < cooldown_duration {
                    return Ok(()); // Still in cooldown
                }
            }
        }

        // Send notifications
        for channel in &config.notification_channels {
            self.send_notification(channel, threshold).await?;
        }

        // Update last alert time
        {
            let mut last_alerts = self.last_alert_times.write().await;
            last_alerts.insert(alert_key, Utc::now());
        }

        Ok(())
    }

    async fn send_notification(&self, channel: &NotificationChannel, threshold: &AlertThreshold) -> TokenResult<()> {
        match channel {
            NotificationChannel::Log => {
                log::warn!("Budget threshold alert: {} exceeded {}%", threshold.name, threshold.percentage);
            },
            NotificationChannel::Email(email) => {
                log::info!("Would send email alert to: {}", email);
                // Email implementation would go here
            },
            NotificationChannel::Webhook(url) => {
                log::info!("Would send webhook alert to: {}", url);
                // Webhook implementation would go here
            },
            NotificationChannel::Slack(config) => {
                log::info!("Would send Slack alert to: {}", config.channel);
                // Slack implementation would go here
            },
        }
        Ok(())
    }

    // Utility methods

    fn scope_to_key(&self, scope: &AlertScope) -> String {
        match scope {
            AlertScope::Global => "global".to_string(),
            AlertScope::Provider(p) => format!("provider:{}", p.to_string()),
            AlertScope::Model(m) => format!("model:{}", m.as_str()),
            AlertScope::User(u) => format!("user:{}", u),
        }
    }

    fn is_same_day(&self, timestamp: DateTime<Utc>) -> bool {
        let now = Utc::now();
        now.date_naive() == timestamp.date_naive()
    }

    fn is_same_hour(&self, timestamp: DateTime<Utc>) -> bool {
        let now = Utc::now();
        now.date_naive() == timestamp.date_naive() && now.hour() == timestamp.hour()
    }

    fn is_same_minute(&self, timestamp: DateTime<Utc>) -> bool {
        let now = Utc::now();
        self.is_same_hour(timestamp) && now.minute() == timestamp.minute()
    }

    fn get_cost_limit_for_scope(&self, scope: &AlertScope, config: &LimitConfig) -> Decimal {
        match scope {
            AlertScope::Global => config.global_limits.daily_cost_limit.unwrap_or(Decimal::ZERO),
            AlertScope::Provider(p) => {
                config.provider_limits.get(p)
                    .and_then(|l| l.daily_cost_limit)
                    .unwrap_or(Decimal::ZERO)
            },
            AlertScope::Model(m) => {
                config.model_limits.get(m)
                    .and_then(|l| l.daily_cost_limit)
                    .unwrap_or(Decimal::ZERO)
            },
            AlertScope::User(u) => {
                config.user_limits.get(u)
                    .and_then(|l| l.daily_cost_limit)
                    .unwrap_or(Decimal::ZERO)
            },
        }
    }

    fn get_token_limit_for_scope(&self, scope: &AlertScope, config: &LimitConfig) -> Decimal {
        match scope {
            AlertScope::Global => Decimal::from(config.global_limits.daily_token_limit.unwrap_or(0)),
            AlertScope::Provider(p) => {
                Decimal::from(config.provider_limits.get(p)
                    .and_then(|l| l.daily_token_limit)
                    .unwrap_or(0))
            },
            AlertScope::Model(m) => {
                Decimal::from(config.model_limits.get(m)
                    .and_then(|l| l.daily_token_limit)
                    .unwrap_or(0))
            },
            AlertScope::User(u) => {
                Decimal::from(config.user_limits.get(u)
                    .and_then(|l| l.daily_token_limit)
                    .unwrap_or(0))
            },
        }
    }

    fn get_request_limit_for_scope(&self, scope: &AlertScope, config: &LimitConfig) -> Decimal {
        match scope {
            AlertScope::Global => Decimal::from(config.global_limits.requests_per_hour.unwrap_or(0)),
            AlertScope::Provider(p) => {
                Decimal::from(config.provider_limits.get(p)
                    .and_then(|l| l.requests_per_minute)
                    .unwrap_or(0))
            },
            AlertScope::Model(_) => Decimal::ZERO, // Models don't have request limits
            AlertScope::User(u) => {
                Decimal::from(config.user_limits.get(u)
                    .and_then(|l| l.requests_per_hour)
                    .unwrap_or(0))
            },
        }
    }
}

/// Throttling decision for request control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThrottleDecision {
    Allow,
    Throttle {
        delay_ms: u32,
        throttle_percentage: f64,
        reason: String,
    },
    Block {
        reason: String,
        retry_after_seconds: u32,
    },
}

/// Type of throttling being applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThrottleType {
    Block,
    RateLimit { delay_ms: u32 },
    CostLimit { percentage_reduction: f64 },
}

/// Current throttling status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottlingStatus {
    pub is_active: bool,
    pub active_throttles: Vec<ActiveThrottle>,
    pub system_load: SystemLoadMetrics,
}

/// Active throttle information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveThrottle {
    pub scope: AlertScope,
    pub throttle_type: ThrottleType,
    pub start_time: DateTime<Utc>,
    pub expected_duration_seconds: u32,
}

/// System load metrics for throttling decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLoadMetrics {
    pub current_requests_per_minute: u32,
    pub current_cost_per_minute: Decimal,
    pub active_throttles: u32,
}

/// Throttling override for emergency situations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottlingOverride {
    pub id: Uuid,
    pub scope: AlertScope,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub reason: String,
    pub authorized_by: String,
    pub is_active: bool,
}

impl Default for LimitConfig {
    fn default() -> Self {
        Self {
            global_limits: GlobalLimits {
                daily_cost_limit: Some(Decimal::from(100)), // $100 daily
                monthly_cost_limit: Some(Decimal::from(2000)), // $2000 monthly
                daily_token_limit: Some(1_000_000), // 1M tokens daily
                monthly_token_limit: Some(30_000_000), // 30M tokens monthly
                requests_per_minute: Some(100),
                requests_per_hour: Some(1000),
                enabled: true,
            },
            provider_limits: HashMap::new(),
            model_limits: HashMap::new(),
            user_limits: HashMap::new(),
            alerting: AlertingConfig {
                enabled: true,
                warning_thresholds: vec![
                    AlertThreshold {
                        name: "High Usage Warning".to_string(),
                        threshold_type: ThresholdType::Cost,
                        percentage: 80.0,
                        scope: AlertScope::Global,
                    },
                    AlertThreshold {
                        name: "Critical Usage Warning".to_string(),
                        threshold_type: ThresholdType::Cost,
                        percentage: 95.0,
                        scope: AlertScope::Global,
                    },
                ],
                notification_channels: vec![NotificationChannel::Log],
                cooldown_minutes: 30,
            },
        }
    }
}