//! Meeting analysis module
//!
//! Analyzes meeting transcripts and summaries to extract
//! accomplishments, action items, and key contributions.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Configuration for meeting analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingConfig {
    /// Enable meeting analysis
    pub enabled: bool,
    /// Your name/email for identifying your contributions
    pub your_name: Option<String>,
    /// Alternative names/aliases to identify you
    pub your_aliases: Vec<String>,
    /// Keywords indicating decisions
    pub decision_keywords: Vec<String>,
    /// Keywords indicating action items
    pub action_keywords: Vec<String>,
    /// Minimum meeting duration (minutes) to analyze
    pub min_duration_mins: u32,
}

impl Default for MeetingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            your_name: None,
            your_aliases: vec![],
            decision_keywords: vec![
                "decided".to_string(),
                "agreed".to_string(),
                "will".to_string(),
                "going to".to_string(),
                "plan is".to_string(),
                "conclusion".to_string(),
            ],
            action_keywords: vec![
                "action item".to_string(),
                "todo".to_string(),
                "follow up".to_string(),
                "next step".to_string(),
                "assigned".to_string(),
                "deadline".to_string(),
                "by end of".to_string(),
            ],
            min_duration_mins: 15,
        }
    }
}

/// Represents a meeting summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSummary {
    pub id: String,
    pub title: String,
    pub date: DateTime<Utc>,
    pub duration_mins: i64,
    pub attendees: Vec<String>,
    pub summary: Option<String>,
    pub key_points: Vec<String>,
    pub decisions: Vec<Decision>,
    pub action_items: Vec<MeetingActionItem>,
    pub your_contributions: Vec<Contribution>,
    pub source: MeetingSource,
}

/// Source of the meeting data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeetingSource {
    Otter,
    Loom,
    Zoom,
    GoogleMeet,
    Teams,
    Manual,
    Transcript,
}

impl std::fmt::Display for MeetingSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Otter => write!(f, "otter"),
            Self::Loom => write!(f, "loom"),
            Self::Zoom => write!(f, "zoom"),
            Self::GoogleMeet => write!(f, "google_meet"),
            Self::Teams => write!(f, "teams"),
            Self::Manual => write!(f, "manual"),
            Self::Transcript => write!(f, "transcript"),
        }
    }
}

/// Represents a decision made in a meeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub text: String,
    pub context: Option<String>,
    pub participants: Vec<String>,
}

/// Represents an action item from a meeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingActionItem {
    pub text: String,
    pub assignee: Option<String>,
    pub deadline: Option<String>,
    pub is_yours: bool,
}

/// Represents your contribution to a meeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub text: String,
    pub contribution_type: ContributionType,
}

/// Types of contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContributionType {
    Presented,
    Proposed,
    Led,
    Explained,
    Resolved,
    Assigned,
    Other,
}

impl std::fmt::Display for ContributionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Presented => write!(f, "presented"),
            Self::Proposed => write!(f, "proposed"),
            Self::Led => write!(f, "led"),
            Self::Explained => write!(f, "explained"),
            Self::Resolved => write!(f, "resolved"),
            Self::Assigned => write!(f, "assigned"),
            Self::Other => write!(f, "contributed"),
        }
    }
}

/// Meeting analyzer
pub struct MeetingAnalyzer {
    config: MeetingConfig,
}

impl MeetingAnalyzer {
    pub fn new(config: MeetingConfig) -> Self {
        Self { config }
    }

    /// Parse an Otter AI export (JSON format)
    pub fn parse_otter_export(&self, json_content: &str) -> Result<MeetingSummary> {
        let data: serde_json::Value =
            serde_json::from_str(json_content).context("Failed to parse Otter JSON")?;

        let title = data["title"]
            .as_str()
            .unwrap_or("Untitled Meeting")
            .to_string();

        let summary = data["summary"].as_str().map(|s| s.to_string());

        let attendees: Vec<String> = data["speakers"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let duration_mins = data["duration_seconds"]
            .as_i64()
            .map(|s| s / 60)
            .unwrap_or(0);

        // Extract transcript for analysis
        let transcript: String = data["transcript"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v["text"].as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        // Analyze the transcript
        let key_points = self.extract_key_points(&transcript);
        let decisions = self.extract_decisions(&transcript);
        let action_items = self.extract_action_items(&transcript);
        let your_contributions = self.extract_contributions(&transcript, &attendees);

        Ok(MeetingSummary {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            date: Utc::now(),
            duration_mins,
            attendees,
            summary,
            key_points,
            decisions,
            action_items,
            your_contributions,
            source: MeetingSource::Otter,
        })
    }

    /// Parse a Loom transcript
    pub fn parse_loom_export(&self, json_content: &str) -> Result<MeetingSummary> {
        let data: serde_json::Value =
            serde_json::from_str(json_content).context("Failed to parse Loom JSON")?;

        let title = data["title"]
            .as_str()
            .unwrap_or("Untitled Video")
            .to_string();

        let summary = data["summary"].as_str().map(|s| s.to_string());

        let transcript = data["transcript"].as_str().unwrap_or("").to_string();

        let duration_mins = data["duration_seconds"]
            .as_i64()
            .map(|s| s / 60)
            .unwrap_or(0);

        let key_points = self.extract_key_points(&transcript);
        let decisions = self.extract_decisions(&transcript);
        let action_items = self.extract_action_items(&transcript);

        Ok(MeetingSummary {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            date: Utc::now(),
            duration_mins,
            attendees: vec![],
            summary,
            key_points,
            decisions,
            action_items,
            your_contributions: vec![],
            source: MeetingSource::Loom,
        })
    }

    /// Parse a manual meeting summary
    pub fn parse_manual_summary(&self, text: &str, title: &str) -> MeetingSummary {
        let key_points = self.extract_key_points(text);
        let decisions = self.extract_decisions(text);
        let action_items = self.extract_action_items(text);

        MeetingSummary {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            date: Utc::now(),
            duration_mins: 0,
            attendees: vec![],
            summary: Some(text.to_string()),
            key_points,
            decisions,
            action_items,
            your_contributions: vec![],
            source: MeetingSource::Manual,
        }
    }

    /// Extract key points from text
    fn extract_key_points(&self, text: &str) -> Vec<String> {
        let mut points = Vec::new();
        let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();

        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.len() > 20 && sentence.len() < 200 {
                // Look for key point indicators
                let indicators = [
                    "important",
                    "key",
                    "main",
                    "significant",
                    "critical",
                    "essential",
                    "primary",
                    "focus",
                ];

                let sentence_lower = sentence.to_lowercase();
                for indicator in indicators {
                    if sentence_lower.contains(indicator) {
                        points.push(sentence.to_string());
                        break;
                    }
                }
            }
        }

        // Deduplicate and limit
        points.truncate(10);
        points
    }

    /// Extract decisions from text
    fn extract_decisions(&self, text: &str) -> Vec<Decision> {
        let mut decisions = Vec::new();
        let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();

        for sentence in sentences {
            let sentence = sentence.trim();
            let sentence_lower = sentence.to_lowercase();

            for keyword in &self.config.decision_keywords {
                if sentence_lower.contains(&keyword.to_lowercase()) {
                    decisions.push(Decision {
                        text: sentence.to_string(),
                        context: None,
                        participants: vec![],
                    });
                    break;
                }
            }
        }

        decisions.truncate(10);
        decisions
    }

    /// Extract action items from text
    fn extract_action_items(&self, text: &str) -> Vec<MeetingActionItem> {
        let mut items = Vec::new();
        let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();

        for sentence in sentences {
            let sentence = sentence.trim();
            let sentence_lower = sentence.to_lowercase();

            for keyword in &self.config.action_keywords {
                if sentence_lower.contains(&keyword.to_lowercase()) {
                    let is_yours = self.is_about_you(&sentence_lower);
                    let assignee = self.extract_assignee(sentence);
                    let deadline = self.extract_deadline(sentence);

                    items.push(MeetingActionItem {
                        text: sentence.to_string(),
                        assignee,
                        deadline,
                        is_yours,
                    });
                    break;
                }
            }
        }

        items.truncate(20);
        items
    }

    /// Extract your contributions from a transcript
    fn extract_contributions(
        &self,
        text: &str,
        _attendees: &[String],
    ) -> Vec<Contribution> {
        let mut contributions = Vec::new();

        // Find sentences where you're speaking or mentioned as contributing
        let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();

        for sentence in sentences {
            let sentence = sentence.trim();
            let sentence_lower = sentence.to_lowercase();

            if self.is_about_you(&sentence_lower) {
                let contribution_type = self.identify_contribution_type(sentence);
                contributions.push(Contribution {
                    text: sentence.to_string(),
                    contribution_type,
                });
            }
        }

        contributions.truncate(10);
        contributions
    }

    /// Check if a sentence is about you
    fn is_about_you(&self, text: &str) -> bool {
        if let Some(name) = &self.config.your_name {
            if text.contains(&name.to_lowercase()) {
                return true;
            }
        }

        for alias in &self.config.your_aliases {
            if text.contains(&alias.to_lowercase()) {
                return true;
            }
        }

        // Also check for first-person indicators at start
        text.starts_with("i ") || text.starts_with("i'll ") || text.starts_with("i'm ")
    }

    /// Identify the type of contribution
    fn identify_contribution_type(&self, text: &str) -> ContributionType {
        let text_lower = text.to_lowercase();

        if text_lower.contains("present") || text_lower.contains("demo") {
            ContributionType::Presented
        } else if text_lower.contains("propos") || text_lower.contains("suggest") {
            ContributionType::Proposed
        } else if text_lower.contains("led") || text_lower.contains("leading") {
            ContributionType::Led
        } else if text_lower.contains("explain") || text_lower.contains("clarif") {
            ContributionType::Explained
        } else if text_lower.contains("resolv") || text_lower.contains("fix") {
            ContributionType::Resolved
        } else if text_lower.contains("assign") || text_lower.contains("delegat") {
            ContributionType::Assigned
        } else {
            ContributionType::Other
        }
    }

    /// Extract assignee from action item text
    fn extract_assignee(&self, text: &str) -> Option<String> {
        // Look for patterns like "assigned to X" or "X will"
        let patterns = [
            Regex::new(r"assigned to (\w+)").ok()?,
            Regex::new(r"(\w+) will").ok()?,
            Regex::new(r"(\w+) to handle").ok()?,
        ];

        for pattern in patterns {
            if let Some(caps) = pattern.captures(text) {
                if let Some(name) = caps.get(1) {
                    return Some(name.as_str().to_string());
                }
            }
        }

        None
    }

    /// Extract deadline from action item text
    fn extract_deadline(&self, text: &str) -> Option<String> {
        let patterns = [
            Regex::new(r"by ((?:end of )?(?:today|tomorrow|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday|next week|next month|EOD|EOW))").ok()?,
            Regex::new(r"deadline[:\s]+(\w+)").ok()?,
            Regex::new(r"due[:\s]+(\w+)").ok()?,
        ];

        for pattern in patterns {
            if let Some(caps) = pattern.captures(text) {
                if let Some(deadline) = caps.get(1) {
                    return Some(deadline.as_str().to_string());
                }
            }
        }

        None
    }

    /// Convert a meeting summary to an activity
    pub fn summary_to_activity(&self, summary: &MeetingSummary) -> Activity {
        let now = Utc::now();

        let description = summary.summary.clone().or_else(|| {
            if !summary.key_points.is_empty() {
                Some(summary.key_points.join("\n- "))
            } else {
                None
            }
        });

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Meeting,
            activity_type: ActivityType::MeetingSummary,
            timestamp: summary.date,
            title: summary.title.clone(),
            description,
            project: None,
            employer: None,
            importance: self.calculate_importance(summary),
            metadata: serde_json::json!({
                "meeting_id": summary.id,
                "duration_mins": summary.duration_mins,
                "attendee_count": summary.attendees.len(),
                "decision_count": summary.decisions.len(),
                "action_item_count": summary.action_items.len(),
                "your_action_items": summary.action_items.iter()
                    .filter(|a| a.is_yours)
                    .map(|a| a.text.clone())
                    .collect::<Vec<_>>(),
                "your_contributions": summary.your_contributions.iter()
                    .map(|c| format!("{}: {}", c.contribution_type, c.text))
                    .collect::<Vec<_>>(),
                "source": summary.source.to_string(),
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// Convert action items to activities
    pub fn action_items_to_activities(&self, summary: &MeetingSummary) -> Vec<Activity> {
        let now = Utc::now();

        summary
            .action_items
            .iter()
            .filter(|item| item.is_yours)
            .map(|item| Activity {
                id: uuid::Uuid::new_v4().to_string(),
                source: ActivitySource::Meeting,
                activity_type: ActivityType::ActionItem,
                timestamp: summary.date,
                title: truncate_text(&item.text, 100),
                description: Some(item.text.clone()),
                project: None,
                employer: None,
                importance: if item.deadline.is_some() {
                    Importance::High
                } else {
                    Importance::Medium
                },
                metadata: serde_json::json!({
                    "meeting_id": summary.id,
                    "meeting_title": summary.title,
                    "assignee": item.assignee,
                    "deadline": item.deadline,
                }),
                created_at: now,
                updated_at: now,
            })
            .collect()
    }

    /// Calculate importance based on meeting characteristics
    fn calculate_importance(&self, summary: &MeetingSummary) -> Importance {
        // High importance if you have action items or made contributions
        if !summary.action_items.iter().any(|a| a.is_yours)
            && summary.your_contributions.is_empty()
        {
            return Importance::Low;
        }

        if summary.duration_mins >= 60 || summary.decisions.len() > 2 {
            return Importance::High;
        }

        Importance::Medium
    }
}

/// Truncate text to a maximum length
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_action_items() {
        let config = MeetingConfig::default();
        let analyzer = MeetingAnalyzer::new(config);

        let text = "We need to follow up with the client. John will update the documentation by Friday. Action item: Review the PR.";
        let items = analyzer.extract_action_items(text);

        assert!(items.len() >= 2);
    }

    #[test]
    fn test_extract_decisions() {
        let config = MeetingConfig::default();
        let analyzer = MeetingAnalyzer::new(config);

        let text = "We decided to use Rust for the backend. The team agreed on the new architecture.";
        let decisions = analyzer.extract_decisions(text);

        assert_eq!(decisions.len(), 2);
    }

    #[test]
    fn test_is_about_you() {
        let mut config = MeetingConfig::default();
        config.your_name = Some("John".to_string());
        let analyzer = MeetingAnalyzer::new(config);

        assert!(analyzer.is_about_you("john will handle this"));
        assert!(analyzer.is_about_you("i'll take care of it"));
        assert!(!analyzer.is_about_you("sarah will do it"));
    }
}
