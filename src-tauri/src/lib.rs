#![allow(clippy::too_many_arguments)]

mod app_log;
mod app_password;
mod audio;
mod audio_convert;
mod autostart;
mod backup;
mod db;
mod holiday_service;
mod emergency_alerts;
mod maintenance;
mod message_storage;
mod music_player;
mod music_schedule;
mod music_storage;
mod operational_day;
mod overview;
mod oref_cities;
mod oref_monitor;
mod os_volume;
mod password_vault;
mod scheduler;
mod system_activity;
mod system_message_schedule;

use db::{
    DbState, EmergencyMessageAudioFile, HolidayEntry, OperatingHoursSettings,
    PlayLogEntry, Schedule, ScheduleOverridesBundle, SystemMessage, Task,
};
use music_storage::{MusicFileEntry, ScannedMusicFile};
use password_vault::{PasswordKind, PasswordVault};
use std::path::Path;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;

pub(crate) struct AppPaths {
    pub(crate) app_data_dir: std::path::PathBuf,
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .try_init();
}

fn map_db_error(error: rusqlite::Error) -> String {
    let message = error.to_string();
    message
        .strip_prefix("Invalid parameter name: ")
        .unwrap_or(&message)
        .to_string()
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(db::APP_NAME);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.maximize();
        let _ = window.set_focus();
        let _ = app.emit("app-lock-required", ());
    }
}

#[tauri::command]
fn has_app_password(vault: State<PasswordVault>) -> Result<bool, String> {
    vault.has_password(PasswordKind::App)
}

#[tauri::command]
fn verify_app_password(vault: State<PasswordVault>, password: String) -> Result<bool, String> {
    vault.verify(PasswordKind::App, &password)
}

#[tauri::command]
fn set_app_password(
    vault: State<PasswordVault>,
    state: State<DbState>,
    current_password: Option<String>,
    new_password: String,
) -> Result<(), String> {
    let has_password = vault.has_password(PasswordKind::App)?;
    if has_password {
        let current = current_password.unwrap_or_default();
        if !vault.verify(PasswordKind::App, &current)? {
            return Err("הסיסמה הנוכחית שגויה".to_string());
        }
    }
    vault.set_password(PasswordKind::App, &new_password)?;
    app_log::settings(&state, "app password updated");
    Ok(())
}

#[tauri::command]
fn clear_app_password(
    vault: State<PasswordVault>,
    state: State<DbState>,
    current_password: String,
) -> Result<(), String> {
    if !vault.verify(PasswordKind::App, &current_password)? {
        return Err("הסיסמה הנוכחית שגויה".to_string());
    }
    vault.clear_password(PasswordKind::App)?;
    app_log::settings(&state, "app password cleared");
    Ok(())
}

#[tauri::command]
fn has_settings_password(vault: State<PasswordVault>) -> Result<bool, String> {
    vault.has_password(PasswordKind::Settings)
}

#[tauri::command]
fn verify_settings_password(
    vault: State<PasswordVault>,
    password: String,
) -> Result<bool, String> {
    vault.verify(PasswordKind::Settings, &password)
}

#[tauri::command]
fn set_settings_password(
    vault: State<PasswordVault>,
    state: State<DbState>,
    current_password: Option<String>,
    new_password: String,
) -> Result<(), String> {
    let has_password = vault.has_password(PasswordKind::Settings)?;
    if has_password {
        let current = current_password.unwrap_or_default();
        if !vault.verify(PasswordKind::Settings, &current)? {
            return Err("סיסמת ההגדרות הנוכחית שגויה".to_string());
        }
    }
    vault.set_password(PasswordKind::Settings, &new_password)?;
    app_log::settings(&state, "settings password updated");
    Ok(())
}

#[tauri::command]
fn clear_settings_password(
    vault: State<PasswordVault>,
    state: State<DbState>,
    current_password: String,
) -> Result<(), String> {
    if !vault.verify(PasswordKind::Settings, &current_password)? {
        return Err("סיסמת ההגדרות הנוכחית שגויה".to_string());
    }
    vault.clear_password(PasswordKind::Settings)?;
    app_log::settings(&state, "settings password cleared");
    Ok(())
}

#[tauri::command]
fn get_lock_music_add(state: State<DbState>) -> Result<bool, String> {
    state.get_lock_music_add().map_err(map_db_error)
}

#[tauri::command]
fn set_lock_music_add(enabled: bool, state: State<DbState>) -> Result<bool, String> {
    let enabled = state.set_lock_music_add(enabled).map_err(map_db_error)?;
    app_log::settings(&state, &format!("lock music add = {enabled}"));
    Ok(enabled)
}

#[tauri::command]
fn get_system_active() -> Result<bool, String> {
    Ok(system_activity::is_active())
}

#[tauri::command]
fn set_system_active(active: bool, state: State<DbState>) -> Result<bool, String> {
    let active = state.set_system_active(active).map_err(map_db_error)?;
    system_activity::set_active(active);
    app_log::system_power(&state, active);
    Ok(active)
}

#[tauri::command]
fn get_day_before_erev_as_thursday(state: State<DbState>) -> Result<bool, String> {
    state
        .get_day_before_erev_as_thursday()
        .map_err(map_db_error)
}

#[tauri::command]
fn set_day_before_erev_as_thursday(enabled: bool, state: State<DbState>) -> Result<bool, String> {
    let enabled = state
        .set_day_before_erev_as_thursday(enabled)
        .map_err(map_db_error)?;
    app_log::settings(
        &state,
        &format!("day before erev as Thursday = {enabled}"),
    );
    Ok(enabled)
}

#[tauri::command]
fn get_tasks(state: State<DbState>) -> Result<Vec<Task>, String> {
    state.get_all_tasks().map_err(|error| error.to_string())
}

#[tauri::command]
fn get_schedules(state: State<DbState>) -> Result<Vec<Schedule>, String> {
    state
        .get_all_schedules()
        .map_err(|error| error.to_string())
}














#[tauri::command]
fn get_play_log(state: State<DbState>, limit: Option<i64>) -> Result<Vec<PlayLogEntry>, String> {
    state
        .get_play_log(limit.unwrap_or(50))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_backup(
    paths: State<AppPaths>,
    state: State<DbState>,
    dest_path: String,
) -> Result<String, String> {
    state
        .checkpoint_for_backup()
        .map_err(|error| error.to_string())?;
    let dest = backup::export_backup(&paths.app_data_dir, Path::new(&dest_path))?;
    app_log::settings(&state, "backup exported");
    Ok(dest)
}

#[tauri::command]
fn import_backup(
    paths: State<AppPaths>,
    state: State<DbState>,
    source_path: String,
) -> Result<(), String> {
    app_log::maintenance(&state, "paused for backup import");
    let _pause = maintenance::PauseGuard::enter();
    let db_path = paths.app_data_dir.join("nusic.db");
    state
        .release_file_locks()
        .map_err(|error| error.to_string())?;
    let _ = std::fs::remove_file(paths.app_data_dir.join("nusic.db-wal"));
    let _ = std::fs::remove_file(paths.app_data_dir.join("nusic.db-shm"));

    backup::import_backup(&paths.app_data_dir, Path::new(&source_path))?;

    state.reopen(&db_path).map_err(|error| error.to_string())?;
    let volumes = state
        .get_audio_volumes()
        .map_err(|error| error.to_string())?;
    audio::init_volumes(&volumes);
    system_activity::load_from_db(&state);
    drop(_pause);
    app_log::maintenance(&state, "resumed after backup import");
    app_log::settings(&state, "backup imported");
    Ok(())
}

#[tauri::command]
fn reset_system(
    paths: State<AppPaths>,
    state: State<DbState>,
    vault: State<PasswordVault>,
    keep_music: bool,
) -> Result<(), String> {
    app_log::maintenance(&state, "paused for system reset");
    backup::reset_system(&paths.app_data_dir, &state, keep_music)?;
    app_log::settings(&state, &format!("system reset (keep_music = {keep_music})"));
    vault.clear_all()?;
    system_activity::set_active(true);
    app_log::maintenance(&state, "resumed after system reset");
    Ok(())
}


#[tauri::command]
fn is_calendar_synced_today(state: State<DbState>) -> Result<bool, String> {
    let today = crate::operational_day::operational_date_string(chrono::Local::now());
    state.is_calendar_synced_on(&today).map_err(map_db_error)
}

#[tauri::command]
fn get_holidays_list(state: State<DbState>) -> Result<Vec<HolidayEntry>, String> {
    holiday_service::get_holidays_list(&state)
}

#[tauri::command]
fn sync_calendar_holidays(
    state: State<DbState>,
    entries: Vec<HolidayEntry>,
) -> Result<Vec<HolidayEntry>, String> {
    holiday_service::sync_calendar_holidays(&state, &entries)
}


#[tauri::command]
fn set_holiday_status(
    state: State<DbState>,
    date: String,
    day_label: String,
    cancel_messages: bool,
    open_time: Option<String>,
    close_time: Option<String>,
) -> Result<Vec<HolidayEntry>, String> {
    let result = holiday_service::set_holiday_status(
        &state,
        &date,
        &day_label,
        cancel_messages,
        open_time,
        close_time,
    )?;
    app_log::settings(
        &state,
        &format!("holiday {date} status = {day_label}, cancel_messages = {cancel_messages}"),
    );
    Ok(result)
}

#[tauri::command]
fn add_custom_holiday(
    state: State<DbState>,
    date: String,
    title: String,
    cancel_messages: Option<bool>,
    day_label: Option<String>,
    open_time: Option<String>,
    close_time: Option<String>,
    hebrew_month: Option<String>,
    hebrew_day: Option<i32>,
) -> Result<Vec<HolidayEntry>, String> {
    let result = holiday_service::add_custom_holiday(
        &state,
        &date,
        &title,
        cancel_messages.unwrap_or(true),
        day_label,
        open_time,
        close_time,
        hebrew_month,
        hebrew_day,
    )?;
    app_log::settings(&state, &format!("holiday added: {title} ({date})"));
    Ok(result)
}

#[tauri::command]
fn ensure_custom_recurrences(
    state: State<DbState>,
    entries: Vec<HolidayEntry>,
) -> Result<Vec<HolidayEntry>, String> {
    holiday_service::ensure_custom_recurrences(&state, &entries)
}

#[tauri::command]
fn delete_custom_holiday(state: State<DbState>, date: String) -> Result<Vec<HolidayEntry>, String> {
    let result = holiday_service::delete_custom_holiday(&state, &date)?;
    app_log::settings(&state, &format!("holiday deleted: {date}"));
    Ok(result)
}



#[tauri::command]
fn get_operating_hours(state: State<DbState>) -> Result<OperatingHoursSettings, String> {
    state.get_operating_hours().map_err(map_db_error)
}

#[tauri::command]
fn set_operating_hours(
    state: State<DbState>,
    settings: OperatingHoursSettings,
) -> Result<OperatingHoursSettings, String> {
    let saved = state
        .set_operating_hours(&settings)
        .map_err(map_db_error)?;
    app_log::settings(&state, "operating hours updated");
    Ok(saved)
}


#[tauri::command]
fn get_emergency_message_settings(
    state: State<DbState>,
) -> Result<std::collections::HashMap<String, bool>, String> {
    state
        .get_emergency_message_settings()
        .map_err(map_db_error)
}

#[tauri::command]
fn set_emergency_message_enabled(
    state: State<DbState>,
    message_type: String,
    enabled: bool,
) -> Result<std::collections::HashMap<String, bool>, String> {
    let settings = state
        .set_emergency_message_enabled(&message_type, enabled)
        .map_err(map_db_error)?;
    app_log::settings(
        &state,
        &format!("emergency '{message_type}' enabled = {enabled}"),
    );
    Ok(settings)
}

#[tauri::command]
fn get_emergency_message_audio_files(
    state: State<DbState>,
) -> Result<Vec<EmergencyMessageAudioFile>, String> {
    state
        .get_emergency_message_audio_files()
        .map_err(map_db_error)
}

#[tauri::command]
fn import_emergency_message_audio(
    state: State<DbState>,
    paths: State<AppPaths>,
    message_type: String,
    source_path: String,
) -> Result<EmergencyMessageAudioFile, String> {
    let old_path = state
        .get_emergency_message_audio_path(&message_type)
        .map_err(|error| error.to_string())?;

    let imported = message_storage::import_emergency_message_audio(
        &paths.app_data_dir,
        &message_type,
        &source_path,
    )?;

    match state.set_emergency_message_audio_path(&message_type, &imported.path) {
        Ok(entry) => {
            if let Some(old_path) = old_path {
                if old_path != imported.path {
                    message_storage::remove_emergency_message_audio_file(&old_path);
                }
            }
            app_log::settings(
                &state,
                &format!("emergency audio updated for '{message_type}'"),
            );
            Ok(entry)
        }
        Err(error) => {
            message_storage::remove_emergency_message_audio_file(&imported.path);
            Err(error.to_string())
        }
    }
}

#[tauri::command]
async fn get_oref_cities(
    paths: State<'_, AppPaths>,
) -> Result<Vec<oref_cities::OrefCity>, String> {
    let dir = paths.app_data_dir.clone();
    tauri::async_runtime::spawn_blocking(move || oref_cities::get_oref_cities(&dir))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
fn get_emergency_monitored_cities(state: State<DbState>) -> Result<Vec<String>, String> {
    state
        .get_emergency_monitored_cities()
        .map_err(map_db_error)
}

#[tauri::command]
fn set_emergency_monitored_cities(
    state: State<DbState>,
    cities: Vec<String>,
) -> Result<Vec<String>, String> {
    let saved = state
        .set_emergency_monitored_cities(&cities)
        .map_err(map_db_error)?;
    app_log::settings(
        &state,
        &format!("emergency monitored cities updated ({})", saved.len()),
    );
    Ok(saved)
}

#[tauri::command]
fn get_system_messages(state: State<DbState>) -> Result<Vec<SystemMessage>, String> {
    state.get_system_messages().map_err(map_db_error)
}

#[tauri::command]
fn add_system_message(
    state: State<DbState>,
    paths: State<AppPaths>,
    title: String,
    source_path: String,
    days_of_week: Vec<u8>,
    schedule_mode: String,
    scheduled_time: Option<String>,
    operating_anchor: Option<String>,
    offset_direction: Option<String>,
    offset_minutes: Option<i64>,
) -> Result<SystemMessage, String> {
    let imported =
        message_storage::import_system_message_audio(&paths.app_data_dir, &source_path)?;

    state
        .add_system_message(
            &title,
            &imported.path,
            &days_of_week,
            &schedule_mode,
            scheduled_time.as_deref(),
            operating_anchor.as_deref(),
            offset_direction.as_deref(),
            offset_minutes,
        )
        .map_err(map_db_error)
}

#[tauri::command]
fn update_system_message(
    state: State<DbState>,
    id: i64,
    title: String,
    days_of_week: Vec<u8>,
    schedule_mode: String,
    scheduled_time: Option<String>,
    operating_anchor: Option<String>,
    offset_direction: Option<String>,
    offset_minutes: Option<i64>,
) -> Result<SystemMessage, String> {
    state
        .update_system_message(
            id,
            &title,
            &days_of_week,
            &schedule_mode,
            scheduled_time.as_deref(),
            operating_anchor.as_deref(),
            offset_direction.as_deref(),
            offset_minutes,
        )
        .map_err(map_db_error)
}

#[tauri::command]
fn set_system_message_enabled(
    state: State<DbState>,
    id: i64,
    enabled: bool,
) -> Result<SystemMessage, String> {
    let message = state
        .set_system_message_enabled(id, enabled)
        .map_err(map_db_error)?;
    app_log::settings(
        &state,
        &format!("system message '{}' enabled = {enabled}", message.title),
    );
    Ok(message)
}

#[tauri::command]
fn update_system_message_audio(
    state: State<DbState>,
    paths: State<AppPaths>,
    id: i64,
    source_path: String,
) -> Result<SystemMessage, String> {
    let existing = state
        .get_system_message(id)
        .map_err(map_db_error)?
        .ok_or_else(|| "הודעת המערכת לא נמצאה".to_string())?;

    let imported =
        message_storage::import_system_message_audio(&paths.app_data_dir, &source_path)?;

    let updated = state
        .update_system_message_audio_path(id, &imported.path)
        .map_err(map_db_error)?;

    if existing.file_path != imported.path {
        message_storage::remove_message_audio_file(&existing.file_path);
    }

    Ok(updated)
}

#[tauri::command]
fn delete_system_message(state: State<DbState>, id: i64) -> Result<(), String> {
    let removed_path = state.delete_system_message(id).map_err(map_db_error)?;
    if let Some(path) = removed_path {
        message_storage::remove_message_audio_file(&path);
    }
    Ok(())
}

#[tauri::command]
fn get_schedule_overrides(
    state: State<DbState>,
    start: String,
    end: String,
) -> Result<ScheduleOverridesBundle, String> {
    state
        .get_schedule_overrides(&start, &end)
        .map_err(|error| error.to_string())
}




#[tauri::command]
fn list_music_files(paths: State<AppPaths>, folder: String) -> Result<Vec<MusicFileEntry>, String> {
    music_storage::list_music_files(&paths.app_data_dir, &folder)
}


#[tauri::command]
fn scan_music_sources(source_paths: Vec<String>) -> Result<Vec<ScannedMusicFile>, String> {
    music_storage::scan_music_sources(&source_paths)
}

#[tauri::command]
fn import_music_files(
    paths: State<AppPaths>,
    state: State<DbState>,
    folder: String,
    source_paths: Vec<String>,
    vocal_warning_acknowledged: bool,
) -> Result<Vec<MusicFileEntry>, String> {
    match music_storage::import_music_files(
        &paths.app_data_dir,
        &folder,
        &source_paths,
        vocal_warning_acknowledged,
    ) {
        Ok(imported) => {
            app_log::write(
                &state,
                &format!("Music imported: {} file(s) → {folder}", imported.len()),
                "music_import",
            );
            Ok(imported)
        }
        Err(error) => {
            app_log::error(&state, &format!("Music import failed ({folder}): {error}"));
            Err(error)
        }
    }
}

#[tauri::command]
fn delete_music_files(
    paths: State<AppPaths>,
    state: State<DbState>,
    folder: String,
    file_paths: Vec<String>,
) -> Result<usize, String> {
    match music_storage::delete_music_files(&paths.app_data_dir, &folder, &file_paths) {
        Ok(deleted) => {
            app_log::write(
                &state,
                &format!("Music deleted: {deleted} file(s) from {folder}"),
                "music_delete",
            );
            Ok(deleted)
        }
        Err(error) => {
            app_log::error(&state, &format!("Music delete failed ({folder}): {error}"));
            Err(error)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    tracing::info!(
        target: "app",
        "[app] - starting - {} v{}",
        db::APP_NAME,
        env!("CARGO_PKG_VERSION")
    );
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
            music_storage::init_music_library(&app_data_dir).map_err(|e| e.to_string())?;
            message_storage::init_message_libraries(&app_data_dir).map_err(|e| e.to_string())?;

            let db_path = app_data_dir.join("nusic.db");
            let db_state = DbState::new(db_path.clone()).map_err(|e| e.to_string())?;
            app_log::app_start(&db_state, env!("CARGO_PKG_VERSION"));
            let password_vault = PasswordVault::new(&app_data_dir)?;
            password_vault.migrate_from_db(&db_state)?;
            let volumes = db_state.get_audio_volumes().map_err(|e| e.to_string())?;
            audio::init_volumes(&volumes);
            system_activity::load_from_db(&db_state);
            os_volume::start_system_volume_guard(app.handle().clone());
            app.manage(db_state);
            app.manage(password_vault);
            app.manage(AppPaths {
                app_data_dir: app_data_dir.clone(),
            });

            let ffmpeg_data_dir = app_data_dir.clone();
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                if audio_convert::install_converter(&ffmpeg_data_dir).is_err() {
                    crate::app_log::from_app(
                        &app_handle,
                        "Audio converter unavailable (ffmpeg)",
                        "converter_error",
                    );
                    let _ = app_handle
                        .notification()
                        .builder()
                        .title("ממיר שמע לא זמין")
                        .body(audio_convert::CONVERTER_INTERNET_REQUIRED_MSG)
                        .show();
                }
            });

            overview::mark_session_start();
            scheduler::start_scheduler(app.handle().clone());
            music_player::start(app.handle().clone());
            oref_cities::ensure_cities_cached_async(app_data_dir.clone());
            oref_monitor::start(app.handle().clone());

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(db::APP_NAME);
                let _ = window.hide();
            }

            let open_app =
                MenuItem::with_id(app, "open_settings", "פתח תוכנה", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "יציאה", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_app, &quit])?;

            let mut tray_builder = TrayIconBuilder::new()
                .tooltip(db::APP_NAME)
                .menu(&menu);
            if let Some(icon) = app.default_window_icon().cloned() {
                tray_builder = tray_builder.icon(icon);
            }
            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open_settings" => {
                        show_main_window(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            if let Ok(exe_path) = std::env::current_exe() {
                if let Err(error) =
                    autostart::enable_autostart(db::AUTOSTART_KEY, &exe_path.to_string_lossy())
                {
                    if let Some(db) = app.try_state::<DbState>() {
                        app_log::warn(
                            db.inner(),
                            &format!("Autostart setup failed: {error}"),
                        );
                    }
                }
            }

            if let Some(db) = app.try_state::<DbState>() {
                app_log::app_ready(db.inner());
            }
            tracing::info!(target: "app", "[app] - ready - initialization complete, background services running");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                let _ = window.app_handle().emit("app-lock-required", ());
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_tasks,
            get_schedules,
            get_play_log,
            overview::get_overview_snapshot,
            export_backup,
            import_backup,
            reset_system,
            has_app_password,
            verify_app_password,
            set_app_password,
            clear_app_password,
            has_settings_password,
            verify_settings_password,
            set_settings_password,
            clear_settings_password,
            get_lock_music_add,
            set_lock_music_add,
            get_system_active,
            set_system_active,
            get_day_before_erev_as_thursday,
            set_day_before_erev_as_thursday,
            get_operating_hours,
            set_operating_hours,
            get_emergency_message_settings,
            set_emergency_message_enabled,
            get_emergency_message_audio_files,
            import_emergency_message_audio,
            get_oref_cities,
            get_emergency_monitored_cities,
            set_emergency_monitored_cities,
            get_system_messages,
            add_system_message,
            update_system_message,
            set_system_message_enabled,
            update_system_message_audio,
            delete_system_message,
            sync_calendar_holidays,
            is_calendar_synced_today,
            get_holidays_list,
            set_holiday_status,
            add_custom_holiday,
            ensure_custom_recurrences,
            delete_custom_holiday,
            get_schedule_overrides,
            list_music_files,
            scan_music_sources,
            import_music_files,
            delete_music_files,
            audio::play_audio,
            audio::play_volume_test_channel,
            audio::get_audio_volumes,
            audio::set_audio_volume_channel,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(db) = app_handle.try_state::<DbState>() {
                    app_log::app_exit(db.inner());
                }
            }
        });
}
