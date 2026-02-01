//! Active window tracking module (Rize-style)
//!
//! Captures metadata about the currently active window including:
//! - Application name
//! - Window title
//! - Process information
//! - Timestamps
//!
//! This module provides privacy-first tracking by only capturing metadata,
//! never the actual content of windows.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Represents a snapshot of the active window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSnapshot {
    pub app_name: String,
    pub window_title: String,
    pub process_id: u64,
    pub process_path: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub category: Option<WindowCategory>,
}

/// Categories for classifying window activity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WindowCategory {
    Development,
    Communication,
    Browser,
    Documentation,
    Design,
    Terminal,
    Meeting,
    Entertainment,
    System,
    Unknown,
}

impl std::fmt::Display for WindowCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => write!(f, "development"),
            Self::Communication => write!(f, "communication"),
            Self::Browser => write!(f, "browser"),
            Self::Documentation => write!(f, "documentation"),
            Self::Design => write!(f, "design"),
            Self::Terminal => write!(f, "terminal"),
            Self::Meeting => write!(f, "meeting"),
            Self::Entertainment => write!(f, "entertainment"),
            Self::System => write!(f, "system"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Configuration for active window tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWindowConfig {
    /// Enable active window tracking
    pub enabled: bool,
    /// Polling interval in seconds
    pub poll_interval_secs: u64,
    /// Minimum duration (in seconds) to record a window session
    pub min_session_duration_secs: u64,
    /// Apps to exclude from tracking
    pub excluded_apps: Vec<String>,
    /// Whether to capture window titles (can be disabled for privacy)
    pub capture_titles: bool,
    /// Whether to try to extract URLs from browsers
    pub capture_urls: bool,
}

impl Default for ActiveWindowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            poll_interval_secs: 5,
            min_session_duration_secs: 30,
            excluded_apps: vec![
                "1Password".to_string(),
                "Keychain Access".to_string(),
                "System Preferences".to_string(),
                "System Settings".to_string(),
            ],
            capture_titles: true,
            capture_urls: false,
        }
    }
}

/// Tracks the currently active window
pub struct ActiveWindowTracker {
    config: ActiveWindowConfig,
    current_window: Option<WindowSnapshot>,
    session_start: Option<DateTime<Utc>>,
    accumulated_sessions: Vec<WindowSession>,
}

/// Represents a tracked window session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSession {
    pub app_name: String,
    pub window_title: String,
    pub category: WindowCategory,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_secs: i64,
    pub process_path: Option<String>,
}

impl ActiveWindowTracker {
    pub fn new(config: ActiveWindowConfig) -> Self {
        Self {
            config,
            current_window: None,
            session_start: None,
            accumulated_sessions: Vec::new(),
        }
    }

    /// Get the current active window information
    pub fn get_active_window(&self) -> Result<Option<WindowSnapshot>> {
        #[cfg(feature = "desktop")]
        {
            use active_win_pos_rs::get_active_window;

            match get_active_window() {
                Ok(window) => {
                    // Check if app is excluded
                    if self
                        .config
                        .excluded_apps
                        .iter()
                        .any(|app| window.app_name.to_lowercase().contains(&app.to_lowercase()))
                    {
                        return Ok(None);
                    }

                    let category = self.categorize_window(&window.app_name, &window.title);

                    Ok(Some(WindowSnapshot {
                        app_name: window.app_name,
                        window_title: if self.config.capture_titles {
                            window.title
                        } else {
                            String::new()
                        },
                        process_id: window.process_id,
                        process_path: Some(window.process_path),
                        timestamp: Utc::now(),
                        category: Some(category),
                    }))
                }
                Err(_) => Ok(None),
            }
        }

        #[cfg(not(feature = "desktop"))]
        {
            Ok(None)
        }
    }

    /// Poll and track the current window, returning a session if one ended
    pub fn poll(&mut self) -> Result<Option<WindowSession>> {
        let new_window = self.get_active_window()?;

        match (&self.current_window, &new_window) {
            (None, Some(new)) => {
                // Starting fresh tracking
                self.current_window = Some(new.clone());
                self.session_start = Some(new.timestamp);
                Ok(None)
            }
            (Some(current), Some(new)) => {
                // Check if we switched to a different window
                if current.app_name != new.app_name || current.window_title != new.window_title {
                    // End current session
                    let session = self.end_current_session()?;

                    // Start new session
                    self.current_window = Some(new.clone());
                    self.session_start = Some(new.timestamp);

                    Ok(session)
                } else {
                    // Same window, update timestamp
                    self.current_window = Some(new.clone());
                    Ok(None)
                }
            }
            (Some(_), None) => {
                // Window lost (screen locked, idle, etc.)
                let session = self.end_current_session()?;
                self.current_window = None;
                self.session_start = None;
                Ok(session)
            }
            (None, None) => Ok(None),
        }
    }

    /// End the current session and return it if it meets minimum duration
    fn end_current_session(&mut self) -> Result<Option<WindowSession>> {
        if let (Some(current), Some(start)) = (&self.current_window, self.session_start) {
            let end_time = Utc::now();
            let duration = end_time.signed_duration_since(start);
            let duration_secs = duration.num_seconds();

            if duration_secs >= self.config.min_session_duration_secs as i64 {
                let session = WindowSession {
                    app_name: current.app_name.clone(),
                    window_title: current.window_title.clone(),
                    category: current.category.clone().unwrap_or(WindowCategory::Unknown),
                    start_time: start,
                    end_time,
                    duration_secs,
                    process_path: current.process_path.clone(),
                };

                self.accumulated_sessions.push(session.clone());
                return Ok(Some(session));
            }
        }

        Ok(None)
    }

    /// Force end the current session (e.g., when stopping tracking)
    pub fn flush(&mut self) -> Result<Option<WindowSession>> {
        self.end_current_session()
    }

    /// Categorize a window based on app name and title
    fn categorize_window(&self, app_name: &str, title: &str) -> WindowCategory {
        let app_lower = app_name.to_lowercase();
        let title_lower = title.to_lowercase();

        // Development tools
        if app_lower.contains("code")
            || app_lower.contains("visual studio")
            || app_lower.contains("intellij")
            || app_lower.contains("pycharm")
            || app_lower.contains("webstorm")
            || app_lower.contains("xcode")
            || app_lower.contains("android studio")
            || app_lower.contains("sublime")
            || app_lower.contains("atom")
            || app_lower.contains("vim")
            || app_lower.contains("neovim")
            || app_lower.contains("emacs")
        {
            return WindowCategory::Development;
        }

        // Terminal
        if app_lower.contains("terminal")
            || app_lower.contains("iterm")
            || app_lower.contains("warp")
            || app_lower.contains("alacritty")
            || app_lower.contains("kitty")
            || app_lower.contains("hyper")
            || app_lower.contains("powershell")
            || app_lower.contains("cmd")
            || app_lower.contains("console")
        {
            return WindowCategory::Terminal;
        }

        // Communication
        if app_lower.contains("slack")
            || app_lower.contains("discord")
            || app_lower.contains("teams")
            || app_lower.contains("messages")
            || app_lower.contains("mail")
            || app_lower.contains("outlook")
            || app_lower.contains("gmail")
            || app_lower.contains("telegram")
            || app_lower.contains("whatsapp")
        {
            return WindowCategory::Communication;
        }

        // Meeting apps
        if app_lower.contains("zoom")
            || app_lower.contains("meet")
            || app_lower.contains("webex")
            || app_lower.contains("facetime")
            || title_lower.contains("meeting")
        {
            return WindowCategory::Meeting;
        }

        // Design tools
        if app_lower.contains("figma")
            || app_lower.contains("sketch")
            || app_lower.contains("photoshop")
            || app_lower.contains("illustrator")
            || app_lower.contains("canva")
            || app_lower.contains("affinity")
        {
            return WindowCategory::Design;
        }

        // Documentation
        if app_lower.contains("notion")
            || app_lower.contains("obsidian")
            || app_lower.contains("roam")
            || app_lower.contains("confluence")
            || app_lower.contains("word")
            || app_lower.contains("docs")
            || app_lower.contains("pages")
            || app_lower.contains("notes")
        {
            return WindowCategory::Documentation;
        }

        // Browser - check for common dev/work patterns
        if app_lower.contains("chrome")
            || app_lower.contains("firefox")
            || app_lower.contains("safari")
            || app_lower.contains("edge")
            || app_lower.contains("brave")
            || app_lower.contains("arc")
        {
            // Check if browser is being used for development-related sites
            if title_lower.contains("github")
                || title_lower.contains("gitlab")
                || title_lower.contains("stackoverflow")
                || title_lower.contains("docs")
                || title_lower.contains("documentation")
                || title_lower.contains("api")
                || title_lower.contains("localhost")
            {
                return WindowCategory::Development;
            }
            return WindowCategory::Browser;
        }

        // System utilities
        if app_lower.contains("finder")
            || app_lower.contains("explorer")
            || app_lower.contains("activity monitor")
            || app_lower.contains("task manager")
            || app_lower.contains("settings")
            || app_lower.contains("preferences")
        {
            return WindowCategory::System;
        }

        WindowCategory::Unknown
    }

    /// Get accumulated sessions and clear them
    pub fn take_sessions(&mut self) -> Vec<WindowSession> {
        std::mem::take(&mut self.accumulated_sessions)
    }

    /// Convert a window session to an activity
    pub fn session_to_activity(&self, session: &WindowSession) -> Activity {
        let now = Utc::now();

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::ActiveWindow,
            activity_type: ActivityType::WindowSession,
            timestamp: session.start_time,
            title: format!("{} - {}", session.app_name, session.category),
            description: if session.window_title.is_empty() {
                None
            } else {
                Some(session.window_title.clone())
            },
            project: None, // Will be detected by ProjectDetector
            employer: None,
            importance: self.session_importance(session),
            metadata: serde_json::json!({
                "app_name": session.app_name,
                "window_title": session.window_title,
                "category": session.category.to_string(),
                "duration_secs": session.duration_secs,
                "start_time": session.start_time.to_rfc3339(),
                "end_time": session.end_time.to_rfc3339(),
                "process_path": session.process_path,
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// Determine importance based on session characteristics
    fn session_importance(&self, session: &WindowSession) -> Importance {
        // Long sessions in development tools are more important
        match session.category {
            WindowCategory::Development | WindowCategory::Terminal => {
                if session.duration_secs > 1800 {
                    // > 30 mins
                    Importance::High
                } else if session.duration_secs > 600 {
                    // > 10 mins
                    Importance::Medium
                } else {
                    Importance::Low
                }
            }
            WindowCategory::Meeting => Importance::Medium,
            WindowCategory::Documentation => {
                if session.duration_secs > 900 {
                    Importance::Medium
                } else {
                    Importance::Low
                }
            }
            _ => Importance::Low,
        }
    }

    /// Get a summary of time spent by category
    pub fn get_category_summary(&self) -> HashMap<WindowCategory, Duration> {
        let mut summary: HashMap<WindowCategory, Duration> = HashMap::new();

        for session in &self.accumulated_sessions {
            let duration = Duration::seconds(session.duration_secs);
            let entry = summary
                .entry(session.category.clone())
                .or_insert(Duration::zero());
            *entry += duration;
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_categorization() {
        let tracker = ActiveWindowTracker::new(ActiveWindowConfig::default());

        assert_eq!(
            tracker.categorize_window("Visual Studio Code", "main.rs"),
            WindowCategory::Development
        );

        assert_eq!(
            tracker.categorize_window("Slack", "general channel"),
            WindowCategory::Communication
        );

        assert_eq!(
            tracker.categorize_window("Chrome", "GitHub - repo"),
            WindowCategory::Development
        );

        assert_eq!(
            tracker.categorize_window("Chrome", "YouTube"),
            WindowCategory::Browser
        );

        assert_eq!(
            tracker.categorize_window("Zoom", "Meeting with team"),
            WindowCategory::Meeting
        );
    }

    #[test]
    fn test_session_importance() {
        let tracker = ActiveWindowTracker::new(ActiveWindowConfig::default());

        let short_dev_session = WindowSession {
            app_name: "VS Code".to_string(),
            window_title: "main.rs".to_string(),
            category: WindowCategory::Development,
            start_time: Utc::now(),
            end_time: Utc::now(),
            duration_secs: 300, // 5 minutes
            process_path: None,
        };

        let long_dev_session = WindowSession {
            app_name: "VS Code".to_string(),
            window_title: "main.rs".to_string(),
            category: WindowCategory::Development,
            start_time: Utc::now(),
            end_time: Utc::now(),
            duration_secs: 3600, // 1 hour
            process_path: None,
        };

        assert_eq!(
            tracker.session_importance(&short_dev_session),
            Importance::Low
        );
        assert_eq!(
            tracker.session_importance(&long_dev_session),
            Importance::High
        );
    }
}
