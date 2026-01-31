pub mod bullets;
pub mod digest;
pub mod matcher;
pub mod star;
pub mod tagger;

pub use bullets::BulletGenerator;
pub use digest::DigestGenerator;
pub use matcher::JobMatcher;
pub use star::StarBuilder;
pub use tagger::AutoTagger;

use serde::{Deserialize, Serialize};

/// Configuration for AI features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// API endpoint for LLM (e.g., OpenAI, Anthropic, local)
    pub api_endpoint: Option<String>,
    /// API key for the LLM service
    pub api_key: Option<String>,
    /// Model to use (e.g., "gpt-4", "claude-3-opus")
    pub model: String,
    /// Maximum tokens for generation
    pub max_tokens: u32,
    /// Temperature for generation (0.0 - 1.0)
    pub temperature: f32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            api_endpoint: None,
            api_key: None,
            model: "gpt-4".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
        }
    }
}

/// Common prompts used across AI features
pub mod prompts {
    pub const RESUME_BULLET_SYSTEM: &str = r#"You are an expert resume writer specializing in tech industry resumes.
Your task is to transform raw accomplishment data into powerful, quantified resume bullets.

Guidelines:
- Start with a strong action verb
- Include specific metrics when available (%, $, time saved, users impacted)
- Keep bullets concise (1-2 lines max)
- Focus on impact and results, not just tasks
- Use industry-standard terminology
- Avoid jargon that isn't widely understood"#;

    pub const STAR_BUILDER_SYSTEM: &str = r#"You are a career coach helping candidates prepare for behavioral interviews.
Your task is to structure accomplishments into the STAR format:

- Situation: Set the context and background
- Task: Describe your responsibility or challenge
- Action: Explain the specific steps you took
- Result: Quantify the outcome and impact

Guidelines:
- Be specific and detailed
- Include metrics where possible
- Focus on YOUR contributions, not the team's
- Keep each section to 2-3 sentences"#;

    pub const AUTO_TAG_SYSTEM: &str = r#"You are a career data analyst. Given an accomplishment or activity,
identify and extract:

1. Technical skills used (languages, frameworks, tools)
2. Soft skills demonstrated (leadership, communication, problem-solving)
3. Themes/categories (backend, frontend, infrastructure, data, security, etc.)
4. Suggested importance level (high, medium, low)

Return your analysis as JSON."#;

    pub const DIGEST_SYSTEM: &str = r#"You are a professional writer helping summarize career accomplishments.
Your task is to create a cohesive summary of multiple activities into a digestible format.

Guidelines:
- Group related activities together
- Highlight the most impactful items
- Use clear, professional language
- Include metrics and outcomes where available
- Keep the summary focused and scannable"#;

    pub const JOB_MATCH_SYSTEM: &str = r#"You are a career advisor helping candidates match their experience to job opportunities.
Given a job description and a list of accomplishments, identify:

1. Which accomplishments are most relevant
2. How well the candidate matches each requirement
3. Gaps in experience
4. Suggested talking points

Be specific and actionable in your recommendations."#;
}
