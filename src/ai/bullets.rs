use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Activity, Accomplishment};

/// Generator for resume bullet points
pub struct BulletGenerator {
    templates: Vec<BulletTemplate>,
}

#[derive(Debug, Clone)]
pub struct BulletTemplate {
    pub name: String,
    pub pattern: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedBullet {
    pub text: String,
    pub style: BulletStyle,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BulletStyle {
    Impact,      // Focus on results and impact
    Technical,   // Focus on technical details
    Leadership,  // Focus on leadership and collaboration
    Concise,     // Short and punchy
}

impl Default for BulletGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl BulletGenerator {
    pub fn new() -> Self {
        Self {
            templates: Self::default_templates(),
        }
    }

    fn default_templates() -> Vec<BulletTemplate> {
        vec![
            BulletTemplate {
                name: "impact".to_string(),
                pattern: "[Action verb] [what you did] resulting in [quantified impact]".to_string(),
                example: "Redesigned checkout flow resulting in 25% increase in conversion rate".to_string(),
            },
            BulletTemplate {
                name: "technical".to_string(),
                pattern: "[Action verb] [technical solution] using [technologies] to [outcome]".to_string(),
                example: "Implemented real-time data pipeline using Kafka and Spark to process 1M events/day".to_string(),
            },
            BulletTemplate {
                name: "leadership".to_string(),
                pattern: "[Led/Mentored/Coordinated] [team/initiative] to [achieve outcome]".to_string(),
                example: "Led cross-functional team of 5 engineers to deliver critical security patch in 48 hours".to_string(),
            },
            BulletTemplate {
                name: "efficiency".to_string(),
                pattern: "[Action verb] [process/system] reducing [metric] by [percentage/amount]".to_string(),
                example: "Automated deployment pipeline reducing release time from 4 hours to 15 minutes".to_string(),
            },
        ]
    }

    /// Generate bullet points from an activity
    pub fn generate_from_activity(&self, activity: &Activity) -> Vec<GeneratedBullet> {
        let mut bullets = Vec::new();

        // Extract key information
        let title = &activity.title;
        let description = activity.description.as_deref().unwrap_or("");
        let impact = activity.metadata.get("impact")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Generate impact-focused bullet
        if !impact.is_empty() {
            bullets.push(GeneratedBullet {
                text: self.format_impact_bullet(title, impact),
                style: BulletStyle::Impact,
                confidence: 0.9,
            });
        }

        // Generate technical bullet if we have tech details
        if let Some(lines_changed) = activity.metadata.get("lines_changed").and_then(|v| v.as_u64()) {
            if lines_changed > 100 {
                bullets.push(GeneratedBullet {
                    text: self.format_technical_bullet(title, lines_changed),
                    style: BulletStyle::Technical,
                    confidence: 0.7,
                });
            }
        }

        // Generate concise bullet (always)
        bullets.push(GeneratedBullet {
            text: self.format_concise_bullet(title, description),
            style: BulletStyle::Concise,
            confidence: 0.8,
        });

        bullets
    }

    /// Generate bullet points from an accomplishment
    pub fn generate_from_accomplishment(&self, accomplishment: &Accomplishment) -> Vec<GeneratedBullet> {
        let mut bullets = Vec::new();

        // If we have STAR data, use it for high-quality bullets
        if let (Some(action), Some(result)) = (&accomplishment.action, &accomplishment.result) {
            bullets.push(GeneratedBullet {
                text: self.format_star_bullet(action, result),
                style: BulletStyle::Impact,
                confidence: 0.95,
            });
        }

        // Generate metric-based bullets
        for metric in &accomplishment.metrics {
            bullets.push(GeneratedBullet {
                text: self.format_metric_bullet(&accomplishment.title, metric),
                style: BulletStyle::Impact,
                confidence: 0.9,
            });
        }

        // If we have generated story, create a concise summary
        if accomplishment.generated_story.is_some() {
            bullets.push(GeneratedBullet {
                text: self.format_concise_bullet(&accomplishment.title, ""),
                style: BulletStyle::Concise,
                confidence: 0.8,
            });
        }

        bullets
    }

    fn format_impact_bullet(&self, title: &str, impact: &str) -> String {
        let action_verb = self.extract_or_add_action_verb(title);
        format!("{} resulting in {}", action_verb, impact.to_lowercase())
    }

    fn format_technical_bullet(&self, title: &str, lines_changed: u64) -> String {
        let action_verb = self.extract_or_add_action_verb(title);
        format!("{} involving {}+ lines of code", action_verb, lines_changed)
    }

    fn format_concise_bullet(&self, title: &str, description: &str) -> String {
        let clean_title = self.extract_or_add_action_verb(title);
        if description.is_empty() {
            clean_title
        } else {
            // Take first sentence of description if it adds value
            let first_sentence = description.split('.').next().unwrap_or("");
            if first_sentence.len() > 10 && !clean_title.contains(first_sentence) {
                format!("{}: {}", clean_title, first_sentence.trim())
            } else {
                clean_title
            }
        }
    }

    fn format_star_bullet(&self, action: &str, result: &str) -> String {
        let action_verb = self.extract_or_add_action_verb(action);
        format!("{}, {}", action_verb, result.to_lowercase())
    }

    fn format_metric_bullet(&self, title: &str, metric: &crate::types::Metric) -> String {
        let action_verb = self.extract_or_add_action_verb(title);
        match metric.metric_type.as_str() {
            "percentage" => format!("{} achieving {}% {}", action_verb, metric.value, metric.description),
            "currency" => format!("{} saving ${} {}", action_verb, metric.value, metric.description),
            "time" => format!("{} reducing {} by {} hours", action_verb, metric.description, metric.value),
            _ => format!("{} with {} {}", action_verb, metric.value, metric.description),
        }
    }

    fn extract_or_add_action_verb(&self, text: &str) -> String {
        let action_verbs = [
            "implemented", "developed", "designed", "built", "created", "led",
            "improved", "optimized", "reduced", "increased", "launched",
            "deployed", "migrated", "refactored", "automated", "integrated",
            "architected", "engineered", "delivered", "shipped", "fixed",
            "resolved", "established", "introduced", "pioneered", "spearheaded",
        ];

        let first_word = text.split_whitespace().next().unwrap_or("").to_lowercase();

        if action_verbs.iter().any(|&v| first_word.starts_with(v)) {
            // Already has action verb, capitalize first letter
            let mut chars = text.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => text.to_string(),
            }
        } else {
            // Add a generic action verb
            format!("Implemented {}", text.to_lowercase())
        }
    }

    /// Get available templates
    pub fn templates(&self) -> &[BulletTemplate] {
        &self.templates
    }

    /// Apply a specific template to content
    pub fn apply_template(&self, template_name: &str, content: &str, impact: &str) -> Option<String> {
        self.templates.iter()
            .find(|t| t.name == template_name)
            .map(|_| self.format_impact_bullet(content, impact))
    }
}

/// Predefined action verbs categorized by type
pub mod action_verbs {
    pub const LEADERSHIP: &[&str] = &[
        "led", "managed", "directed", "coordinated", "supervised",
        "mentored", "coached", "guided", "facilitated", "spearheaded",
    ];

    pub const TECHNICAL: &[&str] = &[
        "implemented", "developed", "engineered", "architected", "designed",
        "built", "created", "programmed", "coded", "debugged",
    ];

    pub const IMPROVEMENT: &[&str] = &[
        "improved", "enhanced", "optimized", "streamlined", "accelerated",
        "reduced", "increased", "expanded", "upgraded", "modernized",
    ];

    pub const DELIVERY: &[&str] = &[
        "delivered", "shipped", "launched", "deployed", "released",
        "published", "completed", "executed", "achieved", "accomplished",
    ];

    pub const COLLABORATION: &[&str] = &[
        "collaborated", "partnered", "coordinated", "aligned", "integrated",
        "unified", "consolidated", "synchronized", "liaised", "bridged",
    ];
}
