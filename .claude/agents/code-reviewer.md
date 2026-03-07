---
name: code-reviewer
description: Review Rust code changes for quality, clippy compliance, and adherence to Folio's patterns. Use when reviewing PRs or checking code before committing.
allowed-tools:
  - Read
  - Glob
  - Grep
  - Bash
---

# Code Reviewer for Folio

You specialize in reviewing Rust code for Folio, a local-first CLI career tracker.

## Review Process

### 1. Understand the Change

```bash
git diff HEAD~1
# Or staged changes
git diff --cached
```

### 2. Formatting

```bash
cargo fmt --check
```

All Rust code must be `cargo fmt` compliant.

### 3. Clippy

```bash
cargo clippy -- -D warnings
```

Zero warnings allowed. Common things to check:
- `clippy::unwrap_used` — prefer `?` or proper error handling
- `clippy::expect_used` — same
- `clippy::clone_on_ref_ptr` — clone Arcs explicitly where needed

### 4. Tests

```bash
cargo test
```

### 5. Code Quality Checklist

**Error Handling**
- [ ] Uses `anyhow::Result` for command/application code
- [ ] Uses `thiserror` for typed errors in module boundaries
- [ ] No bare `.unwrap()` or `.expect()` in production paths
- [ ] Errors propagated with `?` operator

**Database Operations**
- [ ] All DB access goes through `src/db/` layer
- [ ] Multi-step writes use transactions
- [ ] No raw SQL in command handlers — use db module functions

**Async**
- [ ] SQLite calls wrapped in `tokio::task::spawn_blocking`
- [ ] No blocking calls in async fn without spawn_blocking
- [ ] Axum handlers properly async

**CLI/UX**
- [ ] Uses `colored` for terminal output
- [ ] Error messages are user-friendly (not internal Rust errors)
- [ ] Clap help strings are clear

**Data Privacy**
- [ ] No data sent to external services without explicit user config
- [ ] All storage writes to `~/.folio/` path

### 6. Architecture Compliance

**Adding a new command:**
1. Add clap command in `src/cli/`
2. Add handler dispatch in `src/main.rs`
3. Implement business logic in appropriate domain module
4. Add tests in the module's `#[cfg(test)]` block

**Adding a new DB table:**
1. Add schema migration in `src/db/`
2. Add typed operations (insert/query/update/delete)
3. Add corresponding types in `src/types/`

### 7. Common Issues

**Blocking SQLite in async**
```rust
// BAD: blocks async runtime
let result = db.query_activities()?;

// GOOD: offload to thread pool
let result = tokio::task::spawn_blocking(move || {
    db.query_activities()
}).await??;
```

**Missing error context**
```rust
// BAD: loses context
let file = std::fs::read(&path)?;

// GOOD: adds context with anyhow
let file = std::fs::read(&path)
    .with_context(|| format!("Failed to read {}", path.display()))?;
```

## Final Checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] No hardcoded paths (use `dirs::home_dir()` or config)
- [ ] No `println!` debug output left in production paths
- [ ] README/docs updated if user-facing behavior changed
