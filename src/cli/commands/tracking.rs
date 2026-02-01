//! Tracking-related CLI commands

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use std::path::PathBuf;

use crate::config::Config;
use crate::db::Database;
use crate::tracking::{CalendarIntegration, MeetingAnalyzer, ScreenCapturer, TranscriptImporter};

/// Import a transcript file
pub fn import_transcript(
    file: PathBuf,
    title: Option<String>,
    date: Option<String>,
    extract_actions: bool,
) -> Result<()> {
    let config = Config::load()?;
    let db = Database::open()?;

    let content = std::fs::read_to_string(&file)?;
    let extension = file.extension().and_then(|e| e.to_str()).unwrap_or("");

    let importer = TranscriptImporter::new(config.transcript);

    let mut transcript = match extension.to_lowercase().as_str() {
        "vtt" => importer.import_vtt(&content)?,
        "srt" => importer.import_srt(&content)?,
        _ => importer.import_plain_text(&content)?,
    };

    transcript.source_file = Some(file.to_string_lossy().to_string());
    transcript.title = title.or_else(|| file.file_stem().map(|s| s.to_string_lossy().to_string()));

    let meeting_date = date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(12, 0, 0).unwrap())
        .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);

    // Create activity from transcript
    let activity = importer.transcript_to_activity(&transcript, meeting_date);
    db.insert_activity(&activity)?;

    println!(
        "Imported transcript: {}",
        transcript.title.as_deref().unwrap_or("Untitled")
    );
    println!("  Segments: {}", transcript.segments.len());
    println!(
        "  Duration: {} minutes",
        transcript.total_duration.num_minutes()
    );

    // Extract and import action items if requested
    if extract_actions {
        let action_items = importer.extract_action_items(&transcript);
        let activities = importer.action_items_to_activities(&action_items, meeting_date);

        for activity in &activities {
            db.insert_activity(activity)?;
        }

        println!("  Action items extracted: {}", action_items.len());
    }

    println!("Activity ID: {}", activity.id);
    Ok(())
}

/// Import a meeting summary
pub fn import_meeting(file: Option<PathBuf>, title: Option<String>, source: String) -> Result<()> {
    let config = Config::load()?;
    let db = Database::open()?;

    let analyzer = MeetingAnalyzer::new(config.meeting);

    let summary = match (file, source.as_str()) {
        (Some(path), "otter") => {
            let content = std::fs::read_to_string(&path)?;
            analyzer.parse_otter_export(&content)?
        }
        (Some(path), "loom") => {
            let content = std::fs::read_to_string(&path)?;
            analyzer.parse_loom_export(&content)?
        }
        (Some(path), _) => {
            let content = std::fs::read_to_string(&path)?;
            let title = title.unwrap_or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Meeting".to_string())
            });
            analyzer.parse_manual_summary(&content, &title)
        }
        (None, _) => {
            let title = title.unwrap_or_else(|| "Meeting".to_string());
            println!("Enter meeting summary (Ctrl+D to finish):");
            let mut content = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut content)?;
            analyzer.parse_manual_summary(&content, &title)
        }
    };

    // Create activity from meeting summary
    let activity = analyzer.summary_to_activity(&summary);
    db.insert_activity(&activity)?;

    println!("Imported meeting: {}", summary.title);
    println!("  Duration: {} minutes", summary.duration_mins);
    println!("  Attendees: {}", summary.attendees.len());
    println!("  Decisions: {}", summary.decisions.len());
    println!("  Action items: {}", summary.action_items.len());

    // Import action items as separate activities
    let action_activities = analyzer.action_items_to_activities(&summary);
    for action_activity in &action_activities {
        db.insert_activity(action_activity)?;
    }

    if !action_activities.is_empty() {
        println!("  Your action items imported: {}", action_activities.len());
    }

    println!("Activity ID: {}", activity.id);
    Ok(())
}

/// Import calendar events
pub fn import_calendar(file: PathBuf, days: Option<u32>) -> Result<()> {
    let config = Config::load()?;
    let db = Database::open()?;

    let content = std::fs::read_to_string(&file)?;
    let integration = CalendarIntegration::new(config.calendar);

    let events = integration.import_from_ics(&content)?;
    let events = integration.filter_events(events);

    // Filter by days if specified
    let events: Vec<_> = if let Some(days_back) = days {
        let cutoff = Utc::now() - chrono::Duration::days(days_back as i64);
        events
            .into_iter()
            .filter(|e| e.start_time >= cutoff)
            .collect()
    } else {
        events
    };

    let mut imported = 0;
    for event in &events {
        let activity = integration.event_to_activity(event);
        db.insert_activity(&activity)?;
        imported += 1;
    }

    println!("Imported {} calendar events", imported);
    Ok(())
}

/// Record a voice note
pub fn voice(_duration: Option<u32>, transcribe: bool) -> Result<()> {
    let config = Config::load()?;

    println!("Voice note recording is not yet fully implemented.");
    println!("To use this feature, you would need audio recording capabilities.");
    println!();
    println!("Current configuration:");
    println!("  Audio directory: {:?}", config.voice.audio_dir);
    println!(
        "  Transcription provider: {}",
        config.voice.transcription_provider
    );
    println!("  Max duration: {}s", config.voice.max_duration_secs);

    if transcribe {
        if config.voice.openai_api_key.is_some() {
            println!("  OpenAI API key: configured");
        } else {
            println!("  OpenAI API key: not configured");
            println!("  Set OPENAI_API_KEY environment variable to enable transcription");
        }
    }

    Ok(())
}

/// Capture the screen
pub fn screen_capture(title: Option<String>) -> Result<()> {
    let config = Config::load()?;
    let db = Database::open()?;

    let capturer = ScreenCapturer::new(config.screen_capture)?;
    let capture = capturer.capture()?;

    println!("Captured screen:");
    println!("  File: {:?}", capture.file_path);
    println!("  Size: {}x{}", capture.width, capture.height);

    let mut activity = capturer.capture_to_activity(&capture);
    if let Some(title) = title {
        activity.title = title;
    }

    db.insert_activity(&activity)?;
    println!("Activity ID: {}", activity.id);

    Ok(())
}

/// Activity tracking command
pub fn track(foreground: bool, stop: bool, status: bool) -> Result<()> {
    let config = Config::load()?;

    if status {
        println!("Activity Tracking Status:");
        println!(
            "  Active window tracking: {}",
            if config.active_window.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
        println!(
            "  Browser tracking: {}",
            if config.browser.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
        println!(
            "  Poll interval: {}s",
            config.active_window.poll_interval_secs
        );
        return Ok(());
    }

    if stop {
        println!("Stopping activity tracking...");
        // In a real implementation, this would signal the daemon to stop
        println!("Activity tracking stopped.");
        return Ok(());
    }

    if !config.active_window.enabled {
        println!("Active window tracking is disabled.");
        println!("Enable it with: folio config active_window.enabled true");
        return Ok(());
    }

    if foreground {
        println!("Starting activity tracking in foreground...");
        println!("Press Ctrl+C to stop.");
        println!();

        // Simple foreground tracking loop
        use crate::tracking::ActiveWindowTracker;

        let mut tracker = ActiveWindowTracker::new(config.active_window.clone());
        let db = Database::open()?;

        loop {
            if let Ok(Some(session)) = tracker.poll() {
                let activity = tracker.session_to_activity(&session);
                if let Err(e) = db.insert_activity(&activity) {
                    eprintln!("Failed to save activity: {}", e);
                } else {
                    println!(
                        "Tracked: {} - {} ({} secs)",
                        session.app_name, session.category, session.duration_secs
                    );
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(
                config.active_window.poll_interval_secs,
            ));
        }
    } else {
        println!("Starting activity tracking daemon...");
        println!("Use 'folio track --status' to check status.");
        println!("Use 'folio track --stop' to stop tracking.");
        // In a real implementation, this would fork a daemon process
        println!("Daemon mode not yet implemented. Use --foreground for now.");
    }

    Ok(())
}
