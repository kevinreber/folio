use anyhow::Result;
use regex::Regex;
use url::Url;

use super::{GitHubClient, LinearClient};
use crate::types::Activity;

/// Enricher that can fetch details from URLs and convert to activities
pub struct LinkEnricher {
    github_client: Option<GitHubClient>,
    linear_client: Option<LinearClient>,
}

#[derive(Debug, Clone)]
pub enum LinkType {
    GitHubPr {
        owner: String,
        repo: String,
        number: u64,
    },
    GitHubIssue {
        owner: String,
        repo: String,
        number: u64,
    },
    GitHubRepo {
        owner: String,
        repo: String,
    },
    LinearIssue {
        identifier: String,
    },
    JiraIssue {
        project: String,
        number: String,
    },
    Unknown,
}

#[derive(Debug)]
pub struct EnrichedLink {
    pub url: String,
    pub link_type: LinkType,
    pub activity: Option<Activity>,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl LinkEnricher {
    pub fn new(github_token: Option<String>, linear_key: Option<String>) -> Self {
        Self {
            github_client: Some(GitHubClient::new(github_token)),
            linear_client: linear_key.map(LinearClient::new),
        }
    }

    /// Detect the type of link from a URL
    pub fn detect_link_type(&self, url: &str) -> LinkType {
        // Try to parse as URL
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return LinkType::Unknown,
        };

        let host = parsed.host_str().unwrap_or("");
        let path_segments: Vec<&str> = parsed
            .path_segments()
            .map(|s| s.collect())
            .unwrap_or_default();

        // GitHub detection
        if host == "github.com" || host.ends_with(".github.com") {
            if path_segments.len() >= 4 {
                let owner = path_segments[0].to_string();
                let repo = path_segments[1].to_string();

                match path_segments[2] {
                    "pull" => {
                        if let Ok(number) = path_segments[3].parse() {
                            return LinkType::GitHubPr {
                                owner,
                                repo,
                                number,
                            };
                        }
                    }
                    "issues" => {
                        if let Ok(number) = path_segments[3].parse() {
                            return LinkType::GitHubIssue {
                                owner,
                                repo,
                                number,
                            };
                        }
                    }
                    _ => {}
                }
            }

            if path_segments.len() >= 2 {
                return LinkType::GitHubRepo {
                    owner: path_segments[0].to_string(),
                    repo: path_segments[1].to_string(),
                };
            }
        }

        // Linear detection
        if host.contains("linear.app") && path_segments.len() >= 3 && path_segments[1] == "issue" {
            return LinkType::LinearIssue {
                identifier: path_segments[2].to_string(),
            };
        }

        // Jira detection
        if host.contains("atlassian.net") || host.contains("jira") {
            // Try to extract Jira issue key from URL
            let jira_pattern = Regex::new(r"([A-Z][A-Z0-9]+-\d+)").ok();
            if let Some(pattern) = jira_pattern {
                if let Some(captures) = pattern.captures(url) {
                    if let Some(m) = captures.get(1) {
                        let key = m.as_str();
                        let parts: Vec<&str> = key.split('-').collect();
                        if parts.len() == 2 {
                            return LinkType::JiraIssue {
                                project: parts[0].to_string(),
                                number: parts[1].to_string(),
                            };
                        }
                    }
                }
            }
        }

        LinkType::Unknown
    }

    /// Enrich a URL by fetching its details
    pub fn enrich(&self, url: &str) -> Result<EnrichedLink> {
        let link_type = self.detect_link_type(url);

        let mut enriched = EnrichedLink {
            url: url.to_string(),
            link_type: link_type.clone(),
            activity: None,
            title: None,
            description: None,
        };

        match &link_type {
            LinkType::GitHubPr {
                owner,
                repo,
                number,
            } => {
                if let Some(ref client) = self.github_client {
                    match client.get_pr(owner, repo, *number) {
                        Ok(pr) => {
                            enriched.title = Some(pr.title.clone());
                            enriched.description = pr.body.clone();
                            enriched.activity = Some(client.pr_to_activity(&pr));
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to fetch PR details: {}", e);
                        }
                    }
                }
            }
            LinkType::GitHubIssue {
                owner,
                repo,
                number,
            } => {
                if self.github_client.is_some() {
                    // For issues, we'd need a separate API call
                    // For now, just set basic info
                    enriched.title = Some(format!("Issue #{} in {}/{}", number, owner, repo));
                }
            }
            LinkType::LinearIssue { identifier } => {
                // Linear requires the issue to be fetched via GraphQL
                // Would need to implement a get_issue_by_identifier method
                enriched.title = Some(format!("Linear issue {}", identifier));
            }
            LinkType::JiraIssue { project, number } => {
                enriched.title = Some(format!("Jira issue {}-{}", project, number));
            }
            LinkType::GitHubRepo { owner, repo } => {
                enriched.title = Some(format!("Repository: {}/{}", owner, repo));
            }
            LinkType::Unknown => {}
        }

        Ok(enriched)
    }

    /// Extract all links from text
    pub fn extract_links(&self, text: &str) -> Vec<String> {
        let url_pattern =
            Regex::new(r#"https?://[^\s<>\[\](){}"',;]+[^\s<>\[\](){}"',;.!?]"#).unwrap();

        url_pattern
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Enrich all links found in text
    pub fn enrich_all(&self, text: &str) -> Vec<EnrichedLink> {
        self.extract_links(text)
            .into_iter()
            .filter_map(|url| self.enrich(&url).ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_github_pr() {
        let enricher = LinkEnricher::new(None, None);

        match enricher.detect_link_type("https://github.com/owner/repo/pull/123") {
            LinkType::GitHubPr {
                owner,
                repo,
                number,
            } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert_eq!(number, 123);
            }
            _ => panic!("Expected GitHubPr"),
        }
    }

    #[test]
    fn test_detect_linear() {
        let enricher = LinkEnricher::new(None, None);

        match enricher.detect_link_type("https://linear.app/team/issue/TEAM-123/some-title") {
            LinkType::LinearIssue { identifier } => {
                assert_eq!(identifier, "TEAM-123");
            }
            _ => panic!("Expected LinearIssue"),
        }
    }

    #[test]
    fn test_extract_links() {
        let enricher = LinkEnricher::new(None, None);
        let text = "Check out https://github.com/user/repo/pull/1 and also https://linear.app/team/issue/ABC-123/title";
        let links = enricher.extract_links(text);

        assert_eq!(links.len(), 2);
    }
}
