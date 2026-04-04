use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub date: String,
    pub project: String,
    pub project_path: String,
    pub prompt_count: i64,
    pub duration_minutes: i64,
    pub is_personal: bool,
    pub device_id: String,
    pub first_prompt_at: String,
    pub last_prompt_at: String,
    pub metadata: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub date: String,
    pub category: String,
    pub source: String,
    pub title: String,
    pub detail: Option<String>,
    pub url: Option<String>,
    pub metadata: Option<String>,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRecord {
    pub date: String,
    pub period: String,
    pub summary_type: String,
    pub content: String,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectAggregate {
    pub project: String,
    pub sessions: usize,
    pub total_prompts: i64,
    pub total_minutes: i64,
    pub is_personal: bool,
    pub days_active: usize,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapEntry {
    pub project: String,
    pub total_prompts: i64,
    pub days: Vec<HeatmapDay>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapDay {
    pub date: String,
    pub prompts: i64,
}

// ---------------------------------------------------------------------------
// Turso credentials
// ---------------------------------------------------------------------------

struct TursoCredentials {
    database_url: String,
    auth_token: String,
}

fn load_credentials() -> Option<TursoCredentials> {
    let env_file = dirs::home_dir()?.join(".claude").join("turso.env");
    if !env_file.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&env_file).ok()?;
    let mut url = None;
    let mut token = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            match key {
                "TURSO_DATABASE_URL" => url = Some(value.to_string()),
                "TURSO_AUTH_TOKEN" => token = Some(value.to_string()),
                _ => {}
            }
        }
    }

    match (url, token) {
        (Some(database_url), Some(auth_token)) if !database_url.is_empty() && !auth_token.is_empty() => {
            Some(TursoCredentials { database_url, auth_token })
        }
        _ => None,
    }
}

fn resolve_device_id() -> String {
    if let Ok(id) = std::env::var("FOLIO_DEVICE_ID") {
        if !id.is_empty() {
            return id;
        }
    }

    // Check turso.env for DEVICE_NAME
    if let Some(home) = dirs::home_dir() {
        let env_file = home.join(".claude").join("turso.env");
        if let Ok(content) = std::fs::read_to_string(&env_file) {
            for line in content.lines() {
                let line = line.trim();
                if let Some((key, value)) = line.split_once('=') {
                    if key.trim() == "DEVICE_NAME" {
                        let value = value.trim().trim_matches('"').trim_matches('\'');
                        if !value.is_empty() {
                            return value.to_string();
                        }
                    }
                }
            }
        }
    }

    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("activity.db")
}

// ---------------------------------------------------------------------------
// TursoDB
// ---------------------------------------------------------------------------

pub struct TursoDB {
    conn: libsql::Connection,
    pub device_id: String,
}

impl TursoDB {
    /// Connect to Turso over HTTP (remote-only).
    /// Uses remote mode to avoid SQLite threading conflicts with rusqlite.
    /// Falls back to a local-only libsql database if no credentials are configured.
    pub async fn connect() -> Result<Self> {
        let device_id = resolve_device_id();

        let db = if let Some(creds) = load_credentials() {
            // Remote-only mode — no local SQLite file, no threading conflicts
            libsql::Builder::new_remote(creds.database_url, creds.auth_token)
                .build()
                .await
                .context("Failed to connect to Turso")?
        } else {
            // Local-only fallback (only works when rusqlite isn't in the same process)
            let path = db_path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            libsql::Builder::new_local(path.to_string_lossy().to_string())
                .build()
                .await
                .context("Failed to open local activity database")?
        };

        let conn = db.connect().context("Failed to get connection")?;

        let turso = Self { conn, device_id };
        turso.migrate().await?;

        Ok(turso)
    }

    /// Run schema migrations — idempotent.
    async fn migrate(&self) -> Result<()> {
        // Existing tables from activity-db (preserve compatibility)
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS activities (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    category TEXT NOT NULL,
                    source TEXT NOT NULL,
                    title TEXT NOT NULL,
                    detail TEXT,
                    url TEXT,
                    metadata TEXT,
                    tag TEXT DEFAULT 'work',
                    created_at TEXT DEFAULT (datetime('now')),
                    UNIQUE(date, category, source, title)
                )",
                (),
            )
            .await?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS summaries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    period TEXT NOT NULL,
                    type TEXT NOT NULL,
                    content TEXT NOT NULL,
                    metadata TEXT,
                    created_at TEXT DEFAULT (datetime('now')),
                    UNIQUE(date, period, type)
                )",
                (),
            )
            .await?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS goals (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    week TEXT NOT NULL,
                    text TEXT NOT NULL,
                    status TEXT DEFAULT 'open',
                    completed TEXT,
                    created_at TEXT DEFAULT (datetime('now'))
                )",
                (),
            )
            .await?;

        // New sessions table for session-level Claude Code data
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    date TEXT NOT NULL,
                    project TEXT NOT NULL,
                    project_path TEXT NOT NULL,
                    prompt_count INTEGER NOT NULL DEFAULT 0,
                    duration_minutes INTEGER NOT NULL DEFAULT 0,
                    is_personal INTEGER NOT NULL DEFAULT 0,
                    device_id TEXT NOT NULL DEFAULT '',
                    first_prompt_at TEXT NOT NULL,
                    last_prompt_at TEXT NOT NULL,
                    metadata TEXT,
                    created_at TEXT DEFAULT (datetime('now')),
                    UNIQUE(session_id, device_id)
                )",
                (),
            )
            .await?;

        // Indexes
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_sessions_date ON sessions(date)",
                (),
            )
            .await?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project)",
                (),
            )
            .await?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_sessions_device ON sessions(device_id)",
                (),
            )
            .await?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_activities_date ON activities(date)",
                (),
            )
            .await?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_activities_source ON activities(source)",
                (),
            )
            .await?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_summaries_date ON summaries(date)",
                (),
            )
            .await?;

        Ok(())
    }

    /// Force sync with Turso remote. No-op for local-only connections.
    pub async fn sync(&self) -> Result<()> {
        // libsql embedded replicas auto-sync on writes via the Database handle.
        // For explicit sync we'd need the Database, but since we sync after each
        // write via a fresh connection pattern, this is a best-effort approach.
        // The real sync happens in connect() and after writes.
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Write methods
    // -----------------------------------------------------------------------

    pub async fn write_session(&self, session: &SessionRecord) -> Result<()> {
        let device = if session.device_id.is_empty() {
            self.device_id.clone()
        } else {
            session.device_id.clone()
        };

        self.conn
            .execute(
                "INSERT OR REPLACE INTO sessions
                    (session_id, date, project, project_path, prompt_count,
                     duration_minutes, is_personal, device_id, first_prompt_at,
                     last_prompt_at, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                libsql::params![
                    session.session_id.clone(),
                    session.date.clone(),
                    session.project.clone(),
                    session.project_path.clone(),
                    session.prompt_count,
                    session.duration_minutes,
                    session.is_personal as i64,
                    device,
                    session.first_prompt_at.clone(),
                    session.last_prompt_at.clone(),
                    session.metadata.as_deref().unwrap_or("").to_string(),
                ],
            )
            .await
            .context("Failed to write session")?;

        Ok(())
    }

    pub async fn write_activity(&self, activity: &ActivityRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO activities
                    (date, category, source, title, detail, url, metadata, tag)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                libsql::params![
                    activity.date.clone(),
                    activity.category.clone(),
                    activity.source.clone(),
                    activity.title.clone(),
                    activity.detail.as_deref().unwrap_or("").to_string(),
                    activity.url.as_deref().unwrap_or("").to_string(),
                    activity.metadata.as_deref().unwrap_or("").to_string(),
                    activity.tag.clone(),
                ],
            )
            .await
            .context("Failed to write activity")?;

        Ok(())
    }

    pub async fn write_summary(&self, summary: &SummaryRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO summaries
                    (date, period, type, content, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                libsql::params![
                    summary.date.clone(),
                    summary.period.clone(),
                    summary.summary_type.clone(),
                    summary.content.clone(),
                    summary.metadata.as_deref().unwrap_or("").to_string(),
                ],
            )
            .await
            .context("Failed to write summary")?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read methods — for API endpoints
    // -----------------------------------------------------------------------

    /// Aggregate sessions by project for the Claude projects endpoint.
    pub async fn get_sessions_by_project(&self, days: u32) -> Result<Vec<ProjectAggregate>> {
        let cutoff = (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string();

        let mut rows = self
            .conn
            .query(
                "SELECT session_id, date, project, prompt_count, duration_minutes,
                        is_personal, first_prompt_at, last_prompt_at
                 FROM sessions
                 WHERE date >= ?1
                 ORDER BY first_prompt_at",
                libsql::params![cutoff],
            )
            .await
            .context("Failed to query sessions")?;

        let mut stats: HashMap<String, ProjectAggregateBuilder> = HashMap::new();

        while let Some(row) = rows.next().await? {
            let project: String = row.get(2)?;
            let prompt_count: i64 = row.get(3)?;
            let duration_minutes: i64 = row.get(4)?;
            let is_personal: i64 = row.get(5)?;
            let first_prompt_at: String = row.get(6)?;
            let last_prompt_at: String = row.get(7)?;
            let date: String = row.get(1)?;

            let entry = stats.entry(project.clone()).or_insert(ProjectAggregateBuilder {
                project,
                sessions: 0,
                total_prompts: 0,
                total_minutes: 0,
                is_personal: is_personal != 0,
                first_seen: first_prompt_at.clone(),
                last_seen: last_prompt_at.clone(),
                days_active: HashSet::new(),
            });

            entry.sessions += 1;
            entry.total_prompts += prompt_count;
            entry.total_minutes += duration_minutes;

            if first_prompt_at < entry.first_seen {
                entry.first_seen = first_prompt_at;
            }
            if last_prompt_at > entry.last_seen {
                entry.last_seen = last_prompt_at;
            }
            entry.days_active.insert(date);
        }

        let mut projects: Vec<ProjectAggregate> = stats
            .into_values()
            .map(|b| ProjectAggregate {
                project: b.project,
                sessions: b.sessions,
                total_prompts: b.total_prompts,
                total_minutes: b.total_minutes,
                is_personal: b.is_personal,
                days_active: b.days_active.len(),
                first_seen: b.first_seen,
                last_seen: b.last_seen,
            })
            .collect();

        projects.sort_by(|a, b| b.total_prompts.cmp(&a.total_prompts));

        Ok(projects)
    }

    /// Build heatmap data (project x date prompt counts).
    pub async fn get_session_heatmap(&self, days: u32) -> Result<Vec<HeatmapEntry>> {
        let cutoff = (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string();

        let mut rows = self
            .conn
            .query(
                "SELECT project, date, SUM(prompt_count) as prompts
                 FROM sessions
                 WHERE date >= ?1
                 GROUP BY project, date
                 ORDER BY project, date",
                libsql::params![cutoff],
            )
            .await
            .context("Failed to query session heatmap")?;

        let mut project_map: HashMap<String, (i64, Vec<HeatmapDay>)> = HashMap::new();

        while let Some(row) = rows.next().await? {
            let project: String = row.get(0)?;
            let date: String = row.get(1)?;
            let prompts: i64 = row.get(2)?;

            let entry = project_map.entry(project).or_insert((0, Vec::new()));
            entry.0 += prompts;
            entry.1.push(HeatmapDay { date, prompts });
        }

        let mut entries: Vec<HeatmapEntry> = project_map
            .into_iter()
            .map(|(project, (total_prompts, days))| HeatmapEntry {
                project,
                total_prompts,
                days,
            })
            .collect();

        entries.sort_by(|a, b| b.total_prompts.cmp(&a.total_prompts));

        Ok(entries)
    }

    /// Query activities table (for CLI).
    pub async fn query_activities(
        &self,
        date: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
        source: Option<&str>,
        category: Option<&str>,
        tag: Option<&str>,
    ) -> Result<Vec<ActivityRecord>> {
        let mut where_clauses = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();
        let mut idx = 1;

        if let Some(d) = date {
            where_clauses.push(format!("date = ?{idx}"));
            params.push(d.to_string().into());
            idx += 1;
        }
        if let Some(s) = start {
            where_clauses.push(format!("date >= ?{idx}"));
            params.push(s.to_string().into());
            idx += 1;
        }
        if let Some(e) = end {
            where_clauses.push(format!("date <= ?{idx}"));
            params.push(e.to_string().into());
            idx += 1;
        }
        if let Some(s) = source {
            where_clauses.push(format!("source = ?{idx}"));
            params.push(s.to_string().into());
            idx += 1;
        }
        if let Some(c) = category {
            where_clauses.push(format!("category = ?{idx}"));
            params.push(c.to_string().into());
            idx += 1;
        }
        if let Some(t) = tag {
            where_clauses.push(format!("tag = ?{idx}"));
            params.push(t.to_string().into());
            // idx += 1; // last one
        }

        let where_sql = if where_clauses.is_empty() {
            "1=1".to_string()
        } else {
            where_clauses.join(" AND ")
        };

        let sql = format!(
            "SELECT date, category, source, title, detail, url, metadata, tag
             FROM activities WHERE {where_sql} ORDER BY date, category, source"
        );

        let mut rows = self.conn.query(&sql, libsql::params_from_iter(params)).await?;
        let mut results = Vec::new();

        while let Some(row) = rows.next().await? {
            let detail: String = row.get(4)?;
            let url: String = row.get(5)?;
            let metadata: String = row.get(6)?;

            results.push(ActivityRecord {
                date: row.get(0)?,
                category: row.get(1)?,
                source: row.get(2)?,
                title: row.get(3)?,
                detail: if detail.is_empty() { None } else { Some(detail) },
                url: if url.is_empty() { None } else { Some(url) },
                metadata: if metadata.is_empty() { None } else { Some(metadata) },
                tag: row.get(7)?,
            });
        }

        Ok(results)
    }

    /// Query summaries table (for CLI).
    pub async fn query_summaries(
        &self,
        date: Option<&str>,
        period: Option<&str>,
        summary_type: Option<&str>,
    ) -> Result<Vec<SummaryRecord>> {
        let mut where_clauses = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();
        let mut idx = 1;

        if let Some(d) = date {
            where_clauses.push(format!("date = ?{idx}"));
            params.push(d.to_string().into());
            idx += 1;
        }
        if let Some(p) = period {
            where_clauses.push(format!("period = ?{idx}"));
            params.push(p.to_string().into());
            idx += 1;
        }
        if let Some(t) = summary_type {
            where_clauses.push(format!("type = ?{idx}"));
            params.push(t.to_string().into());
            // idx += 1;
        }

        let where_sql = if where_clauses.is_empty() {
            "1=1".to_string()
        } else {
            where_clauses.join(" AND ")
        };

        let sql = format!(
            "SELECT date, period, type, content, metadata
             FROM summaries WHERE {where_sql} ORDER BY date DESC"
        );

        let mut rows = self.conn.query(&sql, libsql::params_from_iter(params)).await?;
        let mut results = Vec::new();

        while let Some(row) = rows.next().await? {
            let metadata: String = row.get(4)?;
            results.push(SummaryRecord {
                date: row.get(0)?,
                period: row.get(1)?,
                summary_type: row.get(2)?,
                content: row.get(3)?,
                metadata: if metadata.is_empty() { None } else { Some(metadata) },
            });
        }

        Ok(results)
    }

    /// Get row counts for stats command.
    pub async fn stats(&self) -> Result<(i64, i64, i64, i64)> {
        let sessions: i64 = self
            .conn
            .query("SELECT COUNT(*) FROM sessions", ())
            .await?
            .next()
            .await?
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        let activities: i64 = self
            .conn
            .query("SELECT COUNT(*) FROM activities", ())
            .await?
            .next()
            .await?
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        let summaries: i64 = self
            .conn
            .query("SELECT COUNT(*) FROM summaries", ())
            .await?
            .next()
            .await?
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        let goals: i64 = self
            .conn
            .query("SELECT COUNT(*) FROM goals", ())
            .await?
            .next()
            .await?
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        Ok((sessions, activities, summaries, goals))
    }
}

// Helper for building project aggregates
struct ProjectAggregateBuilder {
    project: String,
    sessions: usize,
    total_prompts: i64,
    total_minutes: i64,
    is_personal: bool,
    first_seen: String,
    last_seen: String,
    days_active: HashSet<String>,
}
