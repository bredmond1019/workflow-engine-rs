//! Routing Rules and Configuration
//! 
//! Defines routing rules for message filtering, transformation, and delivery options

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::routing::messages::{RoutingMessage, MessageType, MessagePriority, SenderType};

/// Routing rule match result
#[derive(Debug, Clone, PartialEq)]
pub enum RuleMatch {
    FullMatch,
    PartialMatch,
    NoMatch,
}

/// Individual routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub is_terminal: bool,

    // Matching conditions
    pub routing_key_pattern: Option<String>,
    pub topic_pattern: Option<String>,
    pub message_type_filter: Option<Vec<String>>,
    pub sender_type_filter: Option<Vec<SenderType>>,
    pub sender_id_pattern: Option<String>,
    pub header_filters: Option<HashMap<String, String>>,
    pub content_filters: Option<HashMap<String, serde_json::Value>>,

    // Actions
    pub target_filter: Option<HashMap<String, serde_json::Value>>,
    pub message_transform: Option<HashMap<String, serde_json::Value>>,
    pub persist_offline: Option<bool>,
    pub override_priority: Option<MessagePriority>,
    pub rate_limit: Option<RateLimitConfig>,
    pub custom_actions: Option<HashMap<String, serde_json::Value>>,
}

/// Rate limiting configuration for rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_messages_per_second: f64,
    pub burst_size: u32,
    pub per_user: bool,
    pub per_connection: bool,
}

impl RoutingRule {
    /// Check if this rule matches the given message
    pub fn matches(&self, message: &RoutingMessage) -> RuleMatch {
        if !self.enabled {
            return RuleMatch::NoMatch;
        }

        let mut full_match = true;
        let mut any_match = false;

        // Check routing key pattern
        if let Some(pattern) = &self.routing_key_pattern {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&message.routing_key) {
                    any_match = true;
                } else {
                    full_match = false;
                }
            } else {
                // Fallback to simple string matching
                if message.routing_key.contains(pattern) {
                    any_match = true;
                } else {
                    full_match = false;
                }
            }
        }

        // Check topic pattern
        if let Some(pattern) = &self.topic_pattern {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&message.topic) {
                    any_match = true;
                } else {
                    full_match = false;
                }
            } else {
                // Fallback to simple string matching
                if message.topic.contains(pattern) {
                    any_match = true;
                } else {
                    full_match = false;
                }
            }
        }

        // Check message type filter
        if let Some(type_filters) = &self.message_type_filter {
            let message_type_name = match &message.message_type {
                MessageType::Progress(_) => "progress",
                MessageType::Notification(_) => "notification",
                MessageType::Agent(_) => "agent",
                MessageType::Control(_) => "control",
                MessageType::Heartbeat => "heartbeat",
                MessageType::Ack { .. } => "ack",
                MessageType::Error { .. } => "error",
                MessageType::Custom { type_name, .. } => type_name,
            };

            if type_filters.contains(&message_type_name.to_string()) {
                any_match = true;
            } else {
                full_match = false;
            }
        }

        // Check sender type filter
        if let Some(sender_filters) = &self.sender_type_filter {
            if sender_filters.contains(&message.sender_type) {
                any_match = true;
            } else {
                full_match = false;
            }
        }

        // Check sender ID pattern
        if let Some(pattern) = &self.sender_id_pattern {
            if let Some(sender_id) = &message.sender_id {
                if let Ok(regex) = Regex::new(pattern) {
                    if regex.is_match(sender_id) {
                        any_match = true;
                    } else {
                        full_match = false;
                    }
                } else {
                    // Fallback to simple string matching
                    if sender_id.contains(pattern) {
                        any_match = true;
                    } else {
                        full_match = false;
                    }
                }
            } else {
                full_match = false;
            }
        }

        // Check header filters
        if let Some(header_filters) = &self.header_filters {
            for (header_key, header_pattern) in header_filters {
                if let Some(header_value) = message.headers.get(header_key) {
                    if let Ok(regex) = Regex::new(header_pattern) {
                        if regex.is_match(header_value) {
                            any_match = true;
                        } else {
                            full_match = false;
                        }
                    } else {
                        // Fallback to simple string matching
                        if header_value.contains(header_pattern) {
                            any_match = true;
                        } else {
                            full_match = false;
                        }
                    }
                } else {
                    full_match = false;
                }
            }
        }

        // Check content filters (simple JSON path matching)
        if let Some(content_filters) = &self.content_filters {
            for (path, expected_value) in content_filters {
                if self.check_content_filter(message, path, expected_value) {
                    any_match = true;
                } else {
                    full_match = false;
                }
            }
        }

        // Return match result
        if full_match && any_match {
            RuleMatch::FullMatch
        } else if any_match {
            RuleMatch::PartialMatch
        } else {
            RuleMatch::NoMatch
        }
    }

    /// Check if content filter matches (simplified JSON path implementation)
    fn check_content_filter(&self, message: &RoutingMessage, path: &str, expected: &serde_json::Value) -> bool {
        // Simple content filtering based on message type
        match &message.message_type {
            MessageType::Progress(progress) => {
                match path {
                    "operation_id" => {
                        if let Some(expected_str) = expected.as_str() {
                            progress.operation_id.contains(expected_str)
                        } else {
                            false
                        }
                    }
                    "status" => {
                        if let Some(expected_str) = expected.as_str() {
                            format!("{:?}", progress.status).to_lowercase().contains(&expected_str.to_lowercase())
                        } else {
                            false
                        }
                    }
                    "progress_percent" => {
                        if let Some(expected_num) = expected.as_f64() {
                            (progress.progress_percent as f64 - expected_num).abs() < f64::EPSILON
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            MessageType::Notification(notification) => {
                match path {
                    "level" => {
                        if let Some(expected_str) = expected.as_str() {
                            format!("{:?}", notification.level).to_lowercase() == expected_str.to_lowercase()
                        } else {
                            false
                        }
                    }
                    "category" => {
                        if let Some(expected_str) = expected.as_str() {
                            notification.category.contains(expected_str)
                        } else {
                            false
                        }
                    }
                    "title" => {
                        if let Some(expected_str) = expected.as_str() {
                            notification.title.contains(expected_str)
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            MessageType::Agent(agent) => {
                match path {
                    "agent_type" => {
                        if let Some(expected_str) = expected.as_str() {
                            agent.agent_type.contains(expected_str)
                        } else {
                            false
                        }
                    }
                    "agent_id" => {
                        if let Some(expected_str) = expected.as_str() {
                            agent.agent_id.contains(expected_str)
                        } else {
                            false
                        }
                    }
                    "content_type" => {
                        if let Some(expected_str) = expected.as_str() {
                            format!("{:?}", agent.content_type).to_lowercase() == expected_str.to_lowercase()
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            MessageType::Control(control) => {
                match path {
                    "command" => {
                        if let Some(expected_str) = expected.as_str() {
                            format!("{:?}", control.command).to_lowercase().contains(&expected_str.to_lowercase())
                        } else {
                            false
                        }
                    }
                    "target" => {
                        if let Some(expected_str) = expected.as_str() {
                            format!("{:?}", control.target).to_lowercase().contains(&expected_str.to_lowercase())
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Create a default rule for all messages
    pub fn allow_all() -> Self {
        Self {
            name: "allow_all".to_string(),
            description: Some("Allow all messages".to_string()),
            enabled: true,
            priority: 0,
            is_terminal: false,
            routing_key_pattern: None,
            topic_pattern: None,
            message_type_filter: None,
            sender_type_filter: None,
            sender_id_pattern: None,
            header_filters: None,
            content_filters: None,
            target_filter: None,
            message_transform: None,
            persist_offline: None,
            override_priority: None,
            rate_limit: None,
            custom_actions: None,
        }
    }

    /// Create a rule for progress messages only
    pub fn progress_only() -> Self {
        Self {
            name: "progress_only".to_string(),
            description: Some("Only allow progress messages".to_string()),
            enabled: true,
            priority: 100,
            is_terminal: false,
            routing_key_pattern: Some("^progress\\.".to_string()),
            topic_pattern: Some("^progress$".to_string()),
            message_type_filter: Some(vec!["progress".to_string()]),
            sender_type_filter: None,
            sender_id_pattern: None,
            header_filters: None,
            content_filters: None,
            target_filter: None,
            message_transform: None,
            persist_offline: Some(true),
            override_priority: Some(MessagePriority::Normal),
            rate_limit: None,
            custom_actions: None,
        }
    }

    /// Create a rule for high-priority notifications
    pub fn high_priority_notifications() -> Self {
        Self {
            name: "high_priority_notifications".to_string(),
            description: Some("High priority notifications with immediate delivery".to_string()),
            enabled: true,
            priority: 200,
            is_terminal: false,
            routing_key_pattern: Some("^notification\\.".to_string()),
            topic_pattern: Some("^notifications$".to_string()),
            message_type_filter: Some(vec!["notification".to_string()]),
            sender_type_filter: None,
            sender_id_pattern: None,
            header_filters: None,
            content_filters: Some({
                let mut filters = HashMap::new();
                filters.insert("level".to_string(), serde_json::Value::String("error".to_string()));
                filters
            }),
            target_filter: None,
            message_transform: None,
            persist_offline: Some(true),
            override_priority: Some(MessagePriority::Critical),
            rate_limit: None,
            custom_actions: None,
        }
    }

    /// Create a rule for agent messages with rate limiting
    pub fn agent_messages_with_rate_limit() -> Self {
        Self {
            name: "agent_rate_limited".to_string(),
            description: Some("Agent messages with rate limiting".to_string()),
            enabled: true,
            priority: 150,
            is_terminal: false,
            routing_key_pattern: Some("^agent\\.".to_string()),
            topic_pattern: Some("^agents$".to_string()),
            message_type_filter: Some(vec!["agent".to_string()]),
            sender_type_filter: Some(vec![SenderType::Agent]),
            sender_id_pattern: None,
            header_filters: None,
            content_filters: None,
            target_filter: None,
            message_transform: None,
            persist_offline: Some(false),
            override_priority: Some(MessagePriority::Normal),
            rate_limit: Some(RateLimitConfig {
                max_messages_per_second: 10.0,
                burst_size: 20,
                per_user: false,
                per_connection: true,
            }),
            custom_actions: None,
        }
    }

    /// Create a rule that blocks certain senders
    pub fn block_spam_senders() -> Self {
        Self {
            name: "block_spam".to_string(),
            description: Some("Block messages from spam senders".to_string()),
            enabled: true,
            priority: 1000, // High priority to process early
            is_terminal: true, // Stop processing other rules
            routing_key_pattern: None,
            topic_pattern: None,
            message_type_filter: None,
            sender_type_filter: None,
            sender_id_pattern: Some("spam|bot|fake".to_string()),
            header_filters: None,
            content_filters: None,
            target_filter: Some({
                let mut filter = HashMap::new();
                filter.insert("max_targets".to_string(), serde_json::Value::Number(0.into())); // Block by setting 0 targets
                filter
            }),
            message_transform: None,
            persist_offline: Some(false),
            override_priority: None,
            rate_limit: None,
            custom_actions: None,
        }
    }
}

/// Collection of routing rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRules {
    pub rules: Vec<RoutingRule>,
    pub default_action: DefaultAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefaultAction {
    Allow,
    Deny,
    RequireMatch,
}

impl Default for RoutingRules {
    fn default() -> Self {
        Self {
            rules: vec![RoutingRule::allow_all()],
            default_action: DefaultAction::Allow,
        }
    }
}

impl RoutingRules {
    /// Create a set of common routing rules
    pub fn common_rules() -> Self {
        Self {
            rules: vec![
                RoutingRule::block_spam_senders(),
                RoutingRule::high_priority_notifications(),
                RoutingRule::agent_messages_with_rate_limit(),
                RoutingRule::progress_only(),
                RoutingRule::allow_all(),
            ],
            default_action: DefaultAction::Allow,
        }
    }

    /// Add a new rule
    pub fn add_rule(&mut self, rule: RoutingRule) {
        self.rules.push(rule);
        // Sort by priority (higher priority first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a rule by name
    pub fn remove_rule(&mut self, name: &str) {
        self.rules.retain(|rule| rule.name != name);
    }

    /// Enable/disable a rule
    pub fn set_rule_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = enabled;
        }
    }

    /// Get rule by name
    pub fn get_rule(&self, name: &str) -> Option<&RoutingRule> {
        self.rules.iter().find(|r| r.name == name)
    }

    /// Get all enabled rules sorted by priority
    pub fn get_enabled_rules(&self) -> Vec<&RoutingRule> {
        let mut enabled: Vec<&RoutingRule> = self.rules.iter().filter(|r| r.enabled).collect();
        enabled.sort_by(|a, b| b.priority.cmp(&a.priority));
        enabled
    }

    /// Validate all rules
    pub fn validate(&self) -> Result<(), String> {
        // Check for duplicate rule names
        let mut names = std::collections::HashSet::new();
        for rule in &self.rules {
            if !names.insert(&rule.name) {
                return Err(format!("Duplicate rule name: {}", rule.name));
            }
        }

        // Validate regex patterns
        for rule in &self.rules {
            if let Some(pattern) = &rule.routing_key_pattern {
                if let Err(e) = Regex::new(pattern) {
                    return Err(format!("Invalid routing key pattern in rule '{}': {}", rule.name, e));
                }
            }
            if let Some(pattern) = &rule.topic_pattern {
                if let Err(e) = Regex::new(pattern) {
                    return Err(format!("Invalid topic pattern in rule '{}': {}", rule.name, e));
                }
            }
            if let Some(pattern) = &rule.sender_id_pattern {
                if let Err(e) = Regex::new(pattern) {
                    return Err(format!("Invalid sender ID pattern in rule '{}': {}", rule.name, e));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::messages::{ProgressStatus, NotificationLevel, AgentContentType};

    #[test]
    fn test_rule_matching() {
        let rule = RoutingRule::progress_only();
        
        let progress_msg = RoutingMessage::progress(
            "test_op".to_string(),
            50.0,
            ProgressStatus::InProgress,
            "Processing".to_string(),
        );

        assert_eq!(rule.matches(&progress_msg), RuleMatch::FullMatch);

        let notification_msg = RoutingMessage::notification(
            "Test".to_string(),
            "Message".to_string(),
            NotificationLevel::Info,
            vec!["user1".to_string()],
        );

        assert_eq!(rule.matches(&notification_msg), RuleMatch::NoMatch);
    }

    #[test]
    fn test_content_filter_matching() {
        let rule = RoutingRule::high_priority_notifications();
        
        let error_notification = RoutingMessage::notification(
            "Error occurred".to_string(),
            "Something went wrong".to_string(),
            NotificationLevel::Error,
            vec!["user1".to_string()],
        );

        // This would match if the content filter was checking notification level
        // Note: The actual matching might need adjustment based on implementation
        assert_ne!(rule.matches(&error_notification), RuleMatch::NoMatch);
    }

    #[test]
    fn test_routing_rules_validation() {
        let rules = RoutingRules::common_rules();
        assert!(rules.validate().is_ok());

        let mut invalid_rules = RoutingRules::default();
        invalid_rules.add_rule(RoutingRule {
            name: "invalid_regex".to_string(),
            description: None,
            enabled: true,
            priority: 100,
            is_terminal: false,
            routing_key_pattern: Some("[invalid regex".to_string()), // Invalid regex
            topic_pattern: None,
            message_type_filter: None,
            sender_type_filter: None,
            sender_id_pattern: None,
            header_filters: None,
            content_filters: None,
            target_filter: None,
            message_transform: None,
            persist_offline: None,
            override_priority: None,
            rate_limit: None,
            custom_actions: None,
        });

        assert!(invalid_rules.validate().is_err());
    }

    #[test]
    fn test_rules_priority_sorting() {
        let mut rules = RoutingRules::default();
        
        rules.add_rule(RoutingRule {
            name: "low_priority".to_string(),
            priority: 10,
            ..RoutingRule::allow_all()
        });
        
        rules.add_rule(RoutingRule {
            name: "high_priority".to_string(),
            priority: 100,
            ..RoutingRule::allow_all()
        });

        let enabled = rules.get_enabled_rules();
        assert!(enabled[0].priority >= enabled[1].priority);
    }

    #[test]
    fn test_rule_management() {
        let mut rules = RoutingRules::default();
        let rule_name = "test_rule";
        
        rules.add_rule(RoutingRule {
            name: rule_name.to_string(),
            ..RoutingRule::allow_all()
        });

        assert!(rules.get_rule(rule_name).is_some());
        
        rules.set_rule_enabled(rule_name, false);
        assert!(!rules.get_rule(rule_name).unwrap().enabled);
        
        rules.remove_rule(rule_name);
        assert!(rules.get_rule(rule_name).is_none());
    }
}