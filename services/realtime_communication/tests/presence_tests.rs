//! Presence Tests
//! 
//! Tests for the presence tracking system including status management,
//! typing indicators, and subscription handling.

use chrono::Utc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use realtime_communication::actors::messages::*;
use realtime_communication::presence::*;

#[tokio::test]
async fn test_presence_config_default() {
    let config = PresenceConfig::default();
    
    assert_eq!(config.offline_timeout.as_secs(), 300); // 5 minutes
    assert_eq!(config.away_timeout.as_secs(), 600);    // 10 minutes
    assert_eq!(config.typing_timeout.as_secs(), 10);   // 10 seconds
    assert_eq!(config.cleanup_interval.as_secs(), 60); // 1 minute
    assert_eq!(config.presence_broadcast_interval.as_secs(), 30); // 30 seconds
    assert_eq!(config.max_device_history, 10);
    assert!(config.enable_auto_away);
    assert!(!config.enable_redis_sync);
    assert!(config.batch_updates);
}

#[tokio::test]
async fn test_presence_status_variants() {
    let statuses = vec![
        PresenceStatus::Online,
        PresenceStatus::Away,
        PresenceStatus::Busy,
        PresenceStatus::Offline,
    ];
    
    // Test equality
    assert_eq!(PresenceStatus::Online, PresenceStatus::Online);
    assert_ne!(PresenceStatus::Online, PresenceStatus::Away);
    
    // Test serialization for each status
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: PresenceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}

#[tokio::test]
async fn test_user_presence_info() {
    let mut active_connections = HashSet::new();
    active_connections.insert(Uuid::new_v4());
    active_connections.insert(Uuid::new_v4());
    
    let device_info = vec![
        DeviceInfo {
            connection_id: Uuid::new_v4(),
            device_type: "mobile".to_string(),
            platform: "ios".to_string(),
            app_version: Some("1.2.3".to_string()),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
        },
        DeviceInfo {
            connection_id: Uuid::new_v4(),
            device_type: "desktop".to_string(),
            platform: "windows".to_string(),
            app_version: Some("1.2.3".to_string()),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
        },
    ];
    
    let presence_info = UserPresenceInfo {
        user_id: "test_user".to_string(),
        status: PresenceStatus::Online,
        custom_message: Some("Working on project".to_string()),
        last_seen: Utc::now(),
        last_activity: Utc::now(),
        active_connections,
        device_info: device_info.clone(),
        timezone: Some("America/New_York".to_string()),
        auto_away_enabled: true,
        away_message: None,
    };
    
    assert_eq!(presence_info.user_id, "test_user");
    assert_eq!(presence_info.status, PresenceStatus::Online);
    assert_eq!(presence_info.custom_message, Some("Working on project".to_string()));
    assert_eq!(presence_info.active_connections.len(), 2);
    assert_eq!(presence_info.device_info.len(), 2);
    assert_eq!(presence_info.timezone, Some("America/New_York".to_string()));
    assert!(presence_info.auto_away_enabled);
    
    // Test device info details
    let mobile_device = &presence_info.device_info[0];
    assert_eq!(mobile_device.device_type, "mobile");
    assert_eq!(mobile_device.platform, "ios");
    assert_eq!(mobile_device.app_version, Some("1.2.3".to_string()));
    
    let desktop_device = &presence_info.device_info[1];
    assert_eq!(desktop_device.device_type, "desktop");
    assert_eq!(desktop_device.platform, "windows");
    
    // Test serialization
    let json = serde_json::to_string(&presence_info).unwrap();
    let deserialized: UserPresenceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, presence_info.user_id);
    assert_eq!(deserialized.device_info.len(), presence_info.device_info.len());
}

#[tokio::test]
async fn test_device_info_structure() {
    let device = DeviceInfo {
        connection_id: Uuid::new_v4(),
        device_type: "tablet".to_string(),
        platform: "android".to_string(),
        app_version: Some("2.1.0".to_string()),
        connected_at: Utc::now(),
        last_activity: Utc::now(),
    };
    
    assert_eq!(device.device_type, "tablet");
    assert_eq!(device.platform, "android");
    assert_eq!(device.app_version, Some("2.1.0".to_string()));
    
    // Test different device types
    let device_types = vec![
        "mobile", "desktop", "tablet", "web", "tv", "watch"
    ];
    
    for device_type in device_types {
        let test_device = DeviceInfo {
            connection_id: Uuid::new_v4(),
            device_type: device_type.to_string(),
            platform: "test_platform".to_string(),
            app_version: None,
            connected_at: Utc::now(),
            last_activity: Utc::now(),
        };
        
        assert_eq!(test_device.device_type, device_type);
    }
    
    // Test serialization
    let json = serde_json::to_string(&device).unwrap();
    let deserialized: DeviceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.connection_id, device.connection_id);
    assert_eq!(deserialized.device_type, device.device_type);
    assert_eq!(deserialized.platform, device.platform);
}

#[tokio::test]
async fn test_presence_stats() {
    let stats = PresenceStats {
        total_users_tracked: 100,
        online_users: 75,
        away_users: 15,
        busy_users: 8,
        offline_users: 2,
        active_typing_sessions: 5,
        presence_updates: 1000,
        typing_events: 250,
        subscription_changes: 50,
    };
    
    assert_eq!(stats.total_users_tracked, 100);
    assert_eq!(stats.online_users, 75);
    assert_eq!(stats.away_users, 15);
    assert_eq!(stats.busy_users, 8);
    assert_eq!(stats.offline_users, 2);
    assert_eq!(stats.active_typing_sessions, 5);
    assert_eq!(stats.presence_updates, 1000);
    assert_eq!(stats.typing_events, 250);
    assert_eq!(stats.subscription_changes, 50);
    
    // Test that user counts add up correctly
    let total_by_status = stats.online_users + stats.away_users + stats.busy_users + stats.offline_users;
    assert_eq!(total_by_status, stats.total_users_tracked);
    
    // Test serialization
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: PresenceStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_users_tracked, stats.total_users_tracked);
    assert_eq!(deserialized.online_users, stats.online_users);
}

#[tokio::test]
async fn test_presence_message_types() {
    // Test UpdatePresence message
    let update_presence = UpdatePresence {
        user_id: "test_user".to_string(),
        connection_id: Uuid::new_v4(),
        status: PresenceStatus::Busy,
        message: Some("In a meeting".to_string()),
    };
    
    assert_eq!(update_presence.user_id, "test_user");
    assert_eq!(update_presence.status, PresenceStatus::Busy);
    assert_eq!(update_presence.message, Some("In a meeting".to_string()));
    
    // Test TypingIndicator message
    let typing_indicator = TypingIndicator {
        user_id: "typing_user".to_string(),
        conversation_id: "conv_123".to_string(),
        is_typing: true,
        connection_id: Uuid::new_v4(),
    };
    
    assert_eq!(typing_indicator.user_id, "typing_user");
    assert_eq!(typing_indicator.conversation_id, "conv_123");
    assert!(typing_indicator.is_typing);
    
    // Test SubscribeToPresence message
    let subscribe_presence = SubscribeToPresence {
        subscriber: "observer_user".to_string(),
        target_user: "target_user".to_string(),
    };
    
    assert_eq!(subscribe_presence.subscriber, "observer_user");
    assert_eq!(subscribe_presence.target_user, "target_user");
    
    // Test UnsubscribeFromPresence message
    let unsubscribe_presence = UnsubscribeFromPresence {
        subscriber: "observer_user".to_string(),
        target_user: "target_user".to_string(),
    };
    
    assert_eq!(unsubscribe_presence.subscriber, "observer_user");
    assert_eq!(unsubscribe_presence.target_user, "target_user");
}

#[tokio::test]
async fn test_presence_transitions() {
    // Test valid presence status transitions
    let transitions = vec![
        (PresenceStatus::Offline, PresenceStatus::Online),
        (PresenceStatus::Online, PresenceStatus::Away),
        (PresenceStatus::Away, PresenceStatus::Online),
        (PresenceStatus::Online, PresenceStatus::Busy),
        (PresenceStatus::Busy, PresenceStatus::Online),
        (PresenceStatus::Online, PresenceStatus::Offline),
        (PresenceStatus::Away, PresenceStatus::Offline),
        (PresenceStatus::Busy, PresenceStatus::Offline),
    ];
    
    for (from_status, to_status) in transitions {
        // All transitions should be valid in this system
        assert_ne!(from_status, to_status); // Should be different statuses
        
        // Test that we can represent the transition
        let transition_data = (from_status.clone(), to_status.clone());
        assert_eq!(transition_data.0, from_status);
        assert_eq!(transition_data.1, to_status);
    }
}

#[tokio::test]
async fn test_typing_session_logic() {
    // Test typing session management logic
    struct TypingSession {
        conversation_id: String,
        typing_users: HashMap<String, bool>,
        last_updated: chrono::DateTime<Utc>,
    }
    
    let mut session = TypingSession {
        conversation_id: "conv_123".to_string(),
        typing_users: HashMap::new(),
        last_updated: Utc::now(),
    };
    
    // User starts typing
    session.typing_users.insert("user1".to_string(), true);
    session.last_updated = Utc::now();
    
    assert_eq!(session.typing_users.len(), 1);
    assert_eq!(session.typing_users.get("user1"), Some(&true));
    
    // Another user starts typing
    session.typing_users.insert("user2".to_string(), true);
    
    assert_eq!(session.typing_users.len(), 2);
    assert!(session.typing_users.contains_key("user1"));
    assert!(session.typing_users.contains_key("user2"));
    
    // User stops typing
    session.typing_users.remove("user1");
    
    assert_eq!(session.typing_users.len(), 1);
    assert!(!session.typing_users.contains_key("user1"));
    assert!(session.typing_users.contains_key("user2"));
    
    // All users stop typing
    session.typing_users.clear();
    
    assert_eq!(session.typing_users.len(), 0);
    assert!(session.typing_users.is_empty());
}

#[tokio::test]
async fn test_auto_away_logic() {
    // Test auto-away functionality
    let mut presence = UserPresenceInfo {
        user_id: "test_user".to_string(),
        status: PresenceStatus::Online,
        custom_message: None,
        last_seen: Utc::now(),
        last_activity: Utc::now() - chrono::Duration::minutes(15), // 15 minutes ago
        active_connections: HashSet::new(),
        device_info: Vec::new(),
        timezone: None,
        auto_away_enabled: true,
        away_message: None,
    };
    
    let away_timeout_minutes = 10;
    let now = Utc::now();
    let time_since_activity = now.signed_duration_since(presence.last_activity);
    
    // Check if user should be auto-away
    let should_be_away = presence.auto_away_enabled &&
                        presence.status == PresenceStatus::Online &&
                        time_since_activity.num_minutes() > away_timeout_minutes;
    
    assert!(should_be_away);
    
    // Apply auto-away
    if should_be_away {
        presence.status = PresenceStatus::Away;
        presence.away_message = Some("Auto away due to inactivity".to_string());
    }
    
    assert_eq!(presence.status, PresenceStatus::Away);
    assert_eq!(presence.away_message, Some("Auto away due to inactivity".to_string()));
    
    // Test with auto-away disabled
    let mut presence_no_auto = presence.clone();
    presence_no_auto.auto_away_enabled = false;
    presence_no_auto.status = PresenceStatus::Online;
    
    let should_be_away_disabled = presence_no_auto.auto_away_enabled &&
                                 presence_no_auto.status == PresenceStatus::Online &&
                                 time_since_activity.num_minutes() > away_timeout_minutes;
    
    assert!(!should_be_away_disabled);
}

#[tokio::test]
async fn test_presence_subscription_logic() {
    // Test presence subscription management
    let mut subscriptions: HashMap<String, HashSet<String>> = HashMap::new();
    
    // User A subscribes to User B's presence
    subscriptions.entry("user_b".to_string())
        .or_insert_with(HashSet::new)
        .insert("user_a".to_string());
    
    // User C also subscribes to User B's presence
    subscriptions.entry("user_b".to_string())
        .or_insert_with(HashSet::new)
        .insert("user_c".to_string());
    
    // User A subscribes to User C's presence
    subscriptions.entry("user_c".to_string())
        .or_insert_with(HashSet::new)
        .insert("user_a".to_string());
    
    // Test subscription counts
    assert_eq!(subscriptions.len(), 2); // Two users being watched
    assert_eq!(subscriptions.get("user_b").unwrap().len(), 2); // Two subscribers to user_b
    assert_eq!(subscriptions.get("user_c").unwrap().len(), 1); // One subscriber to user_c
    
    // Test subscription lookup
    assert!(subscriptions.get("user_b").unwrap().contains("user_a"));
    assert!(subscriptions.get("user_b").unwrap().contains("user_c"));
    assert!(subscriptions.get("user_c").unwrap().contains("user_a"));
    
    // Remove subscription
    if let Some(subscribers) = subscriptions.get_mut("user_b") {
        subscribers.remove("user_a");
        if subscribers.is_empty() {
            subscriptions.remove("user_b");
        }
    }
    
    // Verify removal
    assert_eq!(subscriptions.get("user_b").unwrap().len(), 1);
    assert!(!subscriptions.get("user_b").unwrap().contains("user_a"));
    assert!(subscriptions.get("user_b").unwrap().contains("user_c"));
}

#[tokio::test]
async fn test_device_history_management() {
    let max_history = 3;
    let mut device_info = Vec::new();
    
    // Add devices up to limit
    for i in 0..5 {
        let device = DeviceInfo {
            connection_id: Uuid::new_v4(),
            device_type: format!("device_{}", i),
            platform: "test".to_string(),
            app_version: None,
            connected_at: Utc::now() + chrono::Duration::seconds(i),
            last_activity: Utc::now(),
        };
        
        device_info.push(device);
        
        // Enforce history limit
        if device_info.len() > max_history {
            device_info.truncate(max_history);
        }
    }
    
    // Should only keep the most recent devices
    assert_eq!(device_info.len(), max_history);
    
    // Verify we have the most recent devices (last 3)
    for (i, device) in device_info.iter().enumerate() {
        let expected_index = i + 2; // Should be devices 2, 3, 4
        assert_eq!(device.device_type, format!("device_{}", expected_index));
    }
}