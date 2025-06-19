/// Test configuration for running tests without external dependencies

use std::env;
use once_cell::sync::Lazy;

/// Test environment configuration
pub struct TestConfig {
    /// Whether to use in-memory database for tests
    pub use_in_memory_db: bool,
    
    /// Whether to use mock MCP servers
    pub use_mock_mcp: bool,
    
    /// Whether to disable external service calls
    pub disable_external_services: bool,
    
    /// Test database URL (if not using in-memory)
    pub test_database_url: Option<String>,
}

impl TestConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            use_in_memory_db: env::var("TEST_USE_IN_MEMORY_DB")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
                
            use_mock_mcp: env::var("TEST_USE_MOCK_MCP")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
                
            disable_external_services: env::var("TEST_DISABLE_EXTERNAL_SERVICES")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
                
            test_database_url: env::var("TEST_DATABASE_URL").ok(),
        }
    }
    
    /// Check if running in CI environment
    pub fn is_ci() -> bool {
        env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok()
    }
    
    /// Check if integration tests should be skipped
    pub fn skip_integration_tests() -> bool {
        env::var("SKIP_INTEGRATION_TESTS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false)
    }
}

/// Global test configuration
pub static TEST_CONFIG: Lazy<TestConfig> = Lazy::new(TestConfig::from_env);

/// Helper macro to skip tests that require external services
#[macro_export]
macro_rules! skip_without_external_services {
    () => {
        if crate::testing::test_config::TEST_CONFIG.disable_external_services {
            eprintln!("Skipping test that requires external services");
            return;
        }
    };
}

/// Helper macro to skip tests in CI environment
#[macro_export]
macro_rules! skip_in_ci {
    () => {
        if crate::testing::test_config::TestConfig::is_ci() {
            eprintln!("Skipping test in CI environment");
            return;
        }
    };
}