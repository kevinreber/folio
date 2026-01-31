use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::types::{Accomplishment, Activity, ActivitySource, ActivityType, Importance};

const SCHEMA_VERSION: i32 = 1;

/// Get the default database path (~/.folio/folio.db)
pub fn default_db_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let folio_dir = home.join(".folio");
    std::fs::create_dir_all(&folio_dir).context("Could not create .folio directory")?;
    Ok(folio_dir.join("folio.db"))
}

/// Database connection wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the database at the default location
    pub fn open() -> Result<Self> {
        let path = default_db_path()?;
        Self::open_at(path)
    }

    /// Open or create the database at a specific path
    pub fn open_at(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)
            .with_context(|| format!("Could not open database at {:?}", path))?;

        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Initialize the database schema
    fn initialize(&self) -> Result<()> {
        // Check schema version
        let version: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM (SELECT 0 as version UNION ALL SELECT version FROM pragma_user_version)",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if version < SCHEMA_VERSION {
            self.run_migrations()?;
        }

        Ok(())
    }

    fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Activities table: raw events from all sources
            CREATE TABLE IF NOT EXISTS activities (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                activity_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                project TEXT,
                employer TEXT,
                importance TEXT NOT NULL DEFAULT 'medium',
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Accomplishments table: enriched, packaged accomplishments
            CREATE TABLE IF NOT EXISTS accomplishments (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                situation TEXT,
                task TEXT,
                action TEXT,
                result TEXT,
                metrics TEXT NOT NULL DEFAULT '[]',
                skills TEXT NOT NULL DEFAULT '[]',
                themes TEXT NOT NULL DEFAULT '[]',
                employer TEXT,
                role TEXT,
                start_date TEXT,
                end_date TEXT,
                generated_bullets TEXT NOT NULL DEFAULT '[]',
                generated_story TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Link table for activities <-> accomplishments (many-to-many)
            CREATE TABLE IF NOT EXISTS activity_accomplishment_links (
                activity_id TEXT NOT NULL,
                accomplishment_id TEXT NOT NULL,
                PRIMARY KEY (activity_id, accomplishment_id),
                FOREIGN KEY (activity_id) REFERENCES activities(id),
                FOREIGN KEY (accomplishment_id) REFERENCES accomplishments(id)
            );

            -- Employment history
            CREATE TABLE IF NOT EXISTS employment (
                id TEXT PRIMARY KEY,
                company TEXT NOT NULL,
                role TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT,
                team_size INTEGER,
                tech_stack TEXT NOT NULL DEFAULT '[]',
                domain TEXT
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_activities_timestamp ON activities(timestamp);
            CREATE INDEX IF NOT EXISTS idx_activities_source ON activities(source);
            CREATE INDEX IF NOT EXISTS idx_activities_employer ON activities(employer);
            CREATE INDEX IF NOT EXISTS idx_accomplishments_employer ON accomplishments(employer);

            -- Update schema version
            PRAGMA user_version = 1;
            "#,
        )?;

        Ok(())
    }

    /// Insert a new activity
    pub fn insert_activity(&self, activity: &Activity) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO activities (
                id, source, activity_type, timestamp, title, description,
                project, employer, importance, metadata, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                activity.id,
                activity.source.to_string(),
                activity.activity_type.to_string(),
                activity.timestamp.to_rfc3339(),
                activity.title,
                activity.description,
                activity.project,
                activity.employer,
                activity.importance.to_string(),
                activity.metadata.to_string(),
                activity.created_at.to_rfc3339(),
                activity.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// Get all activities, ordered by timestamp descending
    pub fn list_activities(&self, limit: Option<u32>) -> Result<Vec<Activity>> {
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let query = format!(
            "SELECT id, source, activity_type, timestamp, title, description,
                    project, employer, importance, metadata, created_at, updated_at
             FROM activities ORDER BY timestamp DESC {}",
            limit_clause
        );

        let mut stmt = self.conn.prepare(&query)?;
        let activities = stmt
            .query_map([], |row| {
                Ok(Activity {
                    id: row.get(0)?,
                    source: row
                        .get::<_, String>(1)?
                        .parse()
                        .unwrap_or(ActivitySource::Manual),
                    activity_type: row
                        .get::<_, String>(2)?
                        .parse()
                        .unwrap_or(ActivityType::ManualEntry),
                    timestamp: parse_datetime(&row.get::<_, String>(3)?),
                    title: row.get(4)?,
                    description: row.get(5)?,
                    project: row.get(6)?,
                    employer: row.get(7)?,
                    importance: row
                        .get::<_, String>(8)?
                        .parse()
                        .unwrap_or(Importance::Medium),
                    metadata: serde_json::from_str(&row.get::<_, String>(9)?)
                        .unwrap_or(serde_json::json!({})),
                    created_at: parse_datetime(&row.get::<_, String>(10)?),
                    updated_at: parse_datetime(&row.get::<_, String>(11)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(activities)
    }

    /// Get a single activity by ID
    pub fn get_activity(&self, id: &str) -> Result<Option<Activity>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, activity_type, timestamp, title, description,
                    project, employer, importance, metadata, created_at, updated_at
             FROM activities WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Activity {
                id: row.get(0)?,
                source: row
                    .get::<_, String>(1)?
                    .parse()
                    .unwrap_or(ActivitySource::Manual),
                activity_type: row
                    .get::<_, String>(2)?
                    .parse()
                    .unwrap_or(ActivityType::ManualEntry),
                timestamp: parse_datetime(&row.get::<_, String>(3)?),
                title: row.get(4)?,
                description: row.get(5)?,
                project: row.get(6)?,
                employer: row.get(7)?,
                importance: row
                    .get::<_, String>(8)?
                    .parse()
                    .unwrap_or(Importance::Medium),
                metadata: serde_json::from_str(&row.get::<_, String>(9)?)
                    .unwrap_or(serde_json::json!({})),
                created_at: parse_datetime(&row.get::<_, String>(10)?),
                updated_at: parse_datetime(&row.get::<_, String>(11)?),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get activity by partial ID match
    pub fn get_activity_by_partial_id(&self, partial_id: &str) -> Result<Option<Activity>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, activity_type, timestamp, title, description,
                    project, employer, importance, metadata, created_at, updated_at
             FROM activities WHERE id LIKE ?1 || '%' LIMIT 1",
        )?;

        let mut rows = stmt.query(params![partial_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Activity {
                id: row.get(0)?,
                source: row
                    .get::<_, String>(1)?
                    .parse()
                    .unwrap_or(ActivitySource::Manual),
                activity_type: row
                    .get::<_, String>(2)?
                    .parse()
                    .unwrap_or(ActivityType::ManualEntry),
                timestamp: parse_datetime(&row.get::<_, String>(3)?),
                title: row.get(4)?,
                description: row.get(5)?,
                project: row.get(6)?,
                employer: row.get(7)?,
                importance: row
                    .get::<_, String>(8)?
                    .parse()
                    .unwrap_or(Importance::Medium),
                metadata: serde_json::from_str(&row.get::<_, String>(9)?)
                    .unwrap_or(serde_json::json!({})),
                created_at: parse_datetime(&row.get::<_, String>(10)?),
                updated_at: parse_datetime(&row.get::<_, String>(11)?),
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete an activity by ID
    pub fn delete_activity(&self, id: &str) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM activities WHERE id = ?1", params![id])?;
        Ok(rows_affected > 0)
    }

    /// Count total activities
    pub fn count_activities(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM activities", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Update an existing activity
    pub fn update_activity(
        &self,
        id: &str,
        title: Option<&str>,
        impact: Option<&str>,
        project: Option<&str>,
        employer: Option<&str>,
        importance: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        // Build dynamic update query
        let mut updates = vec!["updated_at = ?1"];

        if title.is_some() {
            updates.push("title = ?2");
        }

        // Get current activity to preserve metadata
        let current = self
            .get_activity(id)?
            .or_else(|| self.get_activity_by_partial_id(id).ok().flatten())
            .context("Activity not found")?;

        let mut metadata = current.metadata.clone();
        if let Some(impact_val) = impact {
            metadata["impact"] = serde_json::json!(impact_val);
        }

        // Use simple update for now
        self.conn.execute(
            r#"
            UPDATE activities SET
                title = COALESCE(?2, title),
                description = COALESCE(?3, description),
                project = COALESCE(?4, project),
                employer = COALESCE(?5, employer),
                importance = COALESCE(?6, importance),
                metadata = ?7,
                updated_at = ?8
            WHERE id = ?1 OR id LIKE ?1 || '%'
            "#,
            params![
                id,
                title,
                impact,
                project,
                employer,
                importance,
                metadata.to_string(),
                now,
            ],
        )?;

        Ok(())
    }

    /// Check if an activity exists by metadata field
    pub fn activity_exists_by_metadata(&self, key: &str, value: &str) -> Result<bool> {
        let pattern = format!("%\"{}\":\"{}\"%", key, value);
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM activities WHERE metadata LIKE ?1",
            params![pattern],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Insert a new accomplishment
    pub fn insert_accomplishment(&self, accomplishment: &Accomplishment) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO accomplishments (
                id, title, situation, task, action, result,
                metrics, skills, themes, employer, role,
                start_date, end_date, generated_bullets, generated_story,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
            params![
                accomplishment.id,
                accomplishment.title,
                accomplishment.situation,
                accomplishment.task,
                accomplishment.action,
                accomplishment.result,
                serde_json::to_string(&accomplishment.metrics)?,
                serde_json::to_string(&accomplishment.skills)?,
                serde_json::to_string(&accomplishment.themes)?,
                accomplishment.employer,
                accomplishment.role,
                accomplishment.start_date.map(|d| d.to_rfc3339()),
                accomplishment.end_date.map(|d| d.to_rfc3339()),
                serde_json::to_string(&accomplishment.generated_bullets)?,
                accomplishment.generated_story,
                accomplishment.created_at.to_rfc3339(),
                accomplishment.updated_at.to_rfc3339(),
            ],
        )?;

        // Link activities
        for activity_id in &accomplishment.activity_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO activity_accomplishment_links (activity_id, accomplishment_id) VALUES (?1, ?2)",
                params![activity_id, accomplishment.id],
            )?;
        }

        Ok(())
    }

    /// List all accomplishments
    pub fn list_accomplishments(&self) -> Result<Vec<Accomplishment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, situation, task, action, result,
                    metrics, skills, themes, employer, role,
                    start_date, end_date, generated_bullets, generated_story,
                    created_at, updated_at
             FROM accomplishments ORDER BY created_at DESC",
        )?;

        let accomplishments = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;

                Ok(Accomplishment {
                    id: id.clone(),
                    title: row.get(1)?,
                    situation: row.get(2)?,
                    task: row.get(3)?,
                    action: row.get(4)?,
                    result: row.get(5)?,
                    metrics: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                    skills: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                    themes: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                    employer: row.get(9)?,
                    role: row.get(10)?,
                    start_date: row
                        .get::<_, Option<String>>(11)?
                        .map(|s| parse_datetime(&s)),
                    end_date: row
                        .get::<_, Option<String>>(12)?
                        .map(|s| parse_datetime(&s)),
                    generated_bullets: serde_json::from_str(&row.get::<_, String>(13)?)
                        .unwrap_or_default(),
                    generated_story: row.get(14)?,
                    created_at: parse_datetime(&row.get::<_, String>(15)?),
                    updated_at: parse_datetime(&row.get::<_, String>(16)?),
                    activity_ids: vec![], // Would need separate query to fetch
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accomplishments)
    }

    /// Get a single accomplishment by ID
    pub fn get_accomplishment(&self, id: &str) -> Result<Option<Accomplishment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, situation, task, action, result,
                    metrics, skills, themes, employer, role,
                    start_date, end_date, generated_bullets, generated_story,
                    created_at, updated_at
             FROM accomplishments WHERE id = ?1 OR id LIKE ?1 || '%' LIMIT 1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let acc_id: String = row.get(0)?;

            // Get linked activity IDs
            let activity_ids: Vec<String> = self.conn
                .prepare("SELECT activity_id FROM activity_accomplishment_links WHERE accomplishment_id = ?1")?
                .query_map(params![&acc_id], |r| r.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Some(Accomplishment {
                id: acc_id,
                title: row.get(1)?,
                situation: row.get(2)?,
                task: row.get(3)?,
                action: row.get(4)?,
                result: row.get(5)?,
                metrics: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                skills: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                themes: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                employer: row.get(9)?,
                role: row.get(10)?,
                start_date: row
                    .get::<_, Option<String>>(11)?
                    .map(|s| parse_datetime(&s)),
                end_date: row
                    .get::<_, Option<String>>(12)?
                    .map(|s| parse_datetime(&s)),
                generated_bullets: serde_json::from_str(&row.get::<_, String>(13)?)
                    .unwrap_or_default(),
                generated_story: row.get(14)?,
                created_at: parse_datetime(&row.get::<_, String>(15)?),
                updated_at: parse_datetime(&row.get::<_, String>(16)?),
                activity_ids,
            }))
        } else {
            Ok(None)
        }
    }

    /// Search activities by text
    pub fn search_activities(&self, query: &str, limit: u32) -> Result<Vec<Activity>> {
        let pattern = format!("%{}%", query);

        let mut stmt = self.conn.prepare(
            "SELECT id, source, activity_type, timestamp, title, description,
                    project, employer, importance, metadata, created_at, updated_at
             FROM activities
             WHERE title LIKE ?1 OR description LIKE ?1 OR project LIKE ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let activities = stmt
            .query_map(params![pattern, limit], |row| {
                Ok(Activity {
                    id: row.get(0)?,
                    source: row
                        .get::<_, String>(1)?
                        .parse()
                        .unwrap_or(ActivitySource::Manual),
                    activity_type: row
                        .get::<_, String>(2)?
                        .parse()
                        .unwrap_or(ActivityType::ManualEntry),
                    timestamp: parse_datetime(&row.get::<_, String>(3)?),
                    title: row.get(4)?,
                    description: row.get(5)?,
                    project: row.get(6)?,
                    employer: row.get(7)?,
                    importance: row
                        .get::<_, String>(8)?
                        .parse()
                        .unwrap_or(Importance::Medium),
                    metadata: serde_json::from_str(&row.get::<_, String>(9)?)
                        .unwrap_or(serde_json::json!({})),
                    created_at: parse_datetime(&row.get::<_, String>(10)?),
                    updated_at: parse_datetime(&row.get::<_, String>(11)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(activities)
    }
}

fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_query_activity() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open_at(db_path).unwrap();

        let activity = Activity::new_manual("Test accomplishment")
            .with_description("This is a test")
            .with_impact("50% improvement");

        db.insert_activity(&activity).unwrap();

        let activities = db.list_activities(None).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].title, "Test accomplishment");
    }
}
