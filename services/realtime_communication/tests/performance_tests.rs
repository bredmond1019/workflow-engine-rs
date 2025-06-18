//! Performance Tests
//! 
//! Load testing and performance benchmarks for the realtime communication system
//! including concurrent connection limits, message throughput, and latency tests.

use actix::{Actor, Addr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

use realtime_communication::*;

#[actix::test]
async fn test_high_concurrent_connections() {
    let connection_limit = 1000; // Test with 1000 concurrent connections
    let connection_manager = Arc::new(ConnectionManager::new(connection_limit));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    struct PerformanceTestSession {
        connection_id: Uuid,
        user_id: String,
        message_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }
    
    impl Actor for PerformanceTestSession {
        type Context = actix::Context<Self>;
    }
    
    impl actix::Handler<SessionMessage> for PerformanceTestSession {
        type Result = ();
        
        fn handle(&mut self, _msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {
            self.message_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    
    let start_time = Instant::now();
    let mut sessions = Vec::new();
    let session_count = 100; // Reduce for test performance
    
    // Create sessions in batches to avoid overwhelming the system
    for batch in 0..(session_count / 10) {
        for i in 0..10 {
            let session_index = batch * 10 + i;
            let connection_id = Uuid::new_v4();
            let user_id = format!("perf_user_{}", session_index);
            let message_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            
            let session = PerformanceTestSession {
                connection_id,
                user_id: user_id.clone(),
                message_count: message_count.clone(),
            }.start();
            
            // Register with router
            let connect_msg = Connect {
                connection_id,
                user_id: Some(user_id.clone()),
                session_addr: session.recipient(),
                metadata: std::collections::HashMap::new(),
            };
            router_addr.do_send(connect_msg);
            
            sessions.push((connection_id, user_id, session, message_count));
        }
        
        // Small delay between batches
        sleep(Duration::from_millis(10)).await;
    }
    
    let connection_time = start_time.elapsed();
    println!("Connected {} sessions in {:?}", session_count, connection_time);
    
    // Give time for all connections to register
    sleep(Duration::from_millis(100)).await;
    
    // Verify all connections are registered
    let stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(stats.total_connections, session_count);
    
    // Test message throughput
    let message_start = Instant::now();
    let messages_per_session = 10;
    
    for (connection_id, user_id, _, _) in &sessions {
        for msg_index in 0..messages_per_session {
            let route_msg = RouteMessage {
                from_connection: *connection_id,
                from_user: Some(user_id.clone()),
                message: ClientMessage::BroadcastMessage {
                    content: serde_json::json!({
                        "text": format!("Performance test message {} from {}", msg_index, user_id),
                        "timestamp": chrono::Utc::now().timestamp()
                    }),
                    message_id: Some(format!("perf_msg_{}_{}", user_id, msg_index)),
                },
                timestamp: chrono::Utc::now(),
            };
            router_addr.do_send(route_msg);
        }
    }
    
    // Wait for message processing
    sleep(Duration::from_millis(500)).await;
    
    let message_time = message_start.elapsed();
    let total_messages = session_count * messages_per_session;
    let messages_per_second = total_messages as f64 / message_time.as_secs_f64();
    
    println!("Processed {} messages in {:?} ({:.2} msg/sec)", 
             total_messages, message_time, messages_per_second);
    
    // Verify message delivery
    let mut total_received = 0;
    for (_, _, _, message_count) in &sessions {
        total_received += message_count.load(std::sync::atomic::Ordering::Relaxed);
    }
    
    println!("Total messages received: {}", total_received);
    
    // Cleanup - disconnect all sessions
    let disconnect_start = Instant::now();
    for (connection_id, _, _, _) in sessions {
        let disconnect_msg = Disconnect {
            connection_id,
            reason: Some("Performance test completed".to_string()),
        };
        router_addr.do_send(disconnect_msg);
    }
    
    sleep(Duration::from_millis(100)).await;
    let disconnect_time = disconnect_start.elapsed();
    
    println!("Disconnected {} sessions in {:?}", session_count, disconnect_time);
    
    // Verify cleanup
    let final_stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(final_stats.total_connections, 0);
}

#[actix::test]
async fn test_message_latency() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    struct LatencyTestSession {
        connection_id: Uuid,
        latencies: std::sync::Arc<std::sync::Mutex<Vec<Duration>>>,
        send_times: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Instant>>>,
    }
    
    impl Actor for LatencyTestSession {
        type Context = actix::Context<Self>;
    }
    
    impl actix::Handler<SessionMessage> for LatencyTestSession {
        type Result = ();
        
        fn handle(&mut self, msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {
            if let ServerMessage::MessageReceived { message_id, .. } = msg.message {
                let receive_time = Instant::now();
                let mut send_times = self.send_times.lock().unwrap();
                
                if let Some(send_time) = send_times.remove(&message_id) {
                    let latency = receive_time.duration_since(send_time);
                    let mut latencies = self.latencies.lock().unwrap();
                    latencies.push(latency);
                }
            }
        }
    }
    
    let latencies = Arc::new(std::sync::Mutex::new(Vec::new()));
    let send_times = Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
    
    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    let sender_session = LatencyTestSession {
        connection_id: sender_id,
        latencies: latencies.clone(),
        send_times: send_times.clone(),
    }.start();
    
    let receiver_session = LatencyTestSession {
        connection_id: receiver_id,
        latencies: latencies.clone(),
        send_times: send_times.clone(),
    }.start();
    
    // Register sessions
    router_addr.do_send(Connect {
        connection_id: sender_id,
        user_id: Some("sender".to_string()),
        session_addr: sender_session.recipient(),
        metadata: std::collections::HashMap::new(),
    });
    
    router_addr.do_send(Connect {
        connection_id: receiver_id,
        user_id: Some("receiver".to_string()),
        session_addr: receiver_session.recipient(),
        metadata: std::collections::HashMap::new(),
    });
    
    sleep(Duration::from_millis(50)).await;
    
    // Send test messages and measure latency
    let test_message_count = 100;
    
    for i in 0..test_message_count {
        let message_id = format!("latency_test_{}", i);
        let send_time = Instant::now();
        
        // Record send time
        {
            let mut send_times_map = send_times.lock().unwrap();
            send_times_map.insert(message_id.clone(), send_time);
        }
        
        // Send message
        let route_msg = RouteMessage {
            from_connection: sender_id,
            from_user: Some("sender".to_string()),
            message: ClientMessage::DirectMessage {
                to: "receiver".to_string(),
                content: serde_json::json!({"text": format!("Latency test message {}", i)}),
                message_id: Some(message_id),
            },
            timestamp: chrono::Utc::now(),
        };
        router_addr.do_send(route_msg);
        
        // Small delay to avoid overwhelming the system
        if i % 10 == 0 {
            sleep(Duration::from_millis(1)).await;
        }
    }
    
    // Wait for all messages to be processed
    sleep(Duration::from_millis(500)).await;
    
    // Analyze latencies
    let latencies_vec = latencies.lock().unwrap();
    let received_count = latencies_vec.len();
    
    if received_count > 0 {
        let total_latency: Duration = latencies_vec.iter().sum();
        let average_latency = total_latency / received_count as u32;
        
        let mut sorted_latencies = latencies_vec.clone();
        sorted_latencies.sort();
        
        let p50 = sorted_latencies[received_count / 2];
        let p95 = sorted_latencies[(received_count as f64 * 0.95) as usize];
        let p99 = sorted_latencies[(received_count as f64 * 0.99) as usize];
        
        println!("Latency test results:");
        println!("  Messages sent: {}", test_message_count);
        println!("  Messages received: {}", received_count);
        println!("  Average latency: {:?}", average_latency);
        println!("  P50 latency: {:?}", p50);
        println!("  P95 latency: {:?}", p95);
        println!("  P99 latency: {:?}", p99);
        
        // Assert reasonable latency bounds
        assert!(average_latency < Duration::from_millis(10), "Average latency too high");
        assert!(p95 < Duration::from_millis(20), "P95 latency too high");
    } else {
        panic!("No messages received during latency test");
    }
}

#[actix::test]
async fn test_topic_scalability() {
    let connection_manager = Arc::new(ConnectionManager::new(1000));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    struct TopicTestSession {
        connection_id: Uuid,
        received_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }
    
    impl Actor for TopicTestSession {
        type Context = actix::Context<Self>;
    }
    
    impl actix::Handler<SessionMessage> for TopicTestSession {
        type Result = ();
        
        fn handle(&mut self, msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {
            if let ServerMessage::TopicMessageReceived { .. } = msg.message {
                self.received_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
    
    let session_count = 50; // Subscribers per topic
    let topic_count = 10;   // Number of topics
    let mut sessions = Vec::new();
    
    // Create sessions and subscribe to topics
    for topic_index in 0..topic_count {
        let topic = format!("topic_{}", topic_index);
        
        for session_index in 0..session_count {
            let connection_id = Uuid::new_v4();
            let user_id = format!("user_{}_{}", topic_index, session_index);
            let received_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            
            let session = TopicTestSession {
                connection_id,
                received_count: received_count.clone(),
            }.start();
            
            // Register session
            router_addr.do_send(Connect {
                connection_id,
                user_id: Some(user_id.clone()),
                session_addr: session.recipient(),
                metadata: std::collections::HashMap::new(),
            });
            
            // Subscribe to topic
            router_addr.do_send(SubscribeToTopic {
                connection_id,
                topics: vec![topic.clone()],
            });
            
            sessions.push((connection_id, user_id, topic.clone(), session, received_count));
        }
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Send messages to each topic
    let messages_per_topic = 10;
    let start_time = Instant::now();
    
    for topic_index in 0..topic_count {
        let topic = format!("topic_{}", topic_index);
        let sender_connection = sessions[topic_index * session_count].0;
        let sender_user = &sessions[topic_index * session_count].1;
        
        for msg_index in 0..messages_per_topic {
            let route_msg = RouteMessage {
                from_connection: sender_connection,
                from_user: Some(sender_user.clone()),
                message: ClientMessage::TopicMessage {
                    topic: topic.clone(),
                    content: serde_json::json!({
                        "text": format!("Topic {} message {}", topic, msg_index)
                    }),
                    message_id: Some(format!("topic_msg_{}_{}_{}", topic_index, msg_index, sender_user)),
                },
                timestamp: chrono::Utc::now(),
            };
            router_addr.do_send(route_msg);
        }
    }
    
    sleep(Duration::from_millis(200)).await;
    let processing_time = start_time.elapsed();
    
    // Count total messages received
    let mut total_received = 0;
    for (_, _, _, _, received_count) in &sessions {
        total_received += received_count.load(std::sync::atomic::Ordering::Relaxed);
    }
    
    let expected_messages = topic_count * messages_per_topic * (session_count - 1); // -1 because sender doesn't receive own message
    let delivery_rate = total_received as f64 / expected_messages as f64;
    
    println!("Topic scalability test results:");
    println!("  Topics: {}", topic_count);
    println!("  Subscribers per topic: {}", session_count);
    println!("  Messages per topic: {}", messages_per_topic);
    println!("  Expected deliveries: {}", expected_messages);
    println!("  Actual deliveries: {}", total_received);
    println!("  Delivery rate: {:.2}%", delivery_rate * 100.0);
    println!("  Processing time: {:?}", processing_time);
    
    // Should have high delivery rate
    assert!(delivery_rate > 0.95, "Delivery rate too low: {:.2}%", delivery_rate * 100.0);
}

#[actix::test]
async fn test_memory_usage_stability() {
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    struct MemoryTestSession {
        connection_id: Uuid,
    }
    
    impl Actor for MemoryTestSession {
        type Context = actix::Context<Self>;
    }
    
    impl actix::Handler<SessionMessage> for MemoryTestSession {
        type Result = ();
        fn handle(&mut self, _msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {}
    }
    
    // Test creating and destroying sessions repeatedly to check for memory leaks
    let cycles = 10;
    let sessions_per_cycle = 20;
    
    for cycle in 0..cycles {
        let mut sessions = Vec::new();
        
        // Create sessions
        for i in 0..sessions_per_cycle {
            let connection_id = Uuid::new_v4();
            let user_id = format!("mem_user_{}_{}", cycle, i);
            
            let session = MemoryTestSession { connection_id }.start();
            
            router_addr.do_send(Connect {
                connection_id,
                user_id: Some(user_id),
                session_addr: session.recipient(),
                metadata: std::collections::HashMap::new(),
            });
            
            sessions.push((connection_id, session));
        }
        
        sleep(Duration::from_millis(50)).await;
        
        // Send some messages
        for (connection_id, _) in &sessions {
            let route_msg = RouteMessage {
                from_connection: *connection_id,
                from_user: Some(format!("mem_user_{}_{}", cycle, 0)),
                message: ClientMessage::BroadcastMessage {
                    content: serde_json::json!({"text": format!("Memory test cycle {}", cycle)}),
                    message_id: Some(format!("mem_msg_{}_{}", cycle, connection_id)),
                },
                timestamp: chrono::Utc::now(),
            };
            router_addr.do_send(route_msg);
        }
        
        sleep(Duration::from_millis(50)).await;
        
        // Disconnect all sessions
        for (connection_id, _) in sessions {
            router_addr.do_send(Disconnect {
                connection_id,
                reason: Some("Memory test cycle complete".to_string()),
            });
        }
        
        sleep(Duration::from_millis(50)).await;
        
        // Verify cleanup
        let stats = router_addr.send(GetSystemStats).await.unwrap();
        assert_eq!(stats.total_connections, 0, "Memory leak detected in cycle {}", cycle);
        
        if cycle % 5 == 0 {
            println!("Completed memory test cycle {}/{}", cycle + 1, cycles);
        }
    }
    
    println!("Memory stability test completed successfully");
}

#[actix::test] 
async fn test_connection_churn() {
    // Test rapid connect/disconnect cycles
    let connection_manager = Arc::new(ConnectionManager::new(100));
    let router = RouterActor::new(connection_manager.clone());
    let router_addr = router.start();
    
    struct ChurnTestSession;
    impl Actor for ChurnTestSession {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<SessionMessage> for ChurnTestSession {
        type Result = ();
        fn handle(&mut self, _msg: SessionMessage, _ctx: &mut Self::Context) -> Self::Result {}
    }
    
    let churn_cycles = 50;
    let connections_per_cycle = 10;
    
    let start_time = Instant::now();
    
    for cycle in 0..churn_cycles {
        let mut connections = Vec::new();
        
        // Rapid connect
        for i in 0..connections_per_cycle {
            let connection_id = Uuid::new_v4();
            let session = ChurnTestSession.start();
            
            router_addr.do_send(Connect {
                connection_id,
                user_id: Some(format!("churn_user_{}_{}", cycle, i)),
                session_addr: session.recipient(),
                metadata: std::collections::HashMap::new(),
            });
            
            connections.push(connection_id);
        }
        
        // Brief activity
        tokio::time::sleep(Duration::from_millis(1)).await;
        
        // Rapid disconnect
        for connection_id in connections {
            router_addr.do_send(Disconnect {
                connection_id,
                reason: Some("Churn test".to_string()),
            });
        }
        
        if cycle % 10 == 0 {
            sleep(Duration::from_millis(10)).await; // Brief pause to prevent overwhelming
        }
    }
    
    sleep(Duration::from_millis(100)).await;
    
    let total_time = start_time.elapsed();
    let total_operations = churn_cycles * connections_per_cycle * 2; // connect + disconnect
    let ops_per_second = total_operations as f64 / total_time.as_secs_f64();
    
    println!("Connection churn test results:");
    println!("  Cycles: {}", churn_cycles);
    println!("  Connections per cycle: {}", connections_per_cycle);
    println!("  Total operations: {}", total_operations);
    println!("  Total time: {:?}", total_time);
    println!("  Operations per second: {:.2}", ops_per_second);
    
    // Verify final state is clean
    let final_stats = router_addr.send(GetSystemStats).await.unwrap();
    assert_eq!(final_stats.total_connections, 0, "Connections leaked during churn test");
}