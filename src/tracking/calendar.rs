//! Calendar integration module
//!
//! Integrates with calendar services to track meetings and events
//! as activities automatically.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Configuration for calendar integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
    /// Enable calendar integration
    pub enabled: bool,
    /// Google Calendar credentials file path
    pub google_credentials_path: Option<String>,
    /// Calendar IDs to sync (empty = primary calendar)
    pub calendar_ids: Vec<String>,
    /// Days to look back for events
    pub days_back: u32,
    /// Days to look ahead for events
    pub days_ahead: u32,
    /// Minimum meeting duration (minutes) to track
    pub min_duration_mins: u32,
    /// Keywords to identify important meetings
    pub important_keywords: Vec<String>,
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            google_credentials_path: None,
            calendar_ids: vec![],
            days_back: 7,
            days_ahead: 1,
            min_duration_mins: 15,
            important_keywords: vec![
                "1:1".to_string(),
                "review".to_string(),
                "planning".to_string(),
                "standup".to_string(),
                "demo".to_string(),
                "presentation".to_string(),
                "interview".to_string(),
            ],
        }
    }
}

/// Represents a calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub organizer: Option<String>,
    pub is_recurring: bool,
    pub meeting_link: Option<String>,
    pub calendar_id: String,
}

impl CalendarEvent {
    /// Get duration in minutes
    pub fn duration_mins(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }

    /// Check if this is a video meeting
    pub fn is_video_meeting(&self) -> bool {
        self.meeting_link.is_some()
            || self
                .location
                .as_ref()
                .map(|l| {
                    l.contains("zoom")
                        || l.contains("meet.google")
                        || l.contains("teams")
                        || l.contains("webex")
                })
                .unwrap_or(false)
    }
}

/// Calendar integration handler
pub struct CalendarIntegration {
    config: CalendarConfig,
}

impl CalendarIntegration {
    pub fn new(config: CalendarConfig) -> Self {
        Self { config }
    }

    /// Fetch events from a local ICS file
    pub fn import_from_ics(&self, ics_content: &str) -> Result<Vec<CalendarEvent>> {
        let mut events = Vec::new();

        // Simple ICS parser for basic VEVENT components
        let mut current_event: Option<IcsEventBuilder> = None;

        for line in ics_content.lines() {
            let line = line.trim();

            if line == "BEGIN:VEVENT" {
                current_event = Some(IcsEventBuilder::default());
            } else if line == "END:VEVENT" {
                if let Some(builder) = current_event.take() {
                    if let Ok(event) = builder.build() {
                        events.push(event);
                    }
                }
            } else if let Some(ref mut event) = current_event {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.split(';').next().unwrap_or(key);
                    match key {
                        "UID" => event.id = Some(value.to_string()),
                        "SUMMARY" => event.title = Some(value.to_string()),
                        "DESCRIPTION" => event.description = Some(value.to_string()),
                        "DTSTART" => event.start_time = parse_ics_datetime(value),
                        "DTEND" => event.end_time = parse_ics_datetime(value),
                        "LOCATION" => event.location = Some(value.to_string()),
                        "ORGANIZER" => event.organizer = Some(value.to_string()),
                        "RRULE" => event.is_recurring = true,
                        _ => {}
                    }
                }
            }
        }

        Ok(events)
    }

    /// Import events from JSON format (e.g., exported from Google Calendar API)
    pub fn import_from_json(&self, json_content: &str) -> Result<Vec<CalendarEvent>> {
        let events: Vec<CalendarEvent> =
            serde_json::from_str(json_content).context("Failed to parse calendar JSON")?;
        Ok(events)
    }

    /// Convert a calendar event to an activity
    pub fn event_to_activity(&self, event: &CalendarEvent) -> Activity {
        let now = Utc::now();
        let importance = self.calculate_importance(event);

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Calendar,
            activity_type: ActivityType::CalendarEvent,
            timestamp: event.start_time,
            title: event.title.clone(),
            description: event.description.clone(),
            project: None,
            employer: None,
            importance,
            metadata: serde_json::json!({
                "calendar_event_id": event.id,
                "duration_mins": event.duration_mins(),
                "is_video_meeting": event.is_video_meeting(),
                "attendee_count": event.attendees.len(),
                "is_recurring": event.is_recurring,
                "location": event.location,
                "meeting_link": event.meeting_link,
                "organizer": event.organizer,
                "start_time": event.start_time.to_rfc3339(),
                "end_time": event.end_time.to_rfc3339(),
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate importance based on event characteristics
    fn calculate_importance(&self, event: &CalendarEvent) -> Importance {
        let title_lower = event.title.to_lowercase();

        // Check for important keywords
        for keyword in &self.config.important_keywords {
            if title_lower.contains(&keyword.to_lowercase()) {
                return Importance::High;
            }
        }

        // Longer meetings are generally more important
        if event.duration_mins() >= 60 {
            return Importance::High;
        }

        // Meetings with many attendees are important
        if event.attendees.len() > 5 {
            return Importance::Medium;
        }

        // Video meetings suggest importance
        if event.is_video_meeting() {
            return Importance::Medium;
        }

        Importance::Low
    }

    /// Filter events based on configuration
    pub fn filter_events(&self, events: Vec<CalendarEvent>) -> Vec<CalendarEvent> {
        events
            .into_iter()
            .filter(|e| e.duration_mins() >= self.config.min_duration_mins as i64)
            .collect()
    }
}

/// Builder for parsing ICS events
#[derive(Default)]
struct IcsEventBuilder {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    location: Option<String>,
    organizer: Option<String>,
    is_recurring: bool,
}

impl IcsEventBuilder {
    fn build(self) -> Result<CalendarEvent> {
        Ok(CalendarEvent {
            id: self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            title: self.title.context("Missing event title")?,
            description: self.description,
            start_time: self.start_time.context("Missing start time")?,
            end_time: self.end_time.context("Missing end time")?,
            location: self.location,
            attendees: vec![],
            organizer: self.organizer,
            is_recurring: self.is_recurring,
            meeting_link: None,
            calendar_id: "imported".to_string(),
        })
    }
}

/// Parse ICS datetime format (basic support)
fn parse_ics_datetime(value: &str) -> Option<DateTime<Utc>> {
    // Handle formats like: 20240115T100000Z or 20240115T100000
    let value = value.trim_end_matches('Z');

    if value.len() >= 15 {
        let year: i32 = value[0..4].parse().ok()?;
        let month: u32 = value[4..6].parse().ok()?;
        let day: u32 = value[6..8].parse().ok()?;
        let hour: u32 = value[9..11].parse().ok()?;
        let min: u32 = value[11..13].parse().ok()?;
        let sec: u32 = value[13..15].parse().ok()?;

        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|d| d.and_hms_opt(hour, min, sec))
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ics_datetime() {
        let dt = parse_ics_datetime("20240115T100000Z");
        assert!(dt.is_some());

        let dt = dt.unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2024-01-15 10:00:00");
    }

    #[test]
    fn test_event_duration() {
        let event = CalendarEvent {
            id: "test".to_string(),
            title: "Test Meeting".to_string(),
            description: None,
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(1),
            location: None,
            attendees: vec![],
            organizer: None,
            is_recurring: false,
            meeting_link: None,
            calendar_id: "test".to_string(),
        };

        assert_eq!(event.duration_mins(), 60);
    }

    #[test]
    fn test_is_video_meeting() {
        let mut event = CalendarEvent {
            id: "test".to_string(),
            title: "Test Meeting".to_string(),
            description: None,
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(1),
            location: Some("https://zoom.us/j/123456".to_string()),
            attendees: vec![],
            organizer: None,
            is_recurring: false,
            meeting_link: None,
            calendar_id: "test".to_string(),
        };

        assert!(event.is_video_meeting());

        event.location = None;
        event.meeting_link = Some("https://meet.google.com/abc".to_string());
        assert!(event.is_video_meeting());
    }
}
