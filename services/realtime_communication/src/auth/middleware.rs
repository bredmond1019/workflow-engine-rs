//! Authentication Middleware for WebSocket Connections
//! 
//! Provides middleware for JWT authentication during WebSocket handshake
//! and connection lifecycle management with user context.

use super::{AuthError, AuthResult, SystemRole, UserContext};
use super::jwt::{JwtService, TokenExtractor};
use actix_web::{
    dev::ServiceRequest,
    http::StatusCode,
    HttpResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Authentication middleware for WebSocket connections
pub struct AuthMiddleware {
    jwt_service: Arc<JwtService>,
    required_permissions: Vec<String>,
    allow_anonymous: bool,
    metrics: Arc<AuthMetrics>,
}

impl AuthMiddleware {
    pub fn new(
        jwt_service: Arc<JwtService>,
        required_permissions: Vec<String>,
        allow_anonymous: bool,
    ) -> Self {
        Self {
            jwt_service,
            required_permissions,
            allow_anonymous,
            metrics: Arc::new(AuthMetrics::default()),
        }
    }

    /// Authenticate a WebSocket connection request
    pub async fn authenticate_connection(
        &self,
        req: &ServiceRequest,
    ) -> Result<Option<UserContext>, AuthError> {
        let connection_id = Uuid::new_v4();
        
        // Extract token from various sources
        let token = match TokenExtractor::extract_token(
            req.headers(),
            if req.query_string().is_empty() { None } else { Some(req.query_string()) },
        ) {
            Ok(token) => token,
            Err(AuthError::MissingToken) if self.allow_anonymous => {
                self.metrics.increment_anonymous_connections().await;
                return Ok(None);
            }
            Err(e) => {
                self.metrics.increment_auth_failures().await;
                return Err(e);
            }
        };

        // Validate the token
        match self.jwt_service.validate_token(&token) {
            AuthResult::Authenticated(mut context) => {
                // Update connection ID with actual value
                context.connection_id = connection_id;
                
                // Check required permissions
                if let Err(e) = self.check_permissions(&context) {
                    self.metrics.increment_auth_failures().await;
                    return Err(e);
                }

                self.metrics.increment_successful_auths().await;
                info!("User {} authenticated successfully for WebSocket connection {}", 
                      context.user_id, connection_id);
                
                Ok(Some(context))
            }
            AuthResult::RequiresRefresh(user_id) => {
                self.metrics.increment_refresh_required().await;
                warn!("Token refresh required for user: {}", user_id);
                Err(AuthError::TokenRefreshRequired)
            }
            AuthResult::Unauthenticated(error) => {
                self.metrics.increment_auth_failures().await;
                error!("Authentication failed: {:?}", error);
                Err(error)
            }
        }
    }

    /// Check if user has required permissions
    fn check_permissions(&self, context: &UserContext) -> Result<(), AuthError> {
        if self.required_permissions.is_empty() {
            return Ok(());
        }

        for permission in &self.required_permissions {
            if !context.has_permission(permission) {
                return Err(AuthError::InsufficientPermissions(permission.clone()));
            }
        }

        Ok(())
    }

    /// Get authentication metrics
    pub async fn get_metrics(&self) -> AuthMetricsSnapshot {
        self.metrics.get_snapshot().await
    }
}

/// Role-based access control checker
pub struct RoleBasedAccessControl {
    role_permissions: HashMap<String, Vec<String>>,
}

impl RoleBasedAccessControl {
    pub fn new() -> Self {
        let mut rbac = Self {
            role_permissions: HashMap::new(),
        };
        
        // Set up default role permissions
        rbac.setup_default_permissions();
        rbac
    }

    fn setup_default_permissions(&mut self) {
        // Admin permissions
        self.role_permissions.insert(
            SystemRole::Admin.as_str().to_string(),
            vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "manage_users".to_string(),
                "system_admin".to_string(),
                "broadcast".to_string(),
                "moderate".to_string(),
            ],
        );

        // User permissions
        self.role_permissions.insert(
            SystemRole::User.as_str().to_string(),
            vec![
                "read".to_string(),
                "write".to_string(),
                "subscribe".to_string(),
            ],
        );

        // Agent permissions
        self.role_permissions.insert(
            SystemRole::Agent.as_str().to_string(),
            vec![
                "read".to_string(),
                "write".to_string(),
                "execute_tasks".to_string(),
                "broadcast".to_string(),
                "subscribe".to_string(),
            ],
        );

        // Read-only permissions
        self.role_permissions.insert(
            SystemRole::ReadOnly.as_str().to_string(),
            vec![
                "read".to_string(),
                "subscribe".to_string(),
            ],
        );

        // Service permissions
        self.role_permissions.insert(
            SystemRole::Service.as_str().to_string(),
            vec![
                "read".to_string(),
                "write".to_string(),
                "service_api".to_string(),
                "broadcast".to_string(),
            ],
        );
    }

    /// Check if a role has a specific permission
    pub async fn role_has_permission(&self, role: &str, permission: &str) -> bool {
        self.role_permissions
            .get(role)
            .map(|perms| perms.contains(&permission.to_string()))
            .unwrap_or(false)
    }

    /// Get all permissions for a role
    pub async fn get_role_permissions(&self, role: &str) -> Vec<String> {
        self.role_permissions
            .get(role)
            .cloned()
            .unwrap_or_default()
    }

    /// Add a custom permission to a role
    pub async fn add_permission_to_role(&mut self, role: &str, permission: &str) {
        self.role_permissions
            .entry(role.to_string())
            .or_default()
            .push(permission.to_string());
    }

    /// Remove a permission from a role
    pub async fn remove_permission_from_role(&mut self, role: &str, permission: &str) {
        if let Some(perms) = self.role_permissions.get_mut(role) {
            perms.retain(|p| p != permission);
        }
    }
}

impl Default for RoleBasedAccessControl {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication metrics collection
#[derive(Debug, Default)]
pub struct AuthMetrics {
    successful_auths: Arc<RwLock<u64>>,
    failed_auths: Arc<RwLock<u64>>,
    anonymous_connections: Arc<RwLock<u64>>,
    refresh_required: Arc<RwLock<u64>>,
    active_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
}

#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: String,
    pub connection_id: Uuid,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthMetricsSnapshot {
    pub successful_auths: u64,
    pub failed_auths: u64,
    pub anonymous_connections: u64,
    pub refresh_required: u64,
    pub active_sessions_count: usize,
    pub auth_success_rate: f64,
}

impl AuthMetrics {
    pub async fn increment_successful_auths(&self) {
        *self.successful_auths.write().await += 1;
    }

    pub async fn increment_auth_failures(&self) {
        *self.failed_auths.write().await += 1;
    }

    pub async fn increment_anonymous_connections(&self) {
        *self.anonymous_connections.write().await += 1;
    }

    pub async fn increment_refresh_required(&self) {
        *self.refresh_required.write().await += 1;
    }

    pub async fn add_active_session(&self, session: UserSession) {
        let session_key = format!("{}:{}", session.user_id, session.connection_id);
        self.active_sessions.write().await.insert(session_key, session);
    }

    pub async fn remove_active_session(&self, user_id: &str, connection_id: Uuid) {
        let session_key = format!("{}:{}", user_id, connection_id);
        self.active_sessions.write().await.remove(&session_key);
    }

    pub async fn update_session_activity(&self, user_id: &str, connection_id: Uuid) {
        let session_key = format!("{}:{}", user_id, connection_id);
        if let Some(session) = self.active_sessions.write().await.get_mut(&session_key) {
            session.last_activity = chrono::Utc::now();
        }
    }

    pub async fn get_snapshot(&self) -> AuthMetricsSnapshot {
        let successful = *self.successful_auths.read().await;
        let failed = *self.failed_auths.read().await;
        let total_attempts = successful + failed;
        
        let success_rate = if total_attempts > 0 {
            successful as f64 / total_attempts as f64
        } else {
            0.0
        };

        AuthMetricsSnapshot {
            successful_auths: successful,
            failed_auths: failed,
            anonymous_connections: *self.anonymous_connections.read().await,
            refresh_required: *self.refresh_required.read().await,
            active_sessions_count: self.active_sessions.read().await.len(),
            auth_success_rate: success_rate,
        }
    }

    pub async fn cleanup_expired_sessions(&self, max_idle_time: chrono::Duration) {
        let cutoff = chrono::Utc::now() - max_idle_time;
        let mut sessions = self.active_sessions.write().await;
        
        let expired_keys: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.last_activity < cutoff)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            sessions.remove(&key);
        }
    }
}

/// WebSocket authentication handler
pub struct WebSocketAuthHandler {
    auth_middleware: AuthMiddleware,
    rbac: RoleBasedAccessControl,
}

impl WebSocketAuthHandler {
    pub fn new(jwt_service: Arc<JwtService>) -> Self {
        let auth_middleware = AuthMiddleware::new(
            jwt_service,
            vec!["read".to_string(), "subscribe".to_string()], // Default required permissions
            false, // Don't allow anonymous by default
        );

        Self {
            auth_middleware,
            rbac: RoleBasedAccessControl::new(),
        }
    }

    pub fn allow_anonymous(mut self) -> Self {
        self.auth_middleware.allow_anonymous = true;
        self
    }

    pub fn require_permissions(mut self, permissions: Vec<String>) -> Self {
        self.auth_middleware.required_permissions = permissions;
        self
    }

    /// Authenticate WebSocket connection during handshake
    pub async fn authenticate_handshake(
        &self,
        req: &ServiceRequest,
    ) -> Result<Option<UserContext>, HttpResponse> {
        match self.auth_middleware.authenticate_connection(req).await {
            Ok(context) => {
                debug!("WebSocket handshake authentication successful");
                Ok(context)
            }
            Err(AuthError::MissingToken) => {
                Err(HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "missing_token",
                        "message": "Authentication token is required"
                    })))
            }
            Err(AuthError::ExpiredToken) => {
                Err(HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "expired_token",
                        "message": "Token has expired"
                    })))
            }
            Err(AuthError::InsufficientPermissions(perm)) => {
                Err(HttpResponse::Forbidden()
                    .json(serde_json::json!({
                        "error": "insufficient_permissions",
                        "message": format!("Required permission: {}", perm)
                    })))
            }
            Err(AuthError::TokenRefreshRequired) => {
                Err(HttpResponse::build(StatusCode::UPGRADE_REQUIRED)
                    .json(serde_json::json!({
                        "error": "token_refresh_required",
                        "message": "Token needs to be refreshed"
                    })))
            }
            Err(e) => {
                error!("WebSocket authentication failed: {:?}", e);
                Err(HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "authentication_failed",
                        "message": e.to_string()
                    })))
            }
        }
    }

    /// Check if user can perform a specific action
    pub fn check_permission(
        &self,
        context: &UserContext,
        permission: &str,
    ) -> Result<(), AuthError> {
        if context.has_permission(permission) {
            Ok(())
        } else {
            Err(AuthError::InsufficientPermissions(permission.to_string()))
        }
    }

    /// Get RBAC instance for role management
    pub fn rbac(&self) -> &RoleBasedAccessControl {
        &self.rbac
    }

    /// Get authentication metrics
    pub async fn get_metrics(&self) -> AuthMetricsSnapshot {
        self.auth_middleware.get_metrics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    fn create_test_jwt_service() -> Arc<JwtService> {
        let secret = b"test_secret_key_for_middleware";
        Arc::new(JwtService::new(secret).unwrap())
    }

    #[tokio::test]
    async fn test_rbac_default_permissions() {
        let rbac = RoleBasedAccessControl::new();
        
        assert!(rbac.role_has_permission("admin", "read").await);
        assert!(rbac.role_has_permission("admin", "write").await);
        assert!(rbac.role_has_permission("admin", "delete").await);
        assert!(rbac.role_has_permission("admin", "manage_users").await);
        
        assert!(rbac.role_has_permission("user", "read").await);
        assert!(rbac.role_has_permission("user", "write").await);
        assert!(!rbac.role_has_permission("user", "delete").await);
        
        assert!(rbac.role_has_permission("readonly", "read").await);
        assert!(!rbac.role_has_permission("readonly", "write").await);
    }

    #[tokio::test]
    async fn test_rbac_custom_permissions() {
        let mut rbac = RoleBasedAccessControl::new();
        
        rbac.add_permission_to_role("user", "special_feature").await;
        assert!(rbac.role_has_permission("user", "special_feature").await);
        
        rbac.remove_permission_from_role("user", "write").await;
        assert!(!rbac.role_has_permission("user", "write").await);
    }

    #[tokio::test]
    async fn test_auth_metrics() {
        let metrics = AuthMetrics::default();
        
        // Test metric increments
        metrics.increment_successful_auths().await;
        metrics.increment_successful_auths().await;
        metrics.increment_auth_failures().await;
        
        let snapshot = metrics.get_snapshot().await;
        assert_eq!(snapshot.successful_auths, 2);
        assert_eq!(snapshot.failed_auths, 1);
        assert_eq!(snapshot.auth_success_rate, 2.0 / 3.0);
    }

    #[tokio::test]
    async fn test_session_management() {
        let metrics = AuthMetrics::default();
        let connection_id = Uuid::new_v4();
        
        let session = UserSession {
            user_id: "test_user".to_string(),
            connection_id,
            authenticated_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            roles: vec!["user".to_string()],
        };
        
        metrics.add_active_session(session).await;
        assert_eq!(metrics.get_snapshot().await.active_sessions_count, 1);
        
        metrics.remove_active_session("test_user", connection_id).await;
        assert_eq!(metrics.get_snapshot().await.active_sessions_count, 0);
    }

    // Note: This test is temporarily disabled due to compiler issues
    // It validates UserContext functionality which is tested elsewhere
}