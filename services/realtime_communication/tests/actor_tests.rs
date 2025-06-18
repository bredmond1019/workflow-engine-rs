//! Actor System Tests
//! 
//! Unit and integration tests for the router, session, and manager actors
//! including message routing, session management, and actor communication.

use actix::{System, Actor, Addr, Recipient};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use realtime_communication::actors::*;
use realtime_communication::connection::ConnectionManager;

#[actix::test]
async fn test_router_actor_creation() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager);
    
    // Start the router actor
    let router_addr = router.start();
    
    // Test that the router is responsive
    let stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(stats.total_connections, 0);
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.unique_users, 0);
}

#[actix::test]
async fn test_session_manager_creation() {
    let config = ManagerConfig::default();
    let mut manager = SessionManagerActor::new(config, None);
    
    // Create a mock router
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager);
    let router_addr = router.start();
    
    manager.set_router(router_addr);
    
    let manager_addr = manager.start();
    
    // Test that the manager is responsive
    let connections = manager_addr.send(GetConnections { user_id: None }).await.unwrap();
    assert_eq!(connections.len(), 0);
}

#[actix::test]
async fn test_message_routing_flow() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    let connection_id = Uuid::new_v4();
    let user_id = Some("test_user".to_string());
    
    // Create a mock session recipient (in real usage, this would be a SessionActor)
    // For testing, we'll create a simple test actor
    struct TestSessionActor;
    impl Actor for TestSessionActor {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<SessionMessage> for TestSessionActor {
        type Result = ();
        fn handle(&mut self, _msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {
            // Just acknowledge the message
        }
    }
    
    let test_session = TestSessionActor.start();
    let session_recipient = test_session.recipient();
    
    // Register session with router
    let connect_msg = Connect {
        connection_id,
        user_id: user_id.clone(),
        session_addr: session_recipient,
        metadata: HashMap::new(),
    };
    router_addr.do_send(connect_msg);
    
    // Give some time for the message to be processed
    sleep(Duration::from_millis(10)).await;
    
    // Send a direct message
    let route_msg = RouteMessage {
        from_connection: connection_id,
        from_user: user_id.clone(),
        message: ClientMessage::DirectMessage {
            to: "target_user".to_string(),
            content: serde_json::json!({"text": "Hello!"}),
            message_id: Some("msg_123".to_string()),
        },
        timestamp: chrono::Utc::now(),
    };
    router_addr.do_send(route_msg);
    
    // Test successful routing
    sleep(Duration::from_millis(10)).await;
    
    // Test subscription
    let subscribe_msg = SubscribeToTopic {
        connection_id,
        topics: vec!["general".to_string(), "announcements".to_string()],
    };
    router_addr.do_send(subscribe_msg);
    
    sleep(Duration::from_millis(10)).await;
    
    // Send topic message
    let topic_msg = RouteMessage {
        from_connection: connection_id,
        from_user: user_id.clone(),
        message: ClientMessage::TopicMessage {
            topic: "general".to_string(),
            content: serde_json::json!({"text": "Hello everyone!"}),
            message_id: Some("msg_124".to_string()),
        },
        timestamp: chrono::Utc::now(),
    };
    router_addr.do_send(topic_msg);
    
    sleep(Duration::from_millis(10)).await;
    
    // Disconnect
    let disconnect_msg = Disconnect {
        connection_id,
        reason: Some("Test completed".to_string()),
    };
    router_addr.do_send(disconnect_msg);
    
    sleep(Duration::from_millis(10)).await;
}

#[actix::test]
async fn test_session_actor_lifecycle() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    let connection_id = Uuid::new_v4();
    let config = SessionConfig::default();
    
    // Note: SessionActor requires WebSocket context, so we can't easily test it in isolation
    // In a real test, we'd need to set up a full WebSocket connection
    // For now, we'll test the configuration and basic structures
    
    assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
    assert_eq!(config.client_timeout, Duration::from_secs(60));
    assert_eq!(config.max_missed_heartbeats, 3);
    assert!(config.enable_message_buffering);
}

#[actix::test]
async fn test_concurrent_sessions() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    // Create multiple mock sessions
    struct TestSessionActor {
        id: Uuid,
    }
    impl Actor for TestSessionActor {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<SessionMessage> for TestSessionActor {
        type Result = ();
        fn handle(&mut self, _msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {
            // Track received messages in real implementation
        }
    }
    
    let mut sessions = Vec::new();
    let session_count = 10;
    
    for i in 0..session_count {
        let connection_id = Uuid::new_v4();
        let session = TestSessionActor { id: connection_id }.start();
        let session_recipient = session.recipient();
        
        // Register with router
        let connect_msg = Connect {
            connection_id,
            user_id: Some(format!("user_{}", i)),
            session_addr: session_recipient,
            metadata: HashMap::new(),
        };
        router_addr.do_send(connect_msg);
        
        sessions.push((connection_id, session));
    }
    
    // Give time for all connections to register
    sleep(Duration::from_millis(50)).await;
    
    // Send broadcast message
    let broadcast_msg = RouteMessage {
        from_connection: sessions[0].0,
        from_user: Some("user_0".to_string()),
        message: ClientMessage::BroadcastMessage {
            content: serde_json::json!({"text": "Hello everyone!"}),
            message_id: Some("broadcast_123".to_string()),
        },
        timestamp: chrono::Utc::now(),
    };
    router_addr.do_send(broadcast_msg);
    
    sleep(Duration::from_millis(50)).await;
    
    // Get system stats
    let stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(stats.total_connections, session_count);
    
    // Disconnect all sessions
    for (connection_id, _) in sessions {
        let disconnect_msg = Disconnect {
            connection_id,
            reason: Some("Test completed".to_string()),
        };
        router_addr.do_send(disconnect_msg);
    }
    
    sleep(Duration::from_millis(50)).await;
}

#[actix::test]
async fn test_message_priorities() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    // Test message priority ordering
    assert!(MessagePriority::Critical > MessagePriority::High);
    assert!(MessagePriority::High > MessagePriority::Normal);
    assert!(MessagePriority::Normal > MessagePriority::Low);
    
    // Test default priority
    let default_priority = MessagePriority::default();
    assert_eq!(default_priority, MessagePriority::Normal);
    
    // Test priority from string
    assert_eq!(MessagePriority::from("low"), MessagePriority::Low);
    assert_eq!(MessagePriority::from("normal"), MessagePriority::Normal);
    assert_eq!(MessagePriority::from("high"), MessagePriority::High);
    assert_eq!(MessagePriority::from("critical"), MessagePriority::Critical);
    assert_eq!(MessagePriority::from("unknown"), MessagePriority::Normal);
}

#[actix::test]
async fn test_client_message_serialization() {
    // Test serialization of client messages
    let connect_msg = ClientMessage::Connect {
        user_id: Some("test_user".to_string()),
        metadata: {
            let mut map = HashMap::new();
            map.insert("device".to_string(), "mobile".to_string());
            map
        },
    };
    
    let json = serde_json::to_string(&connect_msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ClientMessage::Connect { user_id, metadata } => {
            assert_eq!(user_id, Some("test_user".to_string()));
            assert_eq!(metadata.get("device"), Some(&"mobile".to_string()));
        }
        _ => panic!("Wrong message type"),
    }
    
    // Test direct message
    let direct_msg = ClientMessage::DirectMessage {
        to: "recipient".to_string(),
        content: serde_json::json!({"text": "Hello!"}),
        message_id: Some("msg_123".to_string()),
    };
    
    let json = serde_json::to_string(&direct_msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ClientMessage::DirectMessage { to, content, message_id } => {
            assert_eq!(to, "recipient");
            assert_eq!(content["text"], "Hello!");
            assert_eq!(message_id, Some("msg_123".to_string()));
        }
        _ => panic!("Wrong message type"),
    }
}

#[actix::test]
async fn test_server_message_serialization() {
    // Test server message serialization
    let connected_msg = ServerMessage::Connected {
        connection_id: "conn_123".to_string(),
        server_time: 1234567890,
    };
    
    let json = serde_json::to_string(&connected_msg).unwrap();
    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ServerMessage::Connected { connection_id, server_time } => {
            assert_eq!(connection_id, "conn_123");
            assert_eq!(server_time, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
    
    // Test message received
    let msg_received = ServerMessage::MessageReceived {
        from: "sender".to_string(),
        content: serde_json::json!({"text": "Hello!"}),
        message_id: "msg_123".to_string(),
        timestamp: 1234567890,
    };
    
    let json = serde_json::to_string(&msg_received).unwrap();
    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ServerMessage::MessageReceived { from, content, message_id, timestamp } => {
            assert_eq!(from, "sender");
            assert_eq!(content["text"], "Hello!");
            assert_eq!(message_id, "msg_123");
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

#[actix::test]
async fn test_presence_status() {
    // Test presence status variants
    assert_eq!(PresenceStatus::Online, PresenceStatus::Online);
    assert_ne!(PresenceStatus::Online, PresenceStatus::Away);
    
    // Test serialization
    let status = PresenceStatus::Busy;
    let json = serde_json::to_string(&status).unwrap();
    let deserialized: PresenceStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(status, deserialized);
}

#[actix::test]
async fn test_delivery_status() {
    // Test delivery status variants
    let statuses = vec![
        DeliveryStatus::Sent,
        DeliveryStatus::Delivered,
        DeliveryStatus::Read,
        DeliveryStatus::Failed,
    ];
    
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: DeliveryStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}

#[actix::test]
async fn test_connection_summary() {
    let summary = ConnectionSummary {
        connection_id: Uuid::new_v4(),
        user_id: Some("test_user".to_string()),
        connected_at: chrono::Utc::now(),
        last_activity: chrono::Utc::now(),
        subscriptions: vec!["general".to_string(), "announcements".to_string()],
        presence_status: PresenceStatus::Online,
    };
    
    assert_eq!(summary.user_id, Some("test_user".to_string()));
    assert_eq!(summary.subscriptions.len(), 2);
    assert!(summary.subscriptions.contains(&"general".to_string()));
    assert_eq!(summary.presence_status, PresenceStatus::Online);
    
    // Test serialization
    let json = serde_json::to_string(&summary).unwrap();
    let deserialized: ConnectionSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, summary.user_id);
    assert_eq!(deserialized.subscriptions, summary.subscriptions);
}

#[actix::test]
async fn test_system_stats() {
    let stats = SystemStats {
        total_connections: 100,
        active_connections: 95,
        unique_users: 85,
        messages_routed: 1000,
        messages_delivered: 980,
        messages_failed: 20,
        topics_active: 15,
        uptime_seconds: 3600,
    };
    
    assert_eq!(stats.total_connections, 100);
    assert_eq!(stats.active_connections, 95);
    assert_eq!(stats.unique_users, 85);
    
    // Test serialization
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: SystemStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_connections, stats.total_connections);
    assert_eq!(deserialized.messages_routed, stats.messages_routed);
}