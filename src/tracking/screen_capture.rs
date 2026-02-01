//! Screen capture module
//!
//! Captures screenshots and provides hooks for OCR processing.
//! Useful for capturing demos, evidence of work, and visual documentation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Configuration for screen capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureConfig {
    /// Enable screen capture
    pub enabled: bool,
    /// Directory to store screenshots
    pub capture_dir: PathBuf,
    /// Image format (png, jpg)
    pub format: String,
    /// Quality for JPEG (1-100)
    pub quality: u8,
    /// Capture full screen or active window only
    pub capture_mode: CaptureMode,
    /// Enable OCR on captures
    pub enable_ocr: bool,
    /// Redact sensitive patterns
    pub redact_patterns: Vec<String>,
    /// Auto-capture interval (seconds, 0 = disabled)
    pub auto_capture_interval_secs: u32,
}

/// Capture mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaptureMode {
    FullScreen,
    ActiveWindow,
    Region,
}

impl Default for ScreenCaptureConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            enabled: false,
            capture_dir: home.join(".folio").join("captures"),
            format: "png".to_string(),
            quality: 85,
            capture_mode: CaptureMode::ActiveWindow,
            enable_ocr: false,
            redact_patterns: vec![
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(), // emails
                r"\b\d{3}-\d{2}-\d{4}\b".to_string(), // SSN
                r"\b\d{16}\b".to_string(),            // credit card
                r"password\s*[:=]\s*\S+".to_string(), // passwords
                r"api[_-]?key\s*[:=]\s*\S+".to_string(), // API keys
                r"token\s*[:=]\s*\S+".to_string(),    // tokens
            ],
            auto_capture_interval_secs: 0,
        }
    }
}

/// Represents a screen capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCapture {
    pub id: String,
    pub file_path: PathBuf,
    pub captured_at: DateTime<Utc>,
    pub capture_mode: CaptureMode,
    pub window_title: Option<String>,
    pub app_name: Option<String>,
    pub ocr_text: Option<String>,
    pub width: u32,
    pub height: u32,
}

/// Screen capturer
pub struct ScreenCapturer {
    config: ScreenCaptureConfig,
}

impl ScreenCapturer {
    pub fn new(config: ScreenCaptureConfig) -> Result<Self> {
        // Ensure capture directory exists
        std::fs::create_dir_all(&config.capture_dir)
            .context("Failed to create capture directory")?;

        Ok(Self { config })
    }

    /// Capture the screen
    pub fn capture(&self) -> Result<ScreenCapture> {
        let capture_id = uuid::Uuid::new_v4().to_string();
        let file_path = self.config.capture_dir.join(format!(
            "{}.{}",
            capture_id, self.config.format
        ));

        #[cfg(feature = "desktop")]
        {
            use screenshots::Screen;

            let screens = Screen::all().context("Failed to get screens")?;
            let screen = screens.first().context("No screen found")?;

            let image = screen.capture().context("Failed to capture screen")?;
            let width = image.width();
            let height = image.height();

            // Convert to image format and save
            let img = image::RgbaImage::from_raw(width, height, image.into_raw())
                .context("Failed to create image")?;

            img.save(&file_path)
                .context("Failed to save screenshot")?;

            Ok(ScreenCapture {
                id: capture_id,
                file_path,
                captured_at: Utc::now(),
                capture_mode: self.config.capture_mode.clone(),
                window_title: None,
                app_name: None,
                ocr_text: None,
                width,
                height,
            })
        }

        #[cfg(not(feature = "desktop"))]
        {
            // Create a placeholder when desktop features are not enabled
            Ok(ScreenCapture {
                id: capture_id,
                file_path,
                captured_at: Utc::now(),
                capture_mode: self.config.capture_mode.clone(),
                window_title: None,
                app_name: None,
                ocr_text: None,
                width: 0,
                height: 0,
            })
        }
    }

    /// Capture with window context
    pub fn capture_with_context(
        &self,
        window_title: Option<String>,
        app_name: Option<String>,
    ) -> Result<ScreenCapture> {
        let mut capture = self.capture()?;
        capture.window_title = window_title;
        capture.app_name = app_name;
        Ok(capture)
    }

    /// Import an existing image
    pub fn import_image(&self, source_path: &PathBuf) -> Result<ScreenCapture> {
        let capture_id = uuid::Uuid::new_v4().to_string();
        let extension = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let dest_path = self.config.capture_dir.join(format!("{}.{}", capture_id, extension));

        std::fs::copy(source_path, &dest_path)?;

        // Get image dimensions
        let (width, height) = self.get_image_dimensions(&dest_path).unwrap_or((0, 0));

        Ok(ScreenCapture {
            id: capture_id,
            file_path: dest_path,
            captured_at: Utc::now(),
            capture_mode: CaptureMode::FullScreen,
            window_title: None,
            app_name: None,
            ocr_text: None,
            width,
            height,
        })
    }

    /// Get image dimensions
    fn get_image_dimensions(&self, path: &PathBuf) -> Result<(u32, u32)> {
        let img = image::open(path)?;
        Ok((img.width(), img.height()))
    }

    /// Perform OCR on a capture (placeholder - actual OCR would use tesseract or similar)
    pub fn perform_ocr(&self, capture: &mut ScreenCapture) -> Result<String> {
        // OCR would be implemented here using a library like tesseract
        // For now, return a placeholder
        let text = String::new();
        capture.ocr_text = Some(text.clone());
        Ok(text)
    }

    /// Redact sensitive information from OCR text
    pub fn redact_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        for pattern in &self.config.redact_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                result = regex.replace_all(&result, "[REDACTED]").to_string();
            }
        }

        result
    }

    /// Convert a screen capture to an activity
    pub fn capture_to_activity(&self, capture: &ScreenCapture) -> Activity {
        let now = Utc::now();

        let title = match (&capture.window_title, &capture.app_name) {
            (Some(title), Some(app)) => format!("Captured: {} - {}", app, title),
            (Some(title), None) => format!("Captured: {}", title),
            (None, Some(app)) => format!("Captured: {}", app),
            (None, None) => "Screen capture".to_string(),
        };

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::ScreenCapture,
            activity_type: ActivityType::ManualEntry,
            timestamp: capture.captured_at,
            title,
            description: capture.ocr_text.clone(),
            project: None,
            employer: None,
            importance: Importance::Medium,
            metadata: serde_json::json!({
                "capture_id": capture.id,
                "file_path": capture.file_path.to_string_lossy(),
                "width": capture.width,
                "height": capture.height,
                "capture_mode": format!("{:?}", capture.capture_mode),
                "window_title": capture.window_title,
                "app_name": capture.app_name,
                "has_ocr": capture.ocr_text.is_some(),
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// List all captures
    pub fn list_captures(&self) -> Result<Vec<PathBuf>> {
        let mut captures = Vec::new();

        for entry in std::fs::read_dir(&self.config.capture_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
                        captures.push(path);
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        captures.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).ok();
            let b_time = b.metadata().and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        Ok(captures)
    }

    /// Delete a capture
    pub fn delete_capture(&self, capture_id: &str) -> Result<bool> {
        for ext in &["png", "jpg", "jpeg"] {
            let path = self.config.capture_dir.join(format!("{}.{}", capture_id, ext));
            if path.exists() {
                std::fs::remove_file(path)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get total capture storage size
    pub fn storage_size(&self) -> Result<u64> {
        let mut total = 0u64;

        for entry in std::fs::read_dir(&self.config.capture_dir)? {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                }
            }
        }

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_config_default() {
        let config = ScreenCaptureConfig::default();
        assert_eq!(config.format, "png");
        assert_eq!(config.quality, 85);
        assert!(!config.enabled);
    }

    #[test]
    fn test_redact_text() {
        let config = ScreenCaptureConfig::default();
        let capturer = ScreenCapturer::new(config).unwrap();

        let text = "Email: test@example.com and password: secret123";
        let redacted = capturer.redact_text(text);

        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("test@example.com"));
    }
}
