//! Activity tracking modules for enhanced capture capabilities
//!
//! This module provides various tracking mechanisms inspired by tools like:
//! - Rize.io (active window tracking, idle detection)
//! - Loom (screen capture, transcription)
//! - Otter AI (meeting transcription, action item extraction)

pub mod active_window;
pub mod browser;
pub mod calendar;
pub mod idle;
pub mod meeting;
pub mod project_detection;
pub mod screen_capture;
pub mod session;
pub mod transcript;
pub mod voice;

pub use active_window::ActiveWindowTracker;
pub use calendar::CalendarIntegration;
pub use meeting::MeetingAnalyzer;
pub use screen_capture::ScreenCapturer;
pub use transcript::TranscriptImporter;
