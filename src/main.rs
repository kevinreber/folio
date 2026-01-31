mod ai;
mod cli;
mod config;
mod db;
mod export;
mod integrations;
mod server;
mod tui;
mod types;
mod watcher;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    // Load .env if present
    let _ = dotenvy::dotenv();

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

        Commands::Search {
            query,
            limit,
            project,
            importance,
        } => cli::commands::search(query, limit, project, importance),

        Commands::Edit {
            id,
            title,
            impact,
            project,
            employer,
            importance,
        } => cli::commands::edit(id, title, impact, project, employer, importance),

        Commands::Promote { id, interactive } => cli::commands::promote(id, interactive),

        Commands::Config {
            key,
            value,
            list,
            init,
        } => {
            if init {
                cli::commands::config_init()
            } else {
                cli::commands::config(key, value, list)
            }
        }

        Commands::Export {
            format,
            output,
            brag,
            bullets,
        } => cli::commands::export(format, output, brag, bullets),

        Commands::Import { file, dry_run } => cli::commands::import(file, dry_run),

        Commands::Sync {
            source,
            days,
            repo,
            dry_run,
        } => cli::commands::sync(source, days, repo, dry_run),

        Commands::Digest { period, markdown } => cli::commands::digest(period, markdown),

        Commands::Review { months } => cli::commands::review(months),

        Commands::Match {
            title,
            description,
            file,
        } => cli::commands::match_job(title, description, file),

        Commands::Tui => cli::commands::tui(),

        Commands::Serve { host, port, mcp } => {
            if mcp {
                cli::commands::serve_mcp(host, port)
            } else {
                cli::commands::serve_api(host, port)
            }
        }

        Commands::Watch { daemon } => cli::commands::watch(daemon),
    }
}
