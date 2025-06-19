//! Real-time budget tracking and alerting system for AI cost management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration, Datelike, Timelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;
use crate::ai::tokens::{Model, Provider, CostBreakdown, TokenError, TokenResult};
#[cfg(feature = "streaming")]
use crate::streaming::types::StreamMetadata;

/// Budget configuration for different scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub global_budget: GlobalBudget,
    pub provider_budgets: HashMap<Provider, ProviderBudget>,
    pub user_budgets: HashMap<String, UserBudget>,
    pub project_budgets: HashMap<String, ProjectBudget>,
    pub alert_config: BudgetAlertConfig,
}

/// Global budget limits across all AI usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalBudget {
    pub daily_limit: Option<Decimal>,
    pub weekly_limit: Option<Decimal>,
    pub monthly_limit: Option<Decimal>,
    pub yearly_limit: Option<Decimal>,
    pub enabled: bool,
    pub reset_on_period_start: bool,
}

/// Provider-specific budget limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderBudget {
    pub provider: Provider,
    pub daily_limit: Option<Decimal>,
    pub monthly_limit: Option<Decimal>,
    pub enabled: bool,
    pub priority: BudgetPriority,
}

/// User-specific budget limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBudget {
    pub user_id: String,
    pub daily_limit: Option<Decimal>,
    pub monthly_limit: Option<Decimal>,
    pub enabled: bool,
    pub rollover_unused: bool,
}

/// Project-specific budget limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBudget {
    pub project_id: String,
    pub total_budget: Decimal,
    pub monthly_budget: Option<Decimal>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub enabled: bool,
}

/// Budget priority for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Budget alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlertConfig {
    pub enabled: bool,
    pub warning_thresholds: Vec<f64>, // e.g., [0.5, 0.8, 0.95] for 50%, 80%, 95%
    pub notification_channels: Vec<NotificationChannel>,
    pub alert_frequency: AlertFrequency,
    pub cooldown_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertFrequency {
    Immediate,
    Hourly,
    Daily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Log,
    Email(String),
    Webhook(String),
    Slack(String),
    Discord(String),
}

/// Current budget status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub scope: BudgetScope,
    pub period: BudgetPeriod,
    pub current_spending: Decimal,
    pub budget_limit: Decimal,
    pub remaining_budget: Decimal,
    pub percentage_used: f64,
    pub status: BudgetHealthStatus,
    pub last_updated: DateTime<Utc>,
    pub projected_monthly_cost: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetScope {
    Global,
    Provider(Provider),
    User(String),
    Project(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Custom { start: DateTime<Utc>, end: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetHealthStatus {
    Healthy,      // < 50% used
    Warning,      // 50-80% used
    Critical,     // 80-95% used
    Exceeded,     // > 95% used
    Depleted,     // 100% used
}

/// Budget alert record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub scope: BudgetScope,
    pub alert_type: BudgetAlertType,
    pub current_spending: Decimal,
    pub budget_limit: Decimal,
    pub percentage_used: f64,
    pub message: String,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetAlertType {
    Warning(f64),    // Threshold percentage
    Critical(f64),   // Threshold percentage
    Exceeded,
    Depleted,
    ProjectionAlert, // Projected to exceed budget
}

/// Real-time budget tracking engine
pub struct BudgetTracker {
    config: Arc<RwLock<BudgetConfig>>,
    current_spending: Arc<RwLock<HashMap<String, BudgetSpending>>>,
    alerts: Arc<RwLock<Vec<BudgetAlert>>>,
    last_alert_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

#[derive(Debug, Clone)]
struct BudgetSpending {
    pub daily_spending: Decimal,
    pub weekly_spending: Decimal,
    pub monthly_spending: Decimal,
    pub yearly_spending: Decimal,
    pub last_reset_daily: DateTime<Utc>,
    pub last_reset_weekly: DateTime<Utc>,
    pub last_reset_monthly: DateTime<Utc>,
    pub last_reset_yearly: DateTime<Utc>,
}

impl BudgetTracker {
    /// Create a new budget tracker
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            current_spending: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            last_alert_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request is within budget limits
    pub async fn check_budget_allowed(
        &self,
        provider: &Provider,
        model: &Model,
        cost: &CostBreakdown,
        user_id: Option<&str>,
        project_id: Option<&str>,
    ) -> TokenResult<bool> {
        let config = self.config.read().await;

        // Check global budget
        if config.global_budget.enabled {
            if !self.check_global_budget(&config.global_budget, cost).await? {
                return Ok(false);
            }
        }

        // Check provider budget
        if let Some(provider_budget) = config.provider_budgets.get(provider) {
            if provider_budget.enabled {
                if !self.check_provider_budget(provider_budget, cost).await? {
                    return Ok(false);
                }
            }
        }

        // Check user budget
        if let Some(user_id) = user_id {
            if let Some(user_budget) = config.user_budgets.get(user_id) {
                if user_budget.enabled {
                    if !self.check_user_budget(user_budget, cost).await? {
                        return Ok(false);
                    }
                }
            }
        }

        // Check project budget
        if let Some(project_id) = project_id {
            if let Some(project_budget) = config.project_budgets.get(project_id) {
                if project_budget.enabled {
                    if !self.check_project_budget(project_budget, cost).await? {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Record spending and update budgets
    pub async fn record_spending(
        &self,
        provider: &Provider,
        model: &Model,
        cost: &CostBreakdown,
        user_id: Option<&str>,
        project_id: Option<&str>,
    ) -> TokenResult<()> {
        let now = Utc::now();
        
        // Update global spending
        self.update_spending("global", cost, now).await;

        // Update provider spending
        let provider_key = format!("provider:{}", provider.to_string());
        self.update_spending(&provider_key, cost, now).await;

        // Update user spending
        if let Some(user_id) = user_id {
            let user_key = format!("user:{}", user_id);
            self.update_spending(&user_key, cost, now).await;
        }

        // Update project spending
        if let Some(project_id) = project_id {
            let project_key = format!("project:{}", project_id);
            self.update_spending(&project_key, cost, now).await;
        }

        // Check for budget alerts
        self.check_and_send_budget_alerts(provider, user_id, project_id).await?;

        Ok(())
    }

    /// Record streaming cost in real-time
    #[cfg(feature = "streaming")]
    pub async fn record_streaming_cost(
        &self,
        metadata: &StreamMetadata,
        cost: &CostBreakdown,
        user_id: Option<&str>,
        project_id: Option<&str>,
    ) -> TokenResult<()> {
        let provider = self.parse_provider_from_string(&metadata.provider)?;
        let model = self.parse_model_from_string(&metadata.model)?;
        
        self.record_spending(&provider, &model, cost, user_id, project_id).await
    }

    /// Get current budget status for a scope
    pub async fn get_budget_status(&self, scope: &BudgetScope, period: &BudgetPeriod) -> TokenResult<BudgetStatus> {
        let config = self.config.read().await;
        let spending = self.current_spending.read().await;
        
        let key = self.scope_to_key(scope);
        let budget_spending = spending.get(&key).cloned().unwrap_or_default();
        
        let (current_spending, budget_limit) = match period {
            BudgetPeriod::Daily => {
                let limit = self.get_daily_limit(scope, &config);
                (budget_spending.daily_spending, limit)
            },
            BudgetPeriod::Monthly => {
                let limit = self.get_monthly_limit(scope, &config);
                (budget_spending.monthly_spending, limit)
            },
            BudgetPeriod::Weekly => {
                let limit = self.get_weekly_limit(scope, &config);
                (budget_spending.weekly_spending, limit)
            },
            BudgetPeriod::Yearly => {
                let limit = self.get_yearly_limit(scope, &config);
                (budget_spending.yearly_spending, limit)
            },
            BudgetPeriod::Custom { start: _, end: _ } => {
                // Custom period calculation would go here
                (Decimal::ZERO, Decimal::ZERO)
            },
        };

        let remaining_budget = (budget_limit - current_spending).max(Decimal::ZERO);
        let percentage_used = if budget_limit > Decimal::ZERO {
            (current_spending / budget_limit * Decimal::from(100)).to_f64().unwrap_or(0.0)
        } else {
            0.0
        };

        let status = self.determine_health_status(percentage_used);
        let projected_monthly_cost = self.calculate_projected_monthly_cost(&budget_spending);

        Ok(BudgetStatus {
            scope: scope.clone(),
            period: period.clone(),
            current_spending,
            budget_limit,
            remaining_budget,
            percentage_used,
            status,
            last_updated: Utc::now(),
            projected_monthly_cost,
        })
    }

    /// Get all recent budget alerts
    pub async fn get_recent_alerts(&self, since: Option<DateTime<Utc>>) -> TokenResult<Vec<BudgetAlert>> {
        let alerts = self.alerts.read().await;
        let cutoff = since.unwrap_or_else(|| Utc::now() - Duration::days(7));
        
        Ok(alerts.iter()
            .filter(|a| a.timestamp >= cutoff)
            .cloned()
            .collect())
    }

    /// Update budget configuration
    pub async fn update_config(&self, new_config: BudgetConfig) -> TokenResult<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }

    /// Reset budget spending for a scope
    pub async fn reset_budget(&self, scope: &BudgetScope, period: &BudgetPeriod) -> TokenResult<()> {
        let mut spending = self.current_spending.write().await;
        let key = self.scope_to_key(scope);
        
        if let Some(budget_spending) = spending.get_mut(&key) {
            let now = Utc::now();
            match period {
                BudgetPeriod::Daily => {
                    budget_spending.daily_spending = Decimal::ZERO;
                    budget_spending.last_reset_daily = now;
                },
                BudgetPeriod::Weekly => {
                    budget_spending.weekly_spending = Decimal::ZERO;
                    budget_spending.last_reset_weekly = now;
                },
                BudgetPeriod::Monthly => {
                    budget_spending.monthly_spending = Decimal::ZERO;
                    budget_spending.last_reset_monthly = now;
                },
                BudgetPeriod::Yearly => {
                    budget_spending.yearly_spending = Decimal::ZERO;
                    budget_spending.last_reset_yearly = now;
                },
                BudgetPeriod::Custom { .. } => {
                    // Custom reset logic would go here
                },
            }
        }
        
        Ok(())
    }

    // Helper methods

    async fn check_global_budget(&self, budget: &GlobalBudget, cost: &CostBreakdown) -> TokenResult<bool> {
        let spending = self.current_spending.read().await;
        let global_spending = spending.get("global").cloned().unwrap_or_default();
        
        // Check daily limit
        if let Some(daily_limit) = budget.daily_limit {
            if global_spending.daily_spending + cost.total_cost > daily_limit {
                return Ok(false);
            }
        }

        // Check monthly limit
        if let Some(monthly_limit) = budget.monthly_limit {
            if global_spending.monthly_spending + cost.total_cost > monthly_limit {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn check_provider_budget(&self, budget: &ProviderBudget, cost: &CostBreakdown) -> TokenResult<bool> {
        let spending = self.current_spending.read().await;
        let provider_key = format!("provider:{}", budget.provider.to_string());
        let provider_spending = spending.get(&provider_key).cloned().unwrap_or_default();
        
        if let Some(daily_limit) = budget.daily_limit {
            if provider_spending.daily_spending + cost.total_cost > daily_limit {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn check_user_budget(&self, budget: &UserBudget, cost: &CostBreakdown) -> TokenResult<bool> {
        let spending = self.current_spending.read().await;
        let user_key = format!("user:{}", budget.user_id);
        let user_spending = spending.get(&user_key).cloned().unwrap_or_default();
        
        if let Some(monthly_limit) = budget.monthly_limit {
            if user_spending.monthly_spending + cost.total_cost > monthly_limit {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn check_project_budget(&self, budget: &ProjectBudget, cost: &CostBreakdown) -> TokenResult<bool> {
        let spending = self.current_spending.read().await;
        let project_key = format!("project:{}", budget.project_id);
        let project_spending = spending.get(&project_key).cloned().unwrap_or_default();
        
        // Check against total project budget
        let total_project_cost = project_spending.daily_spending + project_spending.weekly_spending + 
                               project_spending.monthly_spending + project_spending.yearly_spending;
        
        if total_project_cost + cost.total_cost > budget.total_budget {
            return Ok(false);
        }

        Ok(true)
    }

    async fn update_spending(&self, key: &str, cost: &CostBreakdown, timestamp: DateTime<Utc>) {
        let mut spending = self.current_spending.write().await;
        
        let budget_spending = spending.entry(key.to_string()).or_insert_with(|| BudgetSpending {
            daily_spending: Decimal::ZERO,
            weekly_spending: Decimal::ZERO,
            monthly_spending: Decimal::ZERO,
            yearly_spending: Decimal::ZERO,
            last_reset_daily: timestamp,
            last_reset_weekly: timestamp,
            last_reset_monthly: timestamp,
            last_reset_yearly: timestamp,
        });

        // Reset periods if needed
        if self.should_reset_daily(budget_spending.last_reset_daily, timestamp) {
            budget_spending.daily_spending = Decimal::ZERO;
            budget_spending.last_reset_daily = timestamp;
        }

        if self.should_reset_weekly(budget_spending.last_reset_weekly, timestamp) {
            budget_spending.weekly_spending = Decimal::ZERO;
            budget_spending.last_reset_weekly = timestamp;
        }

        if self.should_reset_monthly(budget_spending.last_reset_monthly, timestamp) {
            budget_spending.monthly_spending = Decimal::ZERO;
            budget_spending.last_reset_monthly = timestamp;
        }

        if self.should_reset_yearly(budget_spending.last_reset_yearly, timestamp) {
            budget_spending.yearly_spending = Decimal::ZERO;
            budget_spending.last_reset_yearly = timestamp;
        }

        // Add current cost to all periods
        budget_spending.daily_spending += cost.total_cost;
        budget_spending.weekly_spending += cost.total_cost;
        budget_spending.monthly_spending += cost.total_cost;
        budget_spending.yearly_spending += cost.total_cost;
    }

    async fn check_and_send_budget_alerts(
        &self,
        provider: &Provider,
        user_id: Option<&str>,
        project_id: Option<&str>,
    ) -> TokenResult<()> {
        let config = self.config.read().await;
        
        if !config.alert_config.enabled {
            return Ok(());
        }

        // Check alerts for different scopes
        self.check_scope_alerts(&BudgetScope::Global, &config.alert_config).await?;
        self.check_scope_alerts(&BudgetScope::Provider(provider.clone()), &config.alert_config).await?;
        
        if let Some(user_id) = user_id {
            self.check_scope_alerts(&BudgetScope::User(user_id.to_string()), &config.alert_config).await?;
        }
        
        if let Some(project_id) = project_id {
            self.check_scope_alerts(&BudgetScope::Project(project_id.to_string()), &config.alert_config).await?;
        }

        Ok(())
    }

    async fn check_scope_alerts(&self, scope: &BudgetScope, alert_config: &BudgetAlertConfig) -> TokenResult<()> {
        let budget_status = self.get_budget_status(scope, &BudgetPeriod::Monthly).await?;
        
        for &threshold in &alert_config.warning_thresholds {
            let threshold_percentage = threshold * 100.0;
            
            if budget_status.percentage_used >= threshold_percentage {
                let alert_key = format!("{}:{}", self.scope_to_key(scope), threshold);
                
                // Check cooldown
                if self.is_in_cooldown(&alert_key, alert_config.cooldown_minutes).await {
                    continue;
                }

                let alert_type = if threshold >= 0.95 {
                    BudgetAlertType::Critical(threshold_percentage)
                } else {
                    BudgetAlertType::Warning(threshold_percentage)
                };

                self.send_budget_alert(scope, alert_type, &budget_status, alert_config).await?;
                
                // Update last alert time
                let mut last_alerts = self.last_alert_times.write().await;
                last_alerts.insert(alert_key, Utc::now());
            }
        }

        Ok(())
    }

    async fn send_budget_alert(
        &self,
        scope: &BudgetScope,
        alert_type: BudgetAlertType,
        status: &BudgetStatus,
        config: &BudgetAlertConfig,
    ) -> TokenResult<()> {
        let alert = BudgetAlert {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            scope: scope.clone(),
            alert_type,
            current_spending: status.current_spending,
            budget_limit: status.budget_limit,
            percentage_used: status.percentage_used,
            message: format!(
                "Budget alert for {:?}: {:.2}% used (${:.2} of ${:.2})",
                scope, status.percentage_used, status.current_spending, status.budget_limit
            ),
            acknowledged: false,
        };

        // Store the alert
        let mut alerts = self.alerts.write().await;
        alerts.push(alert.clone());

        // Send notifications
        for channel in &config.notification_channels {
            self.send_notification(channel, &alert).await?;
        }

        Ok(())
    }

    async fn send_notification(&self, channel: &NotificationChannel, alert: &BudgetAlert) -> TokenResult<()> {
        match channel {
            NotificationChannel::Log => {
                log::warn!("Budget Alert: {}", alert.message);
            },
            NotificationChannel::Email(email) => {
                log::info!("Would send email alert to: {}", email);
            },
            NotificationChannel::Webhook(url) => {
                log::info!("Would send webhook alert to: {}", url);
            },
            NotificationChannel::Slack(webhook) => {
                log::info!("Would send Slack alert: {}", webhook);
            },
            NotificationChannel::Discord(webhook) => {
                log::info!("Would send Discord alert: {}", webhook);
            },
        }
        Ok(())
    }

    // Utility methods

    fn scope_to_key(&self, scope: &BudgetScope) -> String {
        match scope {
            BudgetScope::Global => "global".to_string(),
            BudgetScope::Provider(p) => format!("provider:{}", p.to_string()),
            BudgetScope::User(u) => format!("user:{}", u),
            BudgetScope::Project(p) => format!("project:{}", p),
        }
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

    fn should_reset_daily(&self, last_reset: DateTime<Utc>, current: DateTime<Utc>) -> bool {
        last_reset.date_naive() != current.date_naive()
    }

    fn should_reset_weekly(&self, last_reset: DateTime<Utc>, current: DateTime<Utc>) -> bool {
        let days_diff = (current - last_reset).num_days();
        days_diff >= 7 || (current.weekday().num_days_from_monday() < last_reset.weekday().num_days_from_monday() && days_diff > 0)
    }

    fn should_reset_monthly(&self, last_reset: DateTime<Utc>, current: DateTime<Utc>) -> bool {
        last_reset.date_naive().year() != current.date_naive().year() ||
        last_reset.date_naive().month() != current.date_naive().month()
    }

    fn should_reset_yearly(&self, last_reset: DateTime<Utc>, current: DateTime<Utc>) -> bool {
        last_reset.date_naive().year() != current.date_naive().year()
    }

    fn determine_health_status(&self, percentage_used: f64) -> BudgetHealthStatus {
        match percentage_used {
            p if p >= 100.0 => BudgetHealthStatus::Depleted,
            p if p >= 95.0 => BudgetHealthStatus::Exceeded,
            p if p >= 80.0 => BudgetHealthStatus::Critical,
            p if p >= 50.0 => BudgetHealthStatus::Warning,
            _ => BudgetHealthStatus::Healthy,
        }
    }

    fn calculate_projected_monthly_cost(&self, spending: &BudgetSpending) -> Option<Decimal> {
        let now = Utc::now();
        let days_in_month = 30; // Simplified
        let days_elapsed = now.day() as f64;
        
        if days_elapsed > 0.0 {
            let daily_avg = spending.monthly_spending.to_f64()? / days_elapsed;
            let projected = daily_avg * days_in_month as f64;
            Decimal::try_from(projected).ok()
        } else {
            None
        }
    }

    async fn is_in_cooldown(&self, alert_key: &str, cooldown_minutes: u32) -> bool {
        let last_alerts = self.last_alert_times.read().await;
        if let Some(last_alert) = last_alerts.get(alert_key) {
            let cooldown_duration = Duration::minutes(cooldown_minutes as i64);
            return Utc::now() - *last_alert < cooldown_duration;
        }
        false
    }

    fn get_daily_limit(&self, scope: &BudgetScope, config: &BudgetConfig) -> Decimal {
        match scope {
            BudgetScope::Global => config.global_budget.daily_limit.unwrap_or(Decimal::ZERO),
            BudgetScope::Provider(p) => {
                config.provider_budgets.get(p)
                    .and_then(|b| b.daily_limit)
                    .unwrap_or(Decimal::ZERO)
            },
            BudgetScope::User(u) => {
                config.user_budgets.get(u)
                    .and_then(|b| b.daily_limit)
                    .unwrap_or(Decimal::ZERO)
            },
            BudgetScope::Project(_) => Decimal::ZERO, // Projects use total budget
        }
    }

    fn get_monthly_limit(&self, scope: &BudgetScope, config: &BudgetConfig) -> Decimal {
        match scope {
            BudgetScope::Global => config.global_budget.monthly_limit.unwrap_or(Decimal::ZERO),
            BudgetScope::Provider(p) => {
                config.provider_budgets.get(p)
                    .and_then(|b| b.monthly_limit)
                    .unwrap_or(Decimal::ZERO)
            },
            BudgetScope::User(u) => {
                config.user_budgets.get(u)
                    .and_then(|b| b.monthly_limit)
                    .unwrap_or(Decimal::ZERO)
            },
            BudgetScope::Project(p) => {
                config.project_budgets.get(p)
                    .and_then(|b| b.monthly_budget)
                    .unwrap_or(Decimal::ZERO)
            },
        }
    }

    fn get_weekly_limit(&self, _scope: &BudgetScope, config: &BudgetConfig) -> Decimal {
        config.global_budget.weekly_limit.unwrap_or(Decimal::ZERO)
    }

    fn get_yearly_limit(&self, _scope: &BudgetScope, config: &BudgetConfig) -> Decimal {
        config.global_budget.yearly_limit.unwrap_or(Decimal::ZERO)
    }
}

impl Default for BudgetSpending {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            daily_spending: Decimal::ZERO,
            weekly_spending: Decimal::ZERO,
            monthly_spending: Decimal::ZERO,
            yearly_spending: Decimal::ZERO,
            last_reset_daily: now,
            last_reset_weekly: now,
            last_reset_monthly: now,
            last_reset_yearly: now,
        }
    }
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            global_budget: GlobalBudget {
                daily_limit: Some(Decimal::from(50)), // $50 daily
                weekly_limit: Some(Decimal::from(300)), // $300 weekly
                monthly_limit: Some(Decimal::from(1000)), // $1000 monthly
                yearly_limit: Some(Decimal::from(12000)), // $12000 yearly
                enabled: true,
                reset_on_period_start: true,
            },
            provider_budgets: HashMap::new(),
            user_budgets: HashMap::new(),
            project_budgets: HashMap::new(),
            alert_config: BudgetAlertConfig {
                enabled: true,
                warning_thresholds: vec![0.5, 0.8, 0.95],
                notification_channels: vec![NotificationChannel::Log],
                alert_frequency: AlertFrequency::Immediate,
                cooldown_minutes: 30,
            },
        }
    }
}

