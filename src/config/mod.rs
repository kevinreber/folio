use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,
    /// Git integration settings
    pub git: GitConfig,
    /// GitHub integration settings
    pub github: GitHubConfig,
    /// Linear integration settings
    pub linear: LinearConfig,
    /// Jira integration settings
    pub jira: JiraConfig,
    /// AI/LLM settings
    pub ai: AiConfig,
    /// Export settings
    pub export: ExportConfig,
    /// Watcher settings for background monitoring
    pub watcher: WatcherConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Default employer/company for new entries
    pub default_employer: Option<String>,
    /// Default project for new entries
    pub default_project: Option<String>,
    /// User's git email for filtering commits
    pub git_email: Option<String>,
    /// Enable colored output
    pub color: bool,
    /// Date format for display
    pub date_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Enable git scanning
    pub enabled: bool,
    /// Directories to scan for git repos
    pub scan_dirs: Vec<PathBuf>,
    /// Maximum depth to scan for repos
    pub max_depth: usize,
    /// Number of days to look back for commits
    pub days_back: u32,
    /// Minimum lines changed to consider significant
    pub min_lines_changed: usize,
    /// Ignore patterns for repos
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Enable GitHub integration
    pub enabled: bool,
    /// GitHub personal access token
    pub token: Option<String>,
    /// Repositories to track (owner/repo format)
    pub repositories: Vec<String>,
    /// Days to look back for activity
    pub days_back: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearConfig {
    /// Enable Linear integration
    pub enabled: bool,
    /// Linear API key
    pub api_key: Option<String>,
    /// Teams to track
    pub teams: Vec<String>,
    /// Days to look back for activity
    pub days_back: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    /// Enable Jira integration
    pub enabled: bool,
    /// Jira instance URL
    pub url: Option<String>,
    /// Jira API token
    pub api_token: Option<String>,
    /// Jira user email
    pub email: Option<String>,
    /// Projects to track
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Enable AI features
    pub enabled: bool,
    /// API provider (openai, anthropic, local)
    pub provider: String,
    /// API endpoint (for custom/local providers)
    pub api_endpoint: Option<String>,
    /// API key
    pub api_key: Option<String>,
    /// Model to use
    pub model: String,
    /// Temperature for generation
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Default export format
    pub default_format: String,
    /// Default export directory
    pub output_dir: PathBuf,
    /// Include accomplishments in export
    pub include_accomplishments: bool,
    /// Include activities in export
    pub include_activities: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// Enable background watcher
    pub enabled: bool,
    /// Watch interval in seconds
    pub interval_seconds: u64,
    /// Enable desktop notifications
    pub notifications: bool,
    /// Minimum significance score to notify
    pub notify_threshold: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            git: GitConfig::default(),
            github: GitHubConfig::default(),
            linear: LinearConfig::default(),
            jira: JiraConfig::default(),
            ai: AiConfig::default(),
            export: ExportConfig::default(),
            watcher: WatcherConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_employer: None,
            default_project: None,
            git_email: crate::integrations::git::get_git_user_email(),
            color: true,
            date_format: "%Y-%m-%d".to_string(),
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            enabled: true,
            scan_dirs: vec![
                home.join("code"),
                home.join("projects"),
                home.join("work"),
                home.join("dev"),
            ],
            max_depth: 3,
            days_back: 30,
            min_lines_changed: 10,
            ignore_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "vendor".to_string(),
            ],
        }
    }
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token: std::env::var("GITHUB_TOKEN").ok(),
            repositories: vec![],
            days_back: 30,
        }
    }
}

impl Default for LinearConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: std::env::var("LINEAR_API_KEY").ok(),
            teams: vec![],
            days_back: 30,
        }
    }
}

impl Default for JiraConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: std::env::var("JIRA_URL").ok(),
            api_token: std::env::var("JIRA_API_TOKEN").ok(),
            email: std::env::var("JIRA_EMAIL").ok(),
            projects: vec![],
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "openai".to_string(),
            api_endpoint: None,
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            default_format: "markdown".to_string(),
            output_dir: home.join(".folio").join("exports"),
            include_accomplishments: true,
            include_activities: true,
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: 300, // 5 minutes
            notifications: true,
            notify_threshold: 0.5,
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".folio").join("config.toml"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    /// Get a value by key path (e.g., "git.enabled")
    pub fn get(&self, key: &str) -> Option<String> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["general", "default_employer"] => self.general.default_employer.clone(),
            ["general", "default_project"] => self.general.default_project.clone(),
            ["general", "git_email"] => self.general.git_email.clone(),
            ["general", "color"] => Some(self.general.color.to_string()),
            ["general", "date_format"] => Some(self.general.date_format.clone()),
            ["git", "enabled"] => Some(self.git.enabled.to_string()),
            ["git", "days_back"] => Some(self.git.days_back.to_string()),
            ["git", "min_lines_changed"] => Some(self.git.min_lines_changed.to_string()),
            ["github", "enabled"] => Some(self.github.enabled.to_string()),
            ["github", "token"] => self.github.token.clone(),
            ["github", "days_back"] => Some(self.github.days_back.to_string()),
            ["linear", "enabled"] => Some(self.linear.enabled.to_string()),
            ["linear", "api_key"] => self.linear.api_key.clone(),
            ["ai", "enabled"] => Some(self.ai.enabled.to_string()),
            ["ai", "provider"] => Some(self.ai.provider.clone()),
            ["ai", "model"] => Some(self.ai.model.clone()),
            ["watcher", "enabled"] => Some(self.watcher.enabled.to_string()),
            ["watcher", "interval_seconds"] => Some(self.watcher.interval_seconds.to_string()),
            _ => None,
        }
    }

    /// Set a value by key path
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["general", "default_employer"] => {
                self.general.default_employer = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            ["general", "default_project"] => {
                self.general.default_project = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            ["general", "git_email"] => {
                self.general.git_email = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            ["general", "color"] => {
                self.general.color = value.parse().unwrap_or(true);
            }
            ["git", "enabled"] => {
                self.git.enabled = value.parse().unwrap_or(false);
            }
            ["git", "days_back"] => {
                self.git.days_back = value.parse().unwrap_or(30);
            }
            ["git", "min_lines_changed"] => {
                self.git.min_lines_changed = value.parse().unwrap_or(10);
            }
            ["github", "enabled"] => {
                self.github.enabled = value.parse().unwrap_or(false);
            }
            ["github", "token"] => {
                self.github.token = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            ["github", "days_back"] => {
                self.github.days_back = value.parse().unwrap_or(30);
            }
            ["linear", "enabled"] => {
                self.linear.enabled = value.parse().unwrap_or(false);
            }
            ["linear", "api_key"] => {
                self.linear.api_key = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            ["ai", "enabled"] => {
                self.ai.enabled = value.parse().unwrap_or(false);
            }
            ["ai", "provider"] => {
                self.ai.provider = value.to_string();
            }
            ["ai", "model"] => {
                self.ai.model = value.to_string();
            }
            ["ai", "api_key"] => {
                self.ai.api_key = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            ["watcher", "enabled"] => {
                self.watcher.enabled = value.parse().unwrap_or(false);
            }
            ["watcher", "interval_seconds"] => {
                self.watcher.interval_seconds = value.parse().unwrap_or(300);
            }
            _ => {
                anyhow::bail!("Unknown config key: {}", key);
            }
        }

        Ok(())
    }

    /// List all configurable keys
    pub fn list_keys() -> Vec<&'static str> {
        vec![
            "general.default_employer",
            "general.default_project",
            "general.git_email",
            "general.color",
            "general.date_format",
            "git.enabled",
            "git.days_back",
            "git.min_lines_changed",
            "github.enabled",
            "github.token",
            "github.days_back",
            "linear.enabled",
            "linear.api_key",
            "ai.enabled",
            "ai.provider",
            "ai.model",
            "ai.api_key",
            "watcher.enabled",
            "watcher.interval_seconds",
        ]
    }
}
