# Activity Tracking Feature Research

Research into how Rize.io, Loom, and Otter AI capture and process user activity, with potential applications for Folio.

---

## Executive Summary

| Tool | Core Capability | Key Technique | Folio Opportunity |
|------|----------------|---------------|-------------------|
| **Rize.io** | Time/activity tracking | Active window detection | Context-aware accomplishment capture |
| **Loom** | Video recording + transcription | Screen capture + ASR | Demo/presentation evidence gathering |
| **Otter AI** | Meeting transcription | Real-time ASR + speaker diarization | Meeting accomplishment extraction |

---

## 1. Rize.io - Active Window Tracking

### What They Do
- **Continuous window monitoring** - Polls active window every few seconds
- **Metadata-only capture** - App name, window title, URL, timestamps (no screenshots/content)
- **AI categorization** - Auto-categorizes time into work categories
- **Idle detection** - Stops tracking after 2 minutes of inactivity
- **Privacy controls** - URL tracking toggle, app exclusions, data redaction

### Technical Implementation
```rust
// Rust crate: active-win-pos-rs
use active_win_pos_rs::get_active_window;

let window = get_active_window()?;
// Returns: title, app_name, process_path, process_id, window_id, position
```

**Platform APIs:**
- macOS: Accessibility API + Quartz Window Services (requires Screen Recording permission)
- Windows: Win32 API window enumeration
- Linux: X11 only (Wayland not supported for security reasons)

### Potential Folio Features

| Feature | Description | Effort |
|---------|-------------|--------|
| `ActivitySource::ActiveWindow` | New source capturing window metadata | Medium |
| Time-in-context tracking | Know how long you worked before a commit | Medium |
| Project auto-detection | Infer project from window title patterns | Low |
| IDE session tracking | Track VS Code/JetBrains work sessions | Low |
| Browser research tracking | Capture Stack Overflow, docs, GitHub browsing | Medium |
| Idle-aware activity merging | Group contiguous work into sessions | Low |

### Privacy Considerations
- Metadata-only (no screenshots) aligns with Folio's privacy-first approach
- User-configurable app/URL exclusions
- All data stays local

---

## 2. Loom - Video Recording + Transcription

### What They Do
- **Screen + camera recording** - Captures demos, presentations, walkthroughs
- **Automatic transcription** - Speech-to-text using ASR (50+ languages)
- **AI summaries** - Auto-generates titles, chapters, action items
- **Searchable content** - Full-text search across video transcripts
- **Closed captions** - Auto-generated from transcripts

### Technical Components
1. **Screen capture** - Native screen recording APIs
2. **Audio capture** - System audio + microphone
3. **ASR (Automatic Speech Recognition)** - Converts speech to text
4. **NLP processing** - Extracts summaries, action items, chapters

### Potential Folio Features

| Feature | Description | Effort |
|---------|-------------|--------|
| Demo capture integration | Record demo of work, extract transcript as evidence | High |
| Voice note accomplishments | Quick voice memos transcribed to activities | Medium |
| Presentation tracking | Capture when you presented/demoed work | Medium |
| Transcript search | Search across captured audio/video | High |
| Meeting summary import | Import Loom AI summaries as activities | Low |

### Technical Options for Transcription
- **Cloud APIs**: OpenAI Whisper API, Google Speech-to-Text, AWS Transcribe
- **Local/offline**: `whisper.cpp` (Rust bindings available), Vosk
- **Existing integrations**: Import from Loom, Otter, etc. via API

```rust
// Potential Whisper integration
// Crate: whisper-rs
let ctx = WhisperContext::new("ggml-base.en.bin")?;
let transcript = ctx.full(audio_data)?;
```

---

## 3. Otter AI - Meeting Transcription

### What They Do
- **Real-time transcription** - Live speech-to-text during meetings
- **Auto-join meetings** - OtterPilot bot joins Zoom/Meet/Teams automatically
- **Speaker identification** - Diarization to tag who said what
- **Custom vocabulary** - Train on jargon, names, acronyms
- **Action item extraction** - AI identifies tasks with deadlines
- **Slide capture** - Auto-captures presented slides
- **AI chat** - Query your meeting history

### Technical Components
1. **Meeting bot** - Joins video calls as a participant
2. **Real-time ASR** - Streaming transcription
3. **Speaker diarization** - Identifies different speakers
4. **NLP extraction** - Action items, summaries, key points
5. **Calendar integration** - Auto-schedules recording

### Potential Folio Features

| Feature | Description | Effort |
|---------|-------------|--------|
| Meeting accomplishment extraction | Parse meeting transcripts for your contributions | Medium |
| Action item → Activity | Convert assigned tasks to tracked activities | Medium |
| Calendar integration | Track meetings as activities automatically | Low |
| Transcript import | Import from Otter/Zoom/Teams | Low |
| Speaker contribution analysis | Highlight your speaking time/contributions | High |
| Decision tracking | Extract decisions made in meetings | High |

### Integration Approaches
1. **Direct API integration** - Otter API, Zoom API, etc.
2. **File import** - Accept transcript files (.vtt, .srt, .txt)
3. **Calendar sync** - Google Calendar, Outlook integration

---

## Consolidated Feature Roadmap

### Phase 1: Foundation (Low Effort, High Value)

| # | Feature | Source Inspiration | Implementation |
|---|---------|-------------------|----------------|
| 1 | **Active window tracking** | Rize | Add `active-win-pos-rs` crate, new ActivitySource |
| 2 | **Idle detection** | Rize | Timer in watcher module |
| 3 | **Calendar integration** | Otter | Google Calendar API for meeting activities |
| 4 | **Transcript file import** | Loom/Otter | Parse .vtt/.srt files into activities |
| 5 | **Project auto-detection** | Rize | Pattern match window titles to projects |

### Phase 2: Intelligence (Medium Effort)

| # | Feature | Source Inspiration | Implementation |
|---|---------|-------------------|----------------|
| 6 | **Work session grouping** | Rize | Cluster activities by time + context |
| 7 | **Voice note capture** | Loom | Record audio, transcribe with Whisper |
| 8 | **Meeting summary import** | Otter | API integration or structured paste |
| 9 | **Browser context capture** | Rize | Track docs/SO/GitHub as research activities |
| 10 | **Action item extraction** | Otter | NLP on meeting transcripts |

### Phase 3: Advanced (High Effort)

| # | Feature | Source Inspiration | Implementation |
|---|---------|-------------------|----------------|
| 11 | **Screen recording + OCR** | Loom | Capture demos, extract text |
| 12 | **Real-time transcription** | Otter | Local Whisper streaming |
| 13 | **Speaker diarization** | Otter | Identify your voice in meetings |
| 14 | **AI meeting analysis** | Otter | Query meeting history with LLM |
| 15 | **Cross-source correlation** | All | Link commits to meetings to time tracked |

---

## Technical Dependencies

### Rust Crates to Consider

```toml
# Active window detection (Rize-like)
active-win-pos-rs = "0.9"

# Audio/transcription (Loom/Otter-like)
whisper-rs = "0.11"           # Local Whisper bindings
cpal = "0.15"                  # Cross-platform audio capture

# Calendar integration
google-calendar3 = "5.0"       # Google Calendar API

# Transcript parsing
webvtt = "0.1"                 # Parse .vtt subtitle files
```

### Permissions Required (macOS)

| Feature | Permission | User Impact |
|---------|------------|-------------|
| Window title capture | Screen Recording | Must grant in System Preferences |
| Browser URL capture | Accessibility | Must grant in System Preferences |
| Audio recording | Microphone | Standard permission prompt |
| Screen recording | Screen Recording | Standard permission prompt |

---

## Privacy-First Design Principles

Following Folio's existing philosophy:

1. **All data local** - No cloud sync, user owns everything
2. **Opt-in features** - Each tracking source explicitly enabled
3. **Metadata preference** - Capture context, not content when possible
4. **User review** - Activities can be reviewed before promotion
5. **Easy deletion** - Simple data redaction and cleanup
6. **Transparent storage** - SQLite file user can inspect/backup

---

## Sources

- [Rize.io - Automatic Time Tracking](https://rize.io/)
- [Rize Tracking Documentation](https://docs.rize.io/automatic-tracking/tracking-overview)
- [active-win-pos-rs Crate](https://github.com/dimusic/active-win-pos-rs)
- [Loom Transcription Support](https://support.atlassian.com/loom/docs/loom-video-transcription-and-closed-captions/)
- [Loom AI Video Transcription](https://www.loom.com/blog/ai-video-transcription)
- [Otter.ai Overview](https://otter.ai/)
- [Otter Quick Start Guide](https://help.otter.ai/hc/en-us/articles/360049722894-Otter-Quick-Start-Guide)
- [Otter Notetaker Overview](https://help.otter.ai/hc/en-us/articles/4425393298327-Otter-Notetaker-Overview)
- [Zapier - What is Otter.ai](https://zapier.com/blog/otter-ai/)
