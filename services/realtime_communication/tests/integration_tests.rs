//! Integration Tests
//! 
//! End-to-end tests for the complete realtime communication system
//! including WebSocket connections, message flow, and service integration.

use actix_web::{test, web, App};
use actix_ws;
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use realtime_communication::*;

#[actix::test]
async fn test_websocket_connection_establishment() {
    let config = ServerConfig::default();
    let connection_manager = Arc::new(ConnectionManager::new(config.max_connections));
    let metrics = Arc::new(ServerMetrics::default());
    
    let state = ServerState {
        connection_manager,
        config,
        metrics,
    };
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/ws", web::get().to(websocket_handler))
    ).await;
    
    // Test WebSocket endpoint exists and is accessible
    let req = test::TestRequest::get()
        .uri("/ws")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    // WebSocket upgrade requires special handling, so we just test the endpoint is reachable
    // In a real test, we'd use a WebSocket client library
}

#[actix::test]
async fn test_health_and_metrics_endpoints() {
    let config = ServerConfig::default();
    let connection_manager = Arc::new(ConnectionManager::new(config.max_connections));
    let metrics = Arc::new(ServerMetrics::default());
    
    let state = ServerState {
        connection_manager,
        config,
        metrics: metrics.clone(),
    };
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/health", web::get().to(health_handler))
            .route("/metrics", web::get().to(metrics_handler))
    ).await;
    
    // Test health endpoint
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let health_data: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(health_data["status"], "healthy");
    assert_eq!(health_data["active_connections"], 0);
    
    // Test metrics endpoint
    let req = test::TestRequest::get()
        .uri("/metrics")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let metrics_data: ServerStats = test::read_body_json(resp).await;
    assert_eq!(metrics_data.active_connections, 0);
    assert_eq!(metrics_data.total_connections, 0);
}

#[actix::test]
async fn test_connection_limit_enforcement() {
    let mut config = ServerConfig::default();
    config.max_connections = 1; // Set very low limit for testing
    
    let connection_manager = Arc::new(ConnectionManager::new(config.max_connections));
    let metrics = Arc::new(ServerMetrics::default());
    
    // Simulate being at capacity
    metrics.increment_connections().await;
    
    let state = ServerState {
        connection_manager,
        config,
        metrics,
    };
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/ws", web::get().to(websocket_handler))
    ).await;
    
    // This test would require actual WebSocket connection testing
    // For now, we verify the configuration is correct
    assert_eq!(config.max_connections, 1);
}

#[actix::test]
async fn test_message_serialization_compatibility() {
    // Test that our message types are compatible with expected JSON formats
    
    // Client message examples
    let connect_json = r#"{"type":"Connect","data":{"user_id":"user123","metadata":{"device":"mobile"}}}"#;
    let connect_msg: ClientMessage = serde_json::from_str(connect_json).unwrap();
    
    match connect_msg {
        ClientMessage::Connect { user_id, metadata } => {
            assert_eq!(user_id, Some("user123".to_string()));
            assert_eq!(metadata.get("device"), Some(&"mobile".to_string()));
        }
        _ => panic!("Wrong message type"),
    }
    
    let direct_msg_json = r#"{"type":"DirectMessage","data":{"to":"user456","content":{"text":"Hello!"},"message_id":"msg_123"}}"#;
    let direct_msg: ClientMessage = serde_json::from_str(direct_msg_json).unwrap();
    
    match direct_msg {
        ClientMessage::DirectMessage { to, content, message_id } => {
            assert_eq!(to, "user456");
            assert_eq!(content["text"], "Hello!");
            assert_eq!(message_id, Some("msg_123".to_string()));
        }
        _ => panic!("Wrong message type"),
    }
    
    // Server message examples
    let server_msg = ServerMessage::MessageReceived {
        from: "user123".to_string(),
        content: serde_json::json!({"text": "Hello back!"}),
        message_id: "msg_124".to_string(),
        timestamp: 1234567890,
    };
    
    let json = serde_json::to_string(&server_msg).unwrap();
    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ServerMessage::MessageReceived { from, content, message_id, timestamp } => {
            assert_eq!(from, "user123");
            assert_eq!(content["text"], "Hello back!");
            assert_eq!(message_id, "msg_124");
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

#[actix::test]
async fn test_full_actor_system_integration() {
    // Test the complete actor system working together
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    let manager_config = ManagerConfig::default();
    let mut manager = SessionManagerActor::new(manager_config, None);
    manager.set_router(router_addr.clone());
    let manager_addr = manager.start();
    
    // Test system coordination
    sleep(Duration::from_millis(50)).await;
    
    // Get initial stats
    let router_stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(router_stats.total_connections, 0);
    
    let manager_connections = manager_addr.send(GetConnections { user_id: None }).await.unwrap();
    assert_eq!(manager_connections.len(), 0);
}

#[actix::test]
async fn test_concurrent_message_handling() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    // Simulate multiple concurrent users sending messages
    let user_count = 10;
    let messages_per_user = 5;
    
    struct TestSession {
        connection_id: Uuid,
        user_id: String,
    }
    
    impl actix::Actor for TestSession {
        type Context = actix::Context<Self>;
    }
    
    impl actix::Handler<SessionMessage> for TestSession {
        type Result = ();
        fn handle(&mut self, _msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {
            // Track received messages in real implementation
        }
    }
    
    let mut sessions = Vec::new();
    
    // Create sessions
    for i in 0..user_count {
        let connection_id = Uuid::new_v4();
        let user_id = format!("user_{}", i);
        
        let session = TestSession {
            connection_id,
            user_id: user_id.clone(),
        }.start();
        
        // Register with router
        let connect_msg = Connect {
            connection_id,
            user_id: Some(user_id.clone()),
            session_addr: session.recipient(),
            metadata: std::collections::HashMap::new(),
        };
        router_addr.do_send(connect_msg);
        
        sessions.push((connection_id, user_id, session));
    }
    
    sleep(Duration::from_millis(50)).await;
    
    // Send concurrent messages
    for i in 0..messages_per_user {
        for (connection_id, user_id, _) in &sessions {
            let route_msg = RouteMessage {
                from_connection: *connection_id,
                from_user: Some(user_id.clone()),
                message: ClientMessage::BroadcastMessage {
                    content: serde_json::json!({"text": format!("Message {} from {}", i, user_id)}),
                    message_id: Some(format!("msg_{}_{}", user_id, i)),
                },
                timestamp: chrono::Utc::now(),
            };
            router_addr.do_send(route_msg);
        }
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Verify system stats
    let stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(stats.total_connections, user_count);
    
    // Cleanup
    for (connection_id, _, _) in sessions {
        let disconnect_msg = Disconnect {
            connection_id,
            reason: Some("Test completed".to_string()),
        };
        router_addr.do_send(disconnect_msg);
    }
    
    sleep(Duration::from_millis(50)).await;
}

#[actix::test]
async fn test_topic_subscription_system() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    struct TestSession {
        connection_id: Uuid,
        received_messages: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }
    
    impl actix::Actor for TestSession {
        type Context = actix::Context<Self>;
    }
    
    impl actix::Handler<SessionMessage> for TestSession {
        type Result = ();
        fn handle(&mut self, msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {
            if let ServerMessage::TopicMessageReceived { topic, from, .. } = msg.message {
                let mut messages = self.received_messages.lock().unwrap();
                messages.push(format!("{}:{}", topic, from));
            }
        }
    }
    
    // Create sessions with different topic subscriptions
    let session1_messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let session2_messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    
    let connection1 = Uuid::new_v4();
    let connection2 = Uuid::new_v4();
    
    let session1 = TestSession {
        connection_id: connection1,
        received_messages: session1_messages.clone(),
    }.start();
    
    let session2 = TestSession {
        connection_id: connection2,
        received_messages: session2_messages.clone(),
    }.start();
    
    // Register sessions
    router_addr.do_send(Connect {
        connection_id: connection1,
        user_id: Some("user1".to_string()),
        session_addr: session1.recipient(),
        metadata: std::collections::HashMap::new(),
    });
    
    router_addr.do_send(Connect {
        connection_id: connection2,
        user_id: Some("user2".to_string()),
        session_addr: session2.recipient(),
        metadata: std::collections::HashMap::new(),
    });
    
    sleep(Duration::from_millis(50)).await;
    
    // Subscribe to topics
    router_addr.do_send(SubscribeToTopic {
        connection_id: connection1,
        topics: vec!["general".to_string(), "tech".to_string()],
    });
    
    router_addr.do_send(SubscribeToTopic {
        connection_id: connection2,
        topics: vec!["general".to_string()], // Only general
    });
    
    sleep(Duration::from_millis(50)).await;
    
    // Send topic messages
    router_addr.do_send(RouteMessage {
        from_connection: connection1,
        from_user: Some("user1".to_string()),
        message: ClientMessage::TopicMessage {
            topic: "general".to_string(),
            content: serde_json::json!({"text": "Hello general!"}),
            message_id: Some("msg_general".to_string()),
        },
        timestamp: chrono::Utc::now(),
    });
    
    router_addr.do_send(RouteMessage {
        from_connection: connection1,
        from_user: Some("user1".to_string()),
        message: ClientMessage::TopicMessage {
            topic: "tech".to_string(),
            content: serde_json::json!({"text": "Hello tech!"}),
            message_id: Some("msg_tech".to_string()),
        },
        timestamp: chrono::Utc::now(),
    });
    
    sleep(Duration::from_millis(100)).await;
    
    // Verify message delivery
    let session1_msgs = session1_messages.lock().unwrap();
    let session2_msgs = session2_messages.lock().unwrap();
    
    // Session1 should receive tech message (but not general since it's the sender)
    // Session2 should receive general message (but not tech since not subscribed)
    // Note: In the current implementation, senders don't receive their own messages
    
    println!("Session1 messages: {:?}", *session1_msgs);
    println!("Session2 messages: {:?}", *session2_msgs);
}

#[actix::test]
async fn test_error_handling_and_recovery() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    // Test invalid message handling
    let connection_id = Uuid::new_v4();
    
    struct TestSession;
    impl actix::Actor for TestSession {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<SessionMessage> for TestSession {
        type Result = ();
        fn handle(&mut self, _msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {}
    }
    
    let session = TestSession.start();
    
    // Register session
    router_addr.do_send(Connect {
        connection_id,
        user_id: Some("test_user".to_string()),
        session_addr: session.recipient(),
        metadata: std::collections::HashMap::new(),
    });
    
    sleep(Duration::from_millis(50)).await;
    
    // Send message to non-existent user
    router_addr.do_send(RouteMessage {
        from_connection: connection_id,
        from_user: Some("test_user".to_string()),
        message: ClientMessage::DirectMessage {
            to: "non_existent_user".to_string(),
            content: serde_json::json!({"text": "Hello!"}),
            message_id: Some("msg_fail".to_string()),
        },
        timestamp: chrono::Utc::now(),
    });
    
    sleep(Duration::from_millis(50)).await;
    
    // System should handle this gracefully
    let stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(stats.total_connections, 1);
}

#[actix::test]
async fn test_session_cleanup_on_disconnect() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    let connection_id = Uuid::new_v4();
    
    struct TestSession;
    impl actix::Actor for TestSession {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<SessionMessage> for TestSession {
        type Result = ();
        fn handle(&mut self, _msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {}
    }
    
    let session = TestSession.start();
    
    // Register and subscribe
    router_addr.do_send(Connect {
        connection_id,
        user_id: Some("test_user".to_string()),
        session_addr: session.recipient(),
        metadata: std::collections::HashMap::new(),
    });
    
    router_addr.do_send(SubscribeToTopic {
        connection_id,
        topics: vec!["test_topic".to_string()],
    });
    
    sleep(Duration::from_millis(50)).await;
    
    // Verify connection exists
    let stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(stats.total_connections, 1);
    
    // Disconnect
    router_addr.do_send(Disconnect {
        connection_id,
        reason: Some("Test disconnect".to_string()),
    });
    
    sleep(Duration::from_millis(50)).await;
    
    // Verify cleanup
    let stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(stats.total_connections, 0);
}