//! Work session grouping module
//!
//! Groups related activities into cohesive work sessions
//! for better context and understanding of work patterns.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Configuration for session grouping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum gap between activities to consider them part of the same session
    pub max_gap_minutes: u32,
    /// Minimum session duration to be considered significant
    pub min_session_minutes: u32,
    /// Whether to group across different projects
    pub cross_project_sessions: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_gap_minutes: 30,
            min_session_minutes: 15,
            cross_project_sessions: false,
        }
    }
}

/// Represents a grouped work session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSession {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub project: Option<String>,
    pub activities: Vec<SessionActivity>,
    pub categories: HashMap<String, Duration>,
    pub total_duration: Duration,
    pub active_duration: Duration,
}

/// Simplified activity info for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    pub id: String,
    pub title: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub duration: Option<Duration>,
}

impl WorkSession {
    /// Get the number of activities in this session
    pub fn activity_count(&self) -> usize {
        self.activities.len()
    }

    /// Get the dominant category (most time spent)
    pub fn dominant_category(&self) -> Option<&String> {
        self.categories
            .iter()
            .max_by_key(|(_, duration)| duration.num_seconds())
            .map(|(cat, _)| cat)
    }

    /// Check if this is a significant session
    pub fn is_significant(&self, min_minutes: i64) -> bool {
        self.total_duration.num_minutes() >= min_minutes
    }
}

/// Groups activities into work sessions
pub struct SessionGrouper {
    config: SessionConfig,
}

impl SessionGrouper {
    pub fn new(config: SessionConfig) -> Self {
        Self { config }
    }

    /// Group a list of activities into work sessions
    pub fn group_activities(&self, activities: &[Activity]) -> Vec<WorkSession> {
        if activities.is_empty() {
            return vec![];
        }

        // Sort activities by timestamp
        let mut sorted: Vec<&Activity> = activities.iter().collect();
        sorted.sort_by_key(|a| a.timestamp);

        let mut sessions = Vec::new();
        let mut current_session: Option<WorkSessionBuilder> = None;

        for activity in sorted {
            match current_session.take() {
                None => {
                    current_session = Some(WorkSessionBuilder::new(activity));
                }
                Some(mut session) => {
                    let gap = activity.timestamp - session.end_time;

                    // Check if this activity belongs to the current session
                    let same_session = gap.num_minutes() <= self.config.max_gap_minutes as i64
                        && (self.config.cross_project_sessions
                            || session.project == activity.project);

                    if same_session {
                        session.add_activity(activity);
                        current_session = Some(session);
                    } else {
                        // End current session and start a new one
                        if let Some(completed) = session.build() {
                            if completed.total_duration.num_minutes()
                                >= self.config.min_session_minutes as i64
                            {
                                sessions.push(completed);
                            }
                        }
                        current_session = Some(WorkSessionBuilder::new(activity));
                    }
                }
            }
        }

        // Don't forget the last session
        if let Some(session) = current_session {
            if let Some(completed) = session.build() {
                if completed.total_duration.num_minutes() >= self.config.min_session_minutes as i64
                {
                    sessions.push(completed);
                }
            }
        }

        sessions
    }

    /// Convert a work session to an activity for storage
    pub fn session_to_activity(&self, session: &WorkSession) -> Activity {
        let now = Utc::now();

        let title = match &session.project {
            Some(p) => format!("Work session: {}", p),
            None => "Work session".to_string(),
        };

        let description = format!(
            "{} activities over {}",
            session.activity_count(),
            format_duration(session.total_duration)
        );

        Activity {
            id: session.id.clone(),
            source: ActivitySource::ActiveWindow,
            activity_type: ActivityType::WorkSession,
            timestamp: session.start_time,
            title,
            description: Some(description),
            project: session.project.clone(),
            employer: None,
            importance: self.calculate_session_importance(session),
            metadata: serde_json::json!({
                "start_time": session.start_time.to_rfc3339(),
                "end_time": session.end_time.to_rfc3339(),
                "total_duration_mins": session.total_duration.num_minutes(),
                "active_duration_mins": session.active_duration.num_minutes(),
                "activity_count": session.activity_count(),
                "categories": session.categories.iter()
                    .map(|(k, v)| (k.clone(), v.num_minutes()))
                    .collect::<HashMap<_, _>>(),
                "activity_ids": session.activities.iter()
                    .map(|a| a.id.clone())
                    .collect::<Vec<_>>(),
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate importance based on session characteristics
    fn calculate_session_importance(&self, session: &WorkSession) -> Importance {
        let duration_mins = session.total_duration.num_minutes();

        if duration_mins >= 120 {
            // 2+ hours
            Importance::High
        } else if duration_mins >= 60 {
            // 1+ hour
            Importance::Medium
        } else {
            Importance::Low
        }
    }

    /// Get summary statistics for a list of sessions
    pub fn get_summary(&self, sessions: &[WorkSession]) -> SessionSummary {
        let total_duration: Duration = sessions.iter().map(|s| s.total_duration).sum();
        let active_duration: Duration = sessions.iter().map(|s| s.active_duration).sum();

        let mut by_project: HashMap<String, Duration> = HashMap::new();
        let mut by_category: HashMap<String, Duration> = HashMap::new();

        for session in sessions {
            if let Some(project) = &session.project {
                *by_project.entry(project.clone()).or_default() += session.total_duration;
            }
            for (cat, dur) in &session.categories {
                *by_category.entry(cat.clone()).or_default() += *dur;
            }
        }

        SessionSummary {
            session_count: sessions.len(),
            total_duration,
            active_duration,
            by_project,
            by_category,
        }
    }
}

/// Builder for work sessions
struct WorkSessionBuilder {
    id: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    project: Option<String>,
    activities: Vec<SessionActivity>,
    categories: HashMap<String, Duration>,
}

impl WorkSessionBuilder {
    fn new(activity: &Activity) -> Self {
        let mut builder = Self {
            id: uuid::Uuid::new_v4().to_string(),
            start_time: activity.timestamp,
            end_time: activity.timestamp,
            project: activity.project.clone(),
            activities: Vec::new(),
            categories: HashMap::new(),
        };
        builder.add_activity(activity);
        builder
    }

    fn add_activity(&mut self, activity: &Activity) {
        // Update end time
        if activity.timestamp > self.end_time {
            self.end_time = activity.timestamp;
        }

        // Extract duration from metadata if available
        let duration = activity
            .metadata
            .get("duration_secs")
            .and_then(|v| v.as_i64())
            .map(Duration::seconds);

        // Update category time
        if let Some(category) = activity.metadata.get("category").and_then(|v| v.as_str()) {
            if let Some(dur) = duration {
                *self.categories.entry(category.to_string()).or_default() += dur;
            }
        }

        // Add activity reference
        self.activities.push(SessionActivity {
            id: activity.id.clone(),
            title: activity.title.clone(),
            source: activity.source.to_string(),
            timestamp: activity.timestamp,
            duration,
        });
    }

    fn build(self) -> Option<WorkSession> {
        if self.activities.is_empty() {
            return None;
        }

        let total_duration = self.end_time - self.start_time;
        let active_duration: Duration = self.activities.iter().filter_map(|a| a.duration).sum();

        Some(WorkSession {
            id: self.id,
            start_time: self.start_time,
            end_time: self.end_time,
            project: self.project,
            activities: self.activities,
            categories: self.categories,
            total_duration,
            active_duration,
        })
    }
}

/// Summary statistics for multiple sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_count: usize,
    pub total_duration: Duration,
    pub active_duration: Duration,
    pub by_project: HashMap<String, Duration>,
    pub by_category: HashMap<String, Duration>,
}

/// Format a duration in a human-readable way
fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let mins = duration.num_minutes() % 60;

    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_activity(title: &str, mins_ago: i64, project: Option<&str>) -> Activity {
        let now = Utc::now();
        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::ActiveWindow,
            activity_type: ActivityType::WindowSession,
            timestamp: now - Duration::minutes(mins_ago),
            title: title.to_string(),
            description: None,
            project: project.map(|s| s.to_string()),
            employer: None,
            importance: Importance::Medium,
            metadata: serde_json::json!({
                "duration_secs": 300,
                "category": "development"
            }),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_group_activities() {
        let activities = vec![
            create_test_activity("Coding", 60, Some("folio")),
            create_test_activity("Coding more", 50, Some("folio")),
            create_test_activity("Debugging", 40, Some("folio")),
        ];

        let grouper = SessionGrouper::new(SessionConfig::default());
        let sessions = grouper.group_activities(&activities);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].activities.len(), 3);
    }

    #[test]
    fn test_split_by_gap() {
        let activities = vec![
            create_test_activity("Morning work", 120, Some("folio")),
            create_test_activity("Afternoon work", 30, Some("folio")),
        ];

        // Use custom config with no minimum session duration to test gap splitting
        let config = SessionConfig {
            max_gap_minutes: 30,
            min_session_minutes: 0,
            cross_project_sessions: false,
        };
        let grouper = SessionGrouper::new(config);
        let sessions = grouper.group_activities(&activities);

        // Should be split because gap is > 30 minutes
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::minutes(30)), "30m");
        assert_eq!(format_duration(Duration::minutes(90)), "1h 30m");
        assert_eq!(format_duration(Duration::hours(2)), "2h 0m");
    }
}
