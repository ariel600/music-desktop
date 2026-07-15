use crate::audio;
use crate::db::DbState;
use crate::emergency_alerts::{self, MESSAGE_TYPE_UNCONFIGURED};
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, IF_MODIFIED_SINCE, IF_NONE_MATCH};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Mutex;
use tokio::time::interval;

const FETCH_URL: &str = "https://www.oref.org.il/WarningMessages/alert/Alerts.json";
const OREF_HOME_URL: &str = "https://www.oref.org.il/";
const FETCH_INTERVAL_MS: u64 = 2_000;
const DUPLICATE_WINDOW_MS: i64 = 420_000;
const HISTORY_RETENTION_MS: i64 = 24 * 60 * 60 * 1_000;

static OREF_FETCH_UI_LOG: AtomicU64 = AtomicU64::new(0);
static OREF_HANDLE_UI_LOG: AtomicU64 = AtomicU64::new(0);

fn cookie_name_value(set_cookie: &str) -> Option<String> {
    let pair = set_cookie.split(';').next()?.trim();
    if pair.is_empty() || !pair.contains('=') {
        return None;
    }
    Some(pair.to_string())
}

#[derive(Debug, Deserialize)]
struct OrefAlertResponse {
    title: Option<String>,
    desc: Option<String>,
    data: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmergencyAlertPayload {
    pub id: String,
    pub message_type: String,
    pub title: String,
    pub description: Option<String>,
    pub cities: Vec<String>,
    pub received_at: String,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    alert_title: String,
    city: String,
    date: DateTime<Utc>,
}

struct MonitorState {
    client: Client,
    headers: HeaderMap,
    cookies: Option<String>,
    if_modified_since: Option<String>,
    etag: Option<String>,
    history: Vec<HistoryEntry>,
    seen_ids: HashSet<String>,
}

impl MonitorState {
    fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36",
            ),
        );
        headers.insert("Referer", HeaderValue::from_static("https://www.oref.org.il/"));
        headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap_or_else(|_| Client::new()),
            headers,
            cookies: None,
            if_modified_since: None,
            etag: None,
            history: Vec::new(),
            seen_ids: HashSet::new(),
        }
    }

    async fn refresh_cookies(&mut self) {
        match self
            .client
            .get(OREF_HOME_URL)
            .headers(self.headers.clone())
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let cookies = response
                    .headers()
                    .get_all("set-cookie")
                    .iter()
                    .filter_map(|value| value.to_str().ok())
                    .filter_map(cookie_name_value)
                    .collect::<Vec<_>>()
                    .join("; ");

                if !cookies.is_empty() {
                    self.cookies.replace(cookies);
                    tracing::info!(target: "oref", "cookies refreshed");
                }
            }
            Ok(response) => {
                tracing::warn!(target: "oref", "cookie refresh failed ({})", response.status());
            }
            Err(error) => {
                tracing::warn!(target: "oref", "cookie refresh failed ({error})");
            }
        }
    }

    async fn fetch_alert_once(&mut self) -> Result<Option<OrefAlertResponse>, String> {
        let mut headers = self.headers.clone();
        if let Some(value) = &self.if_modified_since {
            if let Ok(header) = HeaderValue::from_str(value) {
                headers.insert(IF_MODIFIED_SINCE, header);
            }
        }
        if let Some(value) = &self.etag {
            if let Ok(header) = HeaderValue::from_str(value) {
                headers.insert(IF_NONE_MATCH, header);
            }
        }
        if let Some(cookies) = &self.cookies {
            if let Ok(header) = HeaderValue::from_str(cookies) {
                headers.insert("Cookie", header);
            }
        }

        let response = self
            .client
            .get(FETCH_URL)
            .headers(headers)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED
            || response.status() == reqwest::StatusCode::FORBIDDEN
        {
            return Err("auth".to_string());
        }

        if let Some(value) = response.headers().get("last-modified") {
            if let Ok(text) = value.to_str() {
                self.if_modified_since = Some(text.to_string());
            }
        }
        if let Some(value) = response.headers().get("etag") {
            if let Ok(text) = value.to_str() {
                self.etag = Some(text.to_string());
            }
        }

        if response.status() != reqwest::StatusCode::OK {
            return Ok(None);
        }

        let body = response.text().await.map_err(|error| error.to_string())?;
        let trimmed = body.trim().trim_start_matches('\u{feff}');
        if trimmed.is_empty() || trimmed == "[]" || trimmed == "{}" || trimmed == "null" {
            return Ok(None);
        }

        let alert = serde_json::from_str::<OrefAlertResponse>(trimmed)
            .map_err(|error| format!("parse: {error}"))?;

        if alert
            .data
            .as_ref()
            .map(|cities| cities.is_empty())
            .unwrap_or(true)
        {
            return Ok(None);
        }

        Ok(Some(alert))
    }

    async fn fetch_alert(&mut self) -> Result<Option<OrefAlertResponse>, String> {
        match self.fetch_alert_once().await {
            Ok(alert) => Ok(alert),
            Err(code) if code == "auth" => {
                tracing::warn!(target: "oref", "auth error (401/403). Refreshing cookies...");
                self.refresh_cookies().await;
                self.fetch_alert_once().await
            }
            Err(error) => Err(error),
        }
    }

    fn prune_history(&mut self, now: DateTime<Utc>) {
        self.history
            .retain(|entry| (now - entry.date).num_milliseconds() <= HISTORY_RETENTION_MS);
    }

    fn is_duplicate(&self, alert_title: &str, city: &str, now: DateTime<Utc>) -> bool {
        self.history.iter().any(|entry| {
            entry.alert_title == alert_title
                && entry.city == city
                && (now - entry.date).num_milliseconds() <= DUPLICATE_WINDOW_MS
        })
    }

    fn remember(&mut self, alert_title: &str, city: &str, now: DateTime<Utc>) {
        self.history.push(HistoryEntry {
            alert_title: alert_title.to_string(),
            city: city.to_string(),
            date: now,
        });
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(crate::db::APP_NAME);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.maximize();
        let _ = window.set_focus();
    }
}

fn notify_desktop(app: &AppHandle, title: &str, cities: &[String]) {
    let body = if cities.is_empty() {
        title.to_string()
    } else {
        format!("{title}\n{}", cities.join(" · "))
    };

    if let Err(error) = app
        .notification()
        .builder()
        .title("פיקוד העורף")
        .body(body)
        .show()
    {
        tracing::warn!(target: "oref", "desktop notification failed: {error}");
    }
}

fn message_type_label(message_type: &str) -> &'static str {
    match message_type {
        "pre-alert" => "התראה מקדימה",
        "red-alert" => "צבע אדום",
        "hostile-aircraft" => "חדירת כלי טיס עוין",
        "end" => "סיום",
        _ => "הודעה לא מוגדרת",
    }
}

async fn handle_alert(
    app: &AppHandle,
    alert: OrefAlertResponse,
    state: &mut MonitorState,
) -> Result<Option<(String, f32, String)>, String> {
    let title = alert.title.unwrap_or_default().trim().to_string();
    if title.is_empty() {
        return Ok(None);
    }

    let description = alert
        .desc
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let cities = alert.data.unwrap_or_default();
    let message_type = emergency_alerts::resolve_message_type(&title);
    let now = Utc::now();

    state.prune_history(now);

    tracing::info!(target: "oref", "alert received: {title} in [{}]", cities.join(", "));

    let db = app
        .try_state::<DbState>()
        .map(|state| state.inner().clone())
        .ok_or_else(|| "Database state is not initialized".to_string())?;

    let monitored = db
        .get_emergency_monitored_cities()
        .unwrap_or_default();

    let mut triggered_cities = Vec::new();
    let mut matched_monitored = false;
    for city in cities {
        if !crate::oref_cities::city_matches(&city, &monitored) {
            continue;
        }
        matched_monitored = true;
        if state.is_duplicate(&title, &city, now) {
            continue;
        }
        state.remember(&title, &city, now);
        triggered_cities.push(city);
    }

    if triggered_cities.is_empty() {
        if !matched_monitored && !monitored.is_empty() {
            tracing::info!(target: "oref", "no monitored cities in alert — skipping");
        } else {
            tracing::info!(target: "oref", "duplicate alert for all cities — skipping");
        }
        return Ok(None);
    }

    tracing::info!(target: "oref", "new match for cities: {}", triggered_cities.join(", "));

    let payload = EmergencyAlertPayload {
        id: format!(
            "{}-{}-{}",
            now.timestamp_millis(),
            message_type,
            state.seen_ids.len()
        ),
        message_type: message_type.to_string(),
        title: title.clone(),
        description: description.clone(),
        cities: triggered_cities.clone(),
        received_at: now.to_rfc3339(),
    };
    state.seen_ids.insert(payload.id.clone());

    let _ = app.emit("emergency-alert", &payload);
    notify_desktop(app, &title, &triggered_cities);
    show_main_window(app);

    let settings = db
        .get_emergency_message_settings()
        .map_err(|error| error.to_string())?;
    let is_enabled = settings.get(message_type).copied().unwrap_or(false);

    if !is_enabled {
        tracing::info!(
            target: "oref",
            "audio skipped — type '{message_type}' ({}) is disabled",
            message_type_label(message_type)
        );
        crate::app_log::emergency_skip(
            &db,
            &title,
            &format!("type disabled: {message_type}"),
        );
        return Ok(None);
    }

    if message_type == MESSAGE_TYPE_UNCONFIGURED {
        tracing::info!(target: "oref", "audio skipped — unmapped alert type");
        crate::app_log::emergency_skip(&db, &title, "unmapped alert type");
        return Ok(None);
    }

    if let Some(audio_path) = db
        .get_emergency_message_audio_path(message_type)
        .map_err(|error| error.to_string())?
    {
        if !audio_path.is_empty() {
            let volume = audio::get_channel_volume(audio::VolumeChannel::Emergency);
            return Ok(Some((audio_path, volume, title)));
        }
        crate::app_log::emergency_skip(&db, &title, &format!("empty audio path: {message_type}"));
    } else {
        tracing::info!(target: "oref", "audio skipped — no file for '{message_type}'");
        crate::app_log::emergency_skip(&db, &title, &format!("no audio file: {message_type}"));
    }

    Ok(None)
}

pub fn start(app: AppHandle) {
    let shared_state = Mutex::new(MonitorState::new());

    tauri::async_runtime::spawn(async move {
        {
            let mut state = shared_state.lock().await;
            state.refresh_cookies().await;
        }

        tracing::info!(target: "oref", "monitor started (poll every {FETCH_INTERVAL_MS}ms)");

        let mut ticker = interval(Duration::from_millis(FETCH_INTERVAL_MS));

        loop {
            ticker.tick().await;

            if !crate::system_activity::allows_playback() {
                continue;
            }

            let alert = {
                let mut state = shared_state.lock().await;
                match state.fetch_alert().await {
                    Ok(alert) => alert,
                    Err(error) => {
                        if !error.starts_with("parse:") {
                            tracing::warn!(target: "oref", "fetch error: {error}");
                            crate::app_log::from_app_rate_limited(
                                &app,
                                &OREF_FETCH_UI_LOG,
                                300,
                                &format!("Oref fetch error: {error}"),
                                "oref_error",
                            );
                        }
                        None
                    }
                }
            };

            let Some(alert) = alert else {
                continue;
            };

            let pending_audio = {
                let mut state = shared_state.lock().await;
                match handle_alert(&app, alert, &mut state).await {
                    Ok(pending) => pending,
                    Err(error) => {
                        tracing::warn!(target: "oref", "handle error: {error}");
                        crate::app_log::from_app_rate_limited(
                            &app,
                            &OREF_HANDLE_UI_LOG,
                            60,
                            &format!("Oref handle error: {error}"),
                            "oref_error",
                        );
                        None
                    }
                }
            };

            if let Some((audio_path, volume, alert_title)) = pending_audio {
                let play_result = tauri::async_runtime::spawn_blocking(move || {
                    audio::play_audio_blocking_for_channel(
                        &audio_path,
                        volume,
                        audio::VolumeChannel::Emergency,
                    )
                })
                .await
                .map_err(|error| error.to_string())
                .and_then(|inner| inner);

                let db = app.try_state::<DbState>().map(|state| state.inner().clone());
                match (&play_result, db) {
                    (Ok(()), Some(db)) => {
                        let _ = db.log_play(
                            None,
                            &format!("Emergency: {alert_title}"),
                            "emergency_ok",
                        );
                    }
                    (Err(error), Some(db)) => {
                        tracing::warn!(target: "oref", "emergency audio error: {error}");
                        let _ = db.log_play(
                            None,
                            &format!("Emergency: {alert_title} ({error})"),
                            "emergency_error",
                        );
                    }
                    (Err(error), None) => {
                        tracing::warn!(target: "oref", "emergency audio error: {error}");
                    }
                    (Ok(()), None) => {}
                }
            }
        }
    });
}
