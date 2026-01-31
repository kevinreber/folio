use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

const LINEAR_API_BASE: &str = "https://api.linear.app/graphql";

/// Linear API client for fetching issues and project data
pub struct LinearClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct LinearResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct LinearIssuesData {
    pub issues: LinearIssueConnection,
}

#[derive(Debug, Deserialize)]
pub struct LinearIssueConnection {
    pub nodes: Vec<LinearIssue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearIssue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub state: LinearState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub url: String,
    pub team: LinearTeam,
    pub project: Option<LinearProject>,
    pub labels: LinearLabelConnection,
    pub estimate: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct LinearState {
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
}

#[derive(Debug, Deserialize)]
pub struct LinearTeam {
    pub name: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct LinearProject {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LinearLabelConnection {
    pub nodes: Vec<LinearLabel>,
}

#[derive(Debug, Deserialize)]
pub struct LinearLabel {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearViewer {
    pub viewer: LinearUser,
}

#[derive(Debug, Deserialize)]
pub struct LinearUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl LinearClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .user_agent("folio-cli/0.2.0")
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
        }
    }

    fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let resp = self
            .client
            .post(LINEAR_API_BASE)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .context("Failed to send Linear API request")?;

        let response: LinearResponse<T> = resp.json().context("Failed to parse Linear response")?;
        Ok(response.data)
    }

    /// Get the authenticated user
    pub fn get_viewer(&self) -> Result<LinearUser> {
        let query = r#"
            query {
                viewer {
                    id
                    name
                    email
                }
            }
        "#;

        let data: LinearViewer = self.query(query, serde_json::json!({}))?;
        Ok(data.viewer)
    }

    /// Fetch completed issues assigned to the user
    pub fn get_completed_issues(&self, days_back: u32) -> Result<Vec<LinearIssue>> {
        let since = Utc::now() - chrono::Duration::days(days_back as i64);

        let query = r#"
            query($filter: IssueFilter) {
                issues(filter: $filter, first: 100) {
                    nodes {
                        id
                        identifier
                        title
                        description
                        priority
                        state {
                            name
                            type
                        }
                        createdAt
                        updatedAt
                        completedAt
                        canceledAt
                        url
                        team {
                            name
                            key
                        }
                        project {
                            name
                        }
                        labels {
                            nodes {
                                name
                            }
                        }
                        estimate
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "filter": {
                "assignee": { "isMe": { "eq": true } },
                "completedAt": { "gte": since.to_rfc3339() },
                "state": { "type": { "eq": "completed" } }
            }
        });

        let data: LinearIssuesData = self.query(query, variables)?;
        Ok(data.issues.nodes)
    }

    /// Fetch issues by team
    pub fn get_team_issues(&self, team_key: &str, days_back: u32) -> Result<Vec<LinearIssue>> {
        let since = Utc::now() - chrono::Duration::days(days_back as i64);

        let query = r#"
            query($filter: IssueFilter) {
                issues(filter: $filter, first: 100) {
                    nodes {
                        id
                        identifier
                        title
                        description
                        priority
                        state {
                            name
                            type
                        }
                        createdAt
                        updatedAt
                        completedAt
                        canceledAt
                        url
                        team {
                            name
                            key
                        }
                        project {
                            name
                        }
                        labels {
                            nodes {
                                name
                            }
                        }
                        estimate
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "updatedAt": { "gte": since.to_rfc3339() }
            }
        });

        let data: LinearIssuesData = self.query(query, variables)?;
        Ok(data.issues.nodes)
    }

    /// Convert a Linear issue to an activity
    pub fn issue_to_activity(&self, issue: &LinearIssue) -> Activity {
        let importance = self.infer_importance(issue);
        let labels: Vec<String> = issue.labels.nodes.iter().map(|l| l.name.clone()).collect();

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Linear,
            activity_type: ActivityType::IssueClosed,
            timestamp: issue.completed_at.unwrap_or(issue.updated_at),
            title: format!("{}: {}", issue.identifier, issue.title),
            description: issue.description.clone(),
            project: issue.project.as_ref().map(|p| p.name.clone()),
            employer: None,
            importance,
            metadata: serde_json::json!({
                "issue_id": issue.id,
                "identifier": issue.identifier,
                "url": issue.url,
                "team": issue.team.name,
                "team_key": issue.team.key,
                "priority": issue.priority,
                "estimate": issue.estimate,
                "labels": labels,
                "state": issue.state.name,
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn infer_importance(&self, issue: &LinearIssue) -> Importance {
        // Priority: 0 = no priority, 1 = urgent, 2 = high, 3 = medium, 4 = low
        match issue.priority {
            1 => Importance::High,   // Urgent
            2 => Importance::High,   // High
            3 => Importance::Medium, // Medium
            4 => Importance::Low,    // Low
            _ => {
                // Check labels
                let labels: Vec<&str> =
                    issue.labels.nodes.iter().map(|l| l.name.as_str()).collect();
                if labels.iter().any(|l| {
                    l.to_lowercase().contains("critical") || l.to_lowercase().contains("urgent")
                }) {
                    Importance::High
                } else {
                    Importance::Medium
                }
            }
        }
    }
}

/// Parse a Linear URL to extract issue identifier
pub fn parse_linear_url(url: &str) -> Option<String> {
    let url = url::Url::parse(url).ok()?;

    if !url.host_str()?.contains("linear.app") {
        return None;
    }

    // Linear URLs are like: https://linear.app/team/issue/TEAM-123/title-slug
    let segments: Vec<&str> = url.path_segments()?.collect();

    if segments.len() >= 3 && segments[1] == "issue" {
        return Some(segments[2].to_string());
    }

    None
}
