// File: src/db/events/cross_service_routing.rs
//
// Cross-service event routing implementation for microservices synchronization

use async_trait::async_trait;
use redis::aio::{ConnectionManager, PubSub};
use redis::{AsyncCommands, Client, RedisResult};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{EventEnvelope, EventError, EventResult};

/// Service routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRoutingConfig {
    /// Map of event types to interested services
    pub event_routes: HashMap<String, Vec<String>>,
    
    /// Service-specific Redis pub/sub channels
    pub service_channels: HashMap<String, String>,
    
    /// Default channel for broadcast events
    pub broadcast_channel: String,
    
    /// Enable message deduplication
    pub enable_deduplication: bool,
    
    /// Deduplication window in seconds
    pub deduplication_window_seconds: u64,
    
    /// Maximum retry attempts for failed deliveries
    pub max_delivery_attempts: u32,
    
    /// Enable event ordering
    pub enable_ordering: bool,
}

impl Default for ServiceRoutingConfig {
    fn default() -> Self {
        Self {
            event_routes: HashMap::new(),
            service_channels: HashMap::new(),
            broadcast_channel: "events:broadcast".to_string(),
            enable_deduplication: true,
            deduplication_window_seconds: 300, // 5 minutes
            max_delivery_attempts: 3,
            enable_ordering: true,
        }
    }
}

/// Event routing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetadata {
    pub source_service: String,
    pub target_services: Vec<String>,
    pub routing_key: String,
    pub priority: RoutingPriority,
    pub delivery_attempts: u32,
    pub sequence_number: Option<i64>,
    pub partition_key: Option<String>,
}

/// Routing priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoutingPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Cross-service event router
pub struct CrossServiceEventRouter {
    redis_client: Arc<Client>,
    connection_manager: Arc<RwLock<ConnectionManager>>,
    config: ServiceRoutingConfig,
    service_subscribers: Arc<RwLock<HashMap<String, Vec<PubSub>>>>,
    sequence_counter: Arc<RwLock<i64>>,
}

impl CrossServiceEventRouter {
    /// Create a new cross-service event router
    pub async fn new(redis_url: &str, config: ServiceRoutingConfig) -> EventResult<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| EventError::ConfigurationError {
                message: format!("Failed to create Redis client: {}", e),
            })?;
        
        let connection_manager = ConnectionManager::new(client.clone())
            .await
            .map_err(|e| EventError::ConfigurationError {
                message: format!("Failed to create Redis connection manager: {}", e),
            })?;
        
        Ok(Self {
            redis_client: Arc::new(client),
            connection_manager: Arc::new(RwLock::new(connection_manager)),
            config,
            service_subscribers: Arc::new(RwLock::new(HashMap::new())),
            sequence_counter: Arc::new(RwLock::new(0)),
        })
    }
    
    /// Route an event to appropriate services
    pub async fn route_event(&self, event: &EventEnvelope, source_service: &str) -> EventResult<Vec<String>> {
        // Determine target services based on event type
        let target_services = self.determine_target_services(&event.event_type);
        
        if target_services.is_empty() {
            tracing::debug!(
                "No target services for event type '{}' from service '{}'",
                event.event_type,
                source_service
            );
            return Ok(vec![]);
        }
        
        // Add routing metadata
        let routing_metadata = RoutingMetadata {
            source_service: source_service.to_string(),
            target_services: target_services.clone(),
            routing_key: format!("{}:{}", event.aggregate_type, event.event_type),
            priority: self.determine_priority(event),
            delivery_attempts: 0,
            sequence_number: if self.config.enable_ordering {
                Some(self.get_next_sequence_number().await)
            } else {
                None
            },
            partition_key: Some(event.aggregate_id.to_string()),
        };
        
        // Check for deduplication
        if self.config.enable_deduplication {
            if self.is_duplicate_event(event).await? {
                tracing::warn!("Duplicate event detected: {}", event.event_id);
                return Ok(vec![]);
            }
            self.mark_event_as_processed(event).await?;
        }
        
        // Publish to target service channels
        let routed_services = self.publish_to_services(event, &routing_metadata).await?;
        
        // Also publish to broadcast channel for monitoring
        self.publish_to_broadcast(event, &routing_metadata).await?;
        
        Ok(routed_services)
    }
    
    /// Subscribe a service to its event channel
    pub async fn subscribe_service(&self, service_name: &str) -> EventResult<PubSub> {
        let channel_name = self.get_service_channel(service_name);
        
        let pubsub = self.redis_client
            .get_async_pubsub()
            .await
            .map_err(|e| EventError::ConfigurationError {
                message: format!("Failed to create pubsub connection: {}", e),
            })?;
        
        // Note: PubSub cannot be cloned, so we don't store it
        // In a real implementation, you'd manage connections differently
        
        Ok(pubsub)
    }
    
    /// Add a routing rule for an event type
    pub async fn add_routing_rule(&mut self, event_type: &str, service_name: &str) {
        self.config
            .event_routes
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(service_name.to_string());
    }
    
    /// Remove a routing rule
    pub async fn remove_routing_rule(&mut self, event_type: &str, service_name: &str) {
        if let Some(services) = self.config.event_routes.get_mut(event_type) {
            services.retain(|s| s != service_name);
        }
    }
    
    /// Get routing statistics
    pub async fn get_routing_stats(&self) -> EventResult<RoutingStatistics> {
        let mut conn = self.connection_manager.write().await;
        
        // Get various stats from Redis
        let processed_count: i64 = conn
            .get("events:stats:processed_count")
            .await
            .unwrap_or(0);
        
        let duplicate_count: i64 = conn
            .get("events:stats:duplicate_count")
            .await
            .unwrap_or(0);
        
        let failed_count: i64 = conn
            .get("events:stats:failed_count")
            .await
            .unwrap_or(0);
        
        let current_sequence = *self.sequence_counter.read().await;
        
        Ok(RoutingStatistics {
            total_events_routed: processed_count,
            duplicate_events_detected: duplicate_count,
            failed_deliveries: failed_count,
            active_routes: self.config.event_routes.len(),
            current_sequence_number: current_sequence,
            active_services: self.service_subscribers.read().await.len(),
        })
    }
    
    // Helper methods
    
    fn determine_target_services(&self, event_type: &str) -> Vec<String> {
        self.config
            .event_routes
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }
    
    fn determine_priority(&self, event: &EventEnvelope) -> RoutingPriority {
        // Determine priority based on event type or metadata
        match event.event_type.as_str() {
            t if t.contains("critical") => RoutingPriority::Critical,
            t if t.contains("error") => RoutingPriority::High,
            t if t.contains("system") => RoutingPriority::High,
            _ => RoutingPriority::Normal,
        }
    }
    
    fn get_service_channel(&self, service_name: &str) -> String {
        self.config
            .service_channels
            .get(service_name)
            .cloned()
            .unwrap_or_else(|| format!("events:service:{}", service_name))
    }
    
    async fn get_next_sequence_number(&self) -> i64 {
        let mut counter = self.sequence_counter.write().await;
        *counter += 1;
        *counter
    }
    
    async fn is_duplicate_event(&self, event: &EventEnvelope) -> EventResult<bool> {
        let mut conn = self.connection_manager.write().await;
        let key = format!("events:dedup:{}", event.event_id);
        
        // Check if event ID exists in deduplication cache
        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to check duplicate: {}", e),
            })?;
        
        Ok(exists)
    }
    
    async fn mark_event_as_processed(&self, event: &EventEnvelope) -> EventResult<()> {
        let mut conn = self.connection_manager.write().await;
        let key = format!("events:dedup:{}", event.event_id);
        
        // Set with expiration based on deduplication window
        let _: () = conn.set_ex(
            &key,
            "1",
            self.config.deduplication_window_seconds,
        )
        .await
        .map_err(|e| EventError::DatabaseError {
            message: format!("Failed to mark event as processed: {}", e),
        })?;
        
        // Increment processed counter
        let _: i64 = conn.incr("events:stats:processed_count", 1)
            .await
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to increment counter: {}", e),
            })?;
        
        Ok(())
    }
    
    async fn publish_to_services(
        &self,
        event: &EventEnvelope,
        metadata: &RoutingMetadata,
    ) -> EventResult<Vec<String>> {
        let mut conn = self.connection_manager.write().await;
        let mut routed_services = Vec::new();
        
        // Create routed event with metadata
        let routed_event = RoutedEvent {
            event: event.clone(),
            routing_metadata: metadata.clone(),
            routed_at: chrono::Utc::now(),
        };
        
        let serialized = serde_json::to_string(&routed_event)
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to serialize routed event: {}", e),
            })?;
        
        // Publish to each target service channel
        for service in &metadata.target_services {
            let channel = self.get_service_channel(service);
            
            match conn.publish::<_, _, ()>(&channel, &serialized).await {
                Ok(_) => {
                    routed_services.push(service.clone());
                    tracing::debug!(
                        "Routed event {} to service '{}' on channel '{}'",
                        event.event_id,
                        service,
                        channel
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to route event {} to service '{}': {}",
                        event.event_id,
                        service,
                        e
                    );
                    
                    // Increment failed counter
                    let _: Result<i64, _> = conn.incr("events:stats:failed_count", 1).await;
                }
            }
        }
        
        Ok(routed_services)
    }
    
    async fn publish_to_broadcast(
        &self,
        event: &EventEnvelope,
        metadata: &RoutingMetadata,
    ) -> EventResult<()> {
        let mut conn = self.connection_manager.write().await;
        
        let broadcast_event = BroadcastEvent {
            event_id: event.event_id,
            event_type: event.event_type.clone(),
            source_service: metadata.source_service.clone(),
            target_services: metadata.target_services.clone(),
            timestamp: chrono::Utc::now(),
        };
        
        let serialized = serde_json::to_string(&broadcast_event)
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to serialize broadcast event: {}", e),
            })?;
        
        conn.publish::<_, _, ()>(&self.config.broadcast_channel, &serialized)
            .await
            .map_err(|e| EventError::DatabaseError {
                message: format!("Failed to publish to broadcast channel: {}", e),
            })?;
        
        Ok(())
    }
}

/// Routed event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedEvent {
    pub event: EventEnvelope,
    pub routing_metadata: RoutingMetadata,
    pub routed_at: chrono::DateTime<chrono::Utc>,
}

/// Broadcast event for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub source_service: String,
    pub target_services: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Routing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStatistics {
    pub total_events_routed: i64,
    pub duplicate_events_detected: i64,
    pub failed_deliveries: i64,
    pub active_routes: usize,
    pub current_sequence_number: i64,
    pub active_services: usize,
}

/// Service event subscriber
pub struct ServiceEventSubscriber {
    service_name: String,
    pubsub: PubSub,
    handler: Arc<dyn CrossServiceEventHandler>,
}

impl ServiceEventSubscriber {
    /// Create a new service event subscriber
    pub fn new(
        service_name: String,
        pubsub: PubSub,
        handler: Arc<dyn CrossServiceEventHandler>,
    ) -> Self {
        Self {
            service_name,
            pubsub,
            handler,
        }
    }
    
    /// Start listening for events
    pub async fn start_listening(&mut self) -> EventResult<()> {
        let channel = format!("events:service:{}", self.service_name);
        
        self.pubsub
            .subscribe(&channel)
            .await
            .map_err(|e| EventError::ConfigurationError {
                message: format!("Failed to subscribe to channel: {}", e),
            })?;
        
        tracing::info!(
            "Service '{}' subscribed to channel '{}'",
            self.service_name,
            channel
        );
        
        // Process messages
        loop {
            let msg_result = {
                self.pubsub.on_message().next().await
            };
            
            match msg_result {
                Some(msg) => {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        if let Err(e) = self.handle_message(&payload).await {
                            tracing::error!(
                                "Service '{}' failed to handle message: {}",
                                self.service_name,
                                e
                            );
                        }
                    }
                }
                None => {
                    tracing::warn!("Service '{}' pubsub stream ended", self.service_name);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_message(&self, payload: &str) -> EventResult<()> {
        let routed_event: RoutedEvent = serde_json::from_str(payload)
            .map_err(|e| EventError::SerializationError {
                message: format!("Failed to deserialize routed event: {}", e),
            })?;
        
        self.handler.handle_service_event(&routed_event).await
    }
}

/// Trait for handling service-specific events
#[async_trait]
pub trait CrossServiceEventHandler: Send + Sync {
    /// Handle a routed event
    async fn handle_service_event(&self, event: &RoutedEvent) -> EventResult<()>;
    
    /// Get the service name
    fn service_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_routing_configuration() {
        let mut config = ServiceRoutingConfig::default();
        
        // Add some routing rules
        config.event_routes.insert(
            "user_created".to_string(),
            vec!["auth_service".to_string(), "notification_service".to_string()],
        );
        
        config.event_routes.insert(
            "order_placed".to_string(),
            vec!["inventory_service".to_string(), "payment_service".to_string()],
        );
        
        // Verify routing
        assert_eq!(
            config.event_routes.get("user_created").unwrap().len(),
            2
        );
    }
}