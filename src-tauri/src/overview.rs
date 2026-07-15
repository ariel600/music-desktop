use crate::audio;
use crate::db::DbState;
use crate::holiday_service;
use crate::music_schedule;
use crate::music_storage;
use crate::operational_day;
use crate::system_message_schedule;
use chrono::{DateTime, Local};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::{AppHandle, Manager, State};

static SESSION_STARTED_AT: OnceLock<DateTime<Local>> = OnceLock::new();

pub fn mark_session_start() {
    let _ = SESSION_STARTED_AT.set(Local::now());
}

fn session_started_at() -> DateTime<Local> {
    *SESSION_STARTED_AT.get_or_init(Local::now)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewNowPlaying {
    pub title: String,
    pub file_path: String,
    pub folder: Option<String>,
    pub artwork_data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewHours {
    pub closed: bool,
    pub open: Option<String>,
    pub close: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewSnapshot {
    pub total_songs: u64,
    pub now_playing: Option<OverviewNowPlaying>,
    pub system_messages_total: u64,
    pub system_messages_today: u64,
    pub music_folder: Option<String>,
    pub emergency_plays_today: u64,
    pub operating_hours: OverviewHours,
    pub session_uptime_seconds: u64,
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.try_state::<crate::AppPaths>()
        .map(|paths| paths.app_data_dir.clone())
        .ok_or_else(|| "App paths are not initialized".to_string())
}

fn count_system_messages_for_today(db: &DbState) -> Result<u64, String> {
    let now = Local::now();
    let operational_date = operational_day::operational_date_string(now);
    let weekday = operational_day::operational_weekday(now);
    let operating_hours = db.get_operating_hours().map_err(|e| e.to_string())?;
    let holiday = db
        .get_holiday_day(&operational_date)
        .map_err(|e| e.to_string())?;
    let holiday_ref = holiday.as_ref();
    let tomorrow_is_erev =
        system_message_schedule::tomorrow_is_erev_chag(db, &operational_date);
    let day_before_erev_as_thursday = db.get_day_before_erev_as_thursday().unwrap_or(false);
    let treat_as_thursday = system_message_schedule::treat_day_as_thursday(
        day_before_erev_as_thursday,
        tomorrow_is_erev,
    );

    let mut count = 0u64;
    for message in db
        .get_active_system_messages()
        .map_err(|e| e.to_string())?
    {
        if message.file_path.trim().is_empty() {
            continue;
        }
        if !system_message_schedule::matches_operational_day(
            &message,
            weekday,
            holiday_ref,
            day_before_erev_as_thursday,
            tomorrow_is_erev,
        ) {
            continue;
        }
        if system_message_schedule::resolve_play_time(
            &message,
            &operational_date,
            weekday,
            &operating_hours,
            holiday_ref,
            treat_as_thursday,
        )
        .is_some()
        {
            count += 1;
        }
    }
    Ok(count)
}

fn resolve_hours(db: &DbState) -> Result<OverviewHours, String> {
    let now = Local::now();
    let operational_date = operational_day::operational_date_string(now);
    let weekday = operational_day::operational_weekday(now);
    let settings = db.get_operating_hours().map_err(|e| e.to_string())?;
    let holiday = db
        .get_holiday_day(&operational_date)
        .map_err(|e| e.to_string())?;

    let Some(hours) = system_message_schedule::resolve_day_hours(
        &operational_date,
        weekday,
        &settings,
        holiday.as_ref(),
    ) else {
        return Ok(OverviewHours {
            closed: true,
            open: None,
            close: None,
        });
    };

    if system_message_schedule::is_closed(&hours) {
        Ok(OverviewHours {
            closed: true,
            open: None,
            close: None,
        })
    } else {
        Ok(OverviewHours {
            closed: false,
            open: Some(hours.open),
            close: Some(hours.close),
        })
    }
}

fn scheduled_music_folder(db: &DbState) -> Option<String> {
    let holidays = holiday_service::get_holidays_list(db).ok()?;
    let settings = db.get_operating_hours().ok()?;
    let now = Local::now();
    let operational_date = operational_day::operational_date_string(now);
    let holiday = db.get_holiday_day(&operational_date).ok().flatten();
    let (_in_window, decision) =
        music_schedule::resolve_today(&holidays, &settings, holiday.as_ref());
    decision.slug().map(str::to_string)
}

#[tauri::command]
pub fn get_overview_snapshot(
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<OverviewSnapshot, String> {
    let app_dir = app_data_dir(&app)?;
    let total_songs = music_storage::count_all_music_files(&app_dir)?;
    let now_playing = audio::get_now_playing().map(|info| OverviewNowPlaying {
        title: info.title,
        file_path: info.file_path,
        folder: info.folder.clone(),
        artwork_data_url: info.artwork_data_url,
    });

    let music_folder = now_playing
        .as_ref()
        .and_then(|playing| playing.folder.clone())
        .or_else(|| scheduled_music_folder(&state));

    let system_messages_total = state
        .get_system_messages()
        .map_err(|e| e.to_string())?
        .len() as u64;
    let system_messages_today = count_system_messages_for_today(&state)?;
    let emergency_plays_today = state
        .count_emergency_plays_on_operational_date()
        .map_err(|e| e.to_string())?;
    let operating_hours = resolve_hours(&state)?;
    let session_uptime_seconds = (Local::now() - session_started_at())
        .num_seconds()
        .max(0) as u64;

    Ok(OverviewSnapshot {
        total_songs,
        now_playing,
        system_messages_total,
        system_messages_today,
        music_folder,
        emergency_plays_today,
        operating_hours,
        session_uptime_seconds,
    })
}

pub fn folder_slug_from_music_path(path: &Path) -> Option<String> {
    let mut seen_music = false;
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy();
        if seen_music {
            if music_storage::MUSIC_FOLDER_SLUGS.contains(&name.as_ref()) {
                return Some(name.into_owned());
            }
            return None;
        }
        if name == music_storage::MUSIC_LIBRARY_DIR {
            seen_music = true;
        }
    }
    None
}

pub fn find_artwork_data_url(audio_path: &Path) -> Option<String> {
    let mut candidates = Vec::new();
    if let Some(stem) = audio_path.file_stem().and_then(|s| s.to_str()) {
        if let Some(parent) = audio_path.parent() {
            for ext in ["jpg", "jpeg", "png", "webp"] {
                candidates.push(parent.join(format!("{stem}.{ext}")));
            }
            for name in [
                "cover.jpg",
                "cover.jpeg",
                "cover.png",
                "folder.jpg",
                "Folder.jpg",
                "front.jpg",
            ] {
                candidates.push(parent.join(name));
            }
        }
    }

    for candidate in candidates {
        if !candidate.is_file() {
            continue;
        }
        let Ok(bytes) = std::fs::read(&candidate) else {
            continue;
        };
        if bytes.is_empty() || bytes.len() > 4_000_000 {
            continue;
        }
        let mime = match candidate
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("png") => "image/png",
            Some("webp") => "image/webp",
            _ => "image/jpeg",
        };
        let encoded = base64_encode(&bytes);
        return Some(format!("data:{mime};base64,{encoded}"));
    }
    None
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
