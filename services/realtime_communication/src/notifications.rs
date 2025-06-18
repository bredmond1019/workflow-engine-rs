//! Notification Delivery System
//! 
//! Real-time notification delivery with preferences, queuing, retry mechanisms,
//! and multiple delivery channels for the communication system.

use actix::{Actor, Addr, Context, Handler, AsyncContext};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::actors::messages::*;
use crate::actors::router::RouterActor;

/// Notification delivery actor
pub struct NotificationDeliveryActor {
    /// Router for sending notifications
    router_addr: Option<Addr<RouterActor>>,
    
    /// User notification preferences
    user_preferences: HashMap<String, NotificationPreferences>,
    
    /// Pending notifications queue
    notification_queue: VecDeque<QueuedNotification>,
    
    /// Failed notifications for retry
    retry_queue: VecDeque<QueuedNotification>,
    
    /// Delivery metrics
    metrics: NotificationMetrics,
    
    /// Configuration
    config: NotificationConfig,
    
    /// Redis client for distributed queuing
    redis_client: Option<Arc<redis::Client>>,
}

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: String,
    pub enable_push: bool,
    pub enable_email: bool,
    pub enable_sms: bool,
    pub enable_in_app: bool,
    pub quiet_hours_start: Option<String>, // Format: "22:00"
    pub quiet_hours_end: Option<String>,   // Format: "08:00"
    pub timezone: Option<String>,
    pub message_types: Vec<String>, // Types of messages to receive
    pub topic_preferences: HashMap<String, bool>, // Per-topic preferences
    pub delivery_delay_seconds: Option<u32>, // Delay for batching
    pub max_notifications_per_hour: Option<u32>,
    pub updated_at: DateTime<Utc>,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            enable_push: true,
            enable_email: true,
            enable_sms: false,
            enable_in_app: true,
            quiet_hours_start: None,
            quiet_hours_end: None,
            timezone: None,
            message_types: vec![
                "direct_message".to_string(),
                "mention".to_string(),
                "system_alert".to_string(),
            ],
            topic_preferences: HashMap::new(),
            delivery_delay_seconds: Some(5), // 5 second batching delay
            max_notifications_per_hour: Some(100),
            updated_at: Utc::now(),
        }
    }
}

/// Queued notification for delivery
#[derive(Debug, Clone)]
struct QueuedNotification {
    id: String,
    user_id: String,
    notification: ServerMessage,
    priority: MessagePriority,
    channels: Vec<DeliveryChannel>,
    scheduled_for: DateTime<Utc>,
    max_retries: u32,
    retry_count: u32,
    created_at: DateTime<Utc>,
    metadata: HashMap<String, String>,
}

/// Delivery channels
#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryChannel {
    InApp,
    Push,
    Email,
    SMS,
    Webhook,
}

/// Notification metrics
#[derive(Debug, Default)]
struct NotificationMetrics {
    total_notifications: u64,
    delivered_notifications: u64,
    failed_notifications: u64,
    retried_notifications: u64,
    queued_notifications: usize,
    average_delivery_time_ms: f64,
    channel_stats: HashMap<DeliveryChannel, ChannelStats>,
}

#[derive(Debug, Default)]
struct ChannelStats {
    sent: u64,
    delivered: u64,
    failed: u64,
    average_delivery_time_ms: f64,
}

/// Notification configuration
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub max_queue_size: usize,
    pub max_retry_attempts: u32,
    pub retry_delay_seconds: u64,
    pub batch_processing_interval: Duration,
    pub delivery_timeout_seconds: u64,
    pub enable_quiet_hours: bool,
    pub enable_rate_limiting: bool,
    pub enable_redis_queue: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10_000,
            max_retry_attempts: 3,
            retry_delay_seconds: 30,
            batch_processing_interval: Duration::from_secs(5),
            delivery_timeout_seconds: 30,
            enable_quiet_hours: true,
            enable_rate_limiting: true,
            enable_redis_queue: false,
        }
    }
}

impl NotificationDeliveryActor {
    pub fn new(
        config: NotificationConfig,
        redis_client: Option<Arc<redis::Client>>,
    ) -> Self {
        Self {
            router_addr: None,
            user_preferences: HashMap::new(),
            notification_queue: VecDeque::new(),
            retry_queue: VecDeque::new(),
            metrics: NotificationMetrics::default(),
            config,
            redis_client,
        }
    }

    /// Set router address
    pub fn set_router(&mut self, router_addr: Addr<RouterActor>) {
        self.router_addr = Some(router_addr);
        info!("Router address set in notification delivery actor");
    }

    /// Queue notification for delivery
    fn queue_notification(
        &mut self,
        user_id: String,
        notification: ServerMessage,
        persistence_required: bool,
        priority: MessagePriority,
    ) -> Result<String, String> {
        // Check queue size limit
        if self.notification_queue.len() >= self.config.max_queue_size {
            warn!("Notification queue full, dropping notification for user: {}", user_id);
            self.metrics.failed_notifications += 1;
            return Err("Queue full".to_string());
        }

        // Get user preferences
        let preferences = self.user_preferences
            .get(&user_id)
            .cloned()
            .unwrap_or_default();

        // Check if user wants this type of notification
        if !self.should_deliver_notification(&user_id, &notification, &preferences) {
            debug!("Notification filtered by user preferences: {}", user_id);
            return Ok("filtered".to_string());
        }

        // Determine delivery channels
        let channels = self.get_delivery_channels(&preferences, &notification);
        if channels.is_empty() {
            debug!("No delivery channels available for user: {}", user_id);
            return Ok("no_channels".to_string());
        }

        // Calculate delivery time (consider quiet hours and batching)
        let scheduled_for = self.calculate_delivery_time(&preferences, priority.clone());

        let notification_id = Uuid::new_v4().to_string();
        let queued_notification = QueuedNotification {
            id: notification_id.clone(),
            user_id: user_id.clone(),
            notification,
            priority,
            channels,
            scheduled_for,
            max_retries: self.config.max_retry_attempts,
            retry_count: 0,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        // Insert based on priority and scheduled time
        let insert_index = self.notification_queue
            .iter()
            .position(|n| {
                n.priority < queued_notification.priority || 
                (n.priority == queued_notification.priority && n.scheduled_for > queued_notification.scheduled_for)
            })
            .unwrap_or(self.notification_queue.len());

        self.notification_queue.insert(insert_index, queued_notification);
        self.metrics.total_notifications += 1;
        self.metrics.queued_notifications = self.notification_queue.len();

        info!("Notification queued: id={}, user_id={}, scheduled_for={}", 
              notification_id, user_id, scheduled_for);

        Ok(notification_id)
    }

    /// Check if notification should be delivered based on preferences
    fn should_deliver_notification(
        &self,
        user_id: &str,
        notification: &ServerMessage,
        preferences: &NotificationPreferences,
    ) -> bool {
        // Check rate limiting
        if self.config.enable_rate_limiting {
            if let Some(max_per_hour) = preferences.max_notifications_per_hour {
                // Implementation would check recent delivery count
                // For now, always allow
            }
        }

        // Check message type preferences
        let message_type = match notification {
            ServerMessage::MessageReceived { .. } => "direct_message",
            ServerMessage::TopicMessageReceived { .. } => "topic_message",
            ServerMessage::BroadcastReceived { .. } => "broadcast",
            ServerMessage::Notification { .. } => "notification",
            ServerMessage::SystemMessage { .. } => "system",
            _ => "other",
        };

        if !preferences.message_types.contains(&message_type.to_string()) {
            return false;
        }

        // Check quiet hours
        if self.config.enable_quiet_hours {
            if let (Some(start), Some(end)) = (&preferences.quiet_hours_start, &preferences.quiet_hours_end) {
                let now = Utc::now();
                // Simplified quiet hours check (would need proper timezone handling)
                let current_hour = now.hour();
                if let (Ok(start_hour), Ok(end_hour)) = (
                    start.split(':').next().unwrap_or("0").parse::<u32>(),
                    end.split(':').next().unwrap_or("0").parse::<u32>()
                ) {
                    if start_hour <= end_hour {
                        if current_hour >= start_hour && current_hour < end_hour {
                            return false; // In quiet hours
                        }
                    } else {
                        // Quiet hours span midnight
                        if current_hour >= start_hour || current_hour < end_hour {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    /// Get delivery channels for notification
    fn get_delivery_channels(
        &self,
        preferences: &NotificationPreferences,
        notification: &ServerMessage,
    ) -> Vec<DeliveryChannel> {
        let mut channels = Vec::new();

        // Always try in-app first if enabled
        if preferences.enable_in_app {
            channels.push(DeliveryChannel::InApp);
        }

        // Add other channels based on notification type and preferences
        match notification {
            ServerMessage::MessageReceived { .. } => {
                if preferences.enable_push {
                    channels.push(DeliveryChannel::Push);
                }
            }
            ServerMessage::Notification { level, .. } => {
                match level {
                    NotificationLevel::Error => {
                        if preferences.enable_push {
                            channels.push(DeliveryChannel::Push);
                        }
                        if preferences.enable_email {
                            channels.push(DeliveryChannel::Email);
                        }
                    }
                    NotificationLevel::Warning => {
                        if preferences.enable_push {
                            channels.push(DeliveryChannel::Push);
                        }
                    }
                    _ => {
                        // Info and success notifications only via in-app
                    }
                }
            }
            ServerMessage::SystemMessage { level, .. } => {
                if level == "critical" {
                    if preferences.enable_push {
                        channels.push(DeliveryChannel::Push);
                    }
                    if preferences.enable_email {
                        channels.push(DeliveryChannel::Email);
                    }
                    if preferences.enable_sms {
                        channels.push(DeliveryChannel::SMS);
                    }
                }
            }
            _ => {
                // Default to in-app only for other message types
            }
        }

        channels
    }

    /// Calculate when notification should be delivered
    fn calculate_delivery_time(
        &self,
        preferences: &NotificationPreferences,
        priority: MessagePriority,
    ) -> DateTime<Utc> {
        let now = Utc::now();

        // Critical messages are delivered immediately
        if priority == MessagePriority::Critical {
            return now;
        }

        // Apply batching delay if configured
        if let Some(delay_seconds) = preferences.delivery_delay_seconds {
            if priority == MessagePriority::Normal || priority == MessagePriority::Low {
                return now + chrono::Duration::seconds(delay_seconds as i64);
            }
        }

        now
    }

    /// Process notification queue
    fn process_notification_queue(&mut self, ctx: &mut Context<Self>) {
        let now = Utc::now();
        let mut processed = 0;
        let batch_size = 10; // Process in batches

        while processed < batch_size && !self.notification_queue.is_empty() {
            if let Some(mut notification) = self.notification_queue.front().cloned() {
                if notification.scheduled_for <= now {
                    // Remove from queue
                    self.notification_queue.pop_front();
                    self.metrics.queued_notifications = self.notification_queue.len();

                    // Attempt delivery
                    let delivery_start = Instant::now();
                    if let Err(e) = self.deliver_notification(&mut notification) {
                        error!("Notification delivery failed: {}", e);
                        self.handle_delivery_failure(notification);
                    } else {
                        self.metrics.delivered_notifications += 1;
                        
                        // Update delivery time metrics
                        let delivery_time = delivery_start.elapsed().as_millis() as f64;
                        self.metrics.average_delivery_time_ms = 
                            (self.metrics.average_delivery_time_ms + delivery_time) / 2.0;
                    }

                    processed += 1;
                } else {
                    // No more notifications ready for delivery
                    break;
                }
            } else {
                break;
            }
        }

        // Process retry queue
        self.process_retry_queue();

        debug!("Processed {} notifications, queue size: {}, retry queue size: {}", 
               processed, self.notification_queue.len(), self.retry_queue.len());
    }

    /// Deliver notification through available channels
    fn deliver_notification(&mut self, notification: &mut QueuedNotification) -> Result<(), String> {
        let mut delivery_success = false;

        for channel in &notification.channels {
            match self.deliver_via_channel(notification, channel) {
                Ok(()) => {
                    delivery_success = true;
                    self.update_channel_stats(channel, true);
                    debug!("Notification delivered via {:?}: {}", channel, notification.id);
                }
                Err(e) => {
                    warn!("Failed to deliver via {:?}: {}", channel, e);
                    self.update_channel_stats(channel, false);
                }
            }
        }

        if delivery_success {
            Ok(())
        } else {
            Err("All delivery channels failed".to_string())
        }
    }

    /// Deliver notification via specific channel
    fn deliver_via_channel(
        &mut self,
        notification: &QueuedNotification,
        channel: &DeliveryChannel,
    ) -> Result<(), String> {
        match channel {
            DeliveryChannel::InApp => {
                // Deliver via WebSocket (router)
                if let Some(router_addr) = &self.router_addr {
                    let session_message = SessionMessage {
                        message: notification.notification.clone(),
                        priority: notification.priority.clone(),
                    };
                    
                    // We'd need a way to route to specific user
                    // For now, just log that we would deliver
                    debug!("Would deliver in-app notification to user: {}", notification.user_id);
                    Ok(())
                } else {
                    Err("Router not available".to_string())
                }
            }
            DeliveryChannel::Push => {
                // Implement push notification delivery
                self.deliver_push_notification(notification)
            }
            DeliveryChannel::Email => {
                // Implement email delivery
                self.deliver_email_notification(notification)
            }
            DeliveryChannel::SMS => {
                // Implement SMS delivery
                self.deliver_sms_notification(notification)
            }
            DeliveryChannel::Webhook => {
                // Implement webhook delivery
                self.deliver_webhook_notification(notification)
            }
        }
    }

    /// Deliver push notification
    fn deliver_push_notification(&self, notification: &QueuedNotification) -> Result<(), String> {
        // Implementation would integrate with push service (FCM, APNs, etc.)
        debug!("Would send push notification to user: {}", notification.user_id);
        Ok(())
    }

    /// Deliver email notification
    fn deliver_email_notification(&self, notification: &QueuedNotification) -> Result<(), String> {
        // Implementation would integrate with email service (SMTP, SES, etc.)
        debug!("Would send email notification to user: {}", notification.user_id);
        Ok(())
    }

    /// Deliver SMS notification
    fn deliver_sms_notification(&self, notification: &QueuedNotification) -> Result<(), String> {
        // Implementation would integrate with SMS service (Twilio, etc.)
        debug!("Would send SMS notification to user: {}", notification.user_id);
        Ok(())
    }

    /// Deliver webhook notification
    fn deliver_webhook_notification(&self, notification: &QueuedNotification) -> Result<(), String> {
        // Implementation would make HTTP POST to user's webhook URL
        debug!("Would send webhook notification to user: {}", notification.user_id);
        Ok(())
    }

    /// Handle delivery failure
    fn handle_delivery_failure(&mut self, mut notification: QueuedNotification) {
        notification.retry_count += 1;

        if notification.retry_count <= notification.max_retries {
            // Schedule for retry
            notification.scheduled_for = Utc::now() + 
                chrono::Duration::seconds(self.config.retry_delay_seconds as i64 * notification.retry_count as i64);
            
            self.retry_queue.push_back(notification);
            self.metrics.retried_notifications += 1;
            
            debug!("Notification scheduled for retry (attempt {}): {}", 
                   notification.retry_count, notification.id);
        } else {
            // Max retries exceeded
            error!("Notification failed after {} retries: {}", 
                   notification.max_retries, notification.id);
            self.metrics.failed_notifications += 1;
        }
    }

    /// Process retry queue
    fn process_retry_queue(&mut self) {
        let now = Utc::now();
        let mut to_retry = Vec::new();

        // Find notifications ready for retry
        while let Some(notification) = self.retry_queue.pop_front() {
            if notification.scheduled_for <= now {
                to_retry.push(notification);
            } else {
                // Put it back (not ready yet)
                self.retry_queue.push_front(notification);
                break;
            }
        }

        // Move ready notifications back to main queue
        for notification in to_retry {
            let insert_index = self.notification_queue
                .iter()
                .position(|n| n.priority < notification.priority)
                .unwrap_or(self.notification_queue.len());
            self.notification_queue.insert(insert_index, notification);
        }
    }

    /// Update channel statistics
    fn update_channel_stats(&mut self, channel: &DeliveryChannel, success: bool) {
        let stats = self.metrics.channel_stats
            .entry(channel.clone())
            .or_insert_with(ChannelStats::default);

        stats.sent += 1;
        if success {
            stats.delivered += 1;
        } else {
            stats.failed += 1;
        }
    }

    /// Update user preferences
    fn update_user_preferences(&mut self, user_id: String, preferences: NotificationPreferences) {
        self.user_preferences.insert(user_id.clone(), preferences);
        info!("Updated notification preferences for user: {}", user_id);
    }

    /// Get notification statistics
    fn get_notification_stats(&self) -> NotificationStats {
        NotificationStats {
            total_notifications: self.metrics.total_notifications,
            delivered_notifications: self.metrics.delivered_notifications,
            failed_notifications: self.metrics.failed_notifications,
            queued_notifications: self.metrics.queued_notifications,
            retry_notifications: self.retry_queue.len(),
            average_delivery_time_ms: self.metrics.average_delivery_time_ms,
            channel_stats: self.metrics.channel_stats.clone(),
        }
    }
}

/// Notification statistics
#[derive(Debug, Serialize)]
pub struct NotificationStats {
    pub total_notifications: u64,
    pub delivered_notifications: u64,
    pub failed_notifications: u64,
    pub queued_notifications: usize,
    pub retry_notifications: usize,
    pub average_delivery_time_ms: f64,
    pub channel_stats: HashMap<DeliveryChannel, ChannelStats>,
}

impl Actor for NotificationDeliveryActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Notification delivery actor started");

        // Start periodic queue processing
        ctx.run_interval(self.config.batch_processing_interval, |act, ctx| {
            act.process_notification_queue(ctx);
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Notification delivery actor stopped");
        
        info!("Final notification stats - Total: {}, Delivered: {}, Failed: {}, Queued: {}",
              self.metrics.total_notifications,
              self.metrics.delivered_notifications,
              self.metrics.failed_notifications,
              self.metrics.queued_notifications);
    }
}

/// Deliver notification handler
impl Handler<DeliverNotification> for NotificationDeliveryActor {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: DeliverNotification, _ctx: &mut Self::Context) -> Self::Result {
        self.queue_notification(
            msg.user_id,
            msg.notification,
            msg.persistence_required,
            MessagePriority::Normal,
        ).map(|_| ())
    }
}

/// Update notification preferences message
#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct UpdateNotificationPreferences {
    pub user_id: String,
    pub preferences: NotificationPreferences,
}

impl Handler<UpdateNotificationPreferences> for NotificationDeliveryActor {
    type Result = ();

    fn handle(&mut self, msg: UpdateNotificationPreferences, _ctx: &mut Self::Context) -> Self::Result {
        self.update_user_preferences(msg.user_id, msg.preferences);
    }
}

/// Get notification stats message
#[derive(actix::Message)]
#[rtype(result = "NotificationStats")]
pub struct GetNotificationStats;

impl Handler<GetNotificationStats> for NotificationDeliveryActor {
    type Result = NotificationStats;

    fn handle(&mut self, _msg: GetNotificationStats, _ctx: &mut Self::Context) -> Self::Result {
        self.get_notification_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_preferences_default() {
        let prefs = NotificationPreferences::default();
        assert!(prefs.enable_push);
        assert!(prefs.enable_email);
        assert!(!prefs.enable_sms);
        assert!(prefs.enable_in_app);
        assert_eq!(prefs.delivery_delay_seconds, Some(5));
        assert_eq!(prefs.max_notifications_per_hour, Some(100));
    }

    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert_eq!(config.max_queue_size, 10_000);
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.retry_delay_seconds, 30);
        assert!(config.enable_quiet_hours);
        assert!(config.enable_rate_limiting);
        assert!(!config.enable_redis_queue);
    }

    #[test]
    fn test_delivery_channel_equality() {
        assert_eq!(DeliveryChannel::InApp, DeliveryChannel::InApp);
        assert_ne!(DeliveryChannel::InApp, DeliveryChannel::Push);
        assert_ne!(DeliveryChannel::Email, DeliveryChannel::SMS);
    }
}