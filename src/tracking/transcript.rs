//! Transcript import module
//!
//! Imports transcripts from VTT, SRT, and text files.
//! Supports extracting action items and key points.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Configuration for transcript import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptConfig {
    /// Enable transcript import
    pub enabled: bool,
    /// Extract action items from transcripts
    pub extract_action_items: bool,
    /// Keywords that indicate action items
    pub action_keywords: Vec<String>,
    /// Minimum segment duration (seconds) to include
    pub min_segment_duration_secs: u32,
}

impl Default for TranscriptConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            extract_action_items: true,
            action_keywords: vec![
                "action item".to_string(),
                "todo".to_string(),
                "to do".to_string(),
                "follow up".to_string(),
                "next step".to_string(),
                "will do".to_string(),
                "need to".to_string(),
                "should".to_string(),
                "must".to_string(),
                "deadline".to_string(),
                "by friday".to_string(),
                "by monday".to_string(),
                "by end of".to_string(),
                "assigned to".to_string(),
            ],
            min_segment_duration_secs: 5,
        }
    }
}

/// Represents a transcript segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub start_time: Duration,
    pub end_time: Duration,
    pub speaker: Option<String>,
    pub text: String,
}

impl TranscriptSegment {
    pub fn duration(&self) -> Duration {
        self.end_time - self.start_time
    }
}

/// Represents a parsed transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub segments: Vec<TranscriptSegment>,
    pub source_file: Option<String>,
    pub title: Option<String>,
    pub total_duration: Duration,
}

impl Transcript {
    /// Get full text without timestamps
    pub fn full_text(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.text.clone())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get segments containing specific keywords
    pub fn find_segments(&self, keywords: &[String]) -> Vec<&TranscriptSegment> {
        self.segments
            .iter()
            .filter(|s| {
                let text_lower = s.text.to_lowercase();
                keywords
                    .iter()
                    .any(|k| text_lower.contains(&k.to_lowercase()))
            })
            .collect()
    }
}

/// Transcript importer
pub struct TranscriptImporter {
    config: TranscriptConfig,
}

impl TranscriptImporter {
    pub fn new(config: TranscriptConfig) -> Self {
        Self { config }
    }

    /// Import from WebVTT format
    pub fn import_vtt(&self, content: &str) -> Result<Transcript> {
        let mut segments = Vec::new();
        let mut lines = content.lines().peekable();

        // Skip WEBVTT header
        while let Some(line) = lines.peek() {
            if line.starts_with("WEBVTT") || line.is_empty() {
                lines.next();
            } else {
                break;
            }
        }

        // Parse cues
        while lines.peek().is_some() {
            // Skip empty lines and cue identifiers
            while let Some(line) = lines.peek() {
                if line.is_empty() || !line.contains("-->") {
                    lines.next();
                } else {
                    break;
                }
            }

            // Parse timestamp line
            if let Some(timestamp_line) = lines.next() {
                if let Some((start, end)) = parse_vtt_timestamp_line(timestamp_line) {
                    // Collect text lines
                    let mut text_lines = Vec::new();
                    while let Some(line) = lines.peek() {
                        if line.is_empty() {
                            break;
                        }
                        text_lines.push(lines.next().unwrap().to_string());
                    }

                    let text = text_lines.join(" ");
                    let (speaker, text) = extract_speaker(&text);

                    if !text.is_empty() {
                        segments.push(TranscriptSegment {
                            start_time: start,
                            end_time: end,
                            speaker,
                            text,
                        });
                    }
                }
            }
        }

        let total_duration = segments
            .last()
            .map(|s| s.end_time)
            .unwrap_or(Duration::zero());

        Ok(Transcript {
            segments,
            source_file: None,
            title: None,
            total_duration,
        })
    }

    /// Import from SRT format
    pub fn import_srt(&self, content: &str) -> Result<Transcript> {
        let mut segments = Vec::new();
        let mut lines = content.lines().peekable();

        while lines.peek().is_some() {
            // Skip sequence number
            if let Some(line) = lines.next() {
                if line.trim().parse::<u32>().is_err() && !line.is_empty() {
                    continue;
                }
            }

            // Parse timestamp line
            if let Some(timestamp_line) = lines.next() {
                if let Some((start, end)) = parse_srt_timestamp_line(timestamp_line) {
                    // Collect text lines until empty line
                    let mut text_lines = Vec::new();
                    while let Some(line) = lines.peek() {
                        if line.is_empty() {
                            lines.next();
                            break;
                        }
                        text_lines.push(lines.next().unwrap().to_string());
                    }

                    let text = text_lines.join(" ");
                    let (speaker, text) = extract_speaker(&text);

                    if !text.is_empty() {
                        segments.push(TranscriptSegment {
                            start_time: start,
                            end_time: end,
                            speaker,
                            text,
                        });
                    }
                }
            }
        }

        let total_duration = segments
            .last()
            .map(|s| s.end_time)
            .unwrap_or(Duration::zero());

        Ok(Transcript {
            segments,
            source_file: None,
            title: None,
            total_duration,
        })
    }

    /// Import from plain text (one line per segment, no timestamps)
    pub fn import_plain_text(&self, content: &str) -> Result<Transcript> {
        let segments: Vec<TranscriptSegment> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .enumerate()
            .map(|(i, line)| {
                let (speaker, text) = extract_speaker(line);
                TranscriptSegment {
                    start_time: Duration::seconds(i as i64 * 30), // Assume 30s per line
                    end_time: Duration::seconds((i + 1) as i64 * 30),
                    speaker,
                    text,
                }
            })
            .collect();

        let total_duration = segments
            .last()
            .map(|s| s.end_time)
            .unwrap_or(Duration::zero());

        Ok(Transcript {
            segments,
            source_file: None,
            title: None,
            total_duration,
        })
    }

    /// Extract action items from transcript
    pub fn extract_action_items(&self, transcript: &Transcript) -> Vec<ActionItem> {
        let mut items = Vec::new();
        let action_segments = transcript.find_segments(&self.config.action_keywords);

        for segment in action_segments {
            // Try to extract the action from context
            let text = &segment.text;
            let item = ActionItem {
                text: text.clone(),
                speaker: segment.speaker.clone(),
                timestamp: segment.start_time,
                confidence: self.calculate_action_confidence(text),
            };
            items.push(item);
        }

        items
    }

    /// Calculate confidence that a segment contains an action item
    fn calculate_action_confidence(&self, text: &str) -> f32 {
        let text_lower = text.to_lowercase();
        let mut score: f32 = 0.0;

        for keyword in &self.config.action_keywords {
            if text_lower.contains(&keyword.to_lowercase()) {
                score += 0.2;
            }
        }

        // Boost for imperative verbs
        let imperative_verbs = ["create", "update", "fix", "add", "remove", "send", "review", "check"];
        for verb in imperative_verbs {
            if text_lower.contains(verb) {
                score += 0.1;
            }
        }

        score.min(1.0)
    }

    /// Convert a transcript to a single activity
    pub fn transcript_to_activity(
        &self,
        transcript: &Transcript,
        meeting_time: DateTime<Utc>,
    ) -> Activity {
        let now = Utc::now();

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Transcript,
            activity_type: ActivityType::TranscriptSegment,
            timestamp: meeting_time,
            title: transcript
                .title
                .clone()
                .unwrap_or_else(|| "Imported Transcript".to_string()),
            description: Some(transcript.full_text()),
            project: None,
            employer: None,
            importance: Importance::Medium,
            metadata: serde_json::json!({
                "source_file": transcript.source_file,
                "segment_count": transcript.segments.len(),
                "total_duration_secs": transcript.total_duration.num_seconds(),
                "speakers": transcript.segments.iter()
                    .filter_map(|s| s.speaker.clone())
                    .collect::<std::collections::HashSet<_>>(),
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// Convert action items to activities
    pub fn action_items_to_activities(
        &self,
        items: &[ActionItem],
        meeting_time: DateTime<Utc>,
    ) -> Vec<Activity> {
        let now = Utc::now();

        items
            .iter()
            .map(|item| Activity {
                id: uuid::Uuid::new_v4().to_string(),
                source: ActivitySource::Transcript,
                activity_type: ActivityType::ActionItem,
                timestamp: meeting_time + item.timestamp,
                title: truncate_text(&item.text, 100),
                description: Some(item.text.clone()),
                project: None,
                employer: None,
                importance: if item.confidence > 0.7 {
                    Importance::High
                } else if item.confidence > 0.4 {
                    Importance::Medium
                } else {
                    Importance::Low
                },
                metadata: serde_json::json!({
                    "speaker": item.speaker,
                    "timestamp_secs": item.timestamp.num_seconds(),
                    "confidence": item.confidence,
                }),
                created_at: now,
                updated_at: now,
            })
            .collect()
    }
}

/// Represents an extracted action item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub text: String,
    pub speaker: Option<String>,
    pub timestamp: Duration,
    pub confidence: f32,
}

/// Parse VTT timestamp line: "00:00:00.000 --> 00:00:05.000"
fn parse_vtt_timestamp_line(line: &str) -> Option<(Duration, Duration)> {
    let parts: Vec<&str> = line.split("-->").collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parse_vtt_timestamp(parts[0].trim())?;
    let end = parse_vtt_timestamp(parts[1].trim().split_whitespace().next()?)?;

    Some((start, end))
}

/// Parse VTT timestamp: "00:00:00.000"
fn parse_vtt_timestamp(ts: &str) -> Option<Duration> {
    let parts: Vec<&str> = ts.split(':').collect();

    match parts.len() {
        2 => {
            // MM:SS.mmm
            let mins: i64 = parts[0].parse().ok()?;
            let secs_parts: Vec<&str> = parts[1].split('.').collect();
            let secs: i64 = secs_parts[0].parse().ok()?;
            let millis: i64 = secs_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            Some(Duration::milliseconds(mins * 60000 + secs * 1000 + millis))
        }
        3 => {
            // HH:MM:SS.mmm
            let hours: i64 = parts[0].parse().ok()?;
            let mins: i64 = parts[1].parse().ok()?;
            let secs_parts: Vec<&str> = parts[2].split('.').collect();
            let secs: i64 = secs_parts[0].parse().ok()?;
            let millis: i64 = secs_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            Some(Duration::milliseconds(
                hours * 3600000 + mins * 60000 + secs * 1000 + millis,
            ))
        }
        _ => None,
    }
}

/// Parse SRT timestamp line: "00:00:00,000 --> 00:00:05,000"
fn parse_srt_timestamp_line(line: &str) -> Option<(Duration, Duration)> {
    let line = line.replace(',', ".");
    parse_vtt_timestamp_line(&line)
}

/// Extract speaker from text: "Speaker Name: text" -> (Some("Speaker Name"), "text")
fn extract_speaker(text: &str) -> (Option<String>, String) {
    // Common patterns: "Name:", "[Name]:", "<Name>"
    let patterns = [
        Regex::new(r"^([A-Za-z\s]+):\s*(.*)$").ok(),
        Regex::new(r"^\[([^\]]+)\]:\s*(.*)$").ok(),
        Regex::new(r"^<([^>]+)>\s*(.*)$").ok(),
    ];

    for pattern in patterns.into_iter().flatten() {
        if let Some(caps) = pattern.captures(text) {
            if let (Some(speaker), Some(content)) = (caps.get(1), caps.get(2)) {
                return (Some(speaker.as_str().to_string()), content.as_str().to_string());
            }
        }
    }

    (None, text.to_string())
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
    fn test_parse_vtt_timestamp() {
        let ts = parse_vtt_timestamp("00:01:30.500");
        assert!(ts.is_some());
        assert_eq!(ts.unwrap().num_milliseconds(), 90500);
    }

    #[test]
    fn test_extract_speaker() {
        let (speaker, text) = extract_speaker("John: Hello everyone");
        assert_eq!(speaker, Some("John".to_string()));
        assert_eq!(text, "Hello everyone");

        let (speaker, text) = extract_speaker("Just regular text");
        assert!(speaker.is_none());
        assert_eq!(text, "Just regular text");
    }

    #[test]
    fn test_import_vtt() {
        let vtt = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
Hello, welcome to the meeting.

00:00:05.000 --> 00:00:10.000
John: Let's discuss the action items.
"#;

        let importer = TranscriptImporter::new(TranscriptConfig::default());
        let transcript = importer.import_vtt(vtt).unwrap();

        assert_eq!(transcript.segments.len(), 2);
        assert_eq!(transcript.segments[1].speaker, Some("John".to_string()));
    }

    #[test]
    fn test_extract_action_items() {
        let transcript = Transcript {
            segments: vec![
                TranscriptSegment {
                    start_time: Duration::seconds(0),
                    end_time: Duration::seconds(10),
                    speaker: Some("Alice".to_string()),
                    text: "We need to update the documentation by Friday".to_string(),
                },
                TranscriptSegment {
                    start_time: Duration::seconds(10),
                    end_time: Duration::seconds(20),
                    speaker: Some("Bob".to_string()),
                    text: "That sounds good to me".to_string(),
                },
            ],
            source_file: None,
            title: None,
            total_duration: Duration::seconds(20),
        };

        let importer = TranscriptImporter::new(TranscriptConfig::default());
        let items = importer.extract_action_items(&transcript);

        assert_eq!(items.len(), 1);
        assert!(items[0].text.contains("update"));
    }
}
