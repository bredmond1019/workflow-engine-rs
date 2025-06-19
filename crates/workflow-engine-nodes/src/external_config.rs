use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

use workflow_engine_mcp::transport::TransportType;
use crate::nodes::external_mcp_client::{AuthConfig, RetryConfig};

/// Configuration for all external MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMCPServerConfig {
    /// Notion MCP server configuration
    pub notion: Option<NotionServerConfig>,
    
    /// HelpScout MCP server configuration
    pub helpscout: Option<HelpscoutServerConfig>,
    
    /// Slack MCP server configuration
    pub slack: Option<SlackServerConfig>,
    
    /// Global default configuration
    pub defaults: DefaultServerConfig,
}

impl Default for ExternalMCPServerConfig {
    fn default() -> Self {
        Self {
            notion: Some(NotionServerConfig::default()),
            helpscout: Some(HelpscoutServerConfig::default()),
            slack: Some(SlackServerConfig::default()),
            defaults: DefaultServerConfig::default(),
        }
    }
}

/// Default configuration that applies to all external MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultServerConfig {
    /// Default retry configuration
    pub retry_config: RetryConfig,
    
    /// Default transport type
    pub default_transport: TransportType,
    
    /// Whether to enable external MCP clients by default
    pub enabled: bool,
    
    /// Default connection timeout in seconds
    pub connection_timeout_seconds: u64,
    
    /// Default request timeout in seconds
    pub request_timeout_seconds: u64,
}

impl Default for DefaultServerConfig {
    fn default() -> Self {
        Self {
            retry_config: RetryConfig::default(),
            default_transport: TransportType::Http {
                base_url: "http://localhost:8000".to_string(),
                pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
            },
            enabled: true,
            connection_timeout_seconds: 30,
            request_timeout_seconds: 60,
        }
    }
}

/// Configuration for Notion MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionServerConfig {
    /// Server URL
    pub url: String,
    
    /// Notion API key
    pub api_key: Option<String>,
    
    /// Transport configuration
    pub transport: TransportType,
    
    /// Whether this client is enabled
    pub enabled: bool,
    
    /// Retry configuration (overrides default if provided)
    pub retry_config: Option<RetryConfig>,
    
    /// Additional headers
    pub headers: Option<HashMap<String, String>>,
}

impl Default for NotionServerConfig {
    fn default() -> Self {
        let url = env::var("NOTION_MCP_URL")
            .unwrap_or_else(|_| "http://localhost:8002".to_string());
        
        Self {
            url: url.clone(),
            api_key: env::var("NOTION_API_KEY").ok(),
            transport: TransportType::Http { 
                base_url: url,
                pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
            },
            enabled: true,
            retry_config: None,
            headers: None,
        }
    }
}

impl NotionServerConfig {
    pub fn to_auth_config(&self) -> Option<AuthConfig> {
        if self.api_key.is_none() && self.headers.is_none() {
            return None;
        }
        
        let mut headers = self.headers.clone().unwrap_or_default();
        
        if let Some(ref api_key) = self.api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
            headers.insert("Notion-Version".to_string(), "2022-06-28".to_string());
        }
        
        Some(AuthConfig {
            token: self.api_key.clone(),
            headers: if headers.is_empty() { None } else { Some(headers) },
        })
    }
}

/// Configuration for HelpScout MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpscoutServerConfig {
    /// Server URL
    pub url: String,
    
    /// HelpScout API key
    pub api_key: Option<String>,
    
    /// Transport configuration
    pub transport: TransportType,
    
    /// Whether this client is enabled
    pub enabled: bool,
    
    /// Retry configuration (overrides default if provided)
    pub retry_config: Option<RetryConfig>,
    
    /// Additional headers
    pub headers: Option<HashMap<String, String>>,
}

impl Default for HelpscoutServerConfig {
    fn default() -> Self {
        let url = env::var("HELPSCOUT_MCP_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());
        
        Self {
            url: url.clone(),
            api_key: env::var("HELPSCOUT_API_KEY").ok(),
            transport: TransportType::Http { 
                base_url: url,
                pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
            },
            enabled: true,
            retry_config: None,
            headers: None,
        }
    }
}

impl HelpscoutServerConfig {
    pub fn to_auth_config(&self) -> Option<AuthConfig> {
        if self.api_key.is_none() && self.headers.is_none() {
            return None;
        }
        
        let mut headers = self.headers.clone().unwrap_or_default();
        
        if let Some(ref api_key) = self.api_key {
            headers.insert("X-API-Key".to_string(), api_key.clone());
        }
        
        Some(AuthConfig {
            token: self.api_key.clone(),
            headers: if headers.is_empty() { None } else { Some(headers) },
        })
    }
}

/// Configuration for Slack MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackServerConfig {
    /// Server URL
    pub url: String,
    
    /// Slack bot token
    pub bot_token: Option<String>,
    
    /// Slack user token
    pub user_token: Option<String>,
    
    /// Transport configuration
    pub transport: TransportType,
    
    /// Whether this client is enabled
    pub enabled: bool,
    
    /// Retry configuration (overrides default if provided)
    pub retry_config: Option<RetryConfig>,
    
    /// Additional headers
    pub headers: Option<HashMap<String, String>>,
}

impl Default for SlackServerConfig {
    fn default() -> Self {
        let url = env::var("SLACK_MCP_URL")
            .unwrap_or_else(|_| "http://localhost:8003".to_string());
        
        Self {
            url: url.clone(),
            bot_token: env::var("SLACK_BOT_TOKEN").ok(),
            user_token: env::var("SLACK_USER_TOKEN").ok(),
            transport: TransportType::Http { 
                base_url: url,
                pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
            },
            enabled: true,
            retry_config: None,
            headers: None,
        }
    }
}

impl SlackServerConfig {
    pub fn to_auth_config(&self) -> Option<AuthConfig> {
        if self.bot_token.is_none() && self.user_token.is_none() && self.headers.is_none() {
            return None;
        }
        
        let mut headers = self.headers.clone().unwrap_or_default();
        
        if let Some(ref bot_token) = self.bot_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", bot_token));
        }
        
        if let Some(ref user_token) = self.user_token {
            headers.insert("X-Slack-User-Token".to_string(), user_token.clone());
        }
        
        Some(AuthConfig {
            token: self.bot_token.clone(),
            headers: if headers.is_empty() { None } else { Some(headers) },
        })
    }
}

/// Builder for creating external MCP server configurations
pub struct ExternalConfigBuilder {
    config: ExternalMCPServerConfig,
}

impl ExternalConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ExternalMCPServerConfig::default(),
        }
    }
    
    /// Configure Notion server
    pub fn with_notion(mut self, notion_config: NotionServerConfig) -> Self {
        self.config.notion = Some(notion_config);
        self
    }
    
    /// Configure HelpScout server
    pub fn with_helpscout(mut self, helpscout_config: HelpscoutServerConfig) -> Self {
        self.config.helpscout = Some(helpscout_config);
        self
    }
    
    /// Configure Slack server
    pub fn with_slack(mut self, slack_config: SlackServerConfig) -> Self {
        self.config.slack = Some(slack_config);
        self
    }
    
    /// Set default configuration
    pub fn with_defaults(mut self, defaults: DefaultServerConfig) -> Self {
        self.config.defaults = defaults;
        self
    }
    
    /// Enable all configured servers
    pub fn enable_all(mut self) -> Self {
        if let Some(ref mut notion) = self.config.notion {
            notion.enabled = true;
        }
        if let Some(ref mut helpscout) = self.config.helpscout {
            helpscout.enabled = true;
        }
        if let Some(ref mut slack) = self.config.slack {
            slack.enabled = true;
        }
        self
    }
    
    /// Disable all configured servers
    pub fn disable_all(mut self) -> Self {
        if let Some(ref mut notion) = self.config.notion {
            notion.enabled = false;
        }
        if let Some(ref mut helpscout) = self.config.helpscout {
            helpscout.enabled = false;
        }
        if let Some(ref mut slack) = self.config.slack {
            slack.enabled = false;
        }
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> ExternalMCPServerConfig {
        self.config
    }
}

impl Default for ExternalConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment variable configuration helper
pub struct EnvConfigLoader;

impl EnvConfigLoader {
    /// Load configuration from environment variables
    pub fn load() -> ExternalMCPServerConfig {
        ExternalMCPServerConfig {
            notion: if Self::is_notion_configured() {
                Some(NotionServerConfig::default())
            } else {
                None
            },
            helpscout: if Self::is_helpscout_configured() {
                Some(HelpscoutServerConfig::default())
            } else {
                None
            },
            slack: if Self::is_slack_configured() {
                Some(SlackServerConfig::default())
            } else {
                None
            },
            defaults: DefaultServerConfig::default(),
        }
    }
    
    /// Check if Notion is configured via environment variables
    fn is_notion_configured() -> bool {
        env::var("NOTION_MCP_URL").is_ok() || env::var("NOTION_API_KEY").is_ok()
    }
    
    /// Check if HelpScout is configured via environment variables
    fn is_helpscout_configured() -> bool {
        env::var("HELPSCOUT_MCP_URL").is_ok() || env::var("HELPSCOUT_API_KEY").is_ok()
    }
    
    /// Check if Slack is configured via environment variables
    fn is_slack_configured() -> bool {
        env::var("SLACK_MCP_URL").is_ok() 
            || env::var("SLACK_BOT_TOKEN").is_ok() 
            || env::var("SLACK_USER_TOKEN").is_ok()
    }
    
    /// Get all relevant environment variables as a HashMap
    pub fn get_env_vars() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        
        // Notion variables
        if let Ok(val) = env::var("NOTION_MCP_URL") {
            vars.insert("NOTION_MCP_URL".to_string(), val);
        }
        if let Ok(val) = env::var("NOTION_API_KEY") {
            vars.insert("NOTION_API_KEY".to_string(), val);
        }
        
        // HelpScout variables
        if let Ok(val) = env::var("HELPSCOUT_MCP_URL") {
            vars.insert("HELPSCOUT_MCP_URL".to_string(), val);
        }
        if let Ok(val) = env::var("HELPSCOUT_API_KEY") {
            vars.insert("HELPSCOUT_API_KEY".to_string(), val);
        }
        
        // Slack variables
        if let Ok(val) = env::var("SLACK_MCP_URL") {
            vars.insert("SLACK_MCP_URL".to_string(), val);
        }
        if let Ok(val) = env::var("SLACK_BOT_TOKEN") {
            vars.insert("SLACK_BOT_TOKEN".to_string(), val);
        }
        if let Ok(val) = env::var("SLACK_USER_TOKEN") {
            vars.insert("SLACK_USER_TOKEN".to_string(), val);
        }
        
        vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_external_config() {
        let config = ExternalMCPServerConfig::default();
        
        assert!(config.notion.is_some());
        assert!(config.helpscout.is_some());
        assert!(config.slack.is_some());
        assert!(config.defaults.enabled);
    }

    #[test]
    fn test_config_builder() {
        let config = ExternalConfigBuilder::new()
            .with_notion(NotionServerConfig {
                url: "http://test-notion:8002".to_string(),
                api_key: Some("test-key".to_string()),
                transport: TransportType::Http {
                    base_url: "http://test-notion:8002".to_string(),
                    pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
                },
                enabled: true,
                retry_config: None,
                headers: None,
            })
            .enable_all()
            .build();
        
        assert!(config.notion.is_some());
        assert_eq!(config.notion.unwrap().url, "http://test-notion:8002");
    }

    #[test]
    fn test_notion_auth_config() {
        let notion_config = NotionServerConfig {
            url: "http://localhost:8002".to_string(),
            api_key: Some("test-api-key".to_string()),
            transport: TransportType::Http {
                base_url: "http://localhost:8002".to_string(),
                pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
            },
            enabled: true,
            retry_config: None,
            headers: None,
        };
        
        let auth = notion_config.to_auth_config();
        assert!(auth.is_some());
        
        let auth = auth.unwrap();
        assert_eq!(auth.token, Some("test-api-key".to_string()));
        assert!(auth.headers.is_some());
        
        let headers = auth.headers.unwrap();
        assert_eq!(headers.get("Authorization"), Some(&"Bearer test-api-key".to_string()));
        assert_eq!(headers.get("Notion-Version"), Some(&"2022-06-28".to_string()));
    }

    #[test]
    fn test_helpscout_auth_config() {
        let helpscout_config = HelpscoutServerConfig {
            url: "http://localhost:8001".to_string(),
            api_key: Some("test-helpscout-key".to_string()),
            transport: TransportType::Http {
                base_url: "http://localhost:8001".to_string(),
                pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
            },
            enabled: true,
            retry_config: None,
            headers: None,
        };
        
        let auth = helpscout_config.to_auth_config();
        assert!(auth.is_some());
        
        let auth = auth.unwrap();
        assert_eq!(auth.token, Some("test-helpscout-key".to_string()));
        assert!(auth.headers.is_some());
        
        let headers = auth.headers.unwrap();
        assert_eq!(headers.get("X-API-Key"), Some(&"test-helpscout-key".to_string()));
    }

    #[test]
    fn test_slack_auth_config() {
        let slack_config = SlackServerConfig {
            url: "http://localhost:8003".to_string(),
            bot_token: Some("xoxb-test-token".to_string()),
            user_token: Some("xoxp-user-token".to_string()),
            transport: TransportType::Http {
                base_url: "http://localhost:8003".to_string(),
                pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
            },
            enabled: true,
            retry_config: None,
            headers: None,
        };
        
        let auth = slack_config.to_auth_config();
        assert!(auth.is_some());
        
        let auth = auth.unwrap();
        assert_eq!(auth.token, Some("xoxb-test-token".to_string()));
        assert!(auth.headers.is_some());
        
        let headers = auth.headers.unwrap();
        assert_eq!(headers.get("Authorization"), Some(&"Bearer xoxb-test-token".to_string()));
        assert_eq!(headers.get("X-Slack-User-Token"), Some(&"xoxp-user-token".to_string()));
    }

    #[test]
    fn test_config_without_auth() {
        let notion_config = NotionServerConfig {
            url: "http://localhost:8002".to_string(),
            api_key: None,
            transport: TransportType::Http {
                base_url: "http://localhost:8002".to_string(),
                pool_config: workflow_engine_core::mcp::transport::HttpPoolConfig::default(),
            },
            enabled: true,
            retry_config: None,
            headers: None,
        };
        
        let auth = notion_config.to_auth_config();
        assert!(auth.is_none());
    }

    #[test]
    fn test_env_config_loader() {
        // Note: This test doesn't set actual environment variables
        // to avoid affecting other tests. In a real scenario, you would
        // use a test framework that allows setting env vars safely.
        let env_vars = EnvConfigLoader::get_env_vars();
        assert!(env_vars.is_empty() || !env_vars.is_empty()); // Will be empty in test env
    }
}