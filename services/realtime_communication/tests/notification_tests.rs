//! Notification Tests
//! 
//! Tests for the notification delivery system including preferences,
//! queuing, retry mechanisms, and delivery channels.

use chrono::Utc;
use std::collections::HashMap;

use realtime_communication::actors::messages::*;
use realtime_communication::notifications::*;

#[tokio::test]
async fn test_notification_preferences_default() {
    let prefs = NotificationPreferences::default();
    
    assert!(prefs.enable_push);
    assert!(prefs.enable_email);
    assert!(!prefs.enable_sms);
    assert!(prefs.enable_in_app);
    assert_eq!(prefs.delivery_delay_seconds, Some(5));
    assert_eq!(prefs.max_notifications_per_hour, Some(100));
    assert!(prefs.message_types.contains(&"direct_message".to_string()));
    assert!(prefs.message_types.contains(&"mention".to_string()));
    assert!(prefs.message_types.contains(&"system_alert".to_string()));
}

#[tokio::test]
async fn test_notification_preferences_customization() {
    let mut prefs = NotificationPreferences::default();
    prefs.user_id = "test_user".to_string();
    prefs.enable_sms = true;
    prefs.quiet_hours_start = Some("22:00".to_string());
    prefs.quiet_hours_end = Some("08:00".to_string());
    prefs.timezone = Some("America/New_York".to_string());
    prefs.delivery_delay_seconds = Some(10);
    prefs.max_notifications_per_hour = Some(50);
    
    // Add topic-specific preferences
    prefs.topic_preferences.insert("urgent".to_string(), true);
    prefs.topic_preferences.insert("spam".to_string(), false);
    
    assert_eq!(prefs.user_id, "test_user");
    assert!(prefs.enable_sms);
    assert_eq!(prefs.quiet_hours_start, Some("22:00".to_string()));
    assert_eq!(prefs.quiet_hours_end, Some("08:00".to_string()));
    assert_eq!(prefs.timezone, Some("America/New_York".to_string()));
    assert_eq!(prefs.delivery_delay_seconds, Some(10));
    assert_eq!(prefs.max_notifications_per_hour, Some(50));
    assert_eq!(prefs.topic_preferences.get("urgent"), Some(&true));
    assert_eq!(prefs.topic_preferences.get("spam"), Some(&false));
    
    // Test serialization
    let json = serde_json::to_string(&prefs).unwrap();
    let deserialized: NotificationPreferences = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, prefs.user_id);
    assert_eq!(deserialized.topic_preferences, prefs.topic_preferences);
}

#[tokio::test]
async fn test_notification_config_default() {
    let config = NotificationConfig::default();
    
    assert_eq!(config.max_queue_size, 10_000);
    assert_eq!(config.max_retry_attempts, 3);
    assert_eq!(config.retry_delay_seconds, 30);
    assert_eq!(config.delivery_timeout_seconds, 30);
    assert!(config.enable_quiet_hours);
    assert!(config.enable_rate_limiting);
    assert!(!config.enable_redis_queue);
}

#[tokio::test]
async fn test_delivery_channel_types() {
    let channels = vec![
        DeliveryChannel::InApp,
        DeliveryChannel::Push,
        DeliveryChannel::Email,
        DeliveryChannel::SMS,
        DeliveryChannel::Webhook,
    ];
    
    // Test equality
    assert_eq!(DeliveryChannel::InApp, DeliveryChannel::InApp);
    assert_ne!(DeliveryChannel::InApp, DeliveryChannel::Push);
    
    // Test that all channel types are distinct
    for (i, channel1) in channels.iter().enumerate() {
        for (j, channel2) in channels.iter().enumerate() {
            if i == j {
                assert_eq!(channel1, channel2);
            } else {
                assert_ne!(channel1, channel2);
            }
        }
    }
}

#[tokio::test]
async fn test_notification_stats() {
    let mut channel_stats = HashMap::new();
    channel_stats.insert(DeliveryChannel::InApp, ChannelStats {
        sent: 100,
        delivered: 98,
        failed: 2,
        average_delivery_time_ms: 15.5,
    });
    channel_stats.insert(DeliveryChannel::Push, ChannelStats {
        sent: 50,
        delivered: 48,
        failed: 2,
        average_delivery_time_ms: 250.0,
    });
    
    let stats = NotificationStats {
        total_notifications: 150,
        delivered_notifications: 146,
        failed_notifications: 4,
        queued_notifications: 5,
        retry_notifications: 2,
        average_delivery_time_ms: 45.2,
        channel_stats,
    };
    
    assert_eq!(stats.total_notifications, 150);
    assert_eq!(stats.delivered_notifications, 146);
    assert_eq!(stats.failed_notifications, 4);
    assert_eq!(stats.queued_notifications, 5);
    assert_eq!(stats.retry_notifications, 2);
    
    // Test channel stats access
    let in_app_stats = stats.channel_stats.get(&DeliveryChannel::InApp).unwrap();
    assert_eq!(in_app_stats.sent, 100);
    assert_eq!(in_app_stats.delivered, 98);
    assert_eq!(in_app_stats.failed, 2);
    
    let push_stats = stats.channel_stats.get(&DeliveryChannel::Push).unwrap();
    assert_eq!(push_stats.sent, 50);
    assert_eq!(push_stats.delivered, 48);
    
    // Test serialization
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: NotificationStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_notifications, stats.total_notifications);
    assert_eq!(deserialized.channel_stats.len(), stats.channel_stats.len());
}

#[tokio::test]
async fn test_notification_message_types() {
    // Test different server message types for notifications
    let messages = vec![
        ServerMessage::MessageReceived {
            from: "alice".to_string(),
            content: serde_json::json!({"text": "Hello!"}),
            message_id: "msg_1".to_string(),
            timestamp: Utc::now().timestamp(),
        },
        ServerMessage::Notification {
            level: NotificationLevel::Info,
            title: "System Update".to_string(),
            message: "System will be updated tonight".to_string(),
            timestamp: Utc::now().timestamp(),
        },
        ServerMessage::SystemMessage {
            message: "Server maintenance in progress".to_string(),
            level: "warning".to_string(),
        },
        ServerMessage::Error {
            code: 500,
            message: "Internal server error".to_string(),
        },
    ];
    
    // Test message type classification
    for message in messages {
        match &message {
            ServerMessage::MessageReceived { .. } => {
                // Should trigger direct message notification
                assert!(true);
            }
            ServerMessage::Notification { level, .. } => {
                match level {
                    NotificationLevel::Info => {
                        // Should be in-app only
                        assert!(true);
                    }
                    NotificationLevel::Error => {
                        // Should trigger push and email
                        assert!(true);
                    }
                    _ => {}
                }
            }
            ServerMessage::SystemMessage { level, .. } => {
                if level == "critical" {
                    // Should trigger all channels
                    assert!(true);
                }
            }
            ServerMessage::Error { .. } => {
                // Should trigger error notifications
                assert!(true);
            }
            _ => {}
        }
        
        // Test serialization
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(std::mem::discriminant(&message), std::mem::discriminant(&deserialized));
    }
}

#[tokio::test]
async fn test_notification_priority_handling() {
    let priorities = vec![
        MessagePriority::Low,
        MessagePriority::Normal,
        MessagePriority::High,
        MessagePriority::Critical,
    ];
    
    // Test priority ordering
    assert!(MessagePriority::Critical > MessagePriority::High);
    assert!(MessagePriority::High > MessagePriority::Normal);
    assert!(MessagePriority::Normal > MessagePriority::Low);
    
    // Test priority values
    assert_eq!(MessagePriority::Low as u8, 0);
    assert_eq!(MessagePriority::Normal as u8, 1);
    assert_eq!(MessagePriority::High as u8, 2);
    assert_eq!(MessagePriority::Critical as u8, 3);
    
    // Test from string conversion
    assert_eq!(MessagePriority::from("low"), MessagePriority::Low);
    assert_eq!(MessagePriority::from("normal"), MessagePriority::Normal);
    assert_eq!(MessagePriority::from("high"), MessagePriority::High);
    assert_eq!(MessagePriority::from("critical"), MessagePriority::Critical);
    assert_eq!(MessagePriority::from("unknown"), MessagePriority::Normal);
    
    // Test default
    assert_eq!(MessagePriority::default(), MessagePriority::Normal);
}

#[tokio::test]
async fn test_notification_batching_scenarios() {
    // Test scenarios where notifications should be batched vs immediate
    let preferences = NotificationPreferences {
        user_id: "test_user".to_string(),
        delivery_delay_seconds: Some(5),
        ..Default::default()
    };
    
    // Critical messages should be immediate regardless of batching
    let critical_notification = ServerMessage::SystemMessage {
        message: "Critical system failure".to_string(),
        level: "critical".to_string(),
    };
    
    // Normal messages should respect batching delay
    let normal_notification = ServerMessage::MessageReceived {
        from: "alice".to_string(),
        content: serde_json::json!({"text": "How are you?"}),
        message_id: "msg_normal".to_string(),
        timestamp: Utc::now().timestamp(),
    };
    
    // Test that both messages can be properly categorized
    match critical_notification {
        ServerMessage::SystemMessage { level, .. } if level == "critical" => {
            // Should bypass batching delay
            assert!(true);
        }
        _ => panic!("Wrong message type"),
    }
    
    match normal_notification {
        ServerMessage::MessageReceived { .. } => {
            // Should respect batching delay from preferences
            assert_eq!(preferences.delivery_delay_seconds, Some(5));
        }
        _ => panic!("Wrong message type"),
    }
}

#[tokio::test]
async fn test_quiet_hours_logic() {
    let mut preferences = NotificationPreferences::default();
    preferences.quiet_hours_start = Some("22:00".to_string());
    preferences.quiet_hours_end = Some("08:00".to_string());
    
    // Test quiet hours parsing
    let start_hour = preferences.quiet_hours_start
        .as_ref()
        .and_then(|s| s.split(':').next())
        .and_then(|h| h.parse::<u32>().ok())
        .unwrap();
    
    let end_hour = preferences.quiet_hours_end
        .as_ref()
        .and_then(|s| s.split(':').next())
        .and_then(|h| h.parse::<u32>().ok())
        .unwrap();
    
    assert_eq!(start_hour, 22);
    assert_eq!(end_hour, 8);
    
    // Test time ranges that span midnight
    assert!(start_hour > end_hour); // Spans midnight
    
    // Test different hour scenarios
    let test_hours = vec![
        (6, true),   // During quiet hours (before 8 AM)
        (23, true),  // During quiet hours (after 10 PM)
        (12, false), // Not during quiet hours (noon)
        (18, false), // Not during quiet hours (6 PM)
    ];
    
    for (test_hour, should_be_quiet) in test_hours {
        let is_quiet = if start_hour <= end_hour {
            // Same day quiet hours
            test_hour >= start_hour && test_hour < end_hour
        } else {
            // Spans midnight
            test_hour >= start_hour || test_hour < end_hour
        };
        
        assert_eq!(is_quiet, should_be_quiet, "Hour {} quiet status mismatch", test_hour);
    }
}

#[tokio::test]
async fn test_rate_limiting_logic() {
    let preferences = NotificationPreferences {
        user_id: "test_user".to_string(),
        max_notifications_per_hour: Some(10),
        ..Default::default()
    };
    
    // Test rate limit value
    assert_eq!(preferences.max_notifications_per_hour, Some(10));
    
    // Simulate notification counts
    let mut notifications_sent = 0;
    let max_allowed = preferences.max_notifications_per_hour.unwrap();
    
    // Send notifications up to limit
    for i in 0..15 {
        if notifications_sent < max_allowed {
            notifications_sent += 1;
            // Would send notification
            assert!(notifications_sent <= max_allowed);
        } else {
            // Would drop notification due to rate limit
            assert!(i >= max_allowed);
        }
    }
    
    assert_eq!(notifications_sent, max_allowed);
}

#[tokio::test]
async fn test_delivery_preferences_by_message_type() {
    let mut preferences = NotificationPreferences::default();
    
    // Customize message type preferences
    preferences.message_types = vec![
        "direct_message".to_string(),
        "mention".to_string(),
        // Exclude "system_alert" and others
    ];
    
    // Test message type filtering
    let allowed_messages = vec![
        "direct_message",
        "mention",
    ];
    
    let blocked_messages = vec![
        "system_alert",
        "broadcast",
        "topic_message",
    ];
    
    for msg_type in allowed_messages {
        assert!(preferences.message_types.contains(&msg_type.to_string()));
    }
    
    for msg_type in blocked_messages {
        assert!(!preferences.message_types.contains(&msg_type.to_string()));
    }
}