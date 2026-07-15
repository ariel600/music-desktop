use crate::audio;
use crate::db::DbState;
use crate::holiday_service;
use crate::operational_day;
use crate::system_activity;
use chrono::{Local, Timelike};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::time::{sleep, Duration as TokioDuration};

/// Startup miss notify + due catch-up share the same window.
const DUE_WINDOW_MINUTES: i64 = 10;
const MAX_LOOKBACK_MINUTES: i64 = 120;

async fn notify_missed_tasks(app: &AppHandle, missed: &[crate::db::Task]) {
    for task in missed {
        let body = format!(
            "ההודעה '{}' בשעה {} לא הושמעה. לחץ על 'פתח תוכנה' כדי לנהל.",
            task.title, task.scheduled_time
        );

        if let Err(error) = app
            .notification()
            .builder()
            .title("הודעה שפוספסה")
            .body(body)
            .show()
        {
            tracing::warn!(target: "scheduler", "שגיאת התראה: {error}");
        }
    }
}

async fn play_tasks(
    db: &DbState,
    tasks: Vec<crate::db::Task>,
    operational_date: &str,
) -> Result<(), String> {
    let holiday = db
        .get_holiday_day(operational_date)
        .map_err(|error| error.to_string())?;

    for task in tasks {
        if let Some(reason) =
            holiday_service::should_skip_task_for_date_with_holiday(
                &task,
                operational_date,
                db,
                holiday.as_ref(),
            )?
        {
            tracing::info!(target: "scheduler", "הודעה '{}' בוטלה ({reason})", task.title);
            let _ = db.log_play(Some(task.id), &task.title, reason);
            db.mark_played(task.id, operational_date)
                .map_err(|error| error.to_string())?;
            continue;
        }

        let file_path = task.file_path.clone();
        let volume = audio::resolve_volume(task.volume);
        let play_result = tauri::async_runtime::spawn_blocking(move || {
            audio::play_audio_blocking_for_channel(
                &file_path,
                volume,
                audio::VolumeChannel::General,
            )
        })
        .await
        .map_err(|error| error.to_string())?;

        if let Err(error) = play_result {
            tracing::warn!(target: "scheduler", "שגיאת השמעה למשימה '{}': {error}", task.title);
            let _ = db.log_play(
                Some(task.id),
                &format!("Schedule: {} ({error})", task.title),
                "error",
            );
            continue;
        }

        let _ = db.log_play(
            Some(task.id),
            &format!("Schedule: {}", task.title),
            "success",
        );
        db.mark_played(task.id, operational_date)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

async fn play_system_messages(
    db: &DbState,
    messages: Vec<crate::db::SystemMessage>,
    operational_date: &str,
) -> Result<(), String> {
    for message in messages {
        let file_path = message.file_path.clone();
        let volume = audio::get_channel_volume(audio::VolumeChannel::System);
        let play_result = tauri::async_runtime::spawn_blocking(move || {
            audio::play_audio_blocking_for_channel(
                &file_path,
                volume,
                audio::VolumeChannel::System,
            )
        })
        .await
        .map_err(|error| error.to_string())?;

        if let Err(error) = play_result {
            tracing::warn!(target: "scheduler", "שגיאת השמעה להודעת מערכת '{}': {error}", message.title);
            let _ = db.log_play(
                Some(message.id),
                &format!("System: {} ({error})", message.title),
                "system_error",
            );
            continue;
        }

        let _ = db.log_play(
            Some(message.id),
            &format!("System: {}", message.title),
            "system_ok",
        );
        db.mark_system_message_played(message.id, operational_date)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn db_from_app(app: &AppHandle) -> Result<DbState, String> {
    app.try_state::<DbState>()
        .map(|state| state.inner().clone())
        .ok_or_else(|| "Database state is not initialized".to_string())
}

async fn run_scheduler_tick(app: &AppHandle, lookback_minutes: i64) -> Result<(), String> {
    let now = Local::now();
    let operational_date = operational_day::operational_date_string(now);
    let weekday = operational_day::operational_weekday(now);

    let db = db_from_app(app)?;

    let due_tasks = db
        .get_due_tasks(lookback_minutes, &operational_date, weekday)
        .map_err(|error| error.to_string())?;

    play_tasks(&db, due_tasks, &operational_date).await?;

    let due_system_messages = db
        .get_due_system_messages(lookback_minutes, &operational_date, weekday)
        .map_err(|error| error.to_string())?;

    play_system_messages(&db, due_system_messages, &operational_date).await?;

    Ok(())
}

async fn check_missed_on_startup(app: &AppHandle) {
    let db = match db_from_app(app) {
        Ok(db) => db,
        Err(error) => {
            tracing::warn!(target: "scheduler", "שגיאת scheduler בהפעלה: {error}");
            crate::app_log::error_from_app(app, &format!("Scheduler startup error: {error}"));
            return;
        }
    };

    let operational_date = operational_day::operational_date_string(Local::now());
    let holiday = db.get_holiday_day(&operational_date).ok().flatten();

    match db.get_missed_tasks(DUE_WINDOW_MINUTES) {
        Ok(missed) => {
            let mut notify = Vec::new();
            for task in missed {
                match holiday_service::should_skip_task_for_date_with_holiday(
                    &task,
                    &operational_date,
                    &db,
                    holiday.as_ref(),
                ) {
                    Ok(Some(_)) => {}
                    Ok(None) => notify.push(task),
                    Err(error) => {
                        tracing::warn!(target: "scheduler", "שגיאת סינון הודעה שפוספסה: {error}");
                        crate::app_log::error(
                            &db,
                            &format!("Missed-message filter error: {error}"),
                        );
                    }
                }
            }
            if !notify.is_empty() {
                for task in &notify {
                    crate::app_log::write(
                        &db,
                        &format!(
                            "Missed schedule message: {} ({})",
                            task.title, task.scheduled_time
                        ),
                        "missed",
                    );
                }
                notify_missed_tasks(app, &notify).await;
            }
        }
        Err(error) => {
            tracing::warn!(target: "scheduler", "שגיאת בדיקת הודעות שפוספסו: {error}");
            crate::app_log::error(&db, &format!("Missed-message check error: {error}"));
        }
    }
}

async fn sleep_until_next_minute() {
    let now = Local::now();
    let ms_into_minute =
        (now.second() as u64 * 1000) + (now.nanosecond() as u64 / 1_000_000);
    let wait_ms = if ms_into_minute == 0 {
        60_000
    } else {
        60_000u64.saturating_sub(ms_into_minute).max(1)
    };
    sleep(TokioDuration::from_millis(wait_ms)).await;
}

fn lookback_minutes_since(window_start: chrono::DateTime<Local>) -> i64 {
    let elapsed = (Local::now() - window_start).num_minutes().max(0) + 1;
    elapsed.clamp(DUE_WINDOW_MINUTES, MAX_LOOKBACK_MINUTES)
}

pub fn start_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tracing::info!(target: "scheduler", "[scheduler] - started - background loop active");
        let _ = app.notification().request_permission();
        sleep(TokioDuration::from_secs(2)).await;

        check_missed_on_startup(&app).await;

        let mut window_start = Local::now() - chrono::Duration::minutes(DUE_WINDOW_MINUTES);

        loop {
            if !system_activity::allows_playback() {
                sleep(TokioDuration::from_secs(1)).await;
                continue;
            }

            let lookback = lookback_minutes_since(window_start);
            let tick_started = Local::now();

            if let Err(error) = run_scheduler_tick(&app, lookback).await {
                tracing::warn!(target: "scheduler", "שגיאת scheduler: {error}");
                crate::app_log::error_from_app(&app, &format!("Scheduler error: {error}"));
            }

            // Advance watermark to tick start so slots that became due during
            // long playback are still inside the next lookback window.
            window_start = tick_started;
            sleep_until_next_minute().await;
        }
    });
}
