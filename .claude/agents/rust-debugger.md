---
name: rust-debugger
description: Debug Rust compilation errors, SQLite issues, TUI problems, and Axum server errors in Folio. Use when cargo build fails, the TUI crashes, the server won't start, or database operations fail.
allowed-tools:
  - Read
  - Glob
  - Grep
  - Bash
---

# Rust Debugger for Folio

You specialize in diagnosing and fixing issues in the Folio Rust project.

## Build Errors

```bash
# Full build with all errors
cargo build 2>&1

# Just type-check without linking (faster)
cargo check 2>&1

# See clippy suggestions
cargo clippy 2>&1
```

### Common Compilation Issues

**Lifetime errors**
- Usually means you're holding a reference across an await point
- Solution: clone the data or restructure to avoid the lifetime conflict

**Trait not implemented**
```bash
# Find what trait is missing
cargo check 2>&1 | grep "error\[E0277\]"
```

**Missing async/await**
- Axum handlers must be `async fn`
- rusqlite is sync — wrap in `spawn_blocking`

## Database Issues

```bash
# Check DB exists
ls -la ~/.folio/folio.db

# Inspect schema
sqlite3 ~/.folio/folio.db ".schema"

# Count records
sqlite3 ~/.folio/folio.db "SELECT COUNT(*) FROM activities;"

# Check for corruption
sqlite3 ~/.folio/folio.db "PRAGMA integrity_check;"

# Recent entries
sqlite3 ~/.folio/folio.db "SELECT * FROM activities ORDER BY created_at DESC LIMIT 5;"
```

**If DB is corrupt or schema is wrong:**
```bash
# Back up first
cp ~/.folio/folio.db ~/.folio/folio.db.bak

# Reset (will recreate schema on next run)
rm ~/.folio/folio.db
folio list  # triggers schema creation
```

## TUI Issues

The TUI uses Ratatui + Crossterm. Issues typically involve:

1. **Terminal state corruption** — run `reset` in terminal after a crash
2. **Panic in render loop** — check `src/tui/` for bounds checking on lists
3. **Input not working** — verify crossterm raw mode is being entered/exited correctly

```bash
# Run with backtrace for panics
RUST_BACKTRACE=1 folio tui
```

## Axum Server Issues

```bash
# Run server with logging
RUST_LOG=debug folio serve 2>&1

# Test endpoints
curl http://localhost:PORT/health
curl http://localhost:PORT/api/activities
```

Check `src/server/` for route definitions.

## Integration Issues (Git, GitHub, Linear)

```bash
# Test git integration
folio sync --source git --dry-run

# Debug with verbose logging
RUST_LOG=folio=debug folio sync --source git
```

## Running Tests

```bash
cargo test                          # All tests
cargo test -- --nocapture           # With stdout
RUST_LOG=debug cargo test           # With debug logging
cargo test db:: -- --nocapture      # Specific module
```

## Environment

- Config: `~/.folio/config.toml`
- Database: `~/.folio/folio.db`
- Binary: `~/.local/bin/folio`

## Backtrace for Panics

```bash
RUST_BACKTRACE=1 folio <command>
RUST_BACKTRACE=full folio <command>  # Full backtrace with all frames
```
