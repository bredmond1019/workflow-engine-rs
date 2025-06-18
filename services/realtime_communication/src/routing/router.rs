//! Message Router Implementation
//! 
//! Handles routing logic for different message types with topic-based routing,
//! filtering, and delivery orchestration.

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use dashmap::DashMap;

use crate::routing::messages::{RoutingMessage, MessageType, MessagePriority, ValidationResult};
use crate::routing::rules::{RoutingRule, RoutingRules, RuleMatch};
use crate::connection::{ConnectionManager, ConnectionInfo};

/// Router metrics
#[derive(Debug, Default)]
pub struct RouterMetrics {
    pub messages_routed: Arc<RwLock<u64>>,
    pub messages_delivered: Arc<RwLock<u64>>,
    pub messages_failed: Arc<RwLock<u64>>,
    pub messages_dropped: Arc<RwLock<u64>>,
    pub routing_errors: Arc<RwLock<u64>>,
    pub route_cache_hits: Arc<RwLock<u64>>,
    pub route_cache_misses: Arc<RwLock<u64>>,
}

impl RouterMetrics {
    pub async fn increment_routed(&self) {
        *self.messages_routed.write().await += 1;
    }

    pub async fn increment_delivered(&self) {
        *self.messages_delivered.write().await += 1;
    }

    pub async fn increment_failed(&self) {
        *self.messages_failed.write().await += 1;
    }

    pub async fn increment_dropped(&self) {
        *self.messages_dropped.write().await += 1;
    }

    pub async fn increment_errors(&self) {
        *self.routing_errors.write().await += 1;
    }

    pub async fn increment_cache_hits(&self) {
        *self.route_cache_hits.write().await += 1;
    }

    pub async fn increment_cache_misses(&self) {
        *self.route_cache_misses.write().await += 1;
    }
}

/// Routing target representation
#[derive(Debug, Clone)]
pub struct RoutingTarget {
    pub connection_id: Uuid,
    pub user_id: Option<String>,
    pub subscriptions: Vec<String>,
    pub priority: MessagePriority,
}

/// Routing decision outcome
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub targets: Vec<RoutingTarget>,
    pub matched_rules: Vec<String>,
    pub should_persist: bool,
    pub delivery_priority: MessagePriority,
    pub transformed_message: Option<RoutingMessage>,
}

/// Message router trait for different routing strategies
#[async_trait::async_trait]
pub trait MessageRouter: Send + Sync {
    /// Route a message to appropriate targets
    async fn route(&self, message: RoutingMessage) -> Result<RoutingDecision, RouterError>;

    /// Get routing statistics
    async fn get_metrics(&self) -> RouterMetrics;

    /// Update routing rules
    async fn update_rules(&self, rules: RoutingRules) -> Result<(), RouterError>;
}

/// Topic-based message router implementation
pub struct TopicMessageRouter {
    connection_manager: Arc<ConnectionManager>,
    routing_rules: Arc<RwLock<RoutingRules>>,
    metrics: RouterMetrics,
    route_cache: Arc<DashMap<String, Vec<RoutingTarget>>>,
    config: RouterConfig,
}

/// Router configuration
#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
    pub max_cache_entries: usize,
    pub max_targets_per_message: usize,
    pub enable_message_transformation: bool,
    pub drop_invalid_messages: bool,
    pub max_message_size_bytes: usize,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_cache_entries: 10_000,
            max_targets_per_message: 1000,
            enable_message_transformation: true,
            drop_invalid_messages: false,
            max_message_size_bytes: 1024 * 1024, // 1MB
        }
    }
}

impl TopicMessageRouter {
    /// Create a new topic-based message router
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        routing_rules: RoutingRules,
        config: RouterConfig,
    ) -> Self {
        Self {
            connection_manager,
            routing_rules: Arc::new(RwLock::new(routing_rules)),
            metrics: RouterMetrics::default(),
            route_cache: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Find targets for a routing key using caching
    async fn find_routing_targets(&self, routing_key: &str, topic: &str) -> Vec<RoutingTarget> {
        let cache_key = format!("{}:{}", routing_key, topic);

        // Check cache first
        if self.config.enable_caching {
            if let Some(cached_targets) = self.route_cache.get(&cache_key) {
                self.metrics.increment_cache_hits().await;
                return cached_targets.clone();
            }
            self.metrics.increment_cache_misses().await;
        }

        // Get all connections and filter by topic subscriptions
        let all_connections = self.connection_manager.get_all_connections().await;
        let mut targets = Vec::new();

        for connection in all_connections {
            if connection.is_subscribed_to(topic) {
                targets.push(RoutingTarget {
                    connection_id: connection.id,
                    user_id: connection.user_id.clone(),
                    subscriptions: connection.subscriptions.clone(),
                    priority: MessagePriority::Normal,
                });
            }
        }

        // Cache the result
        if self.config.enable_caching && targets.len() <= self.config.max_targets_per_message {
            // Implement simple cache size management
            if self.route_cache.len() >= self.config.max_cache_entries {
                // Remove random entries (in production, use LRU)
                let keys_to_remove: Vec<String> = self.route_cache
                    .iter()
                    .take(self.config.max_cache_entries / 10)
                    .map(|entry| entry.key().clone())
                    .collect();
                
                for key in keys_to_remove {
                    self.route_cache.remove(&key);
                }
            }

            self.route_cache.insert(cache_key, targets.clone());
        }

        targets
    }

    /// Apply routing rules to filter and transform targets
    async fn apply_routing_rules(
        &self,
        message: &RoutingMessage,
        targets: Vec<RoutingTarget>,
    ) -> Result<RoutingDecision, RouterError> {
        let rules = self.routing_rules.read().await;
        let mut matched_rules = Vec::new();
        let mut filtered_targets = targets;
        let mut should_persist = message.delivery_options.persist_offline;
        let mut delivery_priority = message.delivery_options.priority.clone();
        let mut transformed_message = None;

        for rule in &rules.rules {
            match rule.matches(message) {
                RuleMatch::FullMatch => {
                    matched_rules.push(rule.name.clone());
                    
                    // Apply rule actions
                    if let Some(filter) = &rule.target_filter {
                        filtered_targets = self.apply_target_filter(filtered_targets, filter);
                    }

                    if let Some(transform) = &rule.message_transform {
                        if self.config.enable_message_transformation {
                            transformed_message = Some(self.apply_message_transform(message, transform));
                        }
                    }

                    if let Some(persist) = rule.persist_offline {
                        should_persist = persist;
                    }

                    if let Some(priority) = &rule.override_priority {
                        delivery_priority = priority.clone();
                    }

                    // If rule is marked as terminal, stop processing
                    if rule.is_terminal {
                        break;
                    }
                }
                RuleMatch::PartialMatch => {
                    debug!("Partial match for rule: {}", rule.name);
                }
                RuleMatch::NoMatch => {
                    // Continue to next rule
                }
            }
        }

        // Apply hard limits
        if filtered_targets.len() > self.config.max_targets_per_message {
            warn!(
                "Too many targets ({}) for message {}, limiting to {}",
                filtered_targets.len(),
                message.id,
                self.config.max_targets_per_message
            );
            filtered_targets.truncate(self.config.max_targets_per_message);
        }

        Ok(RoutingDecision {
            targets: filtered_targets,
            matched_rules,
            should_persist,
            delivery_priority,
            transformed_message,
        })
    }

    /// Apply target filter based on rule configuration
    fn apply_target_filter(&self, targets: Vec<RoutingTarget>, filter: &HashMap<String, serde_json::Value>) -> Vec<RoutingTarget> {
        let mut filtered = targets;

        // Apply user_id filter
        if let Some(user_ids) = filter.get("user_ids") {
            if let Some(user_ids_array) = user_ids.as_array() {
                let allowed_users: Vec<String> = user_ids_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                
                filtered = filtered
                    .into_iter()
                    .filter(|target| {
                        target.user_id.as_ref()
                            .map(|uid| allowed_users.contains(uid))
                            .unwrap_or(false)
                    })
                    .collect();
            }
        }

        // Apply subscription filter
        if let Some(required_subs) = filter.get("required_subscriptions") {
            if let Some(required_subs_array) = required_subs.as_array() {
                let required: Vec<String> = required_subs_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                
                filtered = filtered
                    .into_iter()
                    .filter(|target| {
                        required.iter().all(|req_sub| target.subscriptions.contains(req_sub))
                    })
                    .collect();
            }
        }

        // Apply max_targets limit
        if let Some(max_targets) = filter.get("max_targets") {
            if let Some(max) = max_targets.as_u64() {
                filtered.truncate(max as usize);
            }
        }

        filtered
    }

    /// Apply message transformation based on rule configuration
    fn apply_message_transform(&self, message: &RoutingMessage, transform: &HashMap<String, serde_json::Value>) -> RoutingMessage {
        let mut transformed = message.clone();

        // Transform routing key
        if let Some(new_routing_key) = transform.get("routing_key") {
            if let Some(key) = new_routing_key.as_str() {
                transformed.routing_key = key.to_string();
            }
        }

        // Transform topic
        if let Some(new_topic) = transform.get("topic") {
            if let Some(topic) = new_topic.as_str() {
                transformed.topic = topic.to_string();
            }
        }

        // Add headers
        if let Some(add_headers) = transform.get("add_headers") {
            if let Some(headers_obj) = add_headers.as_object() {
                for (key, value) in headers_obj {
                    if let Some(value_str) = value.as_str() {
                        transformed.headers.insert(key.clone(), value_str.to_string());
                    }
                }
            }
        }

        // Set priority
        if let Some(priority_val) = transform.get("priority") {
            if let Some(priority_str) = priority_val.as_str() {
                match priority_str {
                    "low" => transformed.delivery_options.priority = MessagePriority::Low,
                    "normal" => transformed.delivery_options.priority = MessagePriority::Normal,
                    "high" => transformed.delivery_options.priority = MessagePriority::High,
                    "critical" => transformed.delivery_options.priority = MessagePriority::Critical,
                    _ => {}
                }
            }
        }

        transformed
    }

    /// Add explicit targets based on message configuration
    async fn add_explicit_targets(&self, mut targets: Vec<RoutingTarget>, message: &RoutingMessage) -> Vec<RoutingTarget> {
        // Add targets based on target_users
        for user_id in &message.target_users {
            let user_connections = self.connection_manager.get_user_connections(user_id).await;
            for connection_id in user_connections {
                if let Some(connection) = self.connection_manager.get_connection(&connection_id).await {
                    targets.push(RoutingTarget {
                        connection_id,
                        user_id: Some(user_id.clone()),
                        subscriptions: connection.subscriptions,
                        priority: message.delivery_options.priority.clone(),
                    });
                }
            }
        }

        // Add targets based on target_connections
        for connection_id in &message.target_connections {
            if let Some(connection) = self.connection_manager.get_connection(connection_id).await {
                targets.push(RoutingTarget {
                    connection_id: *connection_id,
                    user_id: connection.user_id,
                    subscriptions: connection.subscriptions,
                    priority: message.delivery_options.priority.clone(),
                });
            }
        }

        // Remove duplicates
        targets.sort_by(|a, b| a.connection_id.cmp(&b.connection_id));
        targets.dedup_by(|a, b| a.connection_id == b.connection_id);

        targets
    }
}

#[async_trait::async_trait]
impl MessageRouter for TopicMessageRouter {
    async fn route(&self, mut message: RoutingMessage) -> Result<RoutingDecision, RouterError> {
        self.metrics.increment_routed().await;

        // Validate message
        match message.validate() {
            ValidationResult::Valid => {}
            ValidationResult::Invalid(errors) => {
                if self.config.drop_invalid_messages {
                    self.metrics.increment_dropped().await;
                    return Err(RouterError::InvalidMessage(
                        errors.into_iter().map(|e| e.message).collect::<Vec<_>>().join(", ")
                    ));
                } else {
                    warn!("Invalid message {}: {:?}", message.id, errors);
                }
            }
        }

        // Check message size
        if message.size_estimate() > self.config.max_message_size_bytes {
            self.metrics.increment_dropped().await;
            return Err(RouterError::MessageTooLarge(message.size_estimate()));
        }

        // Check if message has expired
        if message.is_expired() {
            self.metrics.increment_dropped().await;
            return Err(RouterError::MessageExpired);
        }

        // Sanitize message
        message.sanitize();

        // Find initial routing targets based on topic
        let mut targets = self.find_routing_targets(&message.routing_key, &message.topic).await;

        // Add explicit targets from message
        targets = self.add_explicit_targets(targets, &message).await;

        // Apply routing rules
        let decision = self.apply_routing_rules(&message, targets).await?;

        info!(
            "Routed message {} to {} targets using rules: {:?}",
            message.id,
            decision.targets.len(),
            decision.matched_rules
        );

        Ok(decision)
    }

    async fn get_metrics(&self) -> RouterMetrics {
        RouterMetrics {
            messages_routed: self.metrics.messages_routed.clone(),
            messages_delivered: self.metrics.messages_delivered.clone(),
            messages_failed: self.metrics.messages_failed.clone(),
            messages_dropped: self.metrics.messages_dropped.clone(),
            routing_errors: self.metrics.routing_errors.clone(),
            route_cache_hits: self.metrics.route_cache_hits.clone(),
            route_cache_misses: self.metrics.route_cache_misses.clone(),
        }
    }

    async fn update_rules(&self, rules: RoutingRules) -> Result<(), RouterError> {
        *self.routing_rules.write().await = rules;
        
        // Clear cache when rules change
        if self.config.enable_caching {
            self.route_cache.clear();
        }

        info!("Updated routing rules and cleared cache");
        Ok(())
    }
}

/// Router errors
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    #[error("Message too large: {0} bytes")]
    MessageTooLarge(usize),
    #[error("Message expired")]
    MessageExpired,
    #[error("No targets found")]
    NoTargets,
    #[error("Connection manager error: {0}")]
    ConnectionManager(String),
    #[error("Routing rule error: {0}")]
    RoutingRule(String),
    #[error("Router error: {0}")]
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::messages::{ProgressStatus, NotificationLevel};
    use crate::connection::ConnectionManager;

    #[tokio::test]
    async fn test_topic_router_creation() {
        let connection_manager = Arc::new(ConnectionManager::new(100));
        let rules = RoutingRules::default();
        let config = RouterConfig::default();
        
        let router = TopicMessageRouter::new(connection_manager, rules, config);
        let metrics = router.get_metrics().await;
        
        assert_eq!(*metrics.messages_routed.read().await, 0);
    }

    #[tokio::test]
    async fn test_message_validation_and_routing() {
        let connection_manager = Arc::new(ConnectionManager::new(100));
        let rules = RoutingRules::default();
        let config = RouterConfig::default();
        
        let router = TopicMessageRouter::new(connection_manager, rules, config);
        
        let message = RoutingMessage::progress(
            "test_op".to_string(),
            50.0,
            ProgressStatus::InProgress,
            "Processing".to_string(),
        );

        let result = router.route(message).await;
        assert!(result.is_ok());
        
        let decision = result.unwrap();
        assert_eq!(decision.targets.len(), 0); // No connections subscribed
    }

    #[tokio::test]
    async fn test_invalid_message_handling() {
        let connection_manager = Arc::new(ConnectionManager::new(100));
        let rules = RoutingRules::default();
        let mut config = RouterConfig::default();
        config.drop_invalid_messages = true;
        
        let router = TopicMessageRouter::new(connection_manager, rules, config);
        
        let mut message = RoutingMessage::progress(
            "test_op".to_string(),
            150.0, // Invalid progress
            ProgressStatus::InProgress,
            "Processing".to_string(),
        );

        let result = router.route(message).await;
        assert!(result.is_err());
        
        if let Err(RouterError::InvalidMessage(_)) = result {
            // Expected
        } else {
            panic!("Expected InvalidMessage error");
        }
    }

    #[tokio::test]
    async fn test_message_size_limit() {
        let connection_manager = Arc::new(ConnectionManager::new(100));
        let rules = RoutingRules::default();
        let mut config = RouterConfig::default();
        config.max_message_size_bytes = 100; // Very small limit
        
        let router = TopicMessageRouter::new(connection_manager, rules, config);
        
        let message = RoutingMessage::notification(
            "Very long title that exceeds the message size limit".to_string(),
            "Very long message content that definitely exceeds the size limit".to_string(),
            NotificationLevel::Info,
            vec!["user1".to_string()],
        );

        let result = router.route(message).await;
        assert!(result.is_err());
        
        if let Err(RouterError::MessageTooLarge(_)) = result {
            // Expected
        } else {
            panic!("Expected MessageTooLarge error");
        }
    }
}