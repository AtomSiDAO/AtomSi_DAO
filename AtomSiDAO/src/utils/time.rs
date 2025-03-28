//! Time utilities for the AtomSi DAO
//!
//! This module provides time-related utility functions.

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current timestamp in seconds since the Unix epoch
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Convert a timestamp in seconds to a DateTime<Utc>
pub fn timestamp_to_datetime(timestamp: u64) -> DateTime<Utc> {
    let naive = NaiveDateTime::from_timestamp_opt(timestamp as i64, 0)
        .expect("Invalid timestamp");
    
    DateTime::from_utc(naive, Utc)
}

/// Format a timestamp in a human-readable format
pub fn format_timestamp(timestamp: u64) -> String {
    let datetime = timestamp_to_datetime(timestamp);
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Calculate time difference in a human-readable format
pub fn time_diff(from: u64, to: u64) -> String {
    if from > to {
        return format_duration(from - to) + " ago";
    } else if from < to {
        return "in " + &format_duration(to - from);
    } else {
        return "now".to_string();
    }
}

/// Format a duration in a human-readable format
pub fn format_duration(seconds: u64) -> String {
    let duration = Duration::seconds(seconds as i64);
    
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let secs = duration.num_seconds() % 60;
    
    if days > 0 {
        if days == 1 {
            return "1 day".to_string();
        } else {
            return format!("{} days", days);
        }
    } else if hours > 0 {
        if hours == 1 {
            return "1 hour".to_string();
        } else {
            return format!("{} hours", hours);
        }
    } else if minutes > 0 {
        if minutes == 1 {
            return "1 minute".to_string();
        } else {
            return format!("{} minutes", minutes);
        }
    } else {
        if secs <= 1 {
            return "1 second".to_string();
        } else {
            return format!("{} seconds", secs);
        }
    }
}

/// Check if a timestamp is expired
pub fn is_expired(timestamp: u64) -> bool {
    current_timestamp() > timestamp
}

/// Add seconds to a timestamp
pub fn add_seconds(timestamp: u64, seconds: u64) -> u64 {
    timestamp + seconds
}

/// Add days to a timestamp
pub fn add_days(timestamp: u64, days: u64) -> u64 {
    timestamp + (days * 24 * 60 * 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_current_timestamp() {
        // Just check that it returns a reasonable value (after year 2020)
        assert!(current_timestamp() > 1577836800); // 2020-01-01
    }
    
    #[test]
    fn test_timestamp_to_datetime() {
        // Test a known timestamp (2021-01-01 00:00:00 UTC)
        let dt = timestamp_to_datetime(1609459200);
        assert_eq!(dt.year(), 2021);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }
    
    #[test]
    fn test_format_timestamp() {
        // Test a known timestamp (2021-01-01 00:00:00 UTC)
        let formatted = format_timestamp(1609459200);
        assert_eq!(formatted, "2021-01-01 00:00:00 UTC");
    }
    
    #[test]
    fn test_time_diff() {
        // Test time differences
        assert_eq!(time_diff(100, 100), "now");
        assert_eq!(time_diff(100, 160), "in 1 minute");
        assert_eq!(time_diff(160, 100), "1 minute ago");
        assert_eq!(time_diff(100, 7300), "in 2 hours");
        assert_eq!(time_diff(7300, 100), "2 hours ago");
        assert_eq!(time_diff(100, 86500), "in 1 day");
        assert_eq!(time_diff(86500, 100), "1 day ago");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(1), "1 second");
        assert_eq!(format_duration(30), "30 seconds");
        assert_eq!(format_duration(60), "1 minute");
        assert_eq!(format_duration(120), "2 minutes");
        assert_eq!(format_duration(3600), "1 hour");
        assert_eq!(format_duration(7200), "2 hours");
        assert_eq!(format_duration(86400), "1 day");
        assert_eq!(format_duration(172800), "2 days");
    }
    
    #[test]
    fn test_is_expired() {
        // Test with a past timestamp
        let past = current_timestamp() - 100;
        assert!(is_expired(past));
        
        // Test with a future timestamp
        let future = current_timestamp() + 100;
        assert!(!is_expired(future));
    }
    
    #[test]
    fn test_add_seconds() {
        let timestamp = 1000;
        assert_eq!(add_seconds(timestamp, 500), 1500);
    }
    
    #[test]
    fn test_add_days() {
        let timestamp = 1000;
        // 1 day = 86400 seconds
        assert_eq!(add_days(timestamp, 1), 87400);
    }
} 