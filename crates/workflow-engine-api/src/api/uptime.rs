use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use once_cell::sync::OnceCell;

/// Service uptime tracker
#[derive(Debug, Clone)]
pub struct UptimeTracker {
    inner: Arc<Mutex<UptimeTrackerInner>>,
}

#[derive(Debug)]
struct UptimeTrackerInner {
    start_time: Instant,
    start_timestamp: DateTime<Utc>,
    restart_count: u32,
}

impl UptimeTracker {
    /// Create a new uptime tracker
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(UptimeTrackerInner {
                start_time: Instant::now(),
                start_timestamp: Utc::now(),
                restart_count: 0,
            })),
        }
    }

    /// Get the current uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        if let Ok(inner) = self.inner.lock() {
            inner.start_time.elapsed().as_secs()
        } else {
            0
        }
    }

    /// Get the current uptime as a duration string
    pub fn uptime_duration(&self) -> String {
        let seconds = self.uptime_seconds();
        format_duration(seconds)
    }

    /// Get the service start timestamp
    pub fn start_timestamp(&self) -> DateTime<Utc> {
        if let Ok(inner) = self.inner.lock() {
            inner.start_timestamp
        } else {
            Utc::now()
        }
    }

    /// Get restart count
    pub fn restart_count(&self) -> u32 {
        if let Ok(inner) = self.inner.lock() {
            inner.restart_count
        } else {
            0
        }
    }

    /// Record a service restart (for testing or when service is restarted)
    pub fn record_restart(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.restart_count += 1;
        }
    }

    /// Get detailed uptime information
    pub fn get_uptime_info(&self) -> UptimeInfo {
        if let Ok(inner) = self.inner.lock() {
            let uptime_seconds = inner.start_time.elapsed().as_secs();
            UptimeInfo {
                uptime_seconds,
                uptime_duration: format_duration(uptime_seconds),
                start_timestamp: inner.start_timestamp,
                restart_count: inner.restart_count,
            }
        } else {
            UptimeInfo {
                uptime_seconds: 0,
                uptime_duration: "0 seconds".to_string(),
                start_timestamp: Utc::now(),
                restart_count: 0,
            }
        }
    }
}

impl Default for UptimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed uptime information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UptimeInfo {
    pub uptime_seconds: u64,
    pub uptime_duration: String,
    pub start_timestamp: DateTime<Utc>,
    pub restart_count: u32,
}

/// Format duration in seconds to a human-readable string
fn format_duration(total_seconds: u64) -> String {
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut parts = Vec::new();
    
    if days > 0 {
        parts.push(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
    }
    if hours > 0 {
        parts.push(format!("{} hour{}", hours, if hours == 1 { "" } else { "s" }));
    }
    if minutes > 0 {
        parts.push(format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" }));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{} second{}", seconds, if seconds == 1 { "" } else { "s" }));
    }

    if parts.len() == 1 {
        parts[0].clone()
    } else if parts.len() == 2 {
        format!("{} and {}", parts[0], parts[1])
    } else {
        let last = parts.pop().expect("parts should not be empty when length > 2");
        format!("{}, and {}", parts.join(", "), last)
    }
}

/// Global uptime tracker instance
static GLOBAL_UPTIME_TRACKER: OnceCell<UptimeTracker> = OnceCell::new();

/// Initialize the global uptime tracker
pub fn init_uptime_tracker() -> UptimeTracker {
    GLOBAL_UPTIME_TRACKER.get_or_init(|| UptimeTracker::new()).clone()
}

/// Get the global uptime tracker
pub fn get_uptime_tracker() -> Option<UptimeTracker> {
    GLOBAL_UPTIME_TRACKER.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_uptime_tracker_creation() {
        let tracker = UptimeTracker::new();
        assert_eq!(tracker.restart_count(), 0);
        assert!(tracker.uptime_seconds() < 1); // Should be very small
    }

    #[test]
    fn test_uptime_tracking() {
        let tracker = UptimeTracker::new();
        
        // Sleep for a short time
        thread::sleep(Duration::from_millis(100));
        
        // Uptime should be at least 0 seconds
        assert!(tracker.uptime_seconds() >= 0);
        
        let info = tracker.get_uptime_info();
        assert!(info.uptime_seconds >= 0);
        assert!(!info.uptime_duration.is_empty());
    }

    #[test]
    fn test_restart_tracking() {
        let tracker = UptimeTracker::new();
        assert_eq!(tracker.restart_count(), 0);
        
        tracker.record_restart();
        assert_eq!(tracker.restart_count(), 1);
        
        tracker.record_restart();
        assert_eq!(tracker.restart_count(), 2);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0 seconds");
        assert_eq!(format_duration(1), "1 second");
        assert_eq!(format_duration(2), "2 seconds");
        assert_eq!(format_duration(60), "1 minute");
        assert_eq!(format_duration(61), "1 minute and 1 second");
        assert_eq!(format_duration(3600), "1 hour");
        assert_eq!(format_duration(3661), "1 hour, 1 minute, and 1 second");
        assert_eq!(format_duration(86400), "1 day");
        assert_eq!(format_duration(90061), "1 day, 1 hour, 1 minute, and 1 second");
    }

    #[test]
    fn test_global_uptime_tracker() {
        let tracker1 = init_uptime_tracker();
        let tracker2 = get_uptime_tracker().unwrap();
        
        // Both should refer to the same instance
        tracker1.record_restart();
        assert_eq!(tracker2.restart_count(), 1);
    }
}