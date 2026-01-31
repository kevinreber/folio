use anyhow::Result;

use super::ExportData;
use crate::types::Activity;

/// Export data to YAML format
pub fn export(data: &ExportData) -> Result<String> {
    serde_yaml::to_string(data)
        .map_err(|e| anyhow::anyhow!("Failed to serialize to YAML: {}", e))
}

/// Export only activities to YAML
pub fn export_activities(activities: &[Activity]) -> Result<String> {
    serde_yaml::to_string(activities)
        .map_err(|e| anyhow::anyhow!("Failed to serialize activities to YAML: {}", e))
}

/// Import activities from YAML
pub fn import(content: &str) -> Result<Vec<Activity>> {
    // Try parsing as ExportData first
    if let Ok(data) = serde_yaml::from_str::<ExportData>(content) {
        return Ok(data.activities);
    }

    // Try parsing as array of activities
    serde_yaml::from_str::<Vec<Activity>>(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse YAML: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_roundtrip() {
        let activities = vec![
            Activity::new_manual("Test activity 1"),
            Activity::new_manual("Test activity 2").with_project("Project A"),
        ];

        let yaml = export_activities(&activities).unwrap();
        let imported = import(&yaml).unwrap();

        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].title, "Test activity 1");
    }
}
