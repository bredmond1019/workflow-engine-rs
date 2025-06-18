//! Authentication module for JWT-based access control
//! 
//! Provides JWT validation, role-based access control, and user context
//! management for WebSocket connections.

pub mod jwt;
pub mod middleware;

pub use jwt::*;
pub use middleware::*;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// User authentication context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub connection_id: Uuid,
    pub roles: HashSet<String>,
    pub permissions: HashSet<String>,
    pub session_id: Option<String>,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UserContext {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has all of the specified permissions
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|perm| self.has_permission(perm))
    }

    /// Check if the user context is still valid
    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() < expires_at
        } else {
            true
        }
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub enum AuthResult {
    Authenticated(UserContext),
    Unauthenticated(AuthError),
    RequiresRefresh(String), // token needs refresh
}

/// Authentication errors
#[derive(Debug, Clone, Serialize)]
pub enum AuthError {
    InvalidToken(String),
    ExpiredToken,
    MissingToken,
    InsufficientPermissions(String),
    TokenRefreshRequired,
    InvalidClaims(String),
    SignatureValidationFailed,
    UnknownUser(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AuthError::ExpiredToken => write!(f, "Token has expired"),
            AuthError::MissingToken => write!(f, "Authentication token is required"),
            AuthError::InsufficientPermissions(perm) => write!(f, "Insufficient permissions: {}", perm),
            AuthError::TokenRefreshRequired => write!(f, "Token refresh is required"),
            AuthError::InvalidClaims(msg) => write!(f, "Invalid token claims: {}", msg),
            AuthError::SignatureValidationFailed => write!(f, "Token signature validation failed"),
            AuthError::UnknownUser(user_id) => write!(f, "Unknown user: {}", user_id),
        }
    }
}

impl std::error::Error for AuthError {}

/// Role definitions for the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemRole {
    Admin,
    User,
    Agent,
    ReadOnly,
    Service,
}

impl SystemRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemRole::Admin => "admin",
            SystemRole::User => "user",
            SystemRole::Agent => "agent",
            SystemRole::ReadOnly => "readonly",
            SystemRole::Service => "service",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(SystemRole::Admin),
            "user" => Some(SystemRole::User),
            "agent" => Some(SystemRole::Agent),
            "readonly" => Some(SystemRole::ReadOnly),
            "service" => Some(SystemRole::Service),
            _ => None,
        }
    }

    /// Get default permissions for this role
    pub fn default_permissions(&self) -> HashSet<String> {
        let mut perms = HashSet::new();
        
        match self {
            SystemRole::Admin => {
                perms.insert("read".to_string());
                perms.insert("write".to_string());
                perms.insert("delete".to_string());
                perms.insert("manage_users".to_string());
                perms.insert("system_admin".to_string());
            }
            SystemRole::User => {
                perms.insert("read".to_string());
                perms.insert("write".to_string());
            }
            SystemRole::Agent => {
                perms.insert("read".to_string());
                perms.insert("write".to_string());
                perms.insert("execute_tasks".to_string());
            }
            SystemRole::ReadOnly => {
                perms.insert("read".to_string());
            }
            SystemRole::Service => {
                perms.insert("read".to_string());
                perms.insert("write".to_string());
                perms.insert("service_api".to_string());
            }
        }
        
        perms
    }
}

/// Permission check result
#[derive(Debug, Clone)]
pub enum PermissionResult {
    Allowed,
    Denied { required: String, user_roles: Vec<String> },
    Unauthenticated,
}

impl PermissionResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, PermissionResult::Allowed)
    }
}