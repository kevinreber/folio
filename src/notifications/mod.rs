use anyhow::Result;
use notify_rust::{Notification, Timeout};

use crate::ai::bullets::{BulletGenerator, GeneratedBullet};
use crate::types::{Activity, Importance};

/// Notification types for different Folio events
#[derive(Debug, Clone)]
pub enum FolioNotification {
    /// New activity was captured
    ActivityCaptured {
        activity: Activity,
    },
    /// Resume was updated with new bullet points
    ResumeUpdated {
        activity_title: String,
        bullets: Vec<GeneratedBullet>,
    },
    /// Multiple activities were captured
    BatchActivitiesCaptured {
        count: usize,
        significant_count: usize,
    },
    /// Reminder to capture work
    CaptureReminder {
        days_since_last: u32,
    },
}

/// Desktop notification manager for Folio
pub struct NotificationManager {
    enabled: bool,
    resume_notifications: bool,
    min_importance: Importance,
    bullet_generator: BulletGenerator,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            enabled: true,
            resume_notifications: true,
            min_importance: Importance::Medium,
            bullet_generator: BulletGenerator::new(),
        }
    }

    /// Create a notification manager with custom settings
    pub fn with_settings(
        enabled: bool,
        resume_notifications: bool,
        min_importance: Importance,
    ) -> Self {
        Self {
            enabled,
            resume_notifications,
            min_importance,
            bullet_generator: BulletGenerator::new(),
        }
    }

    /// Check if an activity is significant enough to notify about
    pub fn is_significant(&self, activity: &Activity) -> bool {
        matches!(
            (&activity.importance, &self.min_importance),
            (Importance::High, _)
                | (Importance::Medium, Importance::Medium | Importance::Low)
                | (Importance::Low, Importance::Low)
        )
    }

    /// Send a desktop notification
    pub fn send(&self, notification: FolioNotification) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        match notification {
            FolioNotification::ActivityCaptured { activity } => {
                self.notify_activity_captured(&activity)?;
            }
            FolioNotification::ResumeUpdated {
                activity_title,
                bullets,
            } => {
                self.notify_resume_updated(&activity_title, &bullets)?;
            }
            FolioNotification::BatchActivitiesCaptured {
                count,
                significant_count,
            } => {
                self.notify_batch_captured(count, significant_count)?;
            }
            FolioNotification::CaptureReminder { days_since_last } => {
                self.notify_capture_reminder(days_since_last)?;
            }
        }

        Ok(())
    }

    /// Process a new activity and send appropriate notifications
    /// Returns generated bullets if resume was updated
    pub fn process_activity(&self, activity: &Activity) -> Result<Option<Vec<GeneratedBullet>>> {
        if !self.enabled {
            return Ok(None);
        }

        // Only process significant activities for resume updates
        if !self.is_significant(activity) {
            return Ok(None);
        }

        // Generate resume bullets for significant activities
        let bullets = self.bullet_generator.generate_from_activity(activity);

        if !bullets.is_empty() && self.resume_notifications {
            self.send(FolioNotification::ResumeUpdated {
                activity_title: activity.title.clone(),
                bullets: bullets.clone(),
            })?;
        }

        Ok(Some(bullets))
    }

    fn notify_activity_captured(&self, activity: &Activity) -> Result<()> {
        let importance_emoji = match activity.importance {
            Importance::High => "🔥",
            Importance::Medium => "📝",
            Importance::Low => "📋",
        };

        let body = if let Some(project) = &activity.project {
            format!("{} {}\nProject: {}", importance_emoji, activity.title, project)
        } else {
            format!("{} {}", importance_emoji, activity.title)
        };

        Notification::new()
            .appname("Folio")
            .summary("Activity Captured")
            .body(&body)
            .icon("document-new")
            .timeout(Timeout::Milliseconds(5000))
            .show()?;

        Ok(())
    }

    fn notify_resume_updated(&self, activity_title: &str, bullets: &[GeneratedBullet]) -> Result<()> {
        // Get the best bullet (highest confidence)
        let best_bullet = bullets
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .map(|b| b.text.clone())
            .unwrap_or_else(|| activity_title.to_string());

        // Truncate if too long for notification
        let display_bullet = if best_bullet.len() > 100 {
            format!("{}...", &best_bullet[..97])
        } else {
            best_bullet
        };

        let body = format!(
            "🎉 New resume bullet generated!\n\n\"{}\"\n\n→ View in Folio to edit or customize",
            display_bullet
        );

        Notification::new()
            .appname("Folio")
            .summary("Resume Updated")
            .body(&body)
            .icon("document-save")
            .timeout(Timeout::Milliseconds(8000))
            .show()?;

        Ok(())
    }

    fn notify_batch_captured(&self, count: usize, significant_count: usize) -> Result<()> {
        let body = if significant_count > 0 {
            format!(
                "Captured {} activities ({} significant)\n→ Run 'folio list' to review",
                count, significant_count
            )
        } else {
            format!(
                "Captured {} activities\n→ Run 'folio list' to review",
                count
            )
        };

        Notification::new()
            .appname("Folio")
            .summary("Activities Synced")
            .body(&body)
            .icon("view-refresh")
            .timeout(Timeout::Milliseconds(5000))
            .show()?;

        Ok(())
    }

    fn notify_capture_reminder(&self, days_since_last: u32) -> Result<()> {
        let body = format!(
            "It's been {} days since your last entry.\n→ Run 'folio capture' to log your work",
            days_since_last
        );

        Notification::new()
            .appname("Folio")
            .summary("Time to Capture Your Work!")
            .body(&body)
            .icon("appointment-reminder")
            .timeout(Timeout::Milliseconds(10000))
            .show()?;

        Ok(())
    }
}

/// Console-based notification fallback (for environments without desktop notifications)
pub struct ConsoleNotifier;

impl ConsoleNotifier {
    /// Print a notification to the console with formatting
    pub fn notify(notification: &FolioNotification) {
        match notification {
            FolioNotification::ActivityCaptured { activity } => {
                println!("\n📝 New activity captured!");
                println!("   Title: {}", activity.title);
                if let Some(project) = &activity.project {
                    println!("   Project: {}", project);
                }
                println!("   Importance: {}", activity.importance);
                println!();
            }
            FolioNotification::ResumeUpdated {
                activity_title,
                bullets,
            } => {
                println!("\n🎉 Resume updated!");
                println!("   Based on: {}", activity_title);
                println!("   Generated {} bullet points:", bullets.len());
                for bullet in bullets.iter().take(2) {
                    println!("   • {}", bullet.text);
                }
                if bullets.len() > 2 {
                    println!("   ... and {} more", bullets.len() - 2);
                }
                println!();
            }
            FolioNotification::BatchActivitiesCaptured {
                count,
                significant_count,
            } => {
                println!("\n📊 Batch sync complete!");
                println!("   Captured: {} activities", count);
                println!("   Significant: {}", significant_count);
                println!();
            }
            FolioNotification::CaptureReminder { days_since_last } => {
                println!("\n⏰ Reminder: {} days since last capture", days_since_last);
                println!("   Run 'folio capture' to log your work");
                println!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_significance_check() {
        let manager = NotificationManager::with_settings(true, true, Importance::Medium);

        let high_activity = Activity::new_manual("Test").with_importance(Importance::High);
        let medium_activity = Activity::new_manual("Test").with_importance(Importance::Medium);
        let low_activity = Activity::new_manual("Test").with_importance(Importance::Low);

        assert!(manager.is_significant(&high_activity));
        assert!(manager.is_significant(&medium_activity));
        assert!(!manager.is_significant(&low_activity));
    }

    #[test]
    fn test_low_threshold_significance() {
        let manager = NotificationManager::with_settings(true, true, Importance::Low);

        let low_activity = Activity::new_manual("Test").with_importance(Importance::Low);
        assert!(manager.is_significant(&low_activity));
    }
}
