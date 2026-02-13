# Decision: UI Approach for Non-Technical Users

**Date:** 2026-02-13
**Status:** Decided
**Decision:** Embedded Web UI served by existing Axum server (Option 1)

## Context

Folio is a local-first career accomplishment tracker primarily accessed through the CLI and TUI. We needed a graphical UI so non-technical users can see and track their activities without using the terminal.

## Options Considered

### Option 1: Embedded Web UI via Axum (Chosen)

Build a lightweight SPA (HTML/CSS/JavaScript) and embed it into the existing Axum REST API server using `rust-embed`. Users run `folio serve --open` and the dashboard opens in their default browser.

**Pros:**
- Zero new runtime dependencies for users (no Node.js, no Chromium, no separate process)
- Reuses the existing REST API server and endpoints (`/api/activities`, `/api/stats`, etc.)
- Assets are compiled into the binary via `rust-embed`, so there's nothing extra to install or distribute
- Works on any OS that has a web browser
- Minimal binary size overhead (~50KB for the embedded HTML/CSS/JS)
- The docs site already uses React/TypeScript, so the team knows web technologies
- Easy to iterate on — just edit files in `web-ui/` and rebuild
- No new build toolchain (no npm/webpack/vite in the build pipeline)

**Cons:**
- No native OS integration (system tray, global hotkeys, desktop notifications from the UI)
- Requires the user to keep a terminal open running `folio serve`
- Browser tab, not a standalone window

### Option 2: Tauri Desktop Application

Use Tauri to create a native desktop app. Tauri uses the system's native webview (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux) instead of bundling Chromium.

**Pros:**
- Native desktop window with system tray support
- Small binary size (~5-10MB) compared to Electron
- Can call Rust backend code directly without HTTP, since the backend is already Rust
- Auto-update mechanisms built in
- Native menu bar, global shortcuts, file drag-and-drop

**Cons:**
- Adds significant build complexity (Tauri CLI, system webview dependencies)
- WebView behavior varies across platforms (especially Linux where WebKitGTK version availability differs)
- Requires maintaining a separate Tauri project configuration
- Users need to install a separate application binary
- CI/CD needs platform-specific build pipelines for each OS
- Heavier dependency tree in the Rust project

### Option 3: Electron Desktop Application

Bundle a full Chromium browser with a Node.js backend in a standalone desktop application.

**Pros:**
- Consistent rendering across all platforms (ships its own Chromium)
- Massive ecosystem of npm packages and UI component libraries
- Deep OS integration (system tray, notifications, global shortcuts, auto-updater)
- Most mature desktop web app framework

**Cons:**
- Bundles entire Chromium browser (~150-200MB download, ~500MB on disk)
- High memory usage (~100-300MB RAM just for the shell)
- Requires maintaining a separate Node.js/Electron project alongside the Rust CLI
- IPC between Electron frontend and Rust backend adds complexity
- Overkill for a dashboard/tracking UI
- Slow startup time compared to native apps
- Security surface area of bundled Chromium

## Decision

**Option 1 (Embedded Web UI)** was chosen because:

1. **The infrastructure already exists.** The Axum REST API with all the needed endpoints was already built. Adding a web frontend is purely additive.
2. **Zero friction for users.** No separate app to install. `folio serve --open` and you're there.
3. **Minimal maintenance burden.** Vanilla HTML/CSS/JS with no build toolchain means no npm vulnerabilities, no webpack configs, no framework upgrades.
4. **Binary stays small.** The entire web UI adds ~50KB to the compiled binary via `rust-embed`.
5. **Progressive enhancement path.** If we later need a native window, we can wrap this same web UI in Tauri without rewriting anything.

## Migration Path

If user feedback indicates a need for native desktop features (system tray, global hotkeys, background operation without a terminal), the recommended next step would be:

1. Keep the web UI as-is (it works in any browser)
2. Add a Tauri wrapper that loads the same embedded web UI
3. Use Tauri's Rust backend integration to call folio's existing DB and tracking code directly

This gives us a native app with minimal new code, since the web UI and Rust backend already exist.
