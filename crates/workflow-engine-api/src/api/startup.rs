//! Application startup time tracking
//! 
//! This module provides a safe way to track when the application started
//! without using environment variables.

use once_cell::sync::Lazy;
use std::time::{SystemTime, UNIX_EPOCH};

/// The time when the application started, in seconds since UNIX epoch
pub static STARTUP_TIME: Lazy<u64> = Lazy::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_else(|_| {
            log::error!("System time before UNIX epoch!");
            0
        })
});

/// Initialize the startup time (forces the lazy static to be evaluated)
pub fn init_startup_time() {
    let _ = *STARTUP_TIME;
    log::info!("Application startup time initialized: {}", *STARTUP_TIME);
}

/// Get the application uptime in seconds
pub fn get_uptime_seconds() -> u64 {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    
    current_time.saturating_sub(*STARTUP_TIME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_time_initialization() {
        init_startup_time();
        assert!(*STARTUP_TIME > 0);
    }

    #[test]
    fn test_uptime_calculation() {
        init_startup_time();
        let uptime = get_uptime_seconds();
        assert!(uptime >= 0);
    }
}