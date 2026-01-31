use anyhow::Result;
use serde_json;

use super::ExportData;
use crate::types::Activity;

/// Export data to JSON format
pub fn export(data: &ExportData) -> Result<String> {
    serde_json::to_string_pretty(data)
        .map_err(|e| anyhow::anyhow!("Failed to serialize to JSON: {}", e))
}

/// Export data to compact JSON (single line)
pub fn export_compact(data: &ExportData) -> Result<String> {
    serde_json::to_string(data).map_err(|e| anyhow::anyhow!("Failed to serialize to JSON: {}", e))
}

/// Export only activities to JSON
pub fn export_activities(activities: &[Activity]) -> Result<String> {
    serde_json::to_string_pretty(activities)
        .map_err(|e| anyhow::anyhow!("Failed to serialize activities to JSON: {}", e))
}

/// Import activities from JSON
pub fn import(content: &str) -> Result<Vec<Activity>> {
    // Try parsing as ExportData first
    if let Ok(data) = serde_json::from_str::<ExportData>(content) {
        return Ok(data.activities);
    }

    // Try parsing as array of activities
    serde_json::from_str::<Vec<Activity>>(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Activity, ActivitySource, ActivityType, Importance};
    use chrono::Utc;

    #[test]
    fn test_export_import_roundtrip() {
        let activities = vec![
            Activity::new_manual("Test activity 1"),
            Activity::new_manual("Test activity 2").with_project("Project A"),
        ];

        let json = export_activities(&activities).unwrap();
        let imported = import(&json).unwrap();

        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].title, "Test activity 1");
        assert_eq!(imported[1].project, Some("Project A".to_string()));
    }
}
