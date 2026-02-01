//! Idle detection module
//!
//! Detects when the user is idle (no keyboard/mouse activity)
//! to accurately track active work time.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Configuration for idle detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleConfig {
    /// Enable idle detection
    pub enabled: bool,
    /// Seconds of inactivity before considered idle
    pub idle_threshold_secs: u64,
    /// Seconds of inactivity before tracking is paused
    pub pause_threshold_secs: u64,
}

impl Default for IdleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_threshold_secs: 120,  // 2 minutes like Rize
            pause_threshold_secs: 300, // 5 minutes
        }
    }
}

/// Idle state tracking
#[derive(Debug, Clone, PartialEq)]
pub enum IdleState {
    Active,
    Idle,
    Away,
}

/// Tracks user idle state
pub struct IdleDetector {
    config: IdleConfig,
    last_active: DateTime<Utc>,
    current_state: IdleState,
    idle_start: Option<DateTime<Utc>>,
}

impl IdleDetector {
    pub fn new(config: IdleConfig) -> Self {
        Self {
            config,
            last_active: Utc::now(),
            current_state: IdleState::Active,
            idle_start: None,
        }
    }

    /// Get the current system idle time in seconds
    pub fn get_system_idle_secs(&self) -> Result<u64> {
        #[cfg(feature = "desktop")]
        {
            match user_idle::UserIdle::get_time() {
                Ok(idle) => Ok(idle.as_secs()),
                Err(_) => Ok(0),
            }
        }

        #[cfg(not(feature = "desktop"))]
        {
            Ok(0)
        }
    }

    /// Check and update idle state, returns true if state changed
    pub fn check(&mut self) -> Result<bool> {
        let idle_secs = self.get_system_idle_secs()?;
        let previous_state = self.current_state.clone();

        if idle_secs >= self.config.pause_threshold_secs {
            if self.current_state != IdleState::Away {
                self.current_state = IdleState::Away;
                if self.idle_start.is_none() {
                    self.idle_start = Some(Utc::now() - Duration::seconds(idle_secs as i64));
                }
            }
        } else if idle_secs >= self.config.idle_threshold_secs {
            if self.current_state != IdleState::Idle {
                self.current_state = IdleState::Idle;
                if self.idle_start.is_none() {
                    self.idle_start = Some(Utc::now() - Duration::seconds(idle_secs as i64));
                }
            }
        } else {
            if self.current_state != IdleState::Active {
                self.last_active = Utc::now();
                self.idle_start = None;
            }
            self.current_state = IdleState::Active;
        }

        Ok(self.current_state != previous_state)
    }

    /// Get current idle state
    pub fn state(&self) -> &IdleState {
        &self.current_state
    }

    /// Check if currently idle
    pub fn is_idle(&self) -> bool {
        matches!(self.current_state, IdleState::Idle | IdleState::Away)
    }

    /// Check if tracking should be paused
    pub fn should_pause_tracking(&self) -> bool {
        matches!(self.current_state, IdleState::Away)
    }

    /// Get duration of current idle period
    pub fn idle_duration(&self) -> Option<Duration> {
        self.idle_start.map(|start| Utc::now() - start)
    }

    /// Get time since last activity
    pub fn time_since_active(&self) -> Duration {
        Utc::now() - self.last_active
    }

    /// Reset idle state (called when activity is detected)
    pub fn reset(&mut self) {
        self.current_state = IdleState::Active;
        self.last_active = Utc::now();
        self.idle_start = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_detector_creation() {
        let detector = IdleDetector::new(IdleConfig::default());
        assert_eq!(*detector.state(), IdleState::Active);
        assert!(!detector.is_idle());
    }

    #[test]
    fn test_idle_config_defaults() {
        let config = IdleConfig::default();
        assert_eq!(config.idle_threshold_secs, 120);
        assert_eq!(config.pause_threshold_secs, 300);
    }
}
