pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "folio")]
#[command(about = "Local-first career accomplishment tracker")]
#[command(version)]
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
}
