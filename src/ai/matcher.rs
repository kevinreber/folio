use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::types::{Activity, Accomplishment};
use super::AutoTagger;

/// Matcher for comparing accomplishments against job descriptions
pub struct JobMatcher {
    tagger: AutoTagger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDescription {
    pub title: String,
    pub company: Option<String>,
    pub description: String,
    pub requirements: Vec<String>,
    pub preferred: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub job: JobDescription,
    pub overall_score: f32,
    pub matched_requirements: Vec<RequirementMatch>,
    pub matched_preferred: Vec<RequirementMatch>,
    pub gaps: Vec<String>,
    pub talking_points: Vec<TalkingPoint>,
    pub recommended_activities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementMatch {
    pub requirement: String,
    pub matched: bool,
    pub supporting_activities: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalkingPoint {
    pub topic: String,
    pub point: String,
    pub supporting_activity_id: Option<String>,
}

impl Default for JobMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl JobMatcher {
    pub fn new() -> Self {
        Self {
            tagger: AutoTagger::new(),
        }
    }

    /// Parse a job description from text
    pub fn parse_job_description(&self, title: String, text: &str) -> JobDescription {
        let lines: Vec<&str> = text.lines().collect();
        let mut requirements = Vec::new();
        let mut preferred = Vec::new();
        let mut description = String::new();
        let mut in_requirements = false;
        let mut in_preferred = false;

        for line in lines {
            let line_lower = line.to_lowercase();

            if line_lower.contains("requirement") || line_lower.contains("must have") || line_lower.contains("you will") {
                in_requirements = true;
                in_preferred = false;
                continue;
            }

            if line_lower.contains("preferred") || line_lower.contains("nice to have") || line_lower.contains("bonus") {
                in_requirements = false;
                in_preferred = true;
                continue;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check for bullet points
            let clean_line = trimmed
                .trim_start_matches('-')
                .trim_start_matches('•')
                .trim_start_matches('*')
                .trim();

            if in_requirements && !clean_line.is_empty() {
                requirements.push(clean_line.to_string());
            } else if in_preferred && !clean_line.is_empty() {
                preferred.push(clean_line.to_string());
            } else if !in_requirements && !in_preferred {
                description.push_str(line);
                description.push('\n');
            }
        }

        JobDescription {
            title,
            company: None,
            description: description.trim().to_string(),
            requirements,
            preferred,
        }
    }

    /// Match activities against a job description
    pub fn match_activities(&self, job: &JobDescription, activities: &[Activity]) -> MatchResult {
        let mut matched_requirements = Vec::new();
        let mut matched_preferred = Vec::new();
        let mut gaps = Vec::new();
        let mut talking_points = Vec::new();
        let mut recommended_activities = Vec::new();

        // Extract skills from all activities
        let activity_skills: HashMap<String, Vec<String>> = activities
            .iter()
            .map(|a| {
                let tags = self.tagger.tag(a);
                let mut all_skills = tags.technical_skills;
                all_skills.extend(tags.soft_skills);
                (a.id.clone(), all_skills)
            })
            .collect();

        // Match each requirement
        for req in &job.requirements {
            let req_keywords = self.extract_keywords(req);
            let mut supporting = Vec::new();
            let mut best_confidence = 0.0f32;

            for activity in activities {
                if let Some(skills) = activity_skills.get(&activity.id) {
                    let match_score = self.calculate_match_score(&req_keywords, skills, &activity.title, activity.description.as_deref());
                    if match_score > 0.3 {
                        supporting.push(activity.id.clone());
                        best_confidence = best_confidence.max(match_score);
                    }
                }
            }

            let matched = !supporting.is_empty();
            if !matched {
                gaps.push(req.clone());
            } else if best_confidence > 0.5 {
                // Generate talking point for strong matches
                if let Some(activity_id) = supporting.first() {
                    if let Some(activity) = activities.iter().find(|a| &a.id == activity_id) {
                        talking_points.push(TalkingPoint {
                            topic: req.clone(),
                            point: format!("Demonstrated through: {}", activity.title),
                            supporting_activity_id: Some(activity_id.clone()),
                        });
                    }
                }
            }

            matched_requirements.push(RequirementMatch {
                requirement: req.clone(),
                matched,
                supporting_activities: supporting,
                confidence: best_confidence,
            });
        }

        // Match preferred qualifications
        for pref in &job.preferred {
            let pref_keywords = self.extract_keywords(pref);
            let mut supporting = Vec::new();
            let mut best_confidence = 0.0f32;

            for activity in activities {
                if let Some(skills) = activity_skills.get(&activity.id) {
                    let match_score = self.calculate_match_score(&pref_keywords, skills, &activity.title, activity.description.as_deref());
                    if match_score > 0.3 {
                        supporting.push(activity.id.clone());
                        best_confidence = best_confidence.max(match_score);
                    }
                }
            }

            matched_preferred.push(RequirementMatch {
                requirement: pref.clone(),
                matched: !supporting.is_empty(),
                supporting_activities: supporting,
                confidence: best_confidence,
            });
        }

        // Recommend top activities for the interview
        let mut activity_scores: Vec<(String, f32)> = activities
            .iter()
            .map(|a| {
                let mentions: f32 = matched_requirements
                    .iter()
                    .chain(matched_preferred.iter())
                    .filter(|m| m.supporting_activities.contains(&a.id))
                    .map(|m| m.confidence)
                    .sum();
                (a.id.clone(), mentions)
            })
            .collect();

        activity_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        recommended_activities = activity_scores
            .into_iter()
            .take(5)
            .filter(|(_, score)| *score > 0.0)
            .map(|(id, _)| id)
            .collect();

        // Calculate overall score
        let req_score: f32 = matched_requirements
            .iter()
            .map(|m| if m.matched { m.confidence } else { 0.0 })
            .sum::<f32>()
            / job.requirements.len().max(1) as f32;

        let pref_score: f32 = if job.preferred.is_empty() {
            0.0
        } else {
            matched_preferred
                .iter()
                .map(|m| if m.matched { m.confidence } else { 0.0 })
                .sum::<f32>()
                / job.preferred.len() as f32
        };

        let overall_score = req_score * 0.7 + pref_score * 0.3;

        MatchResult {
            job: job.clone(),
            overall_score,
            matched_requirements,
            matched_preferred,
            gaps,
            talking_points,
            recommended_activities,
        }
    }

    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let stop_words = ["and", "or", "the", "a", "an", "in", "of", "to", "for", "with", "on", "at", "by", "as", "is", "are", "be", "was", "were", "have", "has", "had", "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "can", "need", "years", "year", "experience", "knowledge", "ability", "understanding", "strong", "excellent", "good", "proven", "demonstrated"];

        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2 && !stop_words.contains(w))
            .map(String::from)
            .collect()
    }

    fn calculate_match_score(
        &self,
        keywords: &[String],
        skills: &[String],
        title: &str,
        description: Option<&str>,
    ) -> f32 {
        if keywords.is_empty() {
            return 0.0;
        }

        let title_lower = title.to_lowercase();
        let desc_lower = description.unwrap_or("").to_lowercase();
        let skills_lower: Vec<String> = skills.iter().map(|s| s.to_lowercase()).collect();

        let mut matches = 0;
        for keyword in keywords {
            if skills_lower.iter().any(|s| s.contains(keyword) || keyword.contains(s.as_str())) {
                matches += 2;
            } else if title_lower.contains(keyword) || desc_lower.contains(keyword) {
                matches += 1;
            }
        }

        (matches as f32 / (keywords.len() * 2) as f32).min(1.0)
    }

    /// Generate a markdown report of the match result
    pub fn to_markdown(&self, result: &MatchResult) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Job Match Report: {}\n\n", result.job.title));

        // Overall score
        let score_percentage = (result.overall_score * 100.0) as u32;
        let score_label = match score_percentage {
            90..=100 => "Excellent Match",
            70..=89 => "Strong Match",
            50..=69 => "Good Match",
            30..=49 => "Partial Match",
            _ => "Weak Match",
        };

        md.push_str(&format!("## Overall Score: {}% ({})\n\n", score_percentage, score_label));

        // Requirements matched
        md.push_str("## Requirements\n\n");
        for req in &result.matched_requirements {
            let status = if req.matched { "✅" } else { "❌" };
            md.push_str(&format!("{} {}\n", status, req.requirement));
            if req.matched && !req.supporting_activities.is_empty() {
                md.push_str(&format!("   - Confidence: {:.0}%\n", req.confidence * 100.0));
            }
        }
        md.push_str("\n");

        // Preferred qualifications
        if !result.matched_preferred.is_empty() {
            md.push_str("## Preferred Qualifications\n\n");
            for pref in &result.matched_preferred {
                let status = if pref.matched { "✅" } else { "⬜" };
                md.push_str(&format!("{} {}\n", status, pref.requirement));
            }
            md.push_str("\n");
        }

        // Gaps
        if !result.gaps.is_empty() {
            md.push_str("## Gaps to Address\n\n");
            for gap in &result.gaps {
                md.push_str(&format!("- {}\n", gap));
            }
            md.push_str("\n");
        }

        // Talking points
        if !result.talking_points.is_empty() {
            md.push_str("## Suggested Talking Points\n\n");
            for point in &result.talking_points {
                md.push_str(&format!("### {}\n", point.topic));
                md.push_str(&format!("{}\n\n", point.point));
            }
        }

        // Recommended activities to highlight
        if !result.recommended_activities.is_empty() {
            md.push_str("## Activities to Highlight\n\n");
            md.push_str("Review these activities before your interview:\n");
            for id in &result.recommended_activities {
                md.push_str(&format!("- `{}`\n", &id[..8.min(id.len())]));
            }
        }

        md
    }
}
