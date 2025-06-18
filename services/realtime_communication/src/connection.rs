//! WebSocket connection management
//! 
//! Handles WebSocket connection lifecycle, state tracking, and management
//! for scalable real-time communication.

use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, error};
use dashmap::DashMap;
use serde::{Serialize, Deserialize};

/// Connection state enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
    Error,
}

/// Connection information
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub state: ConnectionState,
    #[serde(skip)]
    pub connected_at: Instant,
    #[serde(skip)]
    pub last_activity: Instant,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub protocol_version: String,
    pub subscriptions: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl ConnectionInfo {
    pub fn new(id: Uuid) -> Self {
        let now = Instant::now();
        Self {
            id,
            user_id: None,
            state: ConnectionState::Connecting,
            connected_at: now,
            last_activity: now,
            ip_address: None,
            user_agent: None,
            protocol_version: "1.0".to_string(),
            subscriptions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn mark_connected(&mut self) {
        self.state = ConnectionState::Connected;
        self.last_activity = Instant::now();
    }

    pub fn mark_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn mark_disconnecting(&mut self) {
        self.state = ConnectionState::Disconnecting;
    }

    pub fn mark_disconnected(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    pub fn mark_error(&mut self) {
        self.state = ConnectionState::Error;
    }

    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    pub fn add_subscription(&mut self, topic: String) {
        if !self.subscriptions.contains(&topic) {
            self.subscriptions.push(topic);
        }
    }

    pub fn remove_subscription(&mut self, topic: &str) {
        self.subscriptions.retain(|t| t != topic);
    }

    pub fn is_subscribed_to(&self, topic: &str) -> bool {
        self.subscriptions.contains(&topic.to_string())
    }
}

/// Connection manager for handling multiple WebSocket connections
pub struct ConnectionManager {
    connections: Arc<DashMap<Uuid, ConnectionInfo>>,
    user_connections: Arc<DashMap<String, Vec<Uuid>>>,
    max_connections: usize,
    connection_count: Arc<RwLock<usize>>,
}

impl ConnectionManager {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            user_connections: Arc::new(DashMap::new()),
            max_connections,
            connection_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Add a new connection
    pub async fn add_connection(&self, mut connection_info: ConnectionInfo) -> Result<(), ConnectionError> {
        let current_count = *self.connection_count.read().await;
        if current_count >= self.max_connections {
            return Err(ConnectionError::CapacityReached);
        }

        connection_info.mark_connected();
        let connection_id = connection_info.id;
        let user_id = connection_info.user_id.clone();

        // Add to connections map
        self.connections.insert(connection_id, connection_info);

        // Add to user connections map if user_id exists
        if let Some(user_id) = user_id {
            self.user_connections
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(connection_id);
        }

        // Update connection count
        let mut count = self.connection_count.write().await;
        *count += 1;

        info!("Connection {} added, total connections: {}", connection_id, *count);
        Ok(())
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_id: &Uuid) -> Option<ConnectionInfo> {
        if let Some((_, mut connection_info)) = self.connections.remove(connection_id) {
            connection_info.mark_disconnected();

            // Remove from user connections map
            if let Some(user_id) = &connection_info.user_id {
                if let Some(mut user_connections) = self.user_connections.get_mut(user_id) {
                    user_connections.retain(|id| id != connection_id);
                    if user_connections.is_empty() {
                        drop(user_connections);
                        self.user_connections.remove(user_id);
                    }
                }
            }

            // Update connection count
            let mut count = self.connection_count.write().await;
            if *count > 0 {
                *count -= 1;
            }

            info!("Connection {} removed, total connections: {}", connection_id, *count);
            Some(connection_info)
        } else {
            None
        }
    }

    /// Get connection information
    pub async fn get_connection(&self, connection_id: &Uuid) -> Option<ConnectionInfo> {
        self.connections.get(connection_id).map(|entry| entry.clone())
    }

    /// Update connection activity
    pub async fn update_activity(&self, connection_id: &Uuid) {
        if let Some(mut connection) = self.connections.get_mut(connection_id) {
            connection.mark_activity();
        }
    }

    /// Get connections for a specific user
    pub async fn get_user_connections(&self, user_id: &str) -> Vec<Uuid> {
        self.user_connections
            .get(user_id)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Get all active connections
    pub async fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        self.connections
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get connections that have timed out
    pub async fn get_timed_out_connections(&self, _now: Instant, timeout: Duration) -> Vec<Uuid> {
        self.connections
            .iter()
            .filter_map(|entry| {
                let connection = entry.value();
                if connection.is_timed_out(timeout) {
                    Some(connection.id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get current connection count
    pub async fn get_connection_count(&self) -> Arc<RwLock<usize>> {
        self.connection_count.clone()
    }

    /// Subscribe connection to topic
    pub async fn subscribe_to_topic(&self, connection_id: &Uuid, topic: String) -> Result<(), ConnectionError> {
        if let Some(mut connection) = self.connections.get_mut(connection_id) {
            connection.add_subscription(topic);
            Ok(())
        } else {
            Err(ConnectionError::NotFound)
        }
    }

    /// Unsubscribe connection from topic
    pub async fn unsubscribe_from_topic(&self, connection_id: &Uuid, topic: &str) -> Result<(), ConnectionError> {
        if let Some(mut connection) = self.connections.get_mut(connection_id) {
            connection.remove_subscription(topic);
            Ok(())
        } else {
            Err(ConnectionError::NotFound)
        }
    }

    /// Get connections subscribed to a topic
    pub async fn get_topic_subscribers(&self, topic: &str) -> Vec<Uuid> {
        self.connections
            .iter()
            .filter_map(|entry| {
                let connection = entry.value();
                if connection.is_subscribed_to(topic) {
                    Some(connection.id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        let total_connections = *self.connection_count.read().await;
        let active_connections = self.connections
            .iter()
            .filter(|entry| entry.value().state == ConnectionState::Connected)
            .count();

        let user_count = self.user_connections.len();

        ConnectionStats {
            total_connections,
            active_connections,
            user_count,
            max_connections: self.max_connections,
        }
    }
}

/// Connection statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub user_count: usize,
    pub max_connections: usize,
}

/// Connection errors
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Connection capacity reached")]
    CapacityReached,
    #[error("Connection not found")]
    NotFound,
    #[error("Invalid connection state")]
    InvalidState,
    #[error("Connection error: {0}")]
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_info_lifecycle() {
        let id = Uuid::new_v4();
        let mut info = ConnectionInfo::new(id);
        
        assert_eq!(info.state, ConnectionState::Connecting);
        
        info.mark_connected();
        assert_eq!(info.state, ConnectionState::Connected);
        
        info.mark_disconnecting();
        assert_eq!(info.state, ConnectionState::Disconnecting);
        
        info.mark_disconnected();
        assert_eq!(info.state, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_connection_manager() {
        let manager = ConnectionManager::new(100);
        let id = Uuid::new_v4();
        let mut info = ConnectionInfo::new(id);
        info.user_id = Some("user123".to_string());
        
        // Add connection
        manager.add_connection(info.clone()).await.unwrap();
        
        // Verify connection exists
        let retrieved = manager.get_connection(&id).await.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.state, ConnectionState::Connected);
        
        // Verify user connections
        let user_connections = manager.get_user_connections("user123").await;
        assert_eq!(user_connections.len(), 1);
        assert_eq!(user_connections[0], id);
        
        // Remove connection
        let removed = manager.remove_connection(&id).await.unwrap();
        assert_eq!(removed.state, ConnectionState::Disconnected);
        
        // Verify connection is removed
        assert!(manager.get_connection(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let manager = ConnectionManager::new(100);
        let id = Uuid::new_v4();
        let info = ConnectionInfo::new(id);
        
        manager.add_connection(info).await.unwrap();
        
        // Subscribe to topic
        manager.subscribe_to_topic(&id, "test_topic".to_string()).await.unwrap();
        
        // Verify subscription
        let subscribers = manager.get_topic_subscribers("test_topic").await;
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], id);
        
        // Unsubscribe from topic
        manager.unsubscribe_from_topic(&id, "test_topic").await.unwrap();
        
        // Verify unsubscription
        let subscribers = manager.get_topic_subscribers("test_topic").await;
        assert_eq!(subscribers.len(), 0);
    }
}