use anyhow::Result;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::ai::{BulletGenerator, DigestGenerator};
use crate::db::Database;
use crate::export::{ExportFormat, Exporter};
use crate::types::Activity;

/// MCP Server for Claude integration
/// Implements the Model Context Protocol for AI assistants

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

/// MCP Server state
pub struct McpState {
    pub db: Mutex<Database>,
}

/// Start the MCP server
pub async fn serve(host: &str, port: u16) -> Result<()> {
    let db = Database::open()?;
    let state = Arc::new(McpState { db: Mutex::new(db) });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", post(handle_mcp_request))
        .route("/mcp", post(handle_mcp_request))
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("Folio MCP server running on http://{}", addr);
    println!("\nAvailable tools:");
    println!("  - folio_list_activities: List recent activities");
    println!("  - folio_search: Search activities");
    println!("  - folio_get_activity: Get activity details");
    println!("  - folio_create_activity: Create new activity");
    println!("  - folio_generate_bullets: Generate resume bullets");
    println!("  - folio_generate_digest: Generate activity digest");
    println!("  - folio_export: Export activities");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_mcp_request(
    State(state): State<Arc<McpState>>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&request),
        "tools/list" => handle_tools_list(&request),
        "tools/call" => handle_tools_call(&state, &request).await,
        "resources/list" => handle_resources_list(&request),
        _ => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(McpError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    };

    Json(response)
}

fn handle_initialize(request: &McpRequest) -> McpResponse {
    McpResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "folio",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        error: None,
    }
}

fn handle_tools_list(request: &McpRequest) -> McpResponse {
    let tools = serde_json::json!({
        "tools": [
            {
                "name": "folio_list_activities",
                "description": "List recent activities from the user's career accomplishment tracker",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of activities to return (default: 10)"
                        },
                        "project": {
                            "type": "string",
                            "description": "Filter by project name"
                        }
                    }
                }
            },
            {
                "name": "folio_search",
                "description": "Search activities by keyword",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum results (default: 10)"
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "folio_get_activity",
                "description": "Get detailed information about a specific activity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Activity ID (can be partial)"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "folio_create_activity",
                "description": "Create a new activity/accomplishment",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Title of the accomplishment"
                        },
                        "description": {
                            "type": "string",
                            "description": "Detailed description"
                        },
                        "impact": {
                            "type": "string",
                            "description": "Impact or result of the accomplishment"
                        },
                        "project": {
                            "type": "string",
                            "description": "Project name"
                        },
                        "importance": {
                            "type": "string",
                            "enum": ["low", "medium", "high"],
                            "description": "Importance level"
                        }
                    },
                    "required": ["title"]
                }
            },
            {
                "name": "folio_generate_bullets",
                "description": "Generate resume bullet points from activities",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "activity_ids": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Activity IDs to generate bullets from (optional, uses recent if not specified)"
                        }
                    }
                }
            },
            {
                "name": "folio_generate_digest",
                "description": "Generate a digest/summary of recent activities",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "period": {
                            "type": "string",
                            "enum": ["daily", "weekly", "monthly"],
                            "description": "Time period for the digest"
                        }
                    }
                }
            },
            {
                "name": "folio_export",
                "description": "Export activities to a format",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "enum": ["markdown", "json", "yaml"],
                            "description": "Export format"
                        }
                    }
                }
            }
        ]
    });

    McpResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(tools),
        error: None,
    }
}

fn handle_resources_list(request: &McpRequest) -> McpResponse {
    McpResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(serde_json::json!({
            "resources": []
        })),
        error: None,
    }
}

async fn handle_tools_call(state: &McpState, request: &McpRequest) -> McpResponse {
    let params = &request.params;
    let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let result = match tool_name {
        "folio_list_activities" => tool_list_activities(state, &arguments).await,
        "folio_search" => tool_search(state, &arguments).await,
        "folio_get_activity" => tool_get_activity(state, &arguments).await,
        "folio_create_activity" => tool_create_activity(state, &arguments).await,
        "folio_generate_bullets" => tool_generate_bullets(state, &arguments).await,
        "folio_generate_digest" => tool_generate_digest(state, &arguments).await,
        "folio_export" => tool_export(state, &arguments).await,
        _ => Err(format!("Unknown tool: {}", tool_name)),
    };

    match result {
        Ok(content) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": content
                }]
            })),
            error: None,
        },
        Err(e) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(McpError {
                code: -32000,
                message: e,
            }),
        },
    }
}

async fn tool_list_activities(state: &McpState, args: &Value) -> Result<String, String> {
    let db = state.db.lock().await;
    let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as u32;

    let activities = db.list_activities(Some(limit)).map_err(|e| e.to_string())?;

    let project_filter = args.get("project").and_then(|p| p.as_str());

    let filtered: Vec<_> = if let Some(project) = project_filter {
        activities
            .into_iter()
            .filter(|a| {
                a.project
                    .as_ref()
                    .map(|p| p.contains(project))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        activities
    };

    let output: Vec<String> = filtered
        .iter()
        .map(|a| {
            format!(
                "- [{}] {} ({})\n  Project: {}\n  Importance: {}",
                &a.id[..8],
                a.title,
                a.timestamp.format("%Y-%m-%d"),
                a.project.as_deref().unwrap_or("None"),
                a.importance
            )
        })
        .collect();

    Ok(format!(
        "Found {} activities:\n\n{}",
        filtered.len(),
        output.join("\n\n")
    ))
}

async fn tool_search(state: &McpState, args: &Value) -> Result<String, String> {
    let query = args
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or("Missing 'query' parameter")?;
    let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;

    let db = state.db.lock().await;
    let activities = db.list_activities(Some(100)).map_err(|e| e.to_string())?;

    let query_lower = query.to_lowercase();
    let results: Vec<_> = activities
        .into_iter()
        .filter(|a| {
            a.title.to_lowercase().contains(&query_lower)
                || a.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || a.project
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
        })
        .take(limit)
        .collect();

    if results.is_empty() {
        return Ok(format!("No activities found matching '{}'", query));
    }

    let output: Vec<String> = results
        .iter()
        .map(|a| format!("- [{}] {}", &a.id[..8], a.title))
        .collect();

    Ok(format!(
        "Found {} results for '{}':\n\n{}",
        results.len(),
        query,
        output.join("\n")
    ))
}

async fn tool_get_activity(state: &McpState, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|i| i.as_str())
        .ok_or("Missing 'id' parameter")?;

    let db = state.db.lock().await;
    let activity = db
        .get_activity(id)
        .map_err(|e| e.to_string())?
        .or_else(|| db.get_activity_by_partial_id(id).ok().flatten())
        .ok_or_else(|| format!("Activity not found: {}", id))?;

    let mut output = format!(
        "Activity: {}\n\nID: {}\nDate: {}\nSource: {}\nImportance: {}\n",
        activity.title,
        activity.id,
        activity.timestamp.format("%Y-%m-%d %H:%M"),
        activity.source,
        activity.importance
    );

    if let Some(project) = &activity.project {
        output.push_str(&format!("Project: {}\n", project));
    }

    if let Some(desc) = &activity.description {
        output.push_str(&format!("\nDescription:\n{}\n", desc));
    }

    if let Some(impact) = activity.metadata.get("impact").and_then(|v| v.as_str()) {
        output.push_str(&format!("\nImpact: {}\n", impact));
    }

    Ok(output)
}

async fn tool_create_activity(state: &McpState, args: &Value) -> Result<String, String> {
    let title = args
        .get("title")
        .and_then(|t| t.as_str())
        .ok_or("Missing 'title' parameter")?;

    let db = state.db.lock().await;

    let importance = args
        .get("importance")
        .and_then(|i| i.as_str())
        .and_then(|i| i.parse().ok())
        .unwrap_or(crate::types::Importance::Medium);

    let mut activity = Activity::new_manual(title).with_importance(importance);

    if let Some(desc) = args.get("description").and_then(|d| d.as_str()) {
        activity = activity.with_description(desc);
    }

    if let Some(impact) = args.get("impact").and_then(|i| i.as_str()) {
        activity = activity.with_impact(impact);
    }

    if let Some(project) = args.get("project").and_then(|p| p.as_str()) {
        activity = activity.with_project(project);
    }

    let id = activity.id.clone();
    db.insert_activity(&activity).map_err(|e| e.to_string())?;

    Ok(format!("Created activity: {} (ID: {})", title, &id[..8]))
}

async fn tool_generate_bullets(state: &McpState, args: &Value) -> Result<String, String> {
    let db = state.db.lock().await;

    let activities = if let Some(ids) = args.get("activity_ids").and_then(|i| i.as_array()) {
        let id_strings: Vec<&str> = ids.iter().filter_map(|i| i.as_str()).collect();
        let mut found = Vec::new();
        for id in id_strings {
            if let Some(a) = db
                .get_activity(id)
                .ok()
                .flatten()
                .or_else(|| db.get_activity_by_partial_id(id).ok().flatten())
            {
                found.push(a);
            }
        }
        found
    } else {
        db.list_activities(Some(10)).map_err(|e| e.to_string())?
    };

    if activities.is_empty() {
        return Ok("No activities found to generate bullets from.".to_string());
    }

    let generator = BulletGenerator::new();
    let mut all_bullets = Vec::new();

    for activity in &activities {
        let bullets = generator.generate_from_activity(activity);
        for bullet in bullets {
            all_bullets.push(format!("- {}", bullet.text));
        }
    }

    Ok(format!(
        "Generated resume bullets:\n\n{}",
        all_bullets.join("\n")
    ))
}

async fn tool_generate_digest(state: &McpState, args: &Value) -> Result<String, String> {
    let db = state.db.lock().await;
    let period = args
        .get("period")
        .and_then(|p| p.as_str())
        .unwrap_or("weekly");

    let activities = db.list_activities(Some(100)).map_err(|e| e.to_string())?;

    let generator = DigestGenerator::new();
    let digest_period = match period {
        "daily" => crate::ai::digest::DigestPeriod::Daily,
        "monthly" => crate::ai::digest::DigestPeriod::Monthly,
        _ => crate::ai::digest::DigestPeriod::Weekly,
    };

    let digest = generator.generate(&activities, digest_period, None);
    let markdown = generator.to_markdown(&digest);

    Ok(markdown)
}

async fn tool_export(state: &McpState, args: &Value) -> Result<String, String> {
    let db = state.db.lock().await;
    let format = args
        .get("format")
        .and_then(|f| f.as_str())
        .unwrap_or("markdown");

    let activities = db.list_activities(None).map_err(|e| e.to_string())?;

    let export_format = match format {
        "json" => ExportFormat::Json,
        "yaml" => ExportFormat::Yaml,
        _ => ExportFormat::Markdown,
    };

    Exporter::export(activities, vec![], export_format).map_err(|e| e.to_string())
}
