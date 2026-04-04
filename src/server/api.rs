use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::config::Config;
use crate::db::Database;
use crate::types::{Accomplishment, Activity, Importance};

/// Embedded web UI assets (compiled into the binary)
#[derive(Embed)]
#[folder = "web-ui/"]
struct WebAssets;

/// Shared application state
pub struct AppState {
    pub db: Mutex<Database>,
}

/// Start the REST API server
pub async fn serve(host: &str, port: u16, open_browser: bool) -> Result<()> {
    let db = Database::open()?;
    let state = Arc::new(AppState { db: Mutex::new(db) });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/activities", get(list_activities))
        .route("/api/activities", post(create_activity))
        .route("/api/activities/search", get(search_activities))
        .route("/api/activities/:id", get(get_activity))
        .route("/api/activities/:id", put(update_activity))
        .route("/api/activities/:id", delete(delete_activity))
        .route("/api/stats", get(get_stats))
        .route("/api/heatmap", get(get_heatmap))
        .route("/api/accomplishments", get(list_accomplishments))
        .route("/api/accomplishments", post(create_accomplishment))
        .route("/api/accomplishments/:id", get(get_accomplishment))
        .route("/api/timeline", get(get_timeline))
        .route("/api/insights", get(get_insights))
        .route("/api/claude/projects", get(get_claude_projects))
        .route("/api/claude/heatmap", get(get_claude_heatmap))
        .route("/api/config", get(get_config))
        .route("/api/config", post(update_config))
        .route("/api/config/discover", get(discover_config))
        .route("/api/export", get(export_data))
        .fallback(get(serve_web_ui))
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let url = format!("http://{}", addr);
    println!("Folio server running on {}", url);
    println!();
    println!("  Dashboard:  {}", url);
    println!("  API:        {}/api/health", url);
    println!();
    println!("API Endpoints:");
    println!("  GET    /api/health              - Health check");
    println!("  GET    /api/activities           - List activities");
    println!("  POST   /api/activities           - Create activity");
    println!("  GET    /api/activities/:id       - Get activity");
    println!("  PUT    /api/activities/:id       - Update activity");
    println!("  DELETE /api/activities/:id       - Delete activity");
    println!("  GET    /api/activities/search?q= - Search");
    println!("  GET    /api/stats               - Get statistics");
    println!("  GET    /api/export?format=       - Export data");

    if open_browser {
        let _ = open_url(&url);
    }

    axum::serve(listener, app).await?;

    Ok(())
}

/// Open a URL in the default browser
fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()?;
    }
    Ok(())
}

/// Serve embedded web UI files
async fn serve_web_ui(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first, then fall back to index.html for SPA routing
    let file_path = if path.is_empty() { "index.html" } else { path };

    match WebAssets::get(file_path) {
        Some(content) => {
            let mime = mime_guess::from_path(file_path)
                .first_or_octet_stream()
                .to_string();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => {
            // SPA fallback: serve index.html for unmatched routes
            match WebAssets::get("index.html") {
                Some(content) => (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/html".to_string())],
                    content.data.to_vec(),
                )
                    .into_response(),
                None => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            }
        }
    }
}

// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    limit: Option<u32>,
    project: Option<String>,
    importance: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    title: String,
    description: Option<String>,
    impact: Option<String>,
    project: Option<String>,
    employer: Option<String>,
    importance: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

// List activities
async fn list_activities(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let limit = query.limit.unwrap_or(50);

    let mut activities = db.list_activities(Some(limit)).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    // Filter by project if specified
    if let Some(project) = query.project {
        activities.retain(|a| a.project.as_ref().map(|p| p == &project).unwrap_or(false));
    }

    // Filter by importance if specified
    if let Some(importance) = query.importance {
        let imp: Importance = importance.parse().unwrap_or(Importance::Medium);
        activities.retain(|a| a.importance == imp);
    }

    Ok(Json(activities))
}

// Create activity
async fn create_activity(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateActivityRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;

    let importance: Importance = req
        .importance
        .as_ref()
        .and_then(|i| i.parse().ok())
        .unwrap_or(Importance::Medium);

    let mut activity = Activity::new_manual(&req.title).with_importance(importance);

    if let Some(desc) = req.description {
        activity = activity.with_description(&desc);
    }

    if let Some(impact) = req.impact {
        activity = activity.with_impact(&impact);
    }

    if let Some(project) = req.project {
        activity = activity.with_project(project);
    }

    if let Some(employer) = req.employer {
        activity = activity.with_employer(employer);
    }

    db.insert_activity(&activity).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    Ok((StatusCode::CREATED, Json(activity)))
}

// Update activity request — all fields optional, only provided fields are updated
#[derive(Debug, Deserialize)]
pub struct UpdateActivityRequest {
    title: Option<String>,
    impact: Option<String>,
    project: Option<String>,
    employer: Option<String>,
    importance: Option<String>,
}

// Update activity
async fn update_activity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateActivityRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let db = state.db.lock().await;

    // Check that the activity exists
    let activity = db
        .get_activity(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .or_else(|| db.get_activity_by_partial_id(&id).ok().flatten());

    match activity {
        Some(a) => {
            db.update_activity(
                &a.id,
                req.title.as_deref(),
                req.impact.as_deref(),
                req.project.as_deref(),
                req.employer.as_deref(),
                req.importance.as_deref(),
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Return updated activity
            let updated = db
                .get_activity(&a.id)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(updated))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

// Get single activity
async fn get_activity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let db = state.db.lock().await;

    let activity = db
        .get_activity(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .or_else(|| db.get_activity_by_partial_id(&id).ok().flatten());

    match activity {
        Some(a) => Ok(Json(a)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// Delete activity
async fn delete_activity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let db = state.db.lock().await;

    // Find the activity first
    let activity = db
        .get_activity(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .or_else(|| db.get_activity_by_partial_id(&id).ok().flatten());

    match activity {
        Some(a) => {
            db.delete_activity(&a.id)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

// Search activities
async fn search_activities(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let limit = query.limit.unwrap_or(50);

    let activities = db.list_activities(Some(100)).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    let search_term = query.q.to_lowercase();
    let results: Vec<_> = activities
        .into_iter()
        .filter(|a| {
            a.title.to_lowercase().contains(&search_term)
                || a.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&search_term))
                    .unwrap_or(false)
                || a.project
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(&search_term))
                    .unwrap_or(false)
        })
        .take(limit as usize)
        .collect();

    Ok(Json(results))
}

// Get statistics
async fn get_stats(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;

    let activities = db.list_activities(None).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    let total = activities.len();
    let high = activities
        .iter()
        .filter(|a| matches!(a.importance, Importance::High))
        .count();
    let medium = activities
        .iter()
        .filter(|a| matches!(a.importance, Importance::Medium))
        .count();
    let low = activities
        .iter()
        .filter(|a| matches!(a.importance, Importance::Low))
        .count();

    let projects: std::collections::HashSet<_> = activities
        .iter()
        .filter_map(|a| a.project.clone())
        .collect();

    let mut by_source: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for a in &activities {
        *by_source.entry(a.source.to_string()).or_insert(0) += 1;
    }

    Ok(Json(serde_json::json!({
        "total_activities": total,
        "by_importance": {
            "high": high,
            "medium": medium,
            "low": low
        },
        "by_source": by_source,
        "projects_count": projects.len()
    })))
}

// Export data
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    format: Option<String>,
}

async fn export_data(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let format = query.format.unwrap_or_else(|| "json".to_string());

    let activities = db.list_activities(None).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    match format.as_str() {
        "json" => Ok(Json(serde_json::json!({
            "activities": activities,
            "exported_at": chrono::Utc::now().to_rfc3339()
        }))),
        _ => Ok(Json(serde_json::json!({
            "activities": activities,
            "exported_at": chrono::Utc::now().to_rfc3339()
        }))),
    }
}

// --- Heatmap endpoint ---

#[derive(Debug, Deserialize)]
pub struct HeatmapQuery {
    months: Option<u32>,
}

async fn get_heatmap(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HeatmapQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let months = query.months.unwrap_or(6);

    let activities = db.list_activities(None).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    let cutoff = chrono::Utc::now() - chrono::Duration::days(months as i64 * 30);

    // Group by date
    let mut by_date: std::collections::HashMap<String, HeatmapDay> =
        std::collections::HashMap::new();

    for a in &activities {
        if a.timestamp < cutoff {
            continue;
        }
        let date_key = a.timestamp.format("%Y-%m-%d").to_string();
        let entry = by_date.entry(date_key.clone()).or_insert(HeatmapDay {
            date: date_key,
            count: 0,
            high: 0,
            medium: 0,
            low: 0,
            sources: std::collections::HashSet::new(),
            projects: std::collections::HashSet::new(),
        });
        entry.count += 1;
        match a.importance {
            Importance::High => entry.high += 1,
            Importance::Medium => entry.medium += 1,
            Importance::Low => entry.low += 1,
        }
        entry.sources.insert(a.source.to_string());
        if let Some(ref p) = a.project {
            entry.projects.insert(p.clone());
        }
    }

    // Convert to sorted vec
    let mut days: Vec<serde_json::Value> = by_date
        .into_values()
        .map(|d| {
            serde_json::json!({
                "date": d.date,
                "count": d.count,
                "high": d.high,
                "medium": d.medium,
                "low": d.low,
                "sources": d.sources.into_iter().collect::<Vec<_>>(),
                "projects": d.projects.into_iter().collect::<Vec<_>>(),
            })
        })
        .collect();
    days.sort_by(|a, b| {
        a.get("date")
            .and_then(|d| d.as_str())
            .cmp(&b.get("date").and_then(|d| d.as_str()))
    });

    Ok(Json(serde_json::json!({
        "months": months,
        "days": days,
    })))
}

struct HeatmapDay {
    date: String,
    count: usize,
    high: usize,
    medium: usize,
    low: usize,
    sources: std::collections::HashSet<String>,
    projects: std::collections::HashSet<String>,
}

// --- Timeline endpoint ---

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    date: Option<String>,
    days: Option<u32>,
}

async fn get_timeline(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TimelineQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;

    let activities = db.list_activities(None).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    // Filter by date range
    let days_back = query.days.unwrap_or(7);
    let (start, end) = if let Some(ref date_str) = query.date {
        // Single day: return only activities on this exact date
        let day_start = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(1));
        let day_end = day_start + chrono::Duration::days(1);
        (day_start, day_end)
    } else {
        let start = chrono::Utc::now() - chrono::Duration::days(days_back as i64);
        let end = chrono::Utc::now() + chrono::Duration::days(1);
        (start, end)
    };

    let filtered: Vec<&Activity> = activities
        .iter()
        .filter(|a| a.timestamp >= start && a.timestamp < end)
        .collect();

    // Group by date
    let mut by_date: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
        std::collections::BTreeMap::new();

    for a in &filtered {
        let date_key = a.timestamp.format("%Y-%m-%d").to_string();
        by_date
            .entry(date_key)
            .or_default()
            .push(serde_json::json!({
                "id": a.id,
                "source": a.source.to_string(),
                "activity_type": a.activity_type.to_string(),
                "timestamp": a.timestamp.to_rfc3339(),
                "title": a.title,
                "project": a.project,
                "importance": a.importance.to_string(),
            }));
    }

    let timeline: Vec<serde_json::Value> = by_date
        .into_iter()
        .rev()
        .map(|(date, activities)| {
            serde_json::json!({
                "date": date,
                "count": activities.len(),
                "activities": activities,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "total": filtered.len(),
        "days": timeline,
    })))
}

// --- Accomplishments endpoints ---

#[derive(Debug, Deserialize)]
pub struct CreateAccomplishmentRequest {
    title: String,
    situation: Option<String>,
    task: Option<String>,
    action: Option<String>,
    result: Option<String>,
    activity_ids: Option<Vec<String>>,
    skills: Option<Vec<String>>,
    themes: Option<Vec<String>>,
    employer: Option<String>,
    role: Option<String>,
}

async fn list_accomplishments(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let accomplishments = db.list_accomplishments().map_err(|e| ApiError {
        error: e.to_string(),
    })?;
    Ok(Json(accomplishments))
}

async fn get_accomplishment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let db = state.db.lock().await;
    let accomplishment = db
        .get_accomplishment(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match accomplishment {
        Some(a) => Ok(Json(a)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_accomplishment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAccomplishmentRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let now = chrono::Utc::now();

    let accomplishment = Accomplishment {
        id: uuid::Uuid::new_v4().to_string(),
        title: req.title,
        situation: req.situation,
        task: req.task,
        action: req.action,
        result: req.result,
        metrics: vec![],
        activity_ids: req.activity_ids.unwrap_or_default(),
        skills: req.skills.unwrap_or_default(),
        themes: req.themes.unwrap_or_default(),
        employer: req.employer,
        role: req.role,
        start_date: None,
        end_date: None,
        generated_bullets: vec![],
        generated_story: None,
        created_at: now,
        updated_at: now,
    };

    db.insert_accomplishment(&accomplishment)
        .map_err(|e| ApiError {
            error: e.to_string(),
        })?;

    Ok((StatusCode::CREATED, Json(accomplishment)))
}

// --- Insights endpoint ---

#[derive(Debug, Deserialize)]
pub struct InsightsQuery {
    date: Option<String>,
    days: Option<u32>,
}

async fn get_insights(
    State(state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let activities = db.list_activities(None).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    let days_back = query.days.unwrap_or(7);
    let (start, end) = if let Some(ref date_str) = query.date {
        let day_start = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(1));
        (day_start, day_start + chrono::Duration::days(1))
    } else {
        let s = chrono::Utc::now() - chrono::Duration::days(days_back as i64);
        (s, chrono::Utc::now() + chrono::Duration::days(1))
    };

    let filtered: Vec<&Activity> = activities
        .iter()
        .filter(|a| a.timestamp >= start && a.timestamp < end)
        .collect();

    let total = filtered.len();

    // By importance
    let high = filtered
        .iter()
        .filter(|a| a.importance.to_string() == "high")
        .count();
    let medium = filtered
        .iter()
        .filter(|a| a.importance.to_string() == "medium")
        .count();
    let low = filtered
        .iter()
        .filter(|a| a.importance.to_string() == "low")
        .count();

    // By source
    let mut by_source: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for a in &filtered {
        *by_source.entry(a.source.to_string()).or_insert(0) += 1;
    }

    // By project (top 5)
    let mut by_project: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for a in &filtered {
        let p = a.project.as_deref().unwrap_or("unknown");
        let normalized =
            normalize_project_name(p, a.metadata.get("project_path").and_then(|v| v.as_str()));
        *by_project.entry(normalized).or_insert(0) += 1;
    }
    let mut top_projects: Vec<(String, usize)> = by_project.into_iter().collect();
    top_projects.sort_by(|a, b| b.1.cmp(&a.1));
    top_projects.truncate(5);

    // Work vs personal
    let mut work_count = 0usize;
    let mut personal_count = 0usize;
    for a in &filtered {
        let tag = a
            .metadata
            .get("identity_tag")
            .and_then(|v| v.as_str())
            .or_else(|| {
                a.metadata
                    .get("is_personal")
                    .and_then(|v| v.as_bool())
                    .map(|b| if b { "personal" } else { "work" })
            })
            .unwrap_or("unknown");
        match tag {
            "work" => work_count += 1,
            "personal" => personal_count += 1,
            _ => {}
        }
    }

    // Generate text insights
    let mut insights: Vec<serde_json::Value> = Vec::new();

    // Impact insight
    if total > 0 {
        let high_pct = (high as f64 / total as f64 * 100.0).round() as usize;
        if high_pct >= 20 {
            insights.push(serde_json::json!({"type": "positive", "text": format!("{}% of your activities are high impact — strong signal of impactful work", high_pct)}));
        } else if high_pct <= 5 && total >= 10 {
            insights.push(serde_json::json!({"type": "suggestion", "text": format!("Only {}% high impact activities. Consider focusing on fewer, higher-impact tasks", high_pct)}));
        }
    }

    // Focus insight
    if let Some((top_proj, top_count)) = top_projects.first() {
        let focus_pct = (*top_count as f64 / total as f64 * 100.0).round() as usize;
        if focus_pct >= 50 {
            insights.push(serde_json::json!({"type": "info", "text": format!("{}% of activity focused on {} — deep work mode", focus_pct, top_proj)}));
        }
        if top_projects.len() >= 5 && focus_pct < 30 {
            insights.push(serde_json::json!({"type": "suggestion", "text": format!("Spread across {} projects with no clear focus. Context switching may be reducing effectiveness", top_projects.len())}));
        }
    }

    // Work/personal balance
    if work_count > 0 && personal_count > 0 {
        let work_pct =
            (work_count as f64 / (work_count + personal_count) as f64 * 100.0).round() as usize;
        insights.push(serde_json::json!({"type": "info", "text": format!("{}% work / {}% personal split", work_pct, 100 - work_pct)}));
    }

    // Source diversity
    if by_source.len() == 1 {
        let only_source = by_source.keys().next().unwrap();
        insights.push(serde_json::json!({"type": "info", "text": format!("All activity from {} — run `folio sync` to pull in other sources", only_source.replace('_', " "))}));
    }

    // Streak: count consecutive days with activity (looking backward from today)
    let mut active_dates: std::collections::HashSet<String> = std::collections::HashSet::new();
    for a in &activities {
        active_dates.insert(a.timestamp.format("%Y-%m-%d").to_string());
    }
    let mut streak = 0i32;
    let mut check = chrono::Utc::now().date_naive();
    loop {
        let date_str = check.format("%Y-%m-%d").to_string();
        if active_dates.contains(&date_str) {
            streak += 1;
            check -= chrono::Duration::days(1);
        } else {
            break;
        }
    }

    // Highlights: high-importance activities in the filtered range
    let highlights: Vec<serde_json::Value> = filtered
        .iter()
        .filter(|a| a.importance.to_string() == "high")
        .take(5)
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "title": a.title,
                "source": a.source.to_string(),
                "project": a.project,
                "timestamp": a.timestamp.to_rfc3339(),
            })
        })
        .collect();

    // Busiest day in the filtered range
    let mut by_day: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for a in &filtered {
        *by_day
            .entry(a.timestamp.format("%Y-%m-%d").to_string())
            .or_insert(0) += 1;
    }
    let busiest_day = by_day
        .iter()
        .max_by_key(|(_, c)| *c)
        .map(|(d, c)| serde_json::json!({"date": d, "count": c}));

    Ok(Json(serde_json::json!({
        "total": total,
        "importance": {"high": high, "medium": medium, "low": low},
        "by_source": by_source,
        "top_projects": top_projects.iter().map(|(p, c)| serde_json::json!({"project": p, "count": c})).collect::<Vec<_>>(),
        "work_personal": {"work": work_count, "personal": personal_count, "unknown": total - work_count - personal_count},
        "insights": insights,
        "streak": streak,
        "highlights": highlights,
        "busiest_day": busiest_day,
    })))
}

// --- Helpers ---

/// Normalize a project name by resolving worktrees to their parent project.
fn normalize_project_name(project_name: &str, project_path: Option<&str>) -> String {
    // If we have the full path, use it for better detection
    if let Some(path) = project_path {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Pattern 1: Claude Code worktrees — .claude/worktrees/<name>
        if let Some(pos) = parts.iter().position(|&s| s == ".claude") {
            if pos > 0 && parts.get(pos + 1) == Some(&"worktrees") && parts.get(pos + 2).is_some() {
                return parts[pos - 1].to_string();
            }
        }
    }

    // Pattern 2: Git worktrees — project--branch-name
    if let Some(idx) = project_name.find("--") {
        return project_name[..idx].to_string();
    }

    // Pattern 3: Global .claude directory
    if project_name == ".claude" {
        return "claude-config".to_string();
    }

    project_name.to_string()
}

// --- Claude Code analytics endpoints ---

#[derive(Debug, Deserialize)]
pub struct ClaudeProjectsQuery {
    days: Option<u32>,
}

async fn get_claude_projects(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ClaudeProjectsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let activities = db.list_activities(None).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    let days = query.days.unwrap_or(30);
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);

    // Filter to claude_code activities within time range
    let claude_activities: Vec<&Activity> = activities
        .iter()
        .filter(|a| a.source.to_string() == "claude_code" && a.timestamp >= cutoff)
        .collect();

    // Aggregate by project
    let mut project_stats: std::collections::HashMap<String, ProjectStats> =
        std::collections::HashMap::new();

    for a in &claude_activities {
        let raw_project = a.project.as_deref().unwrap_or("unknown");
        // Normalize: resolve worktrees to parent project
        let project = normalize_project_name(
            raw_project,
            a.metadata.get("project_path").and_then(|v| v.as_str()),
        );
        let stats = project_stats
            .entry(project.clone())
            .or_insert(ProjectStats {
                project,
                sessions: 0,
                total_prompts: 0,
                total_minutes: 0,
                is_personal: false,
                first_seen: a.timestamp,
                last_seen: a.timestamp,
                days_active: std::collections::HashSet::new(),
            });

        stats.sessions += 1;

        // Extract prompt count and duration from metadata
        if let Some(count) = a.metadata.get("prompt_count").and_then(|v| v.as_u64()) {
            stats.total_prompts += count as usize;
        }
        if let Some(mins) = a.metadata.get("duration_minutes").and_then(|v| v.as_i64()) {
            stats.total_minutes += mins.max(0) as usize;
        }
        if let Some(personal) = a.metadata.get("is_personal").and_then(|v| v.as_bool()) {
            stats.is_personal = personal;
        }

        if a.timestamp < stats.first_seen {
            stats.first_seen = a.timestamp;
        }
        if a.timestamp > stats.last_seen {
            stats.last_seen = a.timestamp;
        }
        stats
            .days_active
            .insert(a.timestamp.format("%Y-%m-%d").to_string());
    }

    // Convert to sorted list
    let mut projects: Vec<serde_json::Value> = project_stats
        .into_values()
        .map(|s| {
            serde_json::json!({
                "project": s.project,
                "sessions": s.sessions,
                "total_prompts": s.total_prompts,
                "total_minutes": s.total_minutes,
                "is_personal": s.is_personal,
                "days_active": s.days_active.len(),
                "first_seen": s.first_seen.to_rfc3339(),
                "last_seen": s.last_seen.to_rfc3339(),
            })
        })
        .collect();

    // Sort by total prompts descending
    projects.sort_by(|a, b| {
        b.get("total_prompts")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .cmp(&a.get("total_prompts").and_then(|v| v.as_u64()).unwrap_or(0))
    });

    Ok(Json(serde_json::json!({
        "days": days,
        "total_sessions": claude_activities.len(),
        "total_projects": projects.len(),
        "projects": projects,
    })))
}

struct ProjectStats {
    project: String,
    sessions: usize,
    total_prompts: usize,
    total_minutes: usize,
    is_personal: bool,
    first_seen: chrono::DateTime<chrono::Utc>,
    last_seen: chrono::DateTime<chrono::Utc>,
    days_active: std::collections::HashSet<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClaudeHeatmapQuery {
    days: Option<u32>,
}

async fn get_claude_heatmap(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ClaudeHeatmapQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    let activities = db.list_activities(None).map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    let days = query.days.unwrap_or(90);
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);

    // Build a project x date matrix
    let mut matrix: std::collections::HashMap<String, std::collections::HashMap<String, usize>> =
        std::collections::HashMap::new();

    for a in &activities {
        if a.source.to_string() != "claude_code" || a.timestamp < cutoff {
            continue;
        }
        let raw_project = a.project.as_deref().unwrap_or("unknown");
        let project = normalize_project_name(
            raw_project,
            a.metadata.get("project_path").and_then(|v| v.as_str()),
        );
        let date = a.timestamp.format("%Y-%m-%d").to_string();
        let prompts = a
            .metadata
            .get("prompt_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        *matrix.entry(project).or_default().entry(date).or_insert(0) += prompts;
    }

    // Convert to response format
    let mut projects: Vec<serde_json::Value> = matrix
        .into_iter()
        .map(|(project, dates)| {
            let total: usize = dates.values().sum();
            let days_data: Vec<serde_json::Value> = dates
                .into_iter()
                .map(|(date, count)| serde_json::json!({"date": date, "prompts": count}))
                .collect();
            serde_json::json!({
                "project": project,
                "total_prompts": total,
                "days": days_data,
            })
        })
        .collect();

    projects.sort_by(|a, b| {
        b.get("total_prompts")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .cmp(&a.get("total_prompts").and_then(|v| v.as_u64()).unwrap_or(0))
    });

    Ok(Json(serde_json::json!({
        "days": days,
        "projects": projects,
    })))
}

// --- Config endpoints ---

async fn get_config() -> Result<impl IntoResponse, ApiError> {
    let config = Config::load().map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    let config_exists = Config::config_path().map(|p| p.exists()).unwrap_or(false);

    // Count repos per scan dir
    let scan_dirs_info: Vec<serde_json::Value> = config
        .git
        .scan_dirs
        .iter()
        .map(|dir| {
            let exists = dir.exists();
            let repo_count = if exists {
                crate::integrations::git::discover_repos_in(
                    std::slice::from_ref(dir),
                    config.git.max_depth,
                )
                .map(|r| r.len())
                .unwrap_or(0)
            } else {
                0
            };
            serde_json::json!({
                "path": dir.display().to_string(),
                "exists": exists,
                "repo_count": repo_count,
            })
        })
        .collect();

    let history_exists = dirs::home_dir()
        .map(|h| h.join(".claude").join("history.jsonl").exists())
        .unwrap_or(false);

    Ok(Json(serde_json::json!({
        "config_exists": config_exists,
        "scan_dirs": scan_dirs_info,
        "email_tags": config.git.email_tags,
        "git_email": config.general.git_email,
        "default_employer": config.general.default_employer,
        "integrations": {
            "git": { "enabled": config.git.enabled },
            "github": { "enabled": config.github.enabled, "configured": config.github.token.is_some() },
            "linear": { "enabled": config.linear.enabled, "configured": config.linear.api_key.is_some() },
            "claude_code": { "configured": history_exists },
        }
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    scan_dirs: Option<Vec<String>>,
    email_tags: Option<std::collections::HashMap<String, String>>,
    default_employer: Option<String>,
    git_email: Option<String>,
}

async fn update_config(
    Json(req): Json<UpdateConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut config = Config::load().map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    if let Some(dirs) = req.scan_dirs {
        config.git.scan_dirs = dirs.into_iter().map(std::path::PathBuf::from).collect();
    }
    if let Some(tags) = req.email_tags {
        config.git.email_tags = tags;
    }
    if let Some(employer) = req.default_employer {
        config.general.default_employer = if employer.is_empty() {
            None
        } else {
            Some(employer)
        };
    }
    if let Some(email) = req.git_email {
        config.general.git_email = if email.is_empty() { None } else { Some(email) };
    }

    config.save().map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

async fn discover_config() -> Result<impl IntoResponse, ApiError> {
    let config = Config::load().map_err(|e| ApiError {
        error: e.to_string(),
    })?;

    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let candidate_dirs = [
        home.join("Documents").join("code"),
        home.join("code"),
        home.join("projects"),
        home.join("work"),
        home.join("dev"),
        home.join("src"),
        home.join("repos"),
    ];

    // Discover directories with repos
    let dirs_info: Vec<serde_json::Value> = candidate_dirs
        .iter()
        .chain(config.git.scan_dirs.iter())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .filter(|dir| dir.exists())
        .map(|dir| {
            let repos = crate::integrations::git::discover_repos_in(std::slice::from_ref(dir), 3)
                .unwrap_or_default();
            serde_json::json!({
                "path": dir.display().to_string(),
                "repo_count": repos.len(),
                "repo_names": repos.iter().take(10).map(|r|
                    r.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
                ).collect::<Vec<_>>(),
                "already_configured": config.git.scan_dirs.contains(dir),
            })
        })
        .filter(|d| d.get("repo_count").and_then(|v| v.as_u64()).unwrap_or(0) > 0)
        .collect();

    // Discover emails
    let scan_dirs: Vec<std::path::PathBuf> = dirs_info
        .iter()
        .filter_map(|d| {
            d.get("path")
                .and_then(|p| p.as_str())
                .map(std::path::PathBuf::from)
        })
        .collect();

    let emails = if !scan_dirs.is_empty() {
        crate::integrations::git::discover_emails(&scan_dirs, 3).unwrap_or_default()
    } else {
        vec![]
    };

    let personal_domains = [
        "gmail.com",
        "outlook.com",
        "hotmail.com",
        "yahoo.com",
        "proton.me",
        "icloud.com",
        "me.com",
    ];

    let emails_info: Vec<serde_json::Value> = emails
        .into_iter()
        .map(|(email, repos)| {
            let suggested = if personal_domains.iter().any(|d| email.ends_with(d)) {
                "personal"
            } else {
                "work"
            };
            let current_tag = config.git.email_tags.get(&email);
            serde_json::json!({
                "email": email,
                "repos": repos,
                "suggested_tag": suggested,
                "current_tag": current_tag,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "dirs": dirs_info,
        "emails": emails_info,
    })))
}
