use crate::db::DbState;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

/// Append an entry to the in-app Logs tab (`play_log` table).
pub fn write(db: &DbState, title: &str, status: &str) {
    if let Err(error) = db.log_play(None, title, status) {
        tracing::warn!(target: "app_log", "failed to write UI log: {error}");
    }
}

pub fn from_app(app: &AppHandle, title: &str, status: &str) {
    if let Some(db) = app.try_state::<DbState>() {
        write(db.inner(), title, status);
    }
}

/// At most one UI log for `slot` every `min_interval_secs` (avoids poll spam).
pub fn from_app_rate_limited(
    app: &AppHandle,
    slot: &'static AtomicU64,
    min_interval_secs: u64,
    title: &str,
    status: &str,
) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let prev = slot.load(Ordering::Relaxed);
    if now.saturating_sub(prev) < min_interval_secs {
        return;
    }
    slot.store(now, Ordering::Relaxed);
    from_app(app, title, status);
}

pub fn app_start(db: &DbState, version: &str) {
    write(db, &format!("App started (v{version})"), "app_start");
}

pub fn app_ready(db: &DbState) {
    write(db, "App ready", "app_ready");
}

pub fn app_exit(db: &DbState) {
    write(db, "App exit", "app_exit");
}

pub fn system_power(db: &DbState, active: bool) {
    if active {
        write(db, "System turned ON", "system_on");
    } else {
        write(db, "System turned OFF", "system_off");
    }
}

pub fn settings(db: &DbState, detail: &str) {
    write(db, &format!("Settings: {detail}"), "settings");
}

pub fn error(db: &DbState, detail: &str) {
    write(db, detail, "error");
}

pub fn error_from_app(app: &AppHandle, detail: &str) {
    from_app(app, detail, "error");
}

pub fn warn(db: &DbState, detail: &str) {
    write(db, detail, "warn");
}

pub fn warn_from_app(app: &AppHandle, detail: &str) {
    from_app(app, detail, "warn");
}

pub fn maintenance(db: &DbState, detail: &str) {
    write(db, &format!("Maintenance: {detail}"), "maintenance");
}

pub fn emergency_skip(db: &DbState, title: &str, reason: &str) {
    write(
        db,
        &format!("Emergency skipped: {title} ({reason})"),
        "emergency_skip",
    );
}
