# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Folio** is a local-first career accomplishment tracker CLI built in Rust. It helps developers capture what they've done, enrich it with impact and context, and package it into interview-ready stories, resume bullets, and performance review summaries.

Key principle: **individual-owned and portable across jobs** — all data lives locally in SQLite.

## Tech Stack

- **Rust** — CLI with rich terminal output
- **SQLite** (`rusqlite`) — Local-first data storage at `~/.folio/folio.db`
- **Ratatui** — Interactive TUI for browsing activities
- **Axum** — REST API server + MCP server
- **git2** — Native Git repository integration
- **clap** — CLI argument parsing with derive macros

## Project Structure

```
folio/
├── src/
│   ├── main.rs           # CLI entry point and command dispatch
│   ├── cli/              # Clap command definitions
│   ├── db/               # SQLite database layer
│   ├── ai/               # LLM integration (enrichment, tagging)
│   ├── config/           # Configuration management
│   ├── export/           # Export formats (Markdown, JSON, YAML)
│   ├── integrations/     # Git, GitHub, Linear integrations
│   ├── notifications/    # Desktop notifications
│   ├── server/           # Axum REST API + MCP server
│   ├── tracking/         # Activity pattern detection
│   ├── tui/              # Ratatui interactive interface
│   ├── types/            # Shared types and data models
│   └── watcher/          # Background git watcher daemon
├── web-ui/               # Embedded web UI assets
│   ├── index.html        # Dashboard
│   └── assets/
├── docs/                 # Extended documentation
├── scripts/              # Build and test scripts
├── Cargo.toml
└── Makefile
```

## Development Commands

### Building

```bash
make build          # Debug build
make release        # Release build (optimized)
make install        # Build release + install to ~/.local/bin
make install-global # Install to /usr/local/bin (requires sudo)
make uninstall      # Remove from ~/.local/bin
```

### Testing

```bash
make test           # Run unit tests (cargo test)
make test-install   # Run installation tests
make test-quick     # Quick install tests (skip build)
make test-all       # All tests
```

### Code Quality

```bash
make fmt            # Format with cargo fmt
make fmt-check      # Check formatting (CI)
make lint           # Run clippy (warnings as errors)
make check          # Run all checks: fmt-check + lint + test
```

### Running

```bash
# Debug run with args
make run ARGS="capture 'Fixed authentication bug' --impact 'Reduced login errors by 80%'"

# Or directly
cargo run -- capture "Fixed authentication bug"
cargo run -- list
cargo run -- tui
cargo run -- serve  # Start REST/MCP server
```

## Code Conventions

### Error Handling

- Use `anyhow::Result` for application-level functions (command handlers, etc.)
- Use `thiserror` for typed errors in library/module boundaries
- No `.unwrap()` or `.expect()` in non-test code without a comment explaining why it's safe

### Database

- All DB operations go through `src/db/`
- Use transactions for multi-step writes
- Schema migrations handled in `src/db/` — add new tables/columns carefully

### CLI Commands

- Commands defined in `src/cli/` using clap derive macros
- Command handlers in `src/main.rs` dispatch to domain modules
- Rich terminal output using `colored` crate

### Async

- Async runtime: Tokio
- `spawn_blocking` for SQLite operations in async contexts (rusqlite is sync)

## Data Storage

- Config: `~/.folio/config.toml`
- Database: `~/.folio/folio.db`
- Logs: `~/.folio/logs/`

## Testing

```bash
cargo test                      # All unit tests
cargo test -- --nocapture       # With output
cargo test db::                 # Test a specific module
```

Test utilities use `tempfile` crate for temporary databases.

## Key Commands

| Command | Description |
|---------|-------------|
| `folio capture "..."` | Capture a new accomplishment |
| `folio list` | List recent activities |
| `folio sync --source git` | Sync from Git repos |
| `folio export --brag` | Export as brag document |
| `folio digest weekly` | Generate weekly summary |
| `folio review` | Generate performance review summary |
| `folio match` | Match against job description |
| `folio tui` | Launch interactive TUI |
| `folio serve` | Start REST API + MCP server |

## Documentation

- `README.md` — User-facing documentation and quick start
- `ARCHITECTURE.md` — Technical architecture deep dive
- `docs/` — Extended technical documentation
- Online docs: https://kevinreber.github.io/folio/

## Pull Request Workflow

Before pushing, run all checks:

```bash
make check  # Runs: cargo fmt --check + cargo clippy + cargo test
```

### Automated Hooks

Claude Code hooks (`.claude/settings.json`) automatically:
- Run `cargo fmt --check && cargo clippy && cargo test` before committing
- Run `cargo fmt` after editing `.rs` files
