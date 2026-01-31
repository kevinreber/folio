mod cli;
mod db;
mod types;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Capture {
            title,
            impact,
            project,
            employer,
            importance,
        } => cli::commands::capture(title, impact, project, employer, importance),

        Commands::List { limit, full } => cli::commands::list(limit, full),

        Commands::Show { id } => cli::commands::show(id),

        Commands::Delete { id, force } => cli::commands::delete(id, force),

        Commands::Stats => cli::commands::stats(),
    }
}
