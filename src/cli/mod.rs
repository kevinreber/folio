pub mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "folio")]
#[command(about = "Local-first career accomplishment tracker")]
#[command(version)]
#[command(after_help = "Examples:
  folio capture \"Implemented new auth system\" --impact \"Reduced login time by 50%\"
  folio sync --source git --days 30
  folio export --format markdown --output brag-doc.md
  folio search \"authentication\" --project my-app
  folio promote abc123 --interactive
  folio tui
")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Capture a new accomplishment or activity
    Capture {
        /// Title or description of what you accomplished
        title: String,

        /// Impact or result of the accomplishment
        #[arg(short, long)]
        impact: Option<String>,

        /// Project this relates to
        #[arg(short, long)]
        project: Option<String>,

        /// Employer/company context
        #[arg(short, long)]
        employer: Option<String>,

        /// Importance level (low, medium, high)
        #[arg(long, default_value = "medium")]
        importance: String,
    },

    /// List recent activities
    List {
        /// Number of items to show
        #[arg(short, long, default_value = "10")]
        limit: u32,

        /// Show full details
        #[arg(long)]
        full: bool,
    },

    /// Show details of a specific activity
    Show {
        /// Activity ID (can be partial)
        id: String,
    },

    /// Delete an activity
    Delete {
        /// Activity ID (can be partial)
        id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show statistics about captured activities
    Stats,

    /// Search activities by keyword
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: u32,

        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,

        /// Filter by importance (low, medium, high)
        #[arg(long)]
        importance: Option<String>,
    },

    /// Edit an existing activity
    Edit {
        /// Activity ID (can be partial)
        id: String,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New impact
        #[arg(short, long)]
        impact: Option<String>,

        /// New project
        #[arg(short, long)]
        project: Option<String>,

        /// New employer
        #[arg(short, long)]
        employer: Option<String>,

        /// New importance level
        #[arg(long)]
        importance: Option<String>,
    },

    /// Promote an activity to a polished accomplishment with STAR format
    Promote {
        /// Activity ID (can be partial)
        id: String,

        /// Interactive mode for detailed STAR story
        #[arg(short, long)]
        interactive: bool,
    },

    /// Manage configuration
    Config {
        /// Configuration key (e.g., github.token)
        key: Option<String>,

        /// Value to set
        value: Option<String>,

        /// List all configuration keys
        #[arg(short, long)]
        list: bool,

        /// Run interactive configuration wizard
        #[arg(long)]
        init: bool,
    },

    /// Export activities and accomplishments
    Export {
        /// Export format (markdown, json, yaml)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Export as brag document
        #[arg(long)]
        brag: bool,

        /// Export as resume bullets
        #[arg(long)]
        bullets: bool,
    },

    /// Import activities from a file
    Import {
        /// File to import (json or yaml)
        file: PathBuf,

        /// Dry run - show what would be imported
        #[arg(long)]
        dry_run: bool,
    },

    /// Sync activities from external sources (git, GitHub, Linear)
    Sync {
        /// Source to sync from (git, github, linear, or all)
        #[arg(short, long)]
        source: Option<String>,

        /// Number of days to look back
        #[arg(short, long, default_value = "30")]
        days: u32,

        /// Specific repository path (for git)
        #[arg(short, long)]
        repo: Option<PathBuf>,

        /// Dry run - show what would be synced
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate a digest/summary of activities
    Digest {
        /// Time period (daily, weekly, monthly, quarterly, yearly)
        #[arg(default_value = "weekly")]
        period: String,

        /// Output as markdown
        #[arg(short, long)]
        markdown: bool,
    },

    /// Generate a performance review summary
    Review {
        /// Number of months to include
        #[arg(short, long, default_value = "6")]
        months: u32,
    },

    /// Match your experience against a job description
    Match {
        /// Job title
        title: String,

        /// Job description text
        #[arg(short, long)]
        description: Option<String>,

        /// Read job description from file
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Launch interactive TUI
    #[command(alias = "ui")]
    Tui,

    /// Start the REST API server
    Serve {
        /// Host to bind to
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Start MCP server instead of REST API
        #[arg(long)]
        mcp: bool,
    },

    /// Watch for new git commits and capture them automatically
    Watch {
        /// Run continuously as a daemon
        #[arg(short, long)]
        daemon: bool,
    },
}
