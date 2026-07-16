use crate::audio;
use crate::db::DbState;
use crate::holiday_service;
use crate::music_schedule::{self, MusicFolderDecision};
use crate::music_storage;
use crate::system_activity;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use tauri::{AppHandle, Manager};
use tokio::time::{sleep, Duration as TokioDuration};

static MUSIC_FOLDER_UI_LOG: AtomicU64 = AtomicU64::new(0);
static MUSIC_HOURS_UI_LOG: AtomicU64 = AtomicU64::new(0);

struct PlaylistState {
    folder: String,
    /// Unplayed songs in the current round (shuffled once when the round starts).
    remaining: Vec<String>,
    /// Songs already played in the current round.
    played: HashSet<String>,
}

impl PlaylistState {
    fn empty() -> Self {
        Self {
            folder: String::new(),
            remaining: Vec::new(),
            played: HashSet::new(),
        }
    }

    fn begin_round(folder: &str, mut files: Vec<String>) -> Self {
        let mut rng = rand::thread_rng();
        files.shuffle(&mut rng);
        Self {
            folder: folder.to_string(),
            remaining: files,
            played: HashSet::new(),
        }
    }

    /// Random order within the folder; no song repeats until every song in the
    /// folder has been played once, then a new shuffled round begins.
    fn next_path(&mut self, folder: &str, files: Vec<String>) -> Option<String> {
        if files.is_empty() {
            *self = Self::empty();
            return None;
        }

        if self.folder != folder {
            *self = Self::begin_round(folder, files);
            return self.take_next();
        }

        let available: HashSet<String> = files.iter().cloned().collect();
        self.remaining.retain(|path| available.contains(path));
        self.played.retain(|path| available.contains(path));

        for path in &files {
            if !self.played.contains(path) && !self.remaining.iter().any(|p| p == path) {
                self.remaining.push(path.clone());
            }
        }

        if self.remaining.is_empty() {
            // Entire folder was heard — start the next shuffled round.
            *self = Self::begin_round(folder, files);
        }

        self.take_next()
    }

    fn take_next(&mut self) -> Option<String> {
        if self.remaining.is_empty() {
            return None;
        }
        let path = self.remaining.remove(0);
        self.played.insert(path.clone());
        Some(path)
    }
}

fn db_from_app(app: &AppHandle) -> Result<DbState, String> {
    app.try_state::<DbState>()
        .map(|state| state.inner().clone())
        .ok_or_else(|| "Database state is not initialized".to_string())
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.try_state::<crate::AppPaths>()
        .map(|paths| paths.app_data_dir.clone())
        .ok_or_else(|| "App paths are not initialized".to_string())
}

fn files_for_folder(app_data_dir: &Path, folder: &str) -> Vec<String> {
    music_storage::list_music_files(app_data_dir, folder)
        .unwrap_or_default()
        .into_iter()
        .map(|entry| entry.path)
        .collect()
}

fn pick_playback_folder(
    app_data_dir: &Path,
    decision: MusicFolderDecision,
) -> Option<&'static str> {
    let slug = decision.slug()?;
    if !files_for_folder(app_data_dir, slug).is_empty() {
        return Some(slug);
    }

    let fallback = music_schedule::fallback_folder(decision)?;
    if fallback != slug && !files_for_folder(app_data_dir, fallback).is_empty() {
        Some(fallback)
    } else {
        None
    }
}

/// Plays one track. Returns `false` when the file is invalid/missing so the
/// loop can skip to the next song without stalling.
async fn play_one_track(app: &AppHandle, path: String) -> bool {
    let track_path = path.clone();
    let volume = audio::get_channel_volume(audio::VolumeChannel::Music);
    match tauri::async_runtime::spawn_blocking(move || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            audio::play_audio_blocking_for_channel(&path, volume, audio::VolumeChannel::Music)
        }))
        .unwrap_or_else(|_| Err("invalid audio file (panic)".to_string()))
    })
    .await
    {
        Ok(Ok(())) => true,
        Ok(Err(error)) => {
            log_track_failed(app, &track_path, &error);
            false
        }
        Err(error) => {
            log_track_failed(app, &track_path, &error.to_string());
            false
        }
    }
}

fn append_ui_log(app: &AppHandle, title: &str, status: &str) {
    crate::app_log::from_app(app, title, status);
}

fn log_track_failed(app: &AppHandle, track_path: &str, error: &str) {
    let file_name = Path::new(track_path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| track_path.to_string());
    tracing::warn!(
        target: "music_player",
        "[music.playback] - track_failed - path={track_path}, error={error}"
    );
    append_ui_log(
        app,
        &format!("Music track failed: {file_name} ({error})"),
        "music_error",
    );
}

fn log_playback_started(app: &AppHandle, folder: &str) {
    tracing::info!(
        target: "music_player",
        "[music.playback] - started - folder={folder}"
    );
    append_ui_log(app, &format!("Music started ({folder})"), "music_started");
}

fn log_playback_stopped(app: &AppHandle, reason: &str, session_active: &mut bool) {
    if !*session_active {
        return;
    }
    tracing::info!(
        target: "music_player",
        "[music.playback] - stopped - reason={reason}"
    );
    append_ui_log(app, &format!("Music stopped ({reason})"), "music_stopped");
    *session_active = false;
}

pub fn start(app: AppHandle) {
    tracing::info!(target: "music_player", "[music_player] - started - watchdog and playback loops active");
    let watchdog_app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            sleep(TokioDuration::from_secs(20)).await;
            if !system_activity::allows_playback() {
                audio::stop_music();
                continue;
            }
            let Ok(db) = db_from_app(&watchdog_app) else {
                continue;
            };
            let Ok(settings) = db.get_operating_hours() else {
                continue;
            };
            let Ok(holidays) = holiday_service::get_holidays_list(&db) else {
                continue;
            };
            let hebrew_date =
                crate::operational_day::hebrew_date_string(chrono::Local::now());
            let today_holiday = db.get_holiday_day(&hebrew_date).ok().flatten();
            let (in_window, decision) =
                music_schedule::resolve_today(&holidays, &settings, today_holiday.as_ref());
            if !in_window || matches!(decision, MusicFolderDecision::Silence) {
                audio::stop_music();
            }
        }
    });

    tauri::async_runtime::spawn(async move {
        let mut playlist = PlaylistState::empty();
        let mut holidays_loaded_for = String::new();
        let mut holidays = Vec::new();
        // True while we are inside an active music window and playing the queue.
        // Not toggled between songs — only on session start / session end.
        let mut session_active = false;

        loop {
            if !system_activity::allows_playback() {
                audio::stop_music();
                log_playback_stopped(&app, "system_inactive", &mut session_active);
                sleep(TokioDuration::from_secs(1)).await;
                continue;
            }

            let db = match db_from_app(&app) {
                Ok(db) => db,
                Err(error) => {
                    tracing::warn!(target: "music_player", "[music.playback] - failed - stage=db, error={error}");
                    crate::app_log::error_from_app(&app, &format!("Music DB error: {error}"));
                    sleep(TokioDuration::from_secs(5)).await;
                    continue;
                }
            };

            let app_dir = match app_data_dir(&app) {
                Ok(dir) => dir,
                Err(error) => {
                    tracing::warn!(target: "music_player", "[music.playback] - failed - stage=app_dir, error={error}");
                    crate::app_log::error_from_app(&app, &format!("Music app-dir error: {error}"));
                    sleep(TokioDuration::from_secs(5)).await;
                    continue;
                }
            };

            let now = chrono::Local::now();
            let hebrew_date = crate::operational_day::hebrew_date_string(now);
            if holidays_loaded_for != hebrew_date {
                match holiday_service::get_holidays_list(&db) {
                    Ok(list) => {
                        holidays = list;
                        holidays_loaded_for = hebrew_date.clone();
                    }
                    Err(error) => {
                        tracing::warn!(
                            target: "music_player",
                            "[music.playback] - failed - stage=holidays, error={error}"
                        );
                        crate::app_log::warn_from_app(
                            &app,
                            &format!("Music: holidays load failed ({error})"),
                        );
                    }
                }
            }

            let settings = match db.get_operating_hours() {
                Ok(settings) => settings,
                Err(error) => {
                    tracing::warn!(
                        target: "music_player",
                        "[music.playback] - failed - stage=hours, error={error}"
                    );
                    crate::app_log::from_app_rate_limited(
                        &app,
                        &MUSIC_HOURS_UI_LOG,
                        120,
                        &format!("Music: failed to load operating hours ({error})"),
                        "error",
                    );
                    sleep(TokioDuration::from_secs(5)).await;
                    continue;
                }
            };

            let today_holiday = db.get_holiday_day(&hebrew_date).ok().flatten();
            let (in_window, decision) =
                music_schedule::resolve_today(&holidays, &settings, today_holiday.as_ref());

            if !in_window || matches!(decision, MusicFolderDecision::Silence) {
                audio::stop_music();
                playlist = PlaylistState::empty();
                log_playback_stopped(&app, "outside_music_window", &mut session_active);
                sleep(TokioDuration::from_secs(15)).await;
                continue;
            }

            let Some(folder) = pick_playback_folder(&app_dir, decision) else {
                audio::stop_music();
                log_playback_stopped(&app, "no_playable_folder", &mut session_active);
                crate::app_log::from_app_rate_limited(
                    &app,
                    &MUSIC_FOLDER_UI_LOG,
                    300,
                    &format!(
                        "Music: no playable folder for decision={decision:?} (library empty or missing)"
                    ),
                    "warn",
                );
                sleep(TokioDuration::from_secs(30)).await;
                continue;
            };

            let files = files_for_folder(&app_dir, folder);
            if files.is_empty() {
                audio::stop_music();
                log_playback_stopped(&app, "empty_folder", &mut session_active);
                crate::app_log::from_app_rate_limited(
                    &app,
                    &MUSIC_FOLDER_UI_LOG,
                    300,
                    &format!("Music: folder '{folder}' is empty during music window"),
                    "warn",
                );
                sleep(TokioDuration::from_secs(30)).await;
                continue;
            }

            let Some(path) = playlist.next_path(folder, files) else {
                sleep(TokioDuration::from_secs(10)).await;
                continue;
            };

            if !session_active {
                log_playback_started(&app, folder);
                session_active = true;
            }

            if play_one_track(&app, path).await {
                sleep(TokioDuration::from_millis(250)).await;
            } else {
                // Bad file — advance quickly, but avoid a tight spin if many
                // files (or the only file) are corrupt.
                sleep(TokioDuration::from_millis(500)).await;
            }
        }
    });
}
