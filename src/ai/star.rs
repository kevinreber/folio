use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Activity, Accomplishment, Metric};

/// Builder for STAR (Situation, Task, Action, Result) formatted stories
pub struct StarBuilder {
    prompts: StarPrompts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarStory {
    pub situation: String,
    pub task: String,
    pub action: String,
    pub result: String,
    pub metrics: Vec<Metric>,
    pub skills: Vec<String>,
    pub full_narrative: String,
}

#[derive(Debug, Clone)]
pub struct StarPrompts {
    pub situation: Vec<String>,
    pub task: Vec<String>,
    pub action: Vec<String>,
    pub result: Vec<String>,
}

impl Default for StarBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StarBuilder {
    pub fn new() -> Self {
        Self {
            prompts: Self::default_prompts(),
        }
    }

    fn default_prompts() -> StarPrompts {
        StarPrompts {
            situation: vec![
                "What was the context or background?".to_string(),
                "What problem or opportunity existed?".to_string(),
                "What were the constraints or challenges?".to_string(),
                "Who were the stakeholders involved?".to_string(),
            ],
            task: vec![
                "What was your specific responsibility?".to_string(),
                "What goal were you trying to achieve?".to_string(),
                "What was expected of you?".to_string(),
                "What would success look like?".to_string(),
            ],
            action: vec![
                "What specific steps did you take?".to_string(),
                "What decisions did you make?".to_string(),
                "What tools or technologies did you use?".to_string(),
                "How did you overcome obstacles?".to_string(),
            ],
            result: vec![
                "What was the outcome?".to_string(),
                "Can you quantify the impact?".to_string(),
                "What did you learn?".to_string(),
                "How did this affect the business/team?".to_string(),
            ],
        }
    }

    /// Get prompts for building a STAR story interactively
    pub fn get_prompts(&self) -> &StarPrompts {
        &self.prompts
    }

    /// Build a STAR story from provided components
    pub fn build_story(
        &self,
        situation: String,
        task: String,
        action: String,
        result: String,
        metrics: Vec<Metric>,
        skills: Vec<String>,
    ) -> StarStory {
        let full_narrative = self.compose_narrative(&situation, &task, &action, &result);

        StarStory {
            situation,
            task,
            action,
            result,
            metrics,
            skills,
            full_narrative,
        }
    }

    /// Build a STAR story from an activity with automatic extraction
    pub fn from_activity(&self, activity: &Activity) -> StarStoryDraft {
        let mut draft = StarStoryDraft::new();

        // Extract situation from context
        if let Some(project) = &activity.project {
            draft.situation_hints.push(format!("Working on the {} project", project));
        }

        if let Some(employer) = &activity.employer {
            draft.situation_hints.push(format!("At {}", employer));
        }

        // Extract task from title
        draft.task_hints.push(activity.title.clone());

        // Extract action from description
        if let Some(desc) = &activity.description {
            draft.action_hints.push(desc.clone());
        }

        // Extract result from impact
        if let Some(impact) = activity.metadata.get("impact").and_then(|v| v.as_str()) {
            draft.result_hints.push(impact.to_string());
        }

        // Extract metrics from metadata
        if let Some(lines) = activity.metadata.get("lines_changed").and_then(|v| v.as_u64()) {
            draft.metrics.push(Metric {
                metric_type: "absolute".to_string(),
                value: lines as f64,
                description: "lines of code changed".to_string(),
            });
        }

        if let Some(files) = activity.metadata.get("files_changed").and_then(|v| v.as_u64()) {
            draft.metrics.push(Metric {
                metric_type: "absolute".to_string(),
                value: files as f64,
                description: "files modified".to_string(),
            });
        }

        draft
    }

    /// Convert a STAR story to an accomplishment
    pub fn to_accomplishment(&self, story: &StarStory, activity_ids: Vec<String>) -> Accomplishment {
        Accomplishment {
            id: uuid::Uuid::new_v4().to_string(),
            title: self.generate_title(&story.action, &story.result),
            situation: Some(story.situation.clone()),
            task: Some(story.task.clone()),
            action: Some(story.action.clone()),
            result: Some(story.result.clone()),
            metrics: story.metrics.clone(),
            activity_ids,
            skills: story.skills.clone(),
            themes: self.extract_themes(&story),
            employer: None,
            role: None,
            start_date: None,
            end_date: None,
            generated_bullets: self.generate_bullets(&story),
            generated_story: Some(story.full_narrative.clone()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn compose_narrative(&self, situation: &str, task: &str, action: &str, result: &str) -> String {
        format!(
            "{}\n\nGiven this context, {}.\n\n{}\n\nAs a result, {}",
            situation, task.to_lowercase(), action, result.to_lowercase()
        )
    }

    fn generate_title(&self, action: &str, result: &str) -> String {
        // Take the first meaningful phrase from action
        let action_start = action.split('.').next().unwrap_or(action);
        if action_start.len() > 60 {
            action_start[..57].to_string() + "..."
        } else {
            action_start.to_string()
        }
    }

    fn generate_bullets(&self, story: &StarStory) -> Vec<String> {
        let mut bullets = Vec::new();

        // Action-result bullet
        if let Some(first_action) = story.action.split('.').next() {
            if let Some(first_result) = story.result.split('.').next() {
                bullets.push(format!(
                    "{}, resulting in {}",
                    first_action.trim(),
                    first_result.trim().to_lowercase()
                ));
            }
        }

        // Metric-based bullets
        for metric in &story.metrics {
            match metric.metric_type.as_str() {
                "percentage" => {
                    bullets.push(format!("Achieved {}% {}", metric.value, metric.description));
                }
                "currency" => {
                    bullets.push(format!("Generated ${} in {}", metric.value, metric.description));
                }
                "time" => {
                    bullets.push(format!("Reduced {} by {} hours", metric.description, metric.value));
                }
                _ => {
                    bullets.push(format!("{} {}", metric.value, metric.description));
                }
            }
        }

        bullets
    }

    fn extract_themes(&self, story: &StarStory) -> Vec<String> {
        let mut themes = Vec::new();
        let full_text = format!(
            "{} {} {} {}",
            story.situation, story.task, story.action, story.result
        ).to_lowercase();

        // Common themes detection
        let theme_patterns: &[(&str, &[&str])] = &[
            ("performance", &["performance", "optimization", "speed", "latency", "throughput"]),
            ("scalability", &["scale", "scaling", "distributed", "microservice"]),
            ("reliability", &["reliability", "uptime", "availability", "resilient"]),
            ("security", &["security", "authentication", "authorization", "vulnerability"]),
            ("infrastructure", &["infrastructure", "deploy", "ci/cd", "kubernetes", "docker"]),
            ("data", &["data", "database", "analytics", "pipeline", "etl"]),
            ("frontend", &["frontend", "ui", "ux", "react", "angular", "vue"]),
            ("backend", &["backend", "api", "server", "rest", "graphql"]),
            ("mobile", &["mobile", "ios", "android", "react native", "flutter"]),
            ("testing", &["test", "testing", "qa", "automation"]),
            ("documentation", &["documentation", "docs", "readme", "wiki"]),
            ("collaboration", &["team", "collaborate", "coordinate", "stakeholder"]),
            ("mentorship", &["mentor", "coach", "train", "onboard"]),
            ("leadership", &["lead", "manage", "direct", "spearhead"]),
        ];

        for (theme, keywords) in theme_patterns {
            if keywords.iter().any(|k| full_text.contains(k)) {
                themes.push(theme.to_string());
            }
        }

        themes
    }
}

/// Draft STAR story with hints extracted from activities
#[derive(Debug, Clone, Default)]
pub struct StarStoryDraft {
    pub situation_hints: Vec<String>,
    pub task_hints: Vec<String>,
    pub action_hints: Vec<String>,
    pub result_hints: Vec<String>,
    pub metrics: Vec<Metric>,
    pub suggested_skills: Vec<String>,
}

impl StarStoryDraft {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the draft has enough information to build a story
    pub fn is_complete(&self) -> bool {
        !self.situation_hints.is_empty()
            && !self.task_hints.is_empty()
            && !self.action_hints.is_empty()
            && !self.result_hints.is_empty()
    }

    /// Convert draft hints to a story (takes first hint of each type)
    pub fn to_story(&self, skills: Vec<String>) -> Option<StarStory> {
        if !self.is_complete() {
            return None;
        }

        let builder = StarBuilder::new();
        Some(builder.build_story(
            self.situation_hints.first()?.clone(),
            self.task_hints.first()?.clone(),
            self.action_hints.first()?.clone(),
            self.result_hints.first()?.clone(),
            self.metrics.clone(),
            skills,
        ))
    }
}

/// Templates for common STAR story patterns
pub mod templates {
    pub const BUG_FIX: &str = r#"
Situation: [Production system] was experiencing [problem] affecting [users/metrics].
Task: I needed to [identify and fix the issue] within [timeframe] to [restore service/prevent impact].
Action: I [investigated by doing X], [identified the root cause as Y], and [implemented fix Z].
Result: [Resolved the issue in X hours], [prevented $Y in losses], [improved system reliability by Z%].
"#;

    pub const FEATURE_DELIVERY: &str = r#"
Situation: [Business/users] needed [capability] to [achieve goal].
Task: I was responsible for [designing and implementing] the [feature] as part of [initiative].
Action: I [designed the architecture], [implemented using technologies X, Y, Z], and [collaborated with team/stakeholders].
Result: [Delivered feature on time], [adopted by X users], [increased metric by Y%].
"#;

    pub const PERFORMANCE_OPTIMIZATION: &str = r#"
Situation: [System/feature] was experiencing [performance issues] causing [impact].
Task: I needed to [improve performance] to meet [target metrics/SLAs].
Action: I [profiled the system], [identified bottlenecks in X], and [implemented optimizations Y, Z].
Result: [Improved performance by X%], [reduced latency from Y to Z], [saved $A in infrastructure costs].
"#;

    pub const TECHNICAL_LEADERSHIP: &str = r#"
Situation: [Team/org] was facing [technical challenge] that required [expertise/coordination].
Task: I was asked to [lead the initiative] to [achieve technical goal].
Action: I [defined the technical strategy], [coordinated across X teams], and [mentored Y engineers].
Result: [Successfully delivered X], [established best practices], [improved team velocity by Y%].
"#;
}
