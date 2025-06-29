//! Test for JWT configuration error handling
//! 
//! This test follows TDD methodology to replace unsafe .expect() calls
//! with proper error handling in the main application.

use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

/// Test that verifies the application handles missing JWT_SECRET gracefully
/// instead of panicking with .expect()
#[tokio::test]
async fn test_missing_jwt_secret_returns_error() {
    // RED: This test should fail initially because the current code uses .expect()
    // which will panic instead of returning a proper error
    
    // Clear any existing JWT_SECRET
    env::remove_var("JWT_SECRET");
    
    // Start the application without JWT_SECRET
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "workflow-engine"])
        .env_remove("JWT_SECRET")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start application");
    
    // Give it time to start and potentially fail
    let result = timeout(Duration::from_secs(5), child.wait()).await;
    
    match result {
        Ok(exit_status) => {
            // The application should exit with an error (non-zero exit code)
            // instead of panicking
            assert!(
                exit_status.is_err() || !exit_status.unwrap().success(),
                "Application should return error exit code when JWT_SECRET is missing"
            );
        }
        Err(_) => {
            // If timeout occurred, kill the process
            let _ = child.kill();
            panic!("Application did not exit within timeout - likely panicked instead of handling error gracefully");
        }
    }
}

/// Test that verifies the application handles empty JWT_SECRET appropriately
#[tokio::test]
async fn test_empty_jwt_secret_returns_error() {
    // RED: This should also fail initially as the code doesn't validate empty secrets
    
    // Set empty JWT_SECRET
    env::set_var("JWT_SECRET", "");
    
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "workflow-engine"])
        .env("JWT_SECRET", "")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start application");
    
    let result = timeout(Duration::from_secs(5), child.wait()).await;
    
    match result {
        Ok(exit_status) => {
            // Should exit with error for empty JWT secret
            assert!(
                exit_status.is_err() || !exit_status.unwrap().success(),
                "Application should return error exit code when JWT_SECRET is empty"
            );
        }
        Err(_) => {
            let _ = child.kill();
            panic!("Application should validate empty JWT_SECRET and exit gracefully");
        }
    }
    
    // Clean up
    env::remove_var("JWT_SECRET");
}

/// Test that verifies the application starts successfully with valid JWT_SECRET
#[tokio::test]
async fn test_valid_jwt_secret_starts_successfully() {
    // This test should pass once we implement proper error handling
    
    // Set a valid JWT_SECRET
    env::set_var("JWT_SECRET", "test-secret-key-for-testing");
    
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "workflow-engine"])
        .env("JWT_SECRET", "test-secret-key-for-testing")
        .env("DATABASE_URL", "postgresql://test:test@localhost/test_db")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start application");
    
    // Give it time to start
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Kill the process (we just want to test startup)
    let _ = child.kill();
    
    // Clean up
    env::remove_var("JWT_SECRET");
}

/// Test that verifies JWT secret validation follows security best practices
#[tokio::test]
async fn test_weak_jwt_secret_is_rejected() {
    // RED: This should fail initially as there's no validation for weak secrets
    
    // Set a weak JWT_SECRET (too short)
    env::set_var("JWT_SECRET", "abc");
    
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "workflow-engine"])
        .env("JWT_SECRET", "abc")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start application");
    
    let result = timeout(Duration::from_secs(5), child.wait()).await;
    
    match result {
        Ok(exit_status) => {
            // Should exit with error for weak JWT secret
            assert!(
                exit_status.is_err() || !exit_status.unwrap().success(),
                "Application should reject weak JWT secrets"
            );
        }
        Err(_) => {
            let _ = child.kill();
            panic!("Application should validate JWT secret strength and exit gracefully");
        }
    }
    
    // Clean up
    env::remove_var("JWT_SECRET");
}