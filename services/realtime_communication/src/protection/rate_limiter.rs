//! Token Bucket Rate Limiting Implementation
//! 
//! Provides flexible rate limiting with token bucket algorithm supporting
//! per-connection, per-user, and global rate limits with burst capacity.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{debug, info};
use dashmap::DashMap;

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_second: f64,
    pub burst_size: u32,
    pub window_size: Duration,
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_second: 100.0,
            burst_size: 200,
            window_size: Duration::from_secs(1),
            enabled: true,
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
pub struct TokenBucket {
    pub tokens: f64,
    pub last_refill: Instant,
    pub max_tokens: f64,
    pub refill_rate: f64, // tokens per second
}

impl TokenBucket {
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            last_refill: Instant::now(),
            max_tokens,
            refill_rate,
        }
    }

    /// Try to consume tokens, returns true if allowed
    pub fn try_consume(&mut self, tokens_requested: f64) -> bool {
        self.refill();
        
        if self.tokens >= tokens_requested {
            self.tokens -= tokens_requested;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = elapsed.as_secs_f64() * self.refill_rate;
        
        self.tokens = (self.tokens + tokens_to_add).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Get current token count
    pub fn current_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }

    /// Check if request would be allowed without consuming tokens
    pub fn would_allow(&mut self, tokens_requested: f64) -> bool {
        self.refill();
        self.tokens >= tokens_requested
    }
}

/// Rate limiting result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed,
    Denied {
        retry_after: Duration,
        limit_type: String,
        current_rate: f64,
        max_rate: f64,
    },
    Error(String),
}

/// Rate limiter implementation with multiple strategies
pub struct RateLimiter {
    // Per-connection rate limits
    connection_buckets: Arc<DashMap<Uuid, TokenBucket>>,
    connection_config: RateLimitConfig,
    
    // Per-user rate limits
    user_buckets: Arc<DashMap<String, TokenBucket>>,
    user_config: RateLimitConfig,
    
    // Global rate limits
    global_bucket: Arc<RwLock<TokenBucket>>,
    global_config: RateLimitConfig,
    
    // Metrics
    metrics: Arc<RateLimitMetrics>,
}

/// Rate limiting metrics
#[derive(Debug, Default)]
pub struct RateLimitMetrics {
    pub requests_allowed: Arc<RwLock<u64>>,
    pub requests_denied: Arc<RwLock<u64>>,
    pub connection_limits_hit: Arc<RwLock<u64>>,
    pub user_limits_hit: Arc<RwLock<u64>>,
    pub global_limits_hit: Arc<RwLock<u64>>,
}

impl RateLimitMetrics {
    pub async fn increment_allowed(&self) {
        *self.requests_allowed.write().await += 1;
    }

    pub async fn increment_denied(&self) {
        *self.requests_denied.write().await += 1;
    }

    pub async fn increment_connection_limits(&self) {
        *self.connection_limits_hit.write().await += 1;
    }

    pub async fn increment_user_limits(&self) {
        *self.user_limits_hit.write().await += 1;
    }

    pub async fn increment_global_limits(&self) {
        *self.global_limits_hit.write().await += 1;
    }

    pub async fn get_stats(&self) -> RateLimitStats {
        RateLimitStats {
            requests_allowed: *self.requests_allowed.read().await,
            requests_denied: *self.requests_denied.read().await,
            connection_limits_hit: *self.connection_limits_hit.read().await,
            user_limits_hit: *self.user_limits_hit.read().await,
            global_limits_hit: *self.global_limits_hit.read().await,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct RateLimitStats {
    pub requests_allowed: u64,
    pub requests_denied: u64,
    pub connection_limits_hit: u64,
    pub user_limits_hit: u64,
    pub global_limits_hit: u64,
}

impl RateLimiter {
    /// Create a new rate limiter with different limits for different scopes
    pub fn new(
        connection_config: RateLimitConfig,
        user_config: RateLimitConfig,
        global_config: RateLimitConfig,
    ) -> Self {
        let global_bucket = TokenBucket::new(
            global_config.burst_size as f64,
            global_config.max_requests_per_second,
        );

        Self {
            connection_buckets: Arc::new(DashMap::new()),
            connection_config,
            user_buckets: Arc::new(DashMap::new()),
            user_config,
            global_bucket: Arc::new(RwLock::new(global_bucket)),
            global_config,
            metrics: Arc::new(RateLimitMetrics::default()),
        }
    }

    /// Check if a request should be allowed
    pub async fn check_rate_limit(
        &self,
        connection_id: Uuid,
        user_id: Option<&str>,
        tokens_requested: f64,
    ) -> RateLimitResult {
        // Check global limit first (most restrictive)
        if self.global_config.enabled {
            let mut global_bucket = self.global_bucket.write().await;
            if !global_bucket.try_consume(tokens_requested) {
                self.metrics.increment_denied().await;
                self.metrics.increment_global_limits().await;
                
                let retry_after = Duration::from_secs_f64(
                    tokens_requested / global_bucket.refill_rate
                );
                
                return RateLimitResult::Denied {
                    retry_after,
                    limit_type: "global".to_string(),
                    current_rate: global_bucket.refill_rate - global_bucket.current_tokens(),
                    max_rate: self.global_config.max_requests_per_second,
                };
            }
        }

        // Check user-level limits
        if let Some(user_id) = user_id {
            if self.user_config.enabled {
                let mut user_bucket = self.user_buckets
                    .entry(user_id.to_string())
                    .or_insert_with(|| TokenBucket::new(
                        self.user_config.burst_size as f64,
                        self.user_config.max_requests_per_second,
                    ));

                if !user_bucket.try_consume(tokens_requested) {
                    // Need to refund global tokens since user limit hit
                    if self.global_config.enabled {
                        let mut global_bucket = self.global_bucket.write().await;
                        global_bucket.tokens = (global_bucket.tokens + tokens_requested)
                            .min(global_bucket.max_tokens);
                    }

                    self.metrics.increment_denied().await;
                    self.metrics.increment_user_limits().await;
                    
                    let retry_after = Duration::from_secs_f64(
                        tokens_requested / user_bucket.refill_rate
                    );
                    
                    return RateLimitResult::Denied {
                        retry_after,
                        limit_type: format!("user:{}", user_id),
                        current_rate: user_bucket.refill_rate - user_bucket.current_tokens(),
                        max_rate: self.user_config.max_requests_per_second,
                    };
                }
            }
        }

        // Check connection-level limits
        if self.connection_config.enabled {
            let mut connection_bucket = self.connection_buckets
                .entry(connection_id)
                .or_insert_with(|| TokenBucket::new(
                    self.connection_config.burst_size as f64,
                    self.connection_config.max_requests_per_second,
                ));

            if !connection_bucket.try_consume(tokens_requested) {
                // Refund tokens from higher levels
                if let Some(user_id) = user_id {
                    if self.user_config.enabled {
                        if let Some(mut user_bucket) = self.user_buckets.get_mut(user_id) {
                            user_bucket.tokens = (user_bucket.tokens + tokens_requested)
                                .min(user_bucket.max_tokens);
                        }
                    }
                }

                if self.global_config.enabled {
                    let mut global_bucket = self.global_bucket.write().await;
                    global_bucket.tokens = (global_bucket.tokens + tokens_requested)
                        .min(global_bucket.max_tokens);
                }

                self.metrics.increment_denied().await;
                self.metrics.increment_connection_limits().await;
                
                let retry_after = Duration::from_secs_f64(
                    tokens_requested / connection_bucket.refill_rate
                );
                
                return RateLimitResult::Denied {
                    retry_after,
                    limit_type: format!("connection:{}", connection_id),
                    current_rate: connection_bucket.refill_rate - connection_bucket.current_tokens(),
                    max_rate: self.connection_config.max_requests_per_second,
                };
            }
        }

        self.metrics.increment_allowed().await;
        debug!(
            "Rate limit check passed for connection {} (user: {:?})",
            connection_id, user_id
        );
        
        RateLimitResult::Allowed
    }

    /// Remove rate limiting state for a connection
    pub async fn remove_connection(&self, connection_id: &Uuid) {
        self.connection_buckets.remove(connection_id);
        debug!("Removed rate limiting state for connection {}", connection_id);
    }

    /// Remove rate limiting state for a user
    pub async fn remove_user(&self, user_id: &str) {
        self.user_buckets.remove(user_id);
        debug!("Removed rate limiting state for user {}", user_id);
    }

    /// Get current rate limit status for a connection
    pub async fn get_connection_status(&self, connection_id: &Uuid) -> Option<RateLimitStatus> {
        if let Some(mut bucket) = self.connection_buckets.get_mut(connection_id) {
            Some(RateLimitStatus {
                current_tokens: bucket.current_tokens(),
                max_tokens: bucket.max_tokens,
                refill_rate: bucket.refill_rate,
                last_refill: bucket.last_refill,
            })
        } else {
            None
        }
    }

    /// Get current rate limit status for a user
    pub async fn get_user_status(&self, user_id: &str) -> Option<RateLimitStatus> {
        if let Some(mut bucket) = self.user_buckets.get_mut(user_id) {
            Some(RateLimitStatus {
                current_tokens: bucket.current_tokens(),
                max_tokens: bucket.max_tokens,
                refill_rate: bucket.refill_rate,
                last_refill: bucket.last_refill,
            })
        } else {
            None
        }
    }

    /// Get global rate limit status
    pub async fn get_global_status(&self) -> RateLimitStatus {
        let mut bucket = self.global_bucket.write().await;
        RateLimitStatus {
            current_tokens: bucket.current_tokens(),
            max_tokens: bucket.max_tokens,
            refill_rate: bucket.refill_rate,
            last_refill: bucket.last_refill,
        }
    }

    /// Update rate limit configuration
    pub async fn update_config(
        &mut self,
        connection_config: Option<RateLimitConfig>,
        user_config: Option<RateLimitConfig>,
        global_config: Option<RateLimitConfig>,
    ) {
        if let Some(config) = connection_config {
            self.connection_config = config;
            info!("Updated connection rate limit configuration");
        }

        if let Some(config) = user_config {
            self.user_config = config;
            info!("Updated user rate limit configuration");
        }

        if let Some(config) = global_config {
            // Update global bucket with new config
            let mut global_bucket = self.global_bucket.write().await;
            *global_bucket = TokenBucket::new(
                config.burst_size as f64,
                config.max_requests_per_second,
            );
            self.global_config = config;
            info!("Updated global rate limit configuration");
        }
    }

    /// Clean up old buckets (should be called periodically)
    pub async fn cleanup_old_buckets(&self, max_age: Duration) {
        let cutoff = Instant::now() - max_age;
        
        // Clean up connection buckets
        let old_connections: Vec<Uuid> = self.connection_buckets
            .iter()
            .filter_map(|entry| {
                if entry.value().last_refill < cutoff {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect();

        for connection_id in old_connections {
            self.connection_buckets.remove(&connection_id);
        }

        // Clean up user buckets
        let old_users: Vec<String> = self.user_buckets
            .iter()
            .filter_map(|entry| {
                if entry.value().last_refill < cutoff {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for user_id in old_users {
            self.user_buckets.remove(&user_id);
        }

        debug!(
            "Cleaned up {} old connection buckets and {} old user buckets",
            self.connection_buckets.len(),
            self.user_buckets.len()
        );
    }

    /// Get rate limiter metrics
    pub async fn get_metrics(&self) -> RateLimitStats {
        self.metrics.get_stats().await
    }
}

/// Rate limit status for monitoring
#[derive(Debug, serde::Serialize)]
pub struct RateLimitStatus {
    pub current_tokens: f64,
    pub max_tokens: f64,
    pub refill_rate: f64,
    #[serde(skip)]
    pub last_refill: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(10.0, 5.0);
        
        // Should allow consuming available tokens
        assert!(bucket.try_consume(5.0));
        assert!(bucket.tokens < 6.0 && bucket.tokens > 4.0); // Allow some tolerance for timing
        
        // Should deny when not enough tokens
        assert!(!bucket.try_consume(10.0));
        // Tokens should remain roughly the same since we can't consume
        assert!(bucket.tokens < 6.0 && bucket.tokens > 4.0);
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10.0, 10.0); // 10 tokens per second
        
        // Consume all tokens
        assert!(bucket.try_consume(10.0));
        assert_eq!(bucket.tokens, 0.0);
        
        // Wait for refill
        sleep(Duration::from_millis(100)).await;
        bucket.refill();
        
        // Should have refilled approximately 1 token
        assert!(bucket.tokens >= 0.9 && bucket.tokens <= 1.1);
    }

    #[tokio::test]
    async fn test_rate_limiter_connection_limit() {
        let config = RateLimitConfig {
            max_requests_per_second: 5.0,
            burst_size: 5,
            window_size: Duration::from_secs(1),
            enabled: true,
        };
        
        let rate_limiter = RateLimiter::new(
            config.clone(),
            RateLimitConfig { enabled: false, ..config.clone() },
            RateLimitConfig { enabled: false, ..config },
        );
        
        let connection_id = Uuid::new_v4();
        
        // Should allow up to burst size
        for _ in 0..5 {
            match rate_limiter.check_rate_limit(connection_id, None, 1.0).await {
                RateLimitResult::Allowed => {}
                other => panic!("Expected allowed, got {:?}", other),
            }
        }
        
        // Should deny next request
        match rate_limiter.check_rate_limit(connection_id, None, 1.0).await {
            RateLimitResult::Denied { limit_type, .. } => {
                assert!(limit_type.starts_with("connection:"));
            }
            other => panic!("Expected denied, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_user_limit() {
        let config = RateLimitConfig {
            max_requests_per_second: 10.0,
            burst_size: 5,
            window_size: Duration::from_secs(1),
            enabled: true,
        };
        
        let rate_limiter = RateLimiter::new(
            RateLimitConfig { enabled: false, ..config.clone() },
            config.clone(),
            RateLimitConfig { enabled: false, ..config },
        );
        
        let connection_id1 = Uuid::new_v4();
        let connection_id2 = Uuid::new_v4();
        let user_id = "test_user";
        
        // Consume user quota with first connection
        for _ in 0..5 {
            match rate_limiter.check_rate_limit(connection_id1, Some(user_id), 1.0).await {
                RateLimitResult::Allowed => {}
                other => panic!("Expected allowed, got {:?}", other),
            }
        }
        
        // Should deny request from second connection for same user
        match rate_limiter.check_rate_limit(connection_id2, Some(user_id), 1.0).await {
            RateLimitResult::Denied { limit_type, .. } => {
                assert!(limit_type.starts_with("user:"));
            }
            other => panic!("Expected denied, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_global_limit() {
        let config = RateLimitConfig {
            max_requests_per_second: 10.0,
            burst_size: 3,
            window_size: Duration::from_secs(1),
            enabled: true,
        };
        
        let rate_limiter = RateLimiter::new(
            RateLimitConfig { enabled: false, ..config.clone() },
            RateLimitConfig { enabled: false, ..config.clone() },
            config,
        );
        
        let connection_id1 = Uuid::new_v4();
        let connection_id2 = Uuid::new_v4();
        
        // Consume global quota
        for _ in 0..3 {
            match rate_limiter.check_rate_limit(connection_id1, Some("user1"), 1.0).await {
                RateLimitResult::Allowed => {}
                other => panic!("Expected allowed, got {:?}", other),
            }
        }
        
        // Should deny request from any connection due to global limit
        match rate_limiter.check_rate_limit(connection_id2, Some("user2"), 1.0).await {
            RateLimitResult::Denied { limit_type, .. } => {
                assert_eq!(limit_type, "global");
            }
            other => panic!("Expected denied, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_cleanup() {
        let config = RateLimitConfig::default();
        let rate_limiter = RateLimiter::new(config.clone(), config.clone(), config);
        
        let connection_id = Uuid::new_v4();
        
        // Create some rate limit state
        rate_limiter.check_rate_limit(connection_id, Some("test_user"), 1.0).await;
        
        assert!(rate_limiter.connection_buckets.contains_key(&connection_id));
        assert!(rate_limiter.user_buckets.contains_key("test_user"));
        
        // Cleanup should remove old buckets
        rate_limiter.cleanup_old_buckets(Duration::from_millis(1)).await;
        
        // Give cleanup time to run
        sleep(Duration::from_millis(10)).await;
        
        // Note: In a real test, we'd need to wait longer or manipulate timestamps
        // This test mainly verifies the cleanup method doesn't panic
    }
}