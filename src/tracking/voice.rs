//! Voice note capture module
//!
//! Records audio and provides hooks for transcription.
//! Transcription is done externally (via API or local Whisper).

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Configuration for voice note capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// Enable voice note capture
    pub enabled: bool,
    /// Directory to store audio files
    pub audio_dir: PathBuf,
    /// Sample rate for recording
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
    /// Maximum recording duration in seconds
    pub max_duration_secs: u32,
    /// Audio format (wav, mp3)
    pub format: String,
    /// Transcription provider (openai, local, none)
    pub transcription_provider: String,
    /// OpenAI API key for Whisper API
    pub openai_api_key: Option<String>,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            enabled: false,
            audio_dir: home.join(".folio").join("audio"),
            sample_rate: 16000,
            channels: 1,
            max_duration_secs: 300, // 5 minutes
            format: "wav".to_string(),
            transcription_provider: "none".to_string(),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
        }
    }
}

/// Represents a voice note recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceNote {
    pub id: String,
    pub file_path: PathBuf,
    pub recorded_at: DateTime<Utc>,
    pub duration_secs: f32,
    pub transcript: Option<String>,
    pub transcription_status: TranscriptionStatus,
}

/// Status of transcription
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TranscriptionStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    Skipped,
}

/// Voice note capture handler
pub struct VoiceNoteCapture {
    config: VoiceConfig,
    is_recording: Arc<AtomicBool>,
}

impl VoiceNoteCapture {
    pub fn new(config: VoiceConfig) -> Result<Self> {
        // Ensure audio directory exists
        std::fs::create_dir_all(&config.audio_dir)
            .context("Failed to create audio directory")?;

        Ok(Self {
            config,
            is_recording: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    /// Start recording a voice note
    #[cfg(feature = "audio")]
    pub fn start_recording(&self) -> Result<RecordingHandle> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use hound::{WavSpec, WavWriter};
        use std::sync::Mutex;

        if self.is_recording.swap(true, Ordering::SeqCst) {
            anyhow::bail!("Already recording");
        }

        let note_id = uuid::Uuid::new_v4().to_string();
        let file_path = self
            .config
            .audio_dir
            .join(format!("{}.wav", note_id));

        let spec = WavSpec {
            channels: self.config.channels,
            sample_rate: self.config.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = WavWriter::create(&file_path, spec)?;
        let writer = Arc::new(Mutex::new(Some(writer)));
        let writer_clone = writer.clone();

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device available")?;

        let config = cpal::StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let is_recording = self.is_recording.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut guard) = writer_clone.lock() {
                    if let Some(ref mut writer) = *guard {
                        for &sample in data {
                            let sample_i16 = (sample * i16::MAX as f32) as i16;
                            let _ = writer.write_sample(sample_i16);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;

        Ok(RecordingHandle {
            note_id,
            file_path,
            start_time: Utc::now(),
            stream: Some(stream),
            writer,
            is_recording,
        })
    }

    /// Start recording (stub for when audio feature is disabled)
    #[cfg(not(feature = "audio"))]
    pub fn start_recording(&self) -> Result<RecordingHandle> {
        if self.is_recording.swap(true, Ordering::SeqCst) {
            anyhow::bail!("Already recording");
        }

        let note_id = uuid::Uuid::new_v4().to_string();
        let file_path = self.config.audio_dir.join(format!("{}.wav", note_id));

        Ok(RecordingHandle {
            note_id,
            file_path,
            start_time: Utc::now(),
            is_recording: self.is_recording.clone(),
        })
    }

    /// Import an existing audio file
    pub fn import_audio(&self, source_path: &PathBuf) -> Result<VoiceNote> {
        let note_id = uuid::Uuid::new_v4().to_string();
        let extension = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");
        let dest_path = self.config.audio_dir.join(format!("{}.{}", note_id, extension));

        std::fs::copy(source_path, &dest_path)?;

        // Try to get duration (basic WAV support)
        let duration_secs = self.get_audio_duration(&dest_path).unwrap_or(0.0);

        Ok(VoiceNote {
            id: note_id,
            file_path: dest_path,
            recorded_at: Utc::now(),
            duration_secs,
            transcript: None,
            transcription_status: TranscriptionStatus::Pending,
        })
    }

    /// Get audio duration from file
    fn get_audio_duration(&self, path: &PathBuf) -> Result<f32> {
        let reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        let duration = reader.duration() as f32 / spec.sample_rate as f32;
        Ok(duration)
    }

    /// Transcribe using OpenAI Whisper API
    pub async fn transcribe_openai(&self, note: &mut VoiceNote) -> Result<String> {
        let api_key = self
            .config
            .openai_api_key
            .as_ref()
            .context("OpenAI API key not configured")?;

        note.transcription_status = TranscriptionStatus::InProgress;

        let client = reqwest::Client::new();
        let file_bytes = std::fs::read(&note.file_path)?;

        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(file_bytes)
                    .file_name(note.file_path.file_name().unwrap().to_string_lossy().to_string())
                    .mime_str("audio/wav")?,
            )
            .text("model", "whisper-1");

        let response = client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: String = response.text().await?;
            note.transcription_status = TranscriptionStatus::Failed(error.clone());
            anyhow::bail!("Transcription failed: {}", error);
        }

        let result: serde_json::Value = response.json().await?;
        let transcript = result["text"]
            .as_str()
            .context("No transcript in response")?
            .to_string();

        note.transcript = Some(transcript.clone());
        note.transcription_status = TranscriptionStatus::Completed;

        Ok(transcript)
    }

    /// Convert a voice note to an activity
    pub fn note_to_activity(&self, note: &VoiceNote) -> Activity {
        let now = Utc::now();

        let title = note
            .transcript
            .as_ref()
            .map(|t| truncate_text(t, 100))
            .unwrap_or_else(|| format!("Voice note ({}s)", note.duration_secs as i32));

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::VoiceNote,
            activity_type: ActivityType::VoiceNote,
            timestamp: note.recorded_at,
            title,
            description: note.transcript.clone(),
            project: None,
            employer: None,
            importance: Importance::Medium,
            metadata: serde_json::json!({
                "voice_note_id": note.id,
                "file_path": note.file_path,
                "duration_secs": note.duration_secs,
                "transcription_status": format!("{:?}", note.transcription_status),
            }),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Handle for an active recording
pub struct RecordingHandle {
    pub note_id: String,
    pub file_path: PathBuf,
    pub start_time: DateTime<Utc>,
    #[cfg(feature = "audio")]
    stream: Option<cpal::Stream>,
    #[cfg(feature = "audio")]
    writer: Arc<std::sync::Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>>,
    is_recording: Arc<AtomicBool>,
}

impl RecordingHandle {
    /// Stop recording and return the voice note
    #[allow(unused_mut)]
    pub fn stop(mut self) -> Result<VoiceNote> {
        self.is_recording.store(false, Ordering::SeqCst);

        #[cfg(feature = "audio")]
        {
            // Drop stream to stop recording
            self.stream.take();

            // Finalize the WAV file
            if let Ok(mut guard) = self.writer.lock() {
                if let Some(writer) = guard.take() {
                    writer.finalize()?;
                }
            }
        }

        let duration_secs = (Utc::now() - self.start_time).num_seconds() as f32;

        Ok(VoiceNote {
            id: self.note_id,
            file_path: self.file_path,
            recorded_at: self.start_time,
            duration_secs,
            transcript: None,
            transcription_status: TranscriptionStatus::Pending,
        })
    }

    /// Get elapsed recording time
    pub fn elapsed_secs(&self) -> f32 {
        (Utc::now() - self.start_time).num_seconds() as f32
    }
}

/// Truncate text to a maximum length
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_config_default() {
        let config = VoiceConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
        assert_eq!(config.format, "wav");
    }

    #[test]
    fn test_transcription_status() {
        let status = TranscriptionStatus::Pending;
        assert_eq!(status, TranscriptionStatus::Pending);
    }
}
