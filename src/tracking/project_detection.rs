//! Project detection module
//!
//! Automatically detects project associations from window titles,
//! file paths, and other context.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for project detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetectionConfig {
    /// Enable project detection
    pub enabled: bool,
    /// Known project mappings (pattern -> project name)
    pub project_mappings: HashMap<String, String>,
    /// Directories that contain projects
    pub project_dirs: Vec<PathBuf>,
    /// File patterns that indicate project roots
    pub project_markers: Vec<String>,
}

impl Default for ProjectDetectionConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            enabled: true,
            project_mappings: HashMap::new(),
            project_dirs: vec![
                home.join("code"),
                home.join("projects"),
                home.join("work"),
                home.join("dev"),
            ],
            project_markers: vec![
                ".git".to_string(),
                "package.json".to_string(),
                "Cargo.toml".to_string(),
                "go.mod".to_string(),
                "pyproject.toml".to_string(),
                "pom.xml".to_string(),
                "build.gradle".to_string(),
            ],
        }
    }
}

/// Project detector
pub struct ProjectDetector {
    config: ProjectDetectionConfig,
    cached_projects: HashMap<String, String>,
}

impl ProjectDetector {
    pub fn new(config: ProjectDetectionConfig) -> Self {
        Self {
            config,
            cached_projects: HashMap::new(),
        }
    }

    /// Detect project from a window title
    pub fn detect_from_title(&self, title: &str) -> Option<String> {
        // Check explicit mappings first
        for (pattern, project) in &self.config.project_mappings {
            if title.to_lowercase().contains(&pattern.to_lowercase()) {
                return Some(project.clone());
            }
        }

        // Try to extract project from common IDE patterns
        if let Some(project) = self.extract_from_ide_title(title) {
            return Some(project);
        }

        // Try to extract from file path in title
        if let Some(project) = self.extract_from_path_in_title(title) {
            return Some(project);
        }

        // Try to extract from GitHub/GitLab URLs in title
        if let Some(project) = self.extract_from_git_url(title) {
            return Some(project);
        }

        None
    }

    /// Detect project from a file path
    pub fn detect_from_path(&mut self, path: &str) -> Option<String> {
        // Check cache first
        if let Some(project) = self.cached_projects.get(path) {
            return Some(project.clone());
        }

        let path_buf = PathBuf::from(path);

        // Walk up the directory tree looking for project markers
        let mut current = path_buf.as_path();
        while let Some(parent) = current.parent() {
            for marker in &self.config.project_markers {
                if parent.join(marker).exists() {
                    if let Some(name) = parent.file_name().and_then(|n| n.to_str()) {
                        self.cached_projects.insert(path.to_string(), name.to_string());
                        return Some(name.to_string());
                    }
                }
            }
            current = parent;
        }

        None
    }

    /// Extract project from IDE window titles
    fn extract_from_ide_title(&self, title: &str) -> Option<String> {
        // VS Code: "filename - project - Visual Studio Code"
        // IntelliJ: "filename - project - IntelliJ IDEA"
        // Sublime: "filename - project - Sublime Text"

        let patterns = [
            // VS Code pattern
            Regex::new(r"(?:.*?)\s*[-–—]\s*([^-–—]+)\s*[-–—]\s*(?:Visual Studio Code|VSCodium|Code - OSS)")
                .ok()?,
            // IntelliJ pattern
            Regex::new(r"(?:.*?)\s*[-–—]\s*\[([^\]]+)\]").ok()?,
            // Generic "file - project" pattern
            Regex::new(r"(?:.*?)\s*[-–—]\s*([^-–—]+?)\s*[-–—]").ok()?,
        ];

        for pattern in patterns {
            if let Some(caps) = pattern.captures(title) {
                if let Some(project) = caps.get(1) {
                    let project_name = project.as_str().trim();
                    if !project_name.is_empty() && project_name.len() < 50 {
                        return Some(project_name.to_string());
                    }
                }
            }
        }

        None
    }

    /// Extract project from file path in window title
    fn extract_from_path_in_title(&self, title: &str) -> Option<String> {
        // Look for path patterns like ~/code/project/ or /Users/name/projects/project/
        let path_pattern = Regex::new(
            r"(?:~|/(?:Users|home)/[^/]+)/(?:code|projects|work|dev)/([^/\s]+)"
        ).ok()?;

        if let Some(caps) = path_pattern.captures(title) {
            if let Some(project) = caps.get(1) {
                return Some(project.as_str().to_string());
            }
        }

        None
    }

    /// Extract project from GitHub/GitLab URL in title
    fn extract_from_git_url(&self, title: &str) -> Option<String> {
        // GitHub: "owner/repo"
        // GitLab: "group/project"
        let git_pattern = Regex::new(
            r"(?:github\.com|gitlab\.com|bitbucket\.org)/([^/]+)/([^/\s?#]+)"
        ).ok()?;

        if let Some(caps) = git_pattern.captures(title) {
            if let Some(repo) = caps.get(2) {
                return Some(repo.as_str().to_string());
            }
        }

        None
    }

    /// Detect project from browser URL
    pub fn detect_from_url(&self, url: &str) -> Option<String> {
        // GitHub repository
        if let Some(project) = self.extract_from_git_url(url) {
            return Some(project);
        }

        // Jira project
        if url.contains("atlassian.net") || url.contains("jira") {
            let jira_pattern = Regex::new(r"/projects?/([A-Z0-9]+)").ok()?;
            if let Some(caps) = jira_pattern.captures(url) {
                if let Some(project) = caps.get(1) {
                    return Some(project.as_str().to_string());
                }
            }
        }

        // Linear project
        if url.contains("linear.app") {
            let linear_pattern = Regex::new(r"/([A-Z]+-\d+)").ok()?;
            if let Some(caps) = linear_pattern.captures(url) {
                if let Some(issue) = caps.get(1) {
                    // Return the project prefix
                    let issue_str = issue.as_str();
                    if let Some(idx) = issue_str.find('-') {
                        return Some(issue_str[..idx].to_string());
                    }
                }
            }
        }

        None
    }

    /// Add a project mapping
    pub fn add_mapping(&mut self, pattern: String, project: String) {
        self.config.project_mappings.insert(pattern, project);
    }

    /// Get all known projects from mappings
    pub fn known_projects(&self) -> Vec<&String> {
        self.config.project_mappings.values().collect()
    }

    /// Clear the project cache
    pub fn clear_cache(&mut self) {
        self.cached_projects.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_vscode_title() {
        let detector = ProjectDetector::new(ProjectDetectionConfig::default());

        let title = "main.rs - folio - Visual Studio Code";
        assert_eq!(detector.detect_from_title(title), Some("folio".to_string()));
    }

    #[test]
    fn test_detect_from_github_url() {
        let detector = ProjectDetector::new(ProjectDetectionConfig::default());

        let title = "Pull Request #123 - github.com/kevinreber/folio";
        assert_eq!(detector.detect_from_title(title), Some("folio".to_string()));
    }

    #[test]
    fn test_detect_from_url() {
        let detector = ProjectDetector::new(ProjectDetectionConfig::default());

        let url = "https://github.com/anthropics/claude-code/pull/123";
        assert_eq!(detector.detect_from_url(url), Some("claude-code".to_string()));
    }

    #[test]
    fn test_custom_mapping() {
        let mut detector = ProjectDetector::new(ProjectDetectionConfig::default());
        detector.add_mapping("PROJ-".to_string(), "My Project".to_string());

        let title = "PROJ-123: Fix bug - Jira";
        assert_eq!(detector.detect_from_title(title), Some("My Project".to_string()));
    }
}
