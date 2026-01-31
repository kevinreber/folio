pub mod markdown;
pub mod json;
pub mod yaml;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::types::{Activity, Accomplishment};

/// Export format enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Markdown,
    Json,
    Yaml,
}

impl std::str::FromStr for ExportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            _ => Err(anyhow::anyhow!("Unknown export format: {}. Use: markdown, json, or yaml", s)),
        }
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Markdown => write!(f, "markdown"),
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
        }
    }
}

/// Export data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub exported_at: DateTime<Utc>,
    pub version: String,
    pub activities: Vec<Activity>,
    pub accomplishments: Vec<Accomplishment>,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub total_activities: usize,
    pub total_accomplishments: usize,
    pub date_range: Option<DateRange>,
    pub projects: Vec<String>,
    pub employers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Main exporter that handles all formats
pub struct Exporter;

impl Exporter {
    /// Export activities and accomplishments to the specified format
    pub fn export(
        activities: Vec<Activity>,
        accomplishments: Vec<Accomplishment>,
        format: ExportFormat,
    ) -> Result<String> {
        let data = Self::build_export_data(activities, accomplishments);

        match format {
            ExportFormat::Markdown => markdown::export(&data),
            ExportFormat::Json => json::export(&data),
            ExportFormat::Yaml => yaml::export(&data),
        }
    }

    /// Export to a file
    pub fn export_to_file(
        activities: Vec<Activity>,
        accomplishments: Vec<Accomplishment>,
        format: ExportFormat,
        path: &Path,
    ) -> Result<()> {
        let content = Self::export(activities, accomplishments, format)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn build_export_data(activities: Vec<Activity>, accomplishments: Vec<Accomplishment>) -> ExportData {
        let mut projects: Vec<String> = activities
            .iter()
            .filter_map(|a| a.project.clone())
            .collect();
        projects.sort();
        projects.dedup();

        let mut employers: Vec<String> = activities
            .iter()
            .filter_map(|a| a.employer.clone())
            .collect();
        employers.sort();
        employers.dedup();

        let date_range = if !activities.is_empty() {
            let mut dates: Vec<_> = activities.iter().map(|a| a.timestamp).collect();
            dates.sort();
            Some(DateRange {
                start: *dates.first().unwrap(),
                end: *dates.last().unwrap(),
            })
        } else {
            None
        };

        ExportData {
            exported_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            metadata: ExportMetadata {
                total_activities: activities.len(),
                total_accomplishments: accomplishments.len(),
                date_range,
                projects,
                employers,
            },
            activities,
            accomplishments,
        }
    }
}

/// Import data from various formats
pub struct Importer;

impl Importer {
    /// Import activities from JSON
    pub fn from_json(content: &str) -> Result<Vec<Activity>> {
        json::import(content)
    }

    /// Import activities from YAML
    pub fn from_yaml(content: &str) -> Result<Vec<Activity>> {
        yaml::import(content)
    }

    /// Import from file, detecting format from extension
    pub fn from_file(path: &Path) -> Result<Vec<Activity>> {
        let content = std::fs::read_to_string(path)?;
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "json" => Self::from_json(&content),
            "yaml" | "yml" => Self::from_yaml(&content),
            _ => Err(anyhow::anyhow!("Unknown file format: {}", extension)),
        }
    }
}
