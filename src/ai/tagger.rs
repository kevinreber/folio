use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::types::{Activity, Importance};

/// Automatic tagger for activities
pub struct AutoTagger {
    skill_patterns: HashMap<String, Vec<String>>,
    theme_patterns: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagResult {
    pub technical_skills: Vec<String>,
    pub soft_skills: Vec<String>,
    pub themes: Vec<String>,
    pub suggested_importance: Importance,
    pub confidence: f32,
}

impl Default for AutoTagger {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoTagger {
    pub fn new() -> Self {
        Self {
            skill_patterns: Self::build_skill_patterns(),
            theme_patterns: Self::build_theme_patterns(),
        }
    }

    fn build_skill_patterns() -> HashMap<String, Vec<String>> {
        let mut patterns: HashMap<String, Vec<String>> = HashMap::new();

        // Helper to convert &str slice to Vec<String>
        fn s(strs: &[&str]) -> Vec<String> {
            strs.iter().map(|s| s.to_string()).collect()
        }

        // Programming Languages
        patterns.insert("Rust".to_string(), s(&["rust", "cargo", ".rs", "rustc"]));
        patterns.insert("Python".to_string(), s(&["python", "pip", "pytest", "django", "flask", "fastapi", ".py"]));
        patterns.insert("JavaScript".to_string(), s(&["javascript", "js", "node", "npm", "yarn", "react", "vue", "angular"]));
        patterns.insert("TypeScript".to_string(), s(&["typescript", "ts", ".tsx", ".ts"]));
        patterns.insert("Go".to_string(), s(&["golang", " go ", "go mod", ".go"]));
        patterns.insert("Java".to_string(), s(&["java", "maven", "gradle", "spring", ".java"]));
        patterns.insert("C++".to_string(), s(&["c++", "cpp", ".cpp", ".hpp"]));
        patterns.insert("Ruby".to_string(), s(&["ruby", "rails", "gem", ".rb"]));
        patterns.insert("PHP".to_string(), s(&["php", "laravel", "symfony", ".php"]));
        patterns.insert("Swift".to_string(), s(&["swift", "xcode", "ios", ".swift"]));
        patterns.insert("Kotlin".to_string(), s(&["kotlin", "android", ".kt"]));

        // Frameworks & Libraries
        patterns.insert("React".to_string(), s(&["react", "jsx", "redux", "next.js", "nextjs"]));
        patterns.insert("Vue".to_string(), s(&["vue", "vuex", "nuxt"]));
        patterns.insert("Angular".to_string(), s(&["angular", "rxjs", "ngrx"]));
        patterns.insert("Django".to_string(), s(&["django"]));
        patterns.insert("Flask".to_string(), s(&["flask"]));
        patterns.insert("FastAPI".to_string(), s(&["fastapi"]));
        patterns.insert("Express".to_string(), s(&["express", "expressjs"]));
        patterns.insert("Spring".to_string(), s(&["spring", "springboot", "spring boot"]));

        // Databases
        patterns.insert("PostgreSQL".to_string(), s(&["postgres", "postgresql", "psql"]));
        patterns.insert("MySQL".to_string(), s(&["mysql", "mariadb"]));
        patterns.insert("MongoDB".to_string(), s(&["mongodb", "mongo"]));
        patterns.insert("Redis".to_string(), s(&["redis"]));
        patterns.insert("Elasticsearch".to_string(), s(&["elasticsearch", "elastic"]));
        patterns.insert("DynamoDB".to_string(), s(&["dynamodb"]));
        patterns.insert("SQLite".to_string(), s(&["sqlite"]));

        // Cloud & Infrastructure
        patterns.insert("AWS".to_string(), s(&["aws", "amazon web services", "ec2", "s3", "lambda"]));
        patterns.insert("GCP".to_string(), s(&["gcp", "google cloud", "gke"]));
        patterns.insert("Azure".to_string(), s(&["azure", "microsoft cloud"]));
        patterns.insert("Docker".to_string(), s(&["docker", "container", "dockerfile"]));
        patterns.insert("Kubernetes".to_string(), s(&["kubernetes", "k8s", "kubectl", "helm"]));
        patterns.insert("Terraform".to_string(), s(&["terraform", "tf"]));
        patterns.insert("CI/CD".to_string(), s(&["ci/cd", "github actions", "jenkins", "gitlab ci", "circleci"]));

        // Tools
        patterns.insert("Git".to_string(), s(&["git", "github", "gitlab", "bitbucket"]));
        patterns.insert("Linux".to_string(), s(&["linux", "ubuntu", "debian", "centos"]));
        patterns.insert("GraphQL".to_string(), s(&["graphql", "apollo"]));
        patterns.insert("REST API".to_string(), s(&["rest api", "restful", "api endpoint"]));
        patterns.insert("gRPC".to_string(), s(&["grpc", "protobuf"]));
        patterns.insert("Kafka".to_string(), s(&["kafka"]));
        patterns.insert("RabbitMQ".to_string(), s(&["rabbitmq"]));

        patterns
    }

    fn build_theme_patterns() -> HashMap<String, Vec<String>> {
        let mut patterns: HashMap<String, Vec<String>> = HashMap::new();

        // Helper to convert &str slice to Vec<String>
        fn s(strs: &[&str]) -> Vec<String> {
            strs.iter().map(|s| s.to_string()).collect()
        }

        patterns.insert("Backend".to_string(), s(&["backend", "server", "api", "microservice", "database"]));
        patterns.insert("Frontend".to_string(), s(&["frontend", "ui", "ux", "component", "css", "html"]));
        patterns.insert("Full-Stack".to_string(), s(&["full-stack", "fullstack", "end-to-end"]));
        patterns.insert("DevOps".to_string(), s(&["devops", "infrastructure", "deploy", "ci/cd", "monitoring"]));
        patterns.insert("Data Engineering".to_string(), s(&["data pipeline", "etl", "data warehouse", "bigquery"]));
        patterns.insert("Machine Learning".to_string(), s(&["machine learning", "ml", "model", "training", "ai"]));
        patterns.insert("Security".to_string(), s(&["security", "authentication", "authorization", "vulnerability", "encryption"]));
        patterns.insert("Performance".to_string(), s(&["performance", "optimization", "latency", "throughput", "caching"]));
        patterns.insert("Scalability".to_string(), s(&["scalability", "scale", "distributed", "horizontal"]));
        patterns.insert("Testing".to_string(), s(&["testing", "test", "qa", "unit test", "integration test"]));
        patterns.insert("Documentation".to_string(), s(&["documentation", "docs", "readme", "wiki", "spec"]));
        patterns.insert("Architecture".to_string(), s(&["architecture", "design", "system design", "technical design"]));
        patterns.insert("Mobile".to_string(), s(&["mobile", "ios", "android", "react native", "flutter"]));
        patterns.insert("Observability".to_string(), s(&["observability", "monitoring", "logging", "tracing", "metrics"]));

        patterns
    }

    /// Tag an activity with skills and themes
    pub fn tag(&self, activity: &Activity) -> TagResult {
        let text = self.build_search_text(activity);
        let text_lower = text.to_lowercase();

        let technical_skills = self.extract_skills(&text_lower);
        let themes = self.extract_themes(&text_lower);
        let soft_skills = self.extract_soft_skills(&text_lower);
        let suggested_importance = self.suggest_importance(activity, &technical_skills, &themes);

        let confidence = self.calculate_confidence(&technical_skills, &themes);

        TagResult {
            technical_skills,
            soft_skills,
            themes,
            suggested_importance,
            confidence,
        }
    }

    /// Tag multiple activities and aggregate results
    pub fn tag_batch(&self, activities: &[Activity]) -> TagResult {
        let mut all_technical: Vec<String> = Vec::new();
        let mut all_soft: Vec<String> = Vec::new();
        let mut all_themes: Vec<String> = Vec::new();
        let mut importance_scores = Vec::new();

        for activity in activities {
            let result = self.tag(activity);
            all_technical.extend(result.technical_skills);
            all_soft.extend(result.soft_skills);
            all_themes.extend(result.themes);
            importance_scores.push(match result.suggested_importance {
                Importance::High => 3,
                Importance::Medium => 2,
                Importance::Low => 1,
            });
        }

        // Deduplicate and sort by frequency
        let technical_skills = Self::dedupe_and_rank(all_technical);
        let soft_skills = Self::dedupe_and_rank(all_soft);
        let themes = Self::dedupe_and_rank(all_themes);

        // Average importance
        let avg_importance = if importance_scores.is_empty() {
            2
        } else {
            importance_scores.iter().sum::<i32>() / importance_scores.len() as i32
        };

        let suggested_importance = match avg_importance {
            3 => Importance::High,
            2 => Importance::Medium,
            _ => Importance::Low,
        };

        TagResult {
            technical_skills,
            soft_skills,
            themes,
            suggested_importance,
            confidence: 0.8,
        }
    }

    fn build_search_text(&self, activity: &Activity) -> String {
        let mut parts = vec![activity.title.clone()];

        if let Some(desc) = &activity.description {
            parts.push(desc.clone());
        }

        if let Some(project) = &activity.project {
            parts.push(project.clone());
        }

        if let Some(impact) = activity.metadata.get("impact").and_then(|v| v.as_str()) {
            parts.push(impact.to_string());
        }

        parts.join(" ")
    }

    fn extract_skills(&self, text: &str) -> Vec<String> {
        let mut found = Vec::new();

        for (skill, patterns) in &self.skill_patterns {
            if patterns.iter().any(|p| text.contains(p)) {
                found.push(skill.clone());
            }
        }

        found
    }

    fn extract_themes(&self, text: &str) -> Vec<String> {
        let mut found = Vec::new();

        for (theme, patterns) in &self.theme_patterns {
            if patterns.iter().any(|p| text.contains(p)) {
                found.push(theme.clone());
            }
        }

        found
    }

    fn extract_soft_skills(&self, text: &str) -> Vec<String> {
        let mut found = Vec::new();

        let soft_skill_patterns = [
            ("Leadership", vec!["led", "managed", "directed", "coordinated", "mentored"]),
            ("Communication", vec!["presented", "communicated", "documented", "explained", "trained"]),
            ("Problem Solving", vec!["solved", "debugged", "investigated", "diagnosed", "troubleshot"]),
            ("Collaboration", vec!["collaborated", "partnered", "worked with", "coordinated", "aligned"]),
            ("Initiative", vec!["initiated", "proposed", "pioneered", "introduced", "championed"]),
            ("Time Management", vec!["deadline", "on time", "prioritized", "scheduled"]),
            ("Adaptability", vec!["adapted", "pivoted", "learned", "transitioned"]),
        ];

        for (skill, patterns) in soft_skill_patterns {
            if patterns.iter().any(|p| text.contains(p)) {
                found.push(skill.to_string());
            }
        }

        found
    }

    fn suggest_importance(&self, activity: &Activity, skills: &[String], themes: &[String]) -> Importance {
        // Use existing importance as base
        let base = &activity.importance;

        // Boost for high-value indicators
        let title_lower = activity.title.to_lowercase();
        let high_value_keywords = ["launch", "ship", "major", "critical", "security", "performance"];
        let low_value_keywords = ["typo", "minor", "small", "cleanup", "format"];

        if high_value_keywords.iter().any(|k| title_lower.contains(k)) {
            return Importance::High;
        }

        if low_value_keywords.iter().any(|k| title_lower.contains(k)) {
            return Importance::Low;
        }

        // Boost for complex work (many skills/themes)
        if skills.len() >= 3 || themes.len() >= 3 {
            return match base {
                Importance::Low => Importance::Medium,
                _ => Importance::High,
            };
        }

        base.clone()
    }

    fn calculate_confidence(&self, skills: &[String], themes: &[String]) -> f32 {
        // More detected patterns = higher confidence
        let pattern_count = skills.len() + themes.len();
        match pattern_count {
            0 => 0.3,
            1..=2 => 0.5,
            3..=5 => 0.7,
            6..=10 => 0.85,
            _ => 0.95,
        }
    }

    fn dedupe_and_rank(items: Vec<String>) -> Vec<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for item in items {
            *counts.entry(item).or_insert(0) += 1;
        }

        let mut ranked: Vec<_> = counts.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));

        ranked.into_iter().map(|(item, _)| item).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_rust_activity() {
        let tagger = AutoTagger::new();
        let activity = Activity::new_manual("Implemented new Rust API endpoint with PostgreSQL")
            .with_description("Created REST API using Rust and PostgreSQL for user management");

        let result = tagger.tag(&activity);

        assert!(result.technical_skills.contains(&"Rust".to_string()));
        assert!(result.technical_skills.contains(&"PostgreSQL".to_string()));
        assert!(result.themes.contains(&"Backend".to_string()));
    }

    #[test]
    fn test_tag_leadership() {
        let tagger = AutoTagger::new();
        let activity = Activity::new_manual("Led team migration to Kubernetes")
            .with_description("Coordinated with 3 teams to migrate services to k8s");

        let result = tagger.tag(&activity);

        assert!(result.technical_skills.contains(&"Kubernetes".to_string()));
        assert!(result.soft_skills.contains(&"Leadership".to_string()));
    }
}
