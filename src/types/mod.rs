use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivitySource {
    Git,
    Github,
    Linear,
    Jira,
    Manual,
    ScreenCapture,
}

impl std::fmt::Display for ActivitySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git => write!(f, "git"),
            Self::Github => write!(f, "github"),
            Self::Linear => write!(f, "linear"),
            Self::Jira => write!(f, "jira"),
            Self::Manual => write!(f, "manual"),
            Self::ScreenCapture => write!(f, "screen_capture"),
        }
    }
}

impl std::str::FromStr for ActivitySource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "git" => Ok(Self::Git),
            "github" => Ok(Self::Github),
            "linear" => Ok(Self::Linear),
            "jira" => Ok(Self::Jira),
            "manual" => Ok(Self::Manual),
            "screen_capture" => Ok(Self::ScreenCapture),
            _ => Err(anyhow::anyhow!("Unknown source: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Commit,
    PrMerged,
    PrReviewed,
    IssueClosed,
    Deploy,
    Incident,
    ManualEntry,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Commit => write!(f, "commit"),
            Self::PrMerged => write!(f, "pr_merged"),
            Self::PrReviewed => write!(f, "pr_reviewed"),
            Self::IssueClosed => write!(f, "issue_closed"),
            Self::Deploy => write!(f, "deploy"),
            Self::Incident => write!(f, "incident"),
            Self::ManualEntry => write!(f, "manual_entry"),
        }
    }
}

impl std::str::FromStr for ActivityType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "commit" => Ok(Self::Commit),
            "pr_merged" => Ok(Self::PrMerged),
            "pr_reviewed" => Ok(Self::PrReviewed),
            "issue_closed" => Ok(Self::IssueClosed),
            "deploy" => Ok(Self::Deploy),
            "incident" => Ok(Self::Incident),
            "manual_entry" => Ok(Self::ManualEntry),
            _ => Err(anyhow::anyhow!("Unknown activity type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Importance {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Importance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

impl std::str::FromStr for Importance {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" | "med" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err(anyhow::anyhow!("Unknown importance: {}", s)),
        }
    }
}

/// Raw activity captured from various sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub source: ActivitySource,
    pub activity_type: ActivityType,
    pub timestamp: DateTime<Utc>,
    pub title: String,
    pub description: Option<String>,
    pub project: Option<String>,
    pub employer: Option<String>,
    pub importance: Importance,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Activity {
    pub fn new_manual(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Manual,
            activity_type: ActivityType::ManualEntry,
            timestamp: now,
            title: title.into(),
            description: None,
            project: None,
            employer: None,
            importance: Importance::Medium,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.metadata["impact"] = serde_json::json!(impact.into());
        self
    }

    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    pub fn with_employer(mut self, employer: impl Into<String>) -> Self {
        self.employer = Some(employer.into());
        self
    }

    pub fn with_importance(mut self, importance: Importance) -> Self {
        self.importance = importance;
        self
    }
}

/// Metric associated with an accomplishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub metric_type: String, // percentage, absolute, currency, time
    pub value: f64,
    pub description: String,
}

/// Enriched accomplishment derived from activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accomplishment {
    pub id: String,
    pub title: String,
    pub situation: Option<String>,
    pub task: Option<String>,
    pub action: Option<String>,
    pub result: Option<String>,
    pub metrics: Vec<Metric>,
    pub activity_ids: Vec<String>,
    pub skills: Vec<String>,
    pub themes: Vec<String>,
    pub employer: Option<String>,
    pub role: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub generated_bullets: Vec<String>,
    pub generated_story: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Employment history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employment {
    pub id: String,
    pub company: String,
    pub role: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub team_size: Option<i32>,
    pub tech_stack: Vec<String>,
    pub domain: Option<String>,
}
