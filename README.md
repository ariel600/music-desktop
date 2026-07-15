# Nusic — Smart Bell System

**Nusic** is a Windows desktop application for automated bell and announcement scheduling in schools, institutions, and other organizations. It runs quietly in the background, plays audio alerts at configured times, and provides a simple management interface for operators.

Built with **Tauri 2**, the app combines a lightweight Rust backend with a modern React frontend—delivering native performance, system tray integration, and reliable background scheduling without a heavy runtime.

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | [Tauri 2](https://v2.tauri.app/) |
| Backend | Rust (commands, scheduler, audio, persistence) |
| Frontend | React 19 + TypeScript + Vite |
| Database | SQLite via `rusqlite` |
| Styling | Tailwind CSS 4 + shadcn/ui (Slider) |
| Audio | `rodio` |
| Async runtime | Tokio |

---

## Key Features

- **Background service with system tray** — Starts hidden, lives in the Windows notification area. Close minimizes to tray; quit is available from the tray menu.
- **Schedule management** — Group bells under named schedules (e.g. *School Day*). Each task supports active days of the week (Sunday–Saturday).
- **Rust scheduling engine** — A background loop runs every 60 seconds, checks the current day and time, and plays due tasks automatically.
- **Missed-bell notifications** — On startup, the app scans the last 10 minutes for missed bells and shows a system tray notification.
- **Global volume control** — Volume slider in the UI updates a shared backend state and persists across restarts via SQLite.
- **Bell board UI** — Calendar-style table view: rows are times, columns are weekdays—similar to professional bell-management systems.
- **Windows autostart** — Registers in the Windows Run registry for automatic launch on boot.

---

## Getting Started

### Prerequisites

**Windows (primary target)**

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (via `rustup`)
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11)

**Linux (development only)**

Install Tauri and audio dependencies, for example on Ubuntu / Pop!_OS:

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  librsvg2-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  libdbus-1-dev \
  libasound2-dev \
  pkg-config
```

### Installation

```bash
git clone <repository-url>
cd nusic
npm install
```

Ensure Rust is on your `PATH`:

```bash
source "$HOME/.cargo/env"   # Linux / macOS
```

### Development

```bash
npm run tauri dev
```

The app launches hidden in the system tray. Use **Open Settings** from the tray menu to open the management window.

### Production build

```bash
npm run tauri build
```

Installers and binaries are written to `src-tauri/target/release/bundle/`.

---

## Project Structure

```
nusic/
├── src/                          # React frontend
│   ├── api.ts                    # Tauri invoke wrappers
│   ├── types.ts                  # Shared TypeScript types
│   ├── components/
│   │   ├── SettingsPage.tsx      # Main dashboard
│   │   ├── ScheduleBoard.tsx     # Weekday bell table view
│   │   ├── TaskForm.tsx          # Create bells & schedules
│   │   ├── TaskList.tsx          # List view
│   │   ├── VolumeControl.tsx     # Global volume slider
│   │   └── ui/slider.tsx         # shadcn/ui Slider
│   └── lib/utils.ts              # Tailwind class helpers
│
└── src-tauri/                    # Rust backend
    ├── src/
    │   ├── main.rs               # Application entry point
    │   ├── lib.rs                # Tauri setup, tray, commands
    │   ├── db.rs                 # SQLite schema & queries
    │   ├── scheduler.rs          # Background bell engine
    │   ├── audio.rs              # Playback & volume state
    │   └── autostart.rs          # Windows registry autostart
    ├── Cargo.toml
    └── tauri.conf.json
```

### Frontend (`src/`)

Handles the operator UI: schedule boards, task creation, volume control, and status display. Communicates with Rust through Tauri commands defined in `api.ts`.

### Backend (`src-tauri/`)

Owns persistence, scheduling, audio playback, tray behavior, and OS integration. SQLite stores tasks, schedules, settings, and volume. The scheduler runs independently of the UI so bells fire even when the window is hidden.

---

## Advanced Technical Details

### Global volume with `AtomicU32`

Playback volume is stored in a process-wide `AtomicU32` in `audio.rs`. The UI and scheduler read the same value without locking, while changes from the settings panel are persisted to the `settings` table in SQLite. This keeps audio responsive and consistent across manual playback and scheduled bells.

```rust
static GLOBAL_VOLUME: AtomicU32 = AtomicU32::new(1.0f32.to_bits());
```

`rodio` applies the volume via `Sink::set_volume()` on every playback.

### Background scheduler with Tokio

`start_scheduler()` spawns an async task on Tauri's Tokio runtime. It:

1. Checks for missed bells in the last 10 minutes on startup (tray notification).
2. Runs a `tokio::time::interval` every 60 seconds.
3. Queries SQLite for tasks matching the current `HH:MM`, weekday, and active schedule.
4. Plays audio on a blocking thread pool and updates `last_played_date` to prevent duplicate plays on the same day.

### Database schema (overview)

| Table | Purpose |
|-------|---------|
| `schedules` | Named bell programs (e.g. *School Day*) |
| `tasks` | Individual bells: title, audio path, time, days, schedule |
| `settings` | Key-value store (e.g. persisted volume) |

---

## License

Private / unlicensed — update this section as needed for your distribution.
