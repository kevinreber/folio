use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::db::Database;
use crate::types::{Activity, Importance};

/// Shared application state
pub struct AppState {
    pub db: Mutex<Database>,
}

/// Start the REST API server
pub async fn serve(host: &str, port: u16) -> Result<()> {
    let db = Database::open()?;
    let state = Arc::new(AppState {
        db: Mutex::new(db),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/activities", get(list_activities))
        .route("/api/activities", post(create_activity))
        .route("/api/activities/:id", get(get_activity))
        .route("/api/activities/:id", delete(delete_activity))
        .route("/api/activities/search", get(search_activities))
        .route("/api/stats", get(get_stats))
        .route("/api/export", get(export_data))
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("Folio API server running on http://{}", addr);
    println!("Endpoints:");
    println!("  GET    /api/health          - Health check");
    println!("  GET    /api/activities      - List activities");
    println!("  POST   /api/activities      - Create activity");
    println!("  GET    /api/activities/:id  - Get activity");
    println!("  DELETE /api/activities/:id  - Delete activity");
    println!("  GET    /api/activities/search?q=query - Search");
    println!("  GET    /api/stats           - Get statistics");
    println!("  GET    /api/export?format=json|yaml - Export data");

    axum::serve(listener, app).await?;

    Ok(())
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

    let mut activities = db.list_activities(Some(limit))
        .map_err(|e| ApiError { error: e.to_string() })?;

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

    let importance: Importance = req.importance
        .as_ref()
        .and_then(|i| i.parse().ok())
        .unwrap_or(Importance::Medium);

    let mut activity = Activity::new_manual(&req.title)
        .with_importance(importance);

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

    db.insert_activity(&activity)
        .map_err(|e| ApiError { error: e.to_string() })?;

    Ok((StatusCode::CREATED, Json(activity)))
}

// Get single activity
async fn get_activity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let db = state.db.lock().await;

    let activity = db.get_activity(&id)
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
    let activity = db.get_activity(&id)
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

    let activities = db.list_activities(Some(100))
        .map_err(|e| ApiError { error: e.to_string() })?;

    let search_term = query.q.to_lowercase();
    let results: Vec<_> = activities
        .into_iter()
        .filter(|a| {
            a.title.to_lowercase().contains(&search_term)
                || a.description.as_ref().map(|d| d.to_lowercase().contains(&search_term)).unwrap_or(false)
                || a.project.as_ref().map(|p| p.to_lowercase().contains(&search_term)).unwrap_or(false)
        })
        .take(limit as usize)
        .collect();

    Ok(Json(results))
}

// Get statistics
async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;

    let activities = db.list_activities(None)
        .map_err(|e| ApiError { error: e.to_string() })?;

    let total = activities.len();
    let high = activities.iter().filter(|a| matches!(a.importance, Importance::High)).count();
    let medium = activities.iter().filter(|a| matches!(a.importance, Importance::Medium)).count();
    let low = activities.iter().filter(|a| matches!(a.importance, Importance::Low)).count();

    let mut projects: std::collections::HashSet<_> = activities
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

    let activities = db.list_activities(None)
        .map_err(|e| ApiError { error: e.to_string() })?;

    match format.as_str() {
        "json" => {
            Ok(Json(serde_json::json!({
                "activities": activities,
                "exported_at": chrono::Utc::now().to_rfc3339()
            })))
        }
        _ => {
            Ok(Json(serde_json::json!({
                "activities": activities,
                "exported_at": chrono::Utc::now().to_rfc3339()
            })))
        }
    }
}
