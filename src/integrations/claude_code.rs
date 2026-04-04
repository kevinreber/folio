use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// A single entry from Claude Code's history.jsonl
#[derive(Debug, Deserialize)]
struct HistoryEntry {
    display: String,
    timestamp: u64,
    project: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(default, rename = "pastedContents")]
    pasted_contents: Option<serde_json::Value>,
}

/// Configuration for Claude Code activity scanning
#[derive(Debug, Clone)]
pub struct ClaudeCodeConfig {
    /// Path to history.jsonl (default: ~/.claude/history.jsonl)
    pub history_path: PathBuf,
    /// Number of days to look back
    pub days_back: u32,
    /// Minimum prompts in a session to consider it significant
    pub min_session_prompts: usize,
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            history_path: home.join(".claude").join("history.jsonl"),
            days_back: 30,
            min_session_prompts: 1,
        }
    }
}

/// Claude Code history scanner
pub struct ClaudeCodeScanner {
    config: ClaudeCodeConfig,
}

/// A group of prompts within a single session, representing a work session
#[derive(Debug)]
struct SessionGroup {
    session_id: String,
    project: String,
    project_name: String,
    prompts: Vec<HistoryEntry>,
    first_timestamp: DateTime<Utc>,
    last_timestamp: DateTime<Utc>,
    is_personal: bool,
}

impl ClaudeCodeScanner {
    pub fn new(config: ClaudeCodeConfig) -> Self {
        Self { config }
    }

    /// Scan history.jsonl and return activities grouped by session
    pub fn scan(&self) -> Result<Vec<Activity>> {
        let path = &self.config.history_path;
        if !path.exists() {
            return Ok(vec![]);
        }

        let file =
            std::fs::File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
        let reader = std::io::BufReader::new(file);

        let cutoff = Utc::now() - chrono::Duration::days(self.config.days_back as i64);

        // Parse all entries within the time window
        let mut entries: Vec<HistoryEntry> = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
                let ts = Utc
                    .timestamp_millis_opt(entry.timestamp as i64)
                    .single()
                    .unwrap_or_else(Utc::now);
                if ts >= cutoff {
                    entries.push(entry);
                }
            }
        }

        // Group by session
        let sessions = self.group_by_session(entries);

        // Convert sessions to activities
        let mut activities: Vec<Activity> = sessions
            .into_iter()
            .filter(|s| s.prompts.len() >= self.config.min_session_prompts)
            .map(|s| self.session_to_activity(s))
            .collect();

        // Sort by timestamp, newest first
        activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(activities)
    }

    fn group_by_session(&self, entries: Vec<HistoryEntry>) -> Vec<SessionGroup> {
        let mut session_map: HashMap<String, Vec<HistoryEntry>> = HashMap::new();

        for entry in entries {
            session_map
                .entry(entry.session_id.clone())
                .or_default()
                .push(entry);
        }

        session_map
            .into_iter()
            .filter_map(|(session_id, mut prompts)| {
                if prompts.is_empty() {
                    return None;
                }

                // Sort by timestamp
                prompts.sort_by_key(|e| e.timestamp);

                let project = prompts[0].project.clone();
                let project_name = extract_project_name(&project);
                let is_personal = project.contains("/personal/");

                let first_ts = Utc
                    .timestamp_millis_opt(prompts[0].timestamp as i64)
                    .single()
                    .unwrap_or_else(Utc::now);
                let last_ts = Utc
                    .timestamp_millis_opt(prompts.last().unwrap().timestamp as i64)
                    .single()
                    .unwrap_or_else(Utc::now);

                Some(SessionGroup {
                    session_id,
                    project,
                    project_name,
                    prompts,
                    first_timestamp: first_ts,
                    last_timestamp: last_ts,
                    is_personal,
                })
            })
            .collect()
    }

    fn session_to_activity(&self, session: SessionGroup) -> Activity {
        let prompt_count = session.prompts.len();

        // Calculate active time: sum gaps between consecutive prompts,
        // but only count gaps under 30 minutes as active work time.
        let duration_mins = if prompt_count <= 1 {
            2 // single prompt ~ 2 minutes estimate
        } else {
            let mut active_mins = 0i64;
            for window in session.prompts.windows(2) {
                let gap = (Utc
                    .timestamp_millis_opt(window[1].timestamp as i64)
                    .single()
                    .unwrap_or_else(Utc::now)
                    - Utc
                        .timestamp_millis_opt(window[0].timestamp as i64)
                        .single()
                        .unwrap_or_else(Utc::now))
                .num_minutes();
                // Only count gaps under 30 min as active; longer = idle/away
                if gap > 0 && gap <= 30 {
                    active_mins += gap;
                } else if gap > 30 {
                    active_mins += 2; // count ~2 min for the prompt itself
                }
            }
            active_mins.max(1)
        };

        // Build a title from the session's work
        let title = self.build_session_title(&session);

        // Extract references (URLs) from prompts
        let references = self.extract_references(&session.prompts);

        // Determine importance
        let importance = self.infer_importance(&session);

        let now = Utc::now();

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::ClaudeCode,
            activity_type: ActivityType::WorkSession,
            timestamp: session.first_timestamp,
            title,
            description: Some(self.build_session_description(&session)),
            project: Some(session.project_name.clone()),
            employer: None,
            importance,
            metadata: serde_json::json!({
                "session_id": session.session_id,
                "project_path": session.project,
                "prompt_count": prompt_count,
                "duration_minutes": duration_mins,
                "is_personal": session.is_personal,
                "references": references,
                "first_prompt": truncate_prompt(&session.prompts[0].display, 200),
            }),
            created_at: now,
            updated_at: now,
        }
    }

    fn build_session_title(&self, session: &SessionGroup) -> String {
        let prompt_count = session.prompts.len();
        let first_prompt = &session.prompts[0].display;

        // Use the first prompt as the title if it's short enough
        if first_prompt.len() <= 80 && prompt_count <= 3 {
            return first_prompt.clone();
        }

        // For longer sessions, summarize
        let tag = if session.is_personal {
            "personal"
        } else {
            "work"
        };
        format!(
            "[{}] {} session on {} ({} prompts)",
            tag, "Claude Code", session.project_name, prompt_count
        )
    }

    fn build_session_description(&self, session: &SessionGroup) -> String {
        let mut lines = Vec::new();

        // Show first few prompts as context
        let show_count = session.prompts.len().min(5);
        for prompt in session.prompts.iter().take(show_count) {
            lines.push(format!("- {}", truncate_prompt(&prompt.display, 150)));
        }
        if session.prompts.len() > show_count {
            lines.push(format!(
                "  ... and {} more prompts",
                session.prompts.len() - show_count
            ));
        }

        lines.join("\n")
    }

    fn extract_references(&self, prompts: &[HistoryEntry]) -> Vec<String> {
        let mut refs = Vec::new();
        let url_patterns = [
            "github.com/",
            "docs.google.com/",
            "confluence.",
            "jira.",
            "linear.app/",
        ];

        for prompt in prompts {
            for word in prompt.display.split_whitespace() {
                if url_patterns.iter().any(|p| word.contains(p)) {
                    let clean = word.trim_matches(|c: char| {
                        !c.is_alphanumeric()
                            && c != '/'
                            && c != ':'
                            && c != '.'
                            && c != '-'
                            && c != '_'
                            && c != '?'
                            && c != '='
                            && c != '&'
                            && c != '#'
                    });
                    if !refs.contains(&clean.to_string()) {
                        refs.push(clean.to_string());
                    }
                }
            }
        }

        refs
    }

    fn infer_importance(&self, session: &SessionGroup) -> Importance {
        let prompt_count = session.prompts.len();
        let duration_mins = (session.last_timestamp - session.first_timestamp).num_minutes();

        // Long sessions with many prompts = high importance
        if prompt_count >= 15 || duration_mins >= 120 {
            return Importance::High;
        }

        // Short sessions = low importance
        if prompt_count <= 2 && duration_mins <= 10 {
            return Importance::Low;
        }

        // Check prompt content for importance signals
        let all_text: String = session
            .prompts
            .iter()
            .map(|p| p.display.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        if all_text.contains("deploy")
            || all_text.contains("production")
            || all_text.contains("incident")
            || all_text.contains("critical")
            || all_text.contains("security")
        {
            return Importance::High;
        }

        Importance::Medium
    }
}

fn extract_project_name(project_path: &str) -> String {
    let parts: Vec<&str> = project_path.split('/').filter(|s| !s.is_empty()).collect();

    // Pattern 1: Claude Code worktrees — .claude/worktrees/<name>
    // e.g., /path/to/inops/.claude/worktrees/decom-rules-ui → inops
    if let Some(pos) = parts.iter().position(|&s| s == ".claude") {
        if pos > 0 && parts.get(pos + 1) == Some(&"worktrees") && parts.get(pos + 2).is_some() {
            return parts[pos - 1].to_string();
        }
    }

    let last = parts.last().unwrap_or(&"unknown");

    // Pattern 2: Git worktrees — project--branch-name
    // e.g., neo-workflow--reviewer → neo-workflow
    if let Some(idx) = last.find("--") {
        return last[..idx].to_string();
    }

    // Pattern 3: Global .claude directory is not a real project
    if *last == ".claude" {
        return "claude-config".to_string();
    }

    last.to_string()
}

fn truncate_prompt(prompt: &str, max_len: usize) -> String {
    let cleaned = prompt
        .replace("[Pasted text]", "")
        .lines()
        .next()
        .unwrap_or(prompt)
        .trim()
        .to_string();

    if cleaned.len() <= max_len {
        cleaned
    } else {
        format!("{}...", &cleaned[..max_len])
    }
}
