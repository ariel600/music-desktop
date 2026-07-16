use chrono::{Duration, Local, NaiveDate, NaiveTime};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub const APP_NAME: &str = "מערכת הודעות חכמה";
pub const AUTOSTART_KEY: &str = "SmartMessageSystem";
pub const DEFAULT_VOLUME: f32 = 1.0;
pub const VOLUME_SETTING_KEY: &str = "volume";
pub const AUDIO_VOLUMES_KEY: &str = "audio_volumes";
pub const APP_PASSWORD_HASH_KEY: &str = "app_password_hash";
pub const SETTINGS_PASSWORD_HASH_KEY: &str = "settings_password_hash";
pub const LOCK_MUSIC_ADD_KEY: &str = "lock_music_add";
pub const SYSTEM_ACTIVE_KEY: &str = "system_active";
pub const DAY_BEFORE_EREV_AS_THURSDAY_KEY: &str = "day_before_erev_as_thursday";
/// Sunday=0 … Saturday=6 (matches frontend WEEKDAYS).
pub const WEEKDAY_THURSDAY: u8 = 4;
pub const EMERGENCY_MESSAGES_ENABLED_KEY: &str = "emergency_messages_enabled";
pub const EMERGENCY_MESSAGE_AUDIO_KEY: &str = "emergency_message_audio_files";
pub const EMERGENCY_MONITORED_CITIES_KEY: &str = "emergency_monitored_cities";
pub const EMERGENCY_MESSAGE_TYPE_IDS: &[&str] = &[
    "pre-alert",
    "red-alert",
    "hostile-aircraft",
    "end",
    "unconfigured",
];
pub const HOLIDAYS_CACHE_KEY: &str = "jewish_holidays_cache";
pub const HOLIDAYS_SYNCED_ON_KEY: &str = "holidays_calendar_synced_on";
pub const OPERATING_HOURS_KEY: &str = "operating_hours";
pub const TISHA_BAV_CLOSED_DEFAULT_KEY: &str = "tisha_bav_closed_default_v1";
pub const DEFAULT_SCHEDULE_NAME: &str = "יום עבודה";
pub const ALL_DAYS: &str = "0,1,2,3,4,5,6";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperatingDayHours {
    pub open: String,
    pub close: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeasonOperatingHours {
    pub sunday: OperatingDayHours,
    pub monday: OperatingDayHours,
    pub tuesday: OperatingDayHours,
    pub wednesday: OperatingDayHours,
    pub thursday: OperatingDayHours,
    pub friday: OperatingDayHours,
    #[serde(rename = "motzei-shabbat")]
    pub motzei_shabbat: OperatingDayHours,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemporaryOperatingHours {
    #[serde(flatten)]
    pub hours: SeasonOperatingHours,
    #[serde(default)]
    pub valid_from: Option<String>,
    #[serde(default)]
    pub valid_to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperatingHoursSettings {
    pub winter: SeasonOperatingHours,
    pub summer: SeasonOperatingHours,
    #[serde(default = "default_temporary_operating_hours_for_serde")]
    pub temporary: TemporaryOperatingHours,
}

fn default_season_operating_hours_for_serde() -> SeasonOperatingHours {
    SeasonOperatingHours {
        sunday: OperatingDayHours {
            open: "00:00".into(),
            close: "00:00".into(),
        },
        monday: OperatingDayHours {
            open: "00:00".into(),
            close: "00:00".into(),
        },
        tuesday: OperatingDayHours {
            open: "00:00".into(),
            close: "00:00".into(),
        },
        wednesday: OperatingDayHours {
            open: "00:00".into(),
            close: "00:00".into(),
        },
        thursday: OperatingDayHours {
            open: "00:00".into(),
            close: "00:00".into(),
        },
        friday: OperatingDayHours {
            open: "00:00".into(),
            close: "00:00".into(),
        },
        motzei_shabbat: OperatingDayHours {
            open: "00:00".into(),
            close: "00:00".into(),
        },
    }
}

fn default_temporary_operating_hours_for_serde() -> TemporaryOperatingHours {
    TemporaryOperatingHours {
        hours: default_season_operating_hours_for_serde(),
        valid_from: None,
        valid_to: None,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmergencyMessageAudioFile {
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemMessage {
    pub id: i64,
    pub title: String,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_name: Option<String>,
    pub is_active: bool,
    pub days_of_week: Vec<u8>,
    pub schedule_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_anchor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_minutes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played_date: Option<String>,
}

pub const SYSTEM_MESSAGE_SCHEDULE_FIXED: &str = "fixed_time";
pub const SYSTEM_MESSAGE_SCHEDULE_RELATIVE: &str = "relative_operating_hours";
pub const SYSTEM_MESSAGE_DAY_HOLIDAY_EVE: u8 = 7;
pub const SYSTEM_MESSAGE_DAY_HOLIDAY: u8 = 8;
pub const SYSTEM_MESSAGE_MAX_DAY: u8 = 8;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Schedule {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub file_path: String,
    pub scheduled_time: String,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_played_date: Option<String>,
    pub schedule_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_name: Option<String>,
    pub days_of_week: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f32>,
    pub cancel_on_holiday: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HolidayEntry {
    pub date: String,
    pub title: String,
    pub holiday_group: String,
    pub day_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hebrew: Option<String>,
    pub cancel_messages: bool,
    pub is_custom: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hebrew_month: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hebrew_day: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LegacyHolidayCache {
    fetched_on: String,
    year: i32,
    holidays: Vec<LegacyHolidayItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LegacyHolidayItem {
    date: String,
    title: String,
    #[serde(default)]
    hebrew: Option<String>,
    #[serde(default)]
    cancel_messages: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayLogEntry {
    pub id: i64,
    pub task_id: Option<i64>,
    pub task_title: String,
    pub played_at: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DayOverrideEntry {
    pub date: String,
    pub schedule_id: i64,
    pub is_disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskOverrideEntry {
    pub date: String,
    pub task_id: i64,
    pub is_disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleOverridesBundle {
    pub day_overrides: Vec<DayOverrideEntry>,
    pub task_overrides: Vec<TaskOverrideEntry>,
}

#[derive(Clone)]
pub struct DbState {
    conn: Arc<Mutex<Connection>>,
}

pub fn parse_days_of_week(raw: &str) -> Vec<u8> {
    let mut days: Vec<u8> = raw
        .split(',')
        .filter_map(|part| part.trim().parse::<u8>().ok())
        .filter(|day| *day <= 6)
        .collect();

    if days.is_empty() {
        days = vec![0, 1, 2, 3, 4, 5, 6];
    }

    days.sort_unstable();
    days.dedup();
    days
}

pub fn format_days_of_week(days: &[u8]) -> String {
    let mut normalized = days.to_vec();
    normalized.sort_unstable();
    normalized.dedup();
    normalized
        .iter()
        .map(|day| day.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn parse_system_message_days(raw: &str) -> Vec<u8> {
    let mut days: Vec<u8> = raw
        .split(',')
        .filter_map(|part| part.trim().parse::<u8>().ok())
        .filter(|day| *day <= SYSTEM_MESSAGE_MAX_DAY)
        .collect();

    days.sort_unstable();
    days.dedup();
    days
}

impl DbState {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        let _ = conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;",
        );
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schedules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                file_path TEXT NOT NULL,
                scheduled_time TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                last_played_date TEXT,
                schedule_id INTEGER,
                days_of_week TEXT NOT NULL DEFAULT '0,1,2,3,4,5,6',
                volume REAL,
                FOREIGN KEY (schedule_id) REFERENCES schedules(id)
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS play_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER,
                task_title TEXT NOT NULL,
                played_at TEXT NOT NULL,
                status TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS holiday_days (
                date TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                holiday_group TEXT NOT NULL,
                day_label TEXT NOT NULL,
                hebrew TEXT,
                cancel_messages INTEGER NOT NULL DEFAULT 1,
                is_custom INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS schedule_day_overrides (
                date TEXT NOT NULL,
                schedule_id INTEGER NOT NULL DEFAULT 0,
                is_disabled INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (date, schedule_id)
            );
            CREATE TABLE IF NOT EXISTS schedule_task_overrides (
                date TEXT NOT NULL,
                task_id INTEGER NOT NULL,
                is_disabled INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (date, task_id)
            );
            CREATE TABLE IF NOT EXISTS system_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                file_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                is_active INTEGER NOT NULL DEFAULT 1,
                days_of_week TEXT NOT NULL DEFAULT '0,1,2,3,4',
                schedule_mode TEXT NOT NULL DEFAULT 'fixed_time',
                scheduled_time TEXT,
                operating_anchor TEXT,
                offset_direction TEXT,
                offset_minutes INTEGER,
                last_played_date TEXT
            );",
        )?;

        let _ = conn.execute(
            "ALTER TABLE system_messages ADD COLUMN days_of_week TEXT NOT NULL DEFAULT '0,1,2,3,4'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE system_messages ADD COLUMN schedule_mode TEXT NOT NULL DEFAULT 'fixed_time'",
            [],
        );
        let _ = conn.execute("ALTER TABLE system_messages ADD COLUMN scheduled_time TEXT", []);
        let _ = conn.execute("ALTER TABLE system_messages ADD COLUMN operating_anchor TEXT", []);
        let _ = conn.execute("ALTER TABLE system_messages ADD COLUMN offset_direction TEXT", []);
        let _ = conn.execute("ALTER TABLE system_messages ADD COLUMN offset_minutes INTEGER", []);
        let _ = conn.execute(
            "ALTER TABLE system_messages ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE system_messages ADD COLUMN last_played_date TEXT",
            [],
        );

        let _ = conn.execute("ALTER TABLE tasks ADD COLUMN last_played_date TEXT", []);
        let _ = conn.execute("ALTER TABLE tasks ADD COLUMN schedule_id INTEGER", []);
        let _ = conn.execute(
            "ALTER TABLE tasks ADD COLUMN days_of_week TEXT NOT NULL DEFAULT '0,1,2,3,4,5,6'",
            [],
        );
        let _ = conn.execute("ALTER TABLE tasks ADD COLUMN volume REAL", []);
        let _ = conn.execute(
            "ALTER TABLE tasks ADD COLUMN cancel_on_holiday INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute("ALTER TABLE holiday_days ADD COLUMN open_time TEXT", []);
        let _ = conn.execute("ALTER TABLE holiday_days ADD COLUMN close_time TEXT", []);
        let _ = conn.execute("ALTER TABLE holiday_days ADD COLUMN hebrew_month TEXT", []);
        let _ = conn.execute("ALTER TABLE holiday_days ADD COLUMN hebrew_day INTEGER", []);

        let tisha_default_applied: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![TISHA_BAV_CLOSED_DEFAULT_KEY],
                |row| row.get(0),
            )
            .ok();
        if tisha_default_applied.as_deref() != Some("1") {
            let _ = conn.execute(
                "UPDATE holiday_days
                 SET cancel_messages = 1, open_time = NULL, close_time = NULL
                 WHERE is_custom = 0 AND (title = 'תשעה באב' OR holiday_group = 'תשעה באב')",
                [],
            );
            let _ = conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, '1')
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![TISHA_BAV_CLOSED_DEFAULT_KEY],
            );
        }

        conn.execute(
            "INSERT OR IGNORE INTO schedules (id, name, is_active) VALUES (1, ?1, 1)",
            params![DEFAULT_SCHEDULE_NAME],
        )?;
        conn.execute(
            "UPDATE schedules SET name = ?1 WHERE name = 'יום לימודים'",
            params![DEFAULT_SCHEDULE_NAME],
        )?;
        conn.execute(
            "UPDATE tasks SET schedule_id = 1 WHERE schedule_id IS NULL",
            [],
        )?;
        conn.execute(
            "UPDATE tasks SET days_of_week = ?1 WHERE days_of_week IS NULL OR days_of_week = ''",
            params![ALL_DAYS],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            params![VOLUME_SETTING_KEY, DEFAULT_VOLUME.to_string()],
        )?;

        Self::migrate_legacy_holidays_cache(&conn)?;

        let _ = conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_tasks_scheduled_time ON tasks(scheduled_time);
             CREATE INDEX IF NOT EXISTS idx_tasks_is_active ON tasks(is_active);
             CREATE INDEX IF NOT EXISTS idx_system_messages_active ON system_messages(is_active);
             CREATE INDEX IF NOT EXISTS idx_holiday_days_hebrew
               ON holiday_days(hebrew_month, hebrew_day, holiday_group);
             CREATE INDEX IF NOT EXISTS idx_play_log_id ON play_log(id DESC);",
        );

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Release the on-disk DB file so it can be replaced (e.g. backup import).
    pub fn release_file_locks(&self) -> Result<()> {
        let mut guard = self.lock_conn()?;
        *guard = Connection::open_in_memory()?;
        tracing::info!(target: "db", "[db.release_locks] - success - swapped to in-memory connection");
        Ok(())
    }

    /// Flush WAL into the main DB file so a filesystem copy is consistent.
    pub fn checkpoint_for_backup(&self) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        tracing::info!(target: "db", "[db.checkpoint] - success - WAL truncated");
        Ok(())
    }

    /// Open `db_path` and swap it into this shared state after a file replace.
    pub fn reopen(&self, db_path: &std::path::Path) -> Result<()> {
        let conn = Connection::open(db_path)?;
        let _ = conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;",
        );
        let mut guard = self.lock_conn()?;
        *guard = conn;
        tracing::info!(target: "db", "[db.reopen] - success - path={}", db_path.display());
        Ok(())
    }

    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| {
            rusqlite::Error::InvalidParameterName("database lock poisoned".into())
        })
    }

    fn map_task_row(row: &rusqlite::Row<'_>) -> Result<Task> {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            file_path: row.get(2)?,
            scheduled_time: row.get(3)?,
            is_active: row.get::<_, i64>(4)? != 0,
            last_played_date: row.get(5)?,
            schedule_id: row.get(6)?,
            schedule_name: row.get(7)?,
            days_of_week: parse_days_of_week(&row.get::<_, String>(8)?),
            volume: row
                .get::<_, Option<f64>>(9)?
                .map(|value| value as f32),
            cancel_on_holiday: row.get::<_, i64>(10)? != 0,
        })
    }

    fn task_query_sql() -> &'static str {
        "SELECT t.id, t.title, t.file_path, t.scheduled_time, t.is_active, t.last_played_date,
                t.schedule_id, s.name, t.days_of_week, t.volume, t.cancel_on_holiday
         FROM tasks t
         LEFT JOIN schedules s ON t.schedule_id = s.id"
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get(0))?;

        match rows.next() {
            Some(value) => Ok(Some(value?)),
            None => Ok(None),
        }
    }

    pub fn clear_setting_key(&self, key: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn default_audio_volumes_from_legacy(&self) -> Result<crate::audio::AudioVolumes> {
        let mut volumes = crate::audio::AudioVolumes::default();
        if let Some(raw) = self.get_setting(VOLUME_SETTING_KEY)? {
            volumes.general = raw
                .parse::<f32>()
                .unwrap_or(DEFAULT_VOLUME)
                .clamp(0.0, 1.0);
        }
        Ok(volumes)
    }

    pub fn get_audio_volumes(&self) -> Result<crate::audio::AudioVolumes> {
        if let Some(raw) = self.get_setting(AUDIO_VOLUMES_KEY)? {
            if let Ok(parsed) = serde_json::from_str::<crate::audio::AudioVolumes>(&raw) {
                return Ok(parsed.clamp_all());
            }
        }

        let volumes = self.default_audio_volumes_from_legacy()?;
        let _ = self.set_audio_volumes(&volumes);
        Ok(volumes)
    }

    pub fn set_audio_volumes(
        &self,
        volumes: &crate::audio::AudioVolumes,
    ) -> Result<crate::audio::AudioVolumes> {
        let clamped = volumes.clone().clamp_all();
        let json = serde_json::to_string(&clamped).map_err(|error| {
            rusqlite::Error::InvalidParameterName(error.to_string())
        })?;
        self.set_setting(AUDIO_VOLUMES_KEY, &json)?;
        self.set_setting(VOLUME_SETTING_KEY, &clamped.general.to_string())?;
        Ok(clamped)
    }

    pub fn set_audio_volume_channel(
        &self,
        channel: &str,
        volume: f32,
    ) -> Result<crate::audio::AudioVolumes> {
        let channel = crate::audio::VolumeChannel::parse(Some(channel));
        let next = self.get_audio_volumes()?.with_channel(channel, volume);
        self.set_audio_volumes(&next)
    }

    pub fn get_lock_music_add(&self) -> Result<bool> {
        let value = self
            .get_setting(LOCK_MUSIC_ADD_KEY)?
            .unwrap_or_else(|| "false".to_string());
        Ok(value == "true" || value == "1")
    }

    pub fn set_lock_music_add(&self, enabled: bool) -> Result<bool> {
        self.set_setting(
            LOCK_MUSIC_ADD_KEY,
            if enabled { "true" } else { "false" },
        )?;
        Ok(enabled)
    }

    pub fn get_system_active(&self) -> Result<bool> {
        let value = self
            .get_setting(SYSTEM_ACTIVE_KEY)?
            .unwrap_or_else(|| "true".to_string());
        Ok(value != "false" && value != "0")
    }

    pub fn set_system_active(&self, active: bool) -> Result<bool> {
        self.set_setting(
            SYSTEM_ACTIVE_KEY,
            if active { "true" } else { "false" },
        )?;
        Ok(active)
    }

    pub fn get_day_before_erev_as_thursday(&self) -> Result<bool> {
        let value = self
            .get_setting(DAY_BEFORE_EREV_AS_THURSDAY_KEY)?
            .unwrap_or_else(|| "false".to_string());
        Ok(value == "true" || value == "1")
    }

    pub fn set_day_before_erev_as_thursday(&self, enabled: bool) -> Result<bool> {
        self.set_setting(
            DAY_BEFORE_EREV_AS_THURSDAY_KEY,
            if enabled { "true" } else { "false" },
        )?;
        Ok(enabled)
    }

        pub fn factory_reset(&self) -> Result<()> {
        tracing::warn!(target: "db", "[db.factory_reset] - started - clearing all tables");
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "DELETE FROM schedule_task_overrides;
             DELETE FROM schedule_day_overrides;
             DELETE FROM tasks;
             DELETE FROM system_messages;
             DELETE FROM play_log;
             DELETE FROM holiday_days;
             DELETE FROM settings;
             DELETE FROM schedules;",
        )?;

        conn.execute(
            "INSERT INTO schedules (id, name, is_active) VALUES (1, ?1, 1)",
            params![DEFAULT_SCHEDULE_NAME],
        )?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            params![VOLUME_SETTING_KEY, DEFAULT_VOLUME.to_string()],
        )?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, '1')",
            params![TISHA_BAV_CLOSED_DEFAULT_KEY],
        )?;

        let volumes = crate::audio::AudioVolumes::default();
        let json = serde_json::to_string(&volumes).map_err(|error| {
            rusqlite::Error::InvalidParameterName(error.to_string())
        })?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            params![AUDIO_VOLUMES_KEY, json],
        )?;

        tracing::info!(target: "db", "[db.factory_reset] - success - defaults reseeded");
        Ok(())
    }

    fn default_day_hours(open: &str, close: &str) -> OperatingDayHours {
        OperatingDayHours {
            open: open.to_string(),
            close: close.to_string(),
        }
    }

    fn default_season_operating_hours() -> SeasonOperatingHours {
        SeasonOperatingHours {
            sunday: Self::default_day_hours("00:00", "00:00"),
            monday: Self::default_day_hours("00:00", "00:00"),
            tuesday: Self::default_day_hours("00:00", "00:00"),
            wednesday: Self::default_day_hours("00:00", "00:00"),
            thursday: Self::default_day_hours("00:00", "00:00"),
            friday: Self::default_day_hours("00:00", "00:00"),
            motzei_shabbat: Self::default_day_hours("00:00", "00:00"),
        }
    }

    fn default_operating_hours_settings() -> OperatingHoursSettings {
        OperatingHoursSettings {
            winter: Self::default_season_operating_hours(),
            summer: Self::default_season_operating_hours(),
            temporary: TemporaryOperatingHours {
                hours: Self::default_season_operating_hours(),
                valid_from: None,
                valid_to: None,
            },
        }
    }

    fn normalize_day_hours(hours: &OperatingDayHours) -> Result<OperatingDayHours> {
        let open = hours.open.trim();
        let close = hours.close.trim();

        if NaiveTime::parse_from_str(open, "%H:%M").is_err() {
            return Err(rusqlite::Error::InvalidParameterName(
                "שעת פתיחה אינה תקינה".into(),
            ));
        }
        if NaiveTime::parse_from_str(close, "%H:%M").is_err() {
            return Err(rusqlite::Error::InvalidParameterName(
                "שעת סגירה אינה תקינה".into(),
            ));
        }

        Ok(OperatingDayHours {
            open: open.to_string(),
            close: close.to_string(),
        })
    }

    fn normalize_season_hours(season: &SeasonOperatingHours) -> Result<SeasonOperatingHours> {
        Ok(SeasonOperatingHours {
            sunday: Self::normalize_day_hours(&season.sunday)?,
            monday: Self::normalize_day_hours(&season.monday)?,
            tuesday: Self::normalize_day_hours(&season.tuesday)?,
            wednesday: Self::normalize_day_hours(&season.wednesday)?,
            thursday: Self::normalize_day_hours(&season.thursday)?,
            friday: Self::normalize_day_hours(&season.friday)?,
            motzei_shabbat: Self::normalize_day_hours(&season.motzei_shabbat)?,
        })
    }

    fn normalize_optional_date(value: &Option<String>) -> Result<Option<String>> {
        match value {
            None => Ok(None),
            Some(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return Ok(None);
                }
                if NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").is_err() {
                    return Err(rusqlite::Error::InvalidParameterName(
                        "תאריך אינו תקין".into(),
                    ));
                }
                Ok(Some(trimmed.to_string()))
            }
        }
    }

    fn normalize_temporary_hours(
        temporary: &TemporaryOperatingHours,
    ) -> Result<TemporaryOperatingHours> {
        let valid_from = Self::normalize_optional_date(&temporary.valid_from)?;
        let valid_to = Self::normalize_optional_date(&temporary.valid_to)?;

        if let (Some(from), Some(to)) = (&valid_from, &valid_to) {
            if from > to {
                return Err(rusqlite::Error::InvalidParameterName(
                    "תאריך ההתחלה חייב להיות לפני תאריך הסיום או שווה לו".into(),
                ));
            }
        }

        Ok(TemporaryOperatingHours {
            hours: Self::normalize_season_hours(&temporary.hours)?,
            valid_from,
            valid_to,
        })
    }

    pub fn get_operating_hours(&self) -> Result<OperatingHoursSettings> {
        if let Some(json) = self.get_setting(OPERATING_HOURS_KEY)? {
            if let Ok(parsed) = serde_json::from_str::<OperatingHoursSettings>(&json) {
                return Ok(OperatingHoursSettings {
                    winter: Self::normalize_season_hours(&parsed.winter)
                        .unwrap_or_else(|_| Self::default_season_operating_hours()),
                    summer: Self::normalize_season_hours(&parsed.summer)
                        .unwrap_or_else(|_| Self::default_season_operating_hours()),
                    temporary: Self::normalize_temporary_hours(&parsed.temporary)
                        .unwrap_or_else(|_| TemporaryOperatingHours {
                            hours: Self::default_season_operating_hours(),
                            valid_from: None,
                            valid_to: None,
                        }),
                });
            }
        }

        Ok(Self::default_operating_hours_settings())
    }

    pub fn set_operating_hours(
        &self,
        settings: &OperatingHoursSettings,
    ) -> Result<OperatingHoursSettings> {
        let normalized = OperatingHoursSettings {
            winter: Self::normalize_season_hours(&settings.winter)?,
            summer: Self::normalize_season_hours(&settings.summer)?,
            temporary: Self::normalize_temporary_hours(&settings.temporary)?,
        };

        let json = serde_json::to_string(&normalized).map_err(|error| {
            rusqlite::Error::InvalidParameterName(error.to_string())
        })?;
        self.set_setting(OPERATING_HOURS_KEY, &json)?;
        Ok(normalized)
    }

    fn default_emergency_message_settings() -> HashMap<String, bool> {
        EMERGENCY_MESSAGE_TYPE_IDS
            .iter()
            .map(|id| ((*id).to_string(), false))
            .collect()
    }

    fn emergency_audio_exists(path: &str) -> bool {
        !path.is_empty() && std::path::Path::new(path).is_file()
    }

    fn persist_emergency_message_settings(
        &self,
        settings: &HashMap<String, bool>,
    ) -> Result<()> {
        let json = serde_json::to_string(settings).map_err(|error| {
            rusqlite::Error::InvalidParameterName(error.to_string())
        })?;
        self.set_setting(EMERGENCY_MESSAGES_ENABLED_KEY, &json)
    }

    fn get_stored_emergency_message_audio_paths(&self) -> Result<HashMap<String, String>> {
        let mut paths = HashMap::new();

        if let Some(json) = self.get_setting(EMERGENCY_MESSAGE_AUDIO_KEY)? {
            if let Ok(parsed) = serde_json::from_str::<HashMap<String, String>>(&json) {
                for id in EMERGENCY_MESSAGE_TYPE_IDS {
                    if let Some(path) = parsed.get(*id) {
                        if !path.is_empty() {
                            paths.insert((*id).to_string(), path.clone());
                        }
                    }
                }
            }
        }

        Ok(paths)
    }

    fn get_valid_emergency_message_audio_paths(&self) -> Result<HashMap<String, String>> {
        Ok(self
            .get_stored_emergency_message_audio_paths()?
            .into_iter()
            .filter(|(_, path)| Self::emergency_audio_exists(path))
            .collect())
    }

    pub fn get_emergency_message_settings(&self) -> Result<HashMap<String, bool>> {
        let valid_audio = self.get_valid_emergency_message_audio_paths()?;
        let mut settings = Self::default_emergency_message_settings();

        if let Some(json) = self.get_setting(EMERGENCY_MESSAGES_ENABLED_KEY)? {
            if let Ok(parsed) = serde_json::from_str::<HashMap<String, bool>>(&json) {
                for id in EMERGENCY_MESSAGE_TYPE_IDS {
                    if let Some(enabled) = parsed.get(*id) {
                        settings.insert((*id).to_string(), *enabled);
                    }
                }
            }
        }

        let mut changed = false;
        for id in EMERGENCY_MESSAGE_TYPE_IDS {
            if *id == "unconfigured" {
                continue;
            }

            if settings.get(*id) == Some(&true) && !valid_audio.contains_key(*id) {
                settings.insert((*id).to_string(), false);
                changed = true;
            }
        }

        if changed {
            self.persist_emergency_message_settings(&settings)?;
        }

        Ok(settings)
    }

    pub fn set_emergency_message_enabled(
        &self,
        message_type: &str,
        enabled: bool,
    ) -> Result<HashMap<String, bool>> {
        if !EMERGENCY_MESSAGE_TYPE_IDS.contains(&message_type) {
            return Err(rusqlite::Error::InvalidParameterName(
                "invalid emergency message type".into(),
            ));
        }

        if enabled && message_type != "unconfigured" {
            let valid_audio = self.get_valid_emergency_message_audio_paths()?;
            if !valid_audio.contains_key(message_type) {
                return Err(rusqlite::Error::InvalidParameterName(
                    "אין אפשרות להפעיל את ההודעה בלי קובץ שמע".into(),
                ));
            }
        }

        let mut settings = self.get_emergency_message_settings()?;
        settings.insert(message_type.to_string(), enabled);
        self.persist_emergency_message_settings(&settings)?;
        Ok(settings)
    }

    pub fn get_emergency_message_audio_path(
        &self,
        message_type: &str,
    ) -> Result<Option<String>> {
        Ok(self
            .get_stored_emergency_message_audio_paths()?
            .get(message_type)
            .cloned())
    }

    pub fn get_emergency_message_audio_files(&self) -> Result<Vec<EmergencyMessageAudioFile>> {
        let paths = self.get_valid_emergency_message_audio_paths()?;

        Ok(EMERGENCY_MESSAGE_TYPE_IDS
            .iter()
            .map(|id| {
                let path = paths.get(*id).cloned();
                let name = path.as_ref().and_then(|value| {
                    std::path::Path::new(value)
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().to_string())
                });

                EmergencyMessageAudioFile {
                    message_type: (*id).to_string(),
                    name,
                    path,
                }
            })
            .collect())
    }

    pub fn set_emergency_message_audio_path(
        &self,
        message_type: &str,
        path: &str,
    ) -> Result<EmergencyMessageAudioFile> {
        if !EMERGENCY_MESSAGE_TYPE_IDS.contains(&message_type) {
            return Err(rusqlite::Error::InvalidParameterName(
                "invalid emergency message type".into(),
            ));
        }

        let mut paths = self.get_stored_emergency_message_audio_paths()?;
        paths.insert(message_type.to_string(), path.to_string());
        let json = serde_json::to_string(&paths).map_err(|error| {
            rusqlite::Error::InvalidParameterName(error.to_string())
        })?;
        self.set_setting(EMERGENCY_MESSAGE_AUDIO_KEY, &json)?;

        Ok(EmergencyMessageAudioFile {
            message_type: message_type.to_string(),
            name: std::path::Path::new(path)
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string()),
            path: Some(path.to_string()),
        })
    }

    pub const DEFAULT_EMERGENCY_MONITORED_CITY: &str = "ירושלים - מרכז";

    fn default_emergency_monitored_cities() -> Vec<String> {
        vec![Self::DEFAULT_EMERGENCY_MONITORED_CITY.to_string()]
    }

    pub fn get_emergency_monitored_cities(&self) -> Result<Vec<String>> {
        let Some(json) = self.get_setting(EMERGENCY_MONITORED_CITIES_KEY)? else {
            let default = Self::default_emergency_monitored_cities();
            self.set_emergency_monitored_cities(&default)?;
            return Ok(default);
        };

        let parsed: Vec<String> = serde_json::from_str(&json).map_err(|error| {
            rusqlite::Error::InvalidParameterName(error.to_string())
        })?;

        let normalized: Vec<String> = parsed
            .into_iter()
            .map(|city| city.trim().to_string())
            .filter(|city| !city.is_empty())
            .collect();

        let single = match normalized.as_slice() {
            [] => Self::default_emergency_monitored_cities(),
            [only] => vec![only.clone()],
            [first, ..] => vec![first.clone()],
        };

        if normalized.len() != 1 {
            self.set_emergency_monitored_cities(&single)?;
        }

        Ok(single)
    }

    pub fn set_emergency_monitored_cities(&self, cities: &[String]) -> Result<Vec<String>> {
        let city = cities
            .iter()
            .map(|value| value.trim())
            .find(|value| !value.is_empty())
            .unwrap_or(Self::DEFAULT_EMERGENCY_MONITORED_CITY)
            .to_string();

        let normalized = vec![city];
        let json = serde_json::to_string(&normalized).map_err(|error| {
            rusqlite::Error::InvalidParameterName(error.to_string())
        })?;
        self.set_setting(EMERGENCY_MONITORED_CITIES_KEY, &json)?;
        Ok(normalized)
    }

    fn system_message_audio_name(file_path: &str) -> Option<String> {
        std::path::Path::new(file_path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
    }

    fn map_system_message_row(row: &rusqlite::Row<'_>) -> Result<SystemMessage> {
        let file_path: String = row.get(2)?;
        Ok(SystemMessage {
            id: row.get(0)?,
            title: row.get(1)?,
            file_path: file_path.clone(),
            audio_name: Self::system_message_audio_name(&file_path),
            is_active: row.get::<_, i64>(3)? != 0,
            days_of_week: parse_system_message_days(&row.get::<_, String>(4)?),
            schedule_mode: row.get(5)?,
            scheduled_time: row.get(6)?,
            operating_anchor: row.get(7)?,
            offset_direction: row.get(8)?,
            offset_minutes: row.get(9)?,
            last_played_date: row.get(10)?,
        })
    }

    fn validate_system_message_schedule(
        schedule_mode: &str,
        scheduled_time: Option<&str>,
        operating_anchor: Option<&str>,
        offset_direction: Option<&str>,
        offset_minutes: Option<i64>,
    ) -> Result<()> {
        match schedule_mode {
            SYSTEM_MESSAGE_SCHEDULE_FIXED => {
                let time = scheduled_time
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        rusqlite::Error::InvalidParameterName(
                            "יש לבחור שעת השמעה".into(),
                        )
                    })?;

                if NaiveTime::parse_from_str(time, "%H:%M").is_err() {
                    return Err(rusqlite::Error::InvalidParameterName(
                        "שעת השמעה אינה תקינה".into(),
                    ));
                }
            }
            SYSTEM_MESSAGE_SCHEDULE_RELATIVE => {
                let anchor = operating_anchor
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        rusqlite::Error::InvalidParameterName(
                            "יש לבחור שעת פתיחה או סגירה".into(),
                        )
                    })?;

                if anchor != "open" && anchor != "close" {
                    return Err(rusqlite::Error::InvalidParameterName(
                        "שעת ייחוס לא תקינה".into(),
                    ));
                }

                let direction = offset_direction
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        rusqlite::Error::InvalidParameterName(
                            "יש לבחור לפני או אחרי".into(),
                        )
                    })?;

                if direction != "before" && direction != "after" {
                    return Err(rusqlite::Error::InvalidParameterName(
                        "כיוון זמן לא תקין".into(),
                    ));
                }

                let minutes = offset_minutes.unwrap_or(0);
                if minutes <= 0 {
                    return Err(rusqlite::Error::InvalidParameterName(
                        "יש להזין מספר דקות חיובי".into(),
                    ));
                }
            }
            _ => {
                return Err(rusqlite::Error::InvalidParameterName(
                    "סוג תזמון לא תקין".into(),
                ));
            }
        }

        Ok(())
    }

    pub fn get_system_messages(&self) -> Result<Vec<SystemMessage>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, file_path, is_active, days_of_week, schedule_mode, scheduled_time,
                    operating_anchor, offset_direction, offset_minutes, last_played_date
             FROM system_messages
             ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], Self::map_system_message_row)?;

        rows.collect()
    }

    pub fn get_active_system_messages(&self) -> Result<Vec<SystemMessage>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, file_path, is_active, days_of_week, schedule_mode, scheduled_time,
                    operating_anchor, offset_direction, offset_minutes, last_played_date
             FROM system_messages
             WHERE is_active = 1
             ORDER BY id",
        )?;
        let rows = stmt.query_map([], Self::map_system_message_row)?;

        rows.collect()
    }

    pub fn get_system_message(&self, id: i64) -> Result<Option<SystemMessage>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, file_path, is_active, days_of_week, schedule_mode, scheduled_time,
                    operating_anchor, offset_direction, offset_minutes, last_played_date
             FROM system_messages
             WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_system_message_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn add_system_message(
        &self,
        title: &str,
        file_path: &str,
        days_of_week: &[u8],
        schedule_mode: &str,
        scheduled_time: Option<&str>,
        operating_anchor: Option<&str>,
        offset_direction: Option<&str>,
        offset_minutes: Option<i64>,
    ) -> Result<SystemMessage> {
        let title = title.trim();
        if title.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "יש להזין כותרת להודעה".into(),
            ));
        }

        if file_path.trim().is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "יש לבחור קובץ שמע".into(),
            ));
        }

        if days_of_week.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "יש לבחור לפחות יום אחד".into(),
            ));
        }

        if days_of_week.iter().any(|day| *day > SYSTEM_MESSAGE_MAX_DAY) {
            return Err(rusqlite::Error::InvalidParameterName(
                "יום לא תקין נבחר".into(),
            ));
        }

        Self::validate_system_message_schedule(
            schedule_mode,
            scheduled_time,
            operating_anchor,
            offset_direction,
            offset_minutes,
        )?;

        let days = format_days_of_week(days_of_week);
        let stored_scheduled_time = if schedule_mode == SYSTEM_MESSAGE_SCHEDULE_FIXED {
            scheduled_time.map(str::trim).map(str::to_string)
        } else {
            None
        };
        let stored_anchor = if schedule_mode == SYSTEM_MESSAGE_SCHEDULE_RELATIVE {
            operating_anchor.map(str::trim).map(str::to_string)
        } else {
            None
        };
        let stored_direction = if schedule_mode == SYSTEM_MESSAGE_SCHEDULE_RELATIVE {
            offset_direction.map(str::trim).map(str::to_string)
        } else {
            None
        };
        let stored_offset_minutes = if schedule_mode == SYSTEM_MESSAGE_SCHEDULE_RELATIVE {
            offset_minutes
        } else {
            None
        };

        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO system_messages
             (title, file_path, is_active, days_of_week, schedule_mode, scheduled_time,
              operating_anchor, offset_direction, offset_minutes)
             VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                title,
                file_path,
                days,
                schedule_mode,
                stored_scheduled_time,
                stored_anchor,
                stored_direction,
                stored_offset_minutes
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(SystemMessage {
            id,
            title: title.to_string(),
            file_path: file_path.to_string(),
            audio_name: Self::system_message_audio_name(file_path),
            is_active: true,
            days_of_week: days_of_week.to_vec(),
            schedule_mode: schedule_mode.to_string(),
            scheduled_time: stored_scheduled_time,
            operating_anchor: stored_anchor,
            offset_direction: stored_direction,
            offset_minutes: stored_offset_minutes,
            last_played_date: None,
        })
    }

    pub fn update_system_message(
        &self,
        id: i64,
        title: &str,
        days_of_week: &[u8],
        schedule_mode: &str,
        scheduled_time: Option<&str>,
        operating_anchor: Option<&str>,
        offset_direction: Option<&str>,
        offset_minutes: Option<i64>,
    ) -> Result<SystemMessage> {
        let existing = self.get_system_message(id)?.ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("הודעת המערכת לא נמצאה".into())
        })?;

        let title = title.trim();
        if title.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "יש להזין כותרת להודעה".into(),
            ));
        }

        if days_of_week.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "יש לבחור לפחות יום אחד".into(),
            ));
        }

        if days_of_week.iter().any(|day| *day > SYSTEM_MESSAGE_MAX_DAY) {
            return Err(rusqlite::Error::InvalidParameterName(
                "יום לא תקין נבחר".into(),
            ));
        }

        Self::validate_system_message_schedule(
            schedule_mode,
            scheduled_time,
            operating_anchor,
            offset_direction,
            offset_minutes,
        )?;

        let days = format_days_of_week(days_of_week);
        let stored_scheduled_time = if schedule_mode == SYSTEM_MESSAGE_SCHEDULE_FIXED {
            scheduled_time.map(str::trim).map(str::to_string)
        } else {
            None
        };
        let stored_anchor = if schedule_mode == SYSTEM_MESSAGE_SCHEDULE_RELATIVE {
            operating_anchor.map(str::trim).map(str::to_string)
        } else {
            None
        };
        let stored_direction = if schedule_mode == SYSTEM_MESSAGE_SCHEDULE_RELATIVE {
            offset_direction.map(str::trim).map(str::to_string)
        } else {
            None
        };
        let stored_offset_minutes = if schedule_mode == SYSTEM_MESSAGE_SCHEDULE_RELATIVE {
            offset_minutes
        } else {
            None
        };

        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE system_messages
             SET title = ?1, days_of_week = ?2, schedule_mode = ?3, scheduled_time = ?4,
                 operating_anchor = ?5, offset_direction = ?6, offset_minutes = ?7
             WHERE id = ?8",
            params![
                title,
                days,
                schedule_mode,
                stored_scheduled_time,
                stored_anchor,
                stored_direction,
                stored_offset_minutes,
                id
            ],
        )?;

        Ok(SystemMessage {
            title: title.to_string(),
            days_of_week: days_of_week.to_vec(),
            schedule_mode: schedule_mode.to_string(),
            scheduled_time: stored_scheduled_time,
            operating_anchor: stored_anchor,
            offset_direction: stored_direction,
            offset_minutes: stored_offset_minutes,
            ..existing
        })
    }

    pub fn set_system_message_enabled(&self, id: i64, enabled: bool) -> Result<SystemMessage> {
        let message = self.get_system_message(id)?.ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("הודעת המערכת לא נמצאה".into())
        })?;

        if enabled && message.file_path.trim().is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "אין אפשרות להפעיל את ההודעה בלי קובץ שמע".into(),
            ));
        }

        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE system_messages SET is_active = ?1 WHERE id = ?2",
            params![enabled as i64, id],
        )?;

        Ok(SystemMessage {
            is_active: enabled,
            ..message
        })
    }

    pub fn update_system_message_audio_path(
        &self,
        id: i64,
        file_path: &str,
    ) -> Result<SystemMessage> {
        if file_path.trim().is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "יש לבחור קובץ שמע".into(),
            ));
        }

        let message = self.get_system_message(id)?.ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("הודעת המערכת לא נמצאה".into())
        })?;

        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE system_messages SET file_path = ?1 WHERE id = ?2",
            params![file_path, id],
        )?;

        Ok(SystemMessage {
            file_path: file_path.to_string(),
            audio_name: Self::system_message_audio_name(file_path),
            ..message
        })
    }

    pub fn delete_system_message(&self, id: i64) -> Result<Option<String>> {
        let message = match self.get_system_message(id)? {
            Some(message) => message,
            None => return Ok(None),
        };

        let conn = self.lock_conn()?;
        conn.execute("DELETE FROM system_messages WHERE id = ?1", params![id])?;
        Ok(Some(message.file_path))
    }

    pub fn mark_system_message_played(&self, id: i64, date: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE system_messages SET last_played_date = ?1 WHERE id = ?2",
            params![date, id],
        )?;
        Ok(())
    }

    pub fn get_due_system_messages(
        &self,
        lookback_minutes: i64,
        operational_date: &str,
        hebrew_date: &str,
        weekday: u8,
    ) -> Result<Vec<SystemMessage>> {
        let now = Local::now();
        let window_start = now - Duration::minutes(lookback_minutes);
        let operating_hours = self.get_operating_hours()?;
        let holiday = self.get_holiday_day(hebrew_date)?;
        let holiday_ref = holiday.as_ref();
        let tomorrow_is_erev =
            crate::system_message_schedule::tomorrow_is_erev_chag(self, operational_date);
        let day_before_erev_as_thursday = self.get_day_before_erev_as_thursday().unwrap_or(false);
        let Some(op_date) = crate::operational_day::parse_operational_date(operational_date) else {
            return Ok(Vec::new());
        };

        let mut due = Vec::new();
        for message in self.get_active_system_messages()? {
            if message.file_path.trim().is_empty() {
                continue;
            }
            if message.last_played_date.as_deref() == Some(operational_date) {
                continue;
            }
            if holiday_ref.is_some_and(|h| h.cancel_messages) {
                continue;
            }
            let treat_as_thursday = crate::system_message_schedule::treat_day_as_thursday(
                day_before_erev_as_thursday,
                tomorrow_is_erev,
            );
            if !crate::system_message_schedule::matches_operational_day(
                &message,
                weekday,
                holiday_ref,
                day_before_erev_as_thursday,
                tomorrow_is_erev,
            ) {
                continue;
            }
            let Some(play_time) = crate::system_message_schedule::resolve_play_time(
                &message,
                operational_date,
                weekday,
                &operating_hours,
                holiday_ref,
                treat_as_thursday,
            ) else {
                continue;
            };
            let Some(play_dt) =
                crate::operational_day::scheduled_wall_datetime(&play_time, op_date)
            else {
                continue;
            };
            if play_dt >= window_start && play_dt <= now {
                due.push(message);
            }
        }
        due.sort_by_key(|a| a.id);
        Ok(due)
    }

    fn migrate_legacy_holidays_cache(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![HOLIDAYS_CACHE_KEY], |row| row.get::<_, String>(0))?;
        let Some(raw) = rows.next() else {
            return Ok(());
        };
        let raw = raw?;

        #[derive(Deserialize)]
        struct LegacyDatesOnly {
            dates: Vec<String>,
        }

        if let Ok(cache) = serde_json::from_str::<LegacyHolidayCache>(&raw) {
            for holiday in cache.holidays {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO holiday_days
                     (date, title, holiday_group, day_label, hebrew, cancel_messages, is_custom)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
                    params![
                        holiday.date,
                        holiday.title,
                        holiday.title,
                        "חג",
                        holiday.hebrew,
                        if holiday.cancel_messages { 1 } else { 0 },
                    ],
                );
            }
        } else if let Ok(legacy) = serde_json::from_str::<LegacyDatesOnly>(&raw) {
            for date in legacy.dates {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO holiday_days
                     (date, title, holiday_group, day_label, cancel_messages, is_custom)
                     VALUES (?1, ?2, ?2, 'חג', 1, 0)",
                    params![date, "חג"],
                );
            }
        }

        let _ = conn.execute(
            "DELETE FROM settings WHERE key = ?1",
            params![HOLIDAYS_CACHE_KEY],
        );
        Ok(())
    }

    fn map_holiday_row(row: &rusqlite::Row<'_>) -> Result<HolidayEntry> {
        Ok(HolidayEntry {
            date: row.get(0)?,
            title: row.get(1)?,
            holiday_group: row.get(2)?,
            day_label: row.get(3)?,
            hebrew: row.get(4)?,
            cancel_messages: row.get::<_, i64>(5)? != 0,
            is_custom: row.get::<_, i64>(6)? != 0,
            open_time: row.get(7)?,
            close_time: row.get(8)?,
            hebrew_month: row.get(9)?,
            hebrew_day: row.get(10)?,
        })
    }

    pub fn get_holiday_days_for_year(&self, year: i32) -> Result<Vec<HolidayEntry>> {
        let conn = self.lock_conn()?;
        let prefix = format!("{year}-%");
        let mut stmt = conn.prepare(
            "SELECT date, title, holiday_group, day_label, hebrew, cancel_messages, is_custom,
                    open_time, close_time, hebrew_month, hebrew_day
             FROM holiday_days WHERE date LIKE ?1 ORDER BY date",
        )?;

        let holidays = stmt
            .query_map(params![prefix], Self::map_holiday_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(holidays)
    }

    pub fn get_holiday_days_for_years(&self, years: &[i32]) -> Result<Vec<HolidayEntry>> {
        let mut all = Vec::new();
        for year in years {
            all.extend(self.get_holiday_days_for_year(*year)?);
        }
        all.sort_by(|a, b| a.date.cmp(&b.date));
        all.dedup_by(|a, b| a.date == b.date);
        Ok(all)
    }

    pub fn get_holiday_day(&self, date: &str) -> Result<Option<HolidayEntry>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT date, title, holiday_group, day_label, hebrew, cancel_messages, is_custom,
                    open_time, close_time, hebrew_month, hebrew_day
             FROM holiday_days WHERE date = ?1",
        )?;

        let mut rows = stmt.query_map(params![date], Self::map_holiday_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn find_latest_holiday_by_hebrew(
        &self,
        hebrew_month: &str,
        hebrew_day: i32,
        holiday_group: &str,
    ) -> Result<Option<HolidayEntry>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT date, title, holiday_group, day_label, hebrew, cancel_messages, is_custom,
                    open_time, close_time, hebrew_month, hebrew_day
             FROM holiday_days
             WHERE hebrew_month = ?1 AND hebrew_day = ?2 AND holiday_group = ?3
             ORDER BY date DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query_map(
            params![hebrew_month, hebrew_day, holiday_group],
            Self::map_holiday_row,
        )?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn upsert_hebcal_holiday(&self, entry: &HolidayEntry) -> Result<()> {
        if let Some(existing) = self.get_holiday_day(&entry.date)? {
            if existing.is_custom {
                return Ok(());
            }

            let conn = self.lock_conn()?;
            conn.execute(
                "UPDATE holiday_days
                 SET title = ?1, holiday_group = ?2, hebrew = ?3,
                     hebrew_month = COALESCE(?4, hebrew_month),
                     hebrew_day = COALESCE(?5, hebrew_day)
                 WHERE date = ?6",
                params![
                    entry.title,
                    entry.holiday_group,
                    entry.hebrew,
                    entry.hebrew_month,
                    entry.hebrew_day,
                    entry.date,
                ],
            )?;
            return Ok(());
        }

        let mut cancel_messages = entry.cancel_messages;
        let mut open_time = entry.open_time.clone();
        let mut close_time = entry.close_time.clone();
        let mut day_label = entry.day_label.clone();

        if let (Some(month), Some(day)) = (&entry.hebrew_month, entry.hebrew_day) {
            if let Some(previous) =
                self.find_latest_holiday_by_hebrew(month, day, &entry.holiday_group)?
            {
                cancel_messages = previous.cancel_messages;
                open_time = previous.open_time;
                close_time = previous.close_time;
                day_label = previous.day_label;
            }
        }

        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO holiday_days
             (date, title, holiday_group, day_label, hebrew, cancel_messages, is_custom,
              open_time, close_time, hebrew_month, hebrew_day)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, ?9, ?10)",
            params![
                entry.date,
                entry.title,
                entry.holiday_group,
                day_label,
                entry.hebrew,
                cancel_messages as i64,
                open_time,
                close_time,
                entry.hebrew_month,
                entry.hebrew_day,
            ],
        )?;
        Ok(())
    }

    pub fn set_holiday_status(
        &self,
        date: &str,
        day_label: &str,
        cancel_messages: bool,
        open_time: Option<&str>,
        close_time: Option<&str>,
    ) -> Result<()> {
        let Some(existing) = self.get_holiday_day(date)? else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        };

        let conn = self.lock_conn()?;
        if let (Some(month), Some(day)) = (existing.hebrew_month, existing.hebrew_day) {
            conn.execute(
                "UPDATE holiday_days
                 SET day_label = ?1, cancel_messages = ?2, open_time = ?3, close_time = ?4
                 WHERE hebrew_month = ?5 AND hebrew_day = ?6 AND holiday_group = ?7",
                params![
                    day_label,
                    cancel_messages as i64,
                    open_time,
                    close_time,
                    month,
                    day,
                    existing.holiday_group,
                ],
            )?;
        } else {
            let updated = conn.execute(
                "UPDATE holiday_days
                 SET day_label = ?1, cancel_messages = ?2, open_time = ?3, close_time = ?4
                 WHERE date = ?5",
                params![
                    day_label,
                    cancel_messages as i64,
                    open_time,
                    close_time,
                    date,
                ],
            )?;
            if updated == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
        }
        Ok(())
    }

    pub fn add_custom_holiday_day(&self, entry: &HolidayEntry) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO holiday_days
             (date, title, holiday_group, day_label, hebrew, cancel_messages, is_custom,
              open_time, close_time, hebrew_month, hebrew_day)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9, ?10)
             ON CONFLICT(date) DO UPDATE SET
               title = excluded.title,
               holiday_group = excluded.holiday_group,
               day_label = excluded.day_label,
               hebrew = excluded.hebrew,
               cancel_messages = excluded.cancel_messages,
               is_custom = 1,
               open_time = excluded.open_time,
               close_time = excluded.close_time,
               hebrew_month = excluded.hebrew_month,
               hebrew_day = excluded.hebrew_day",
            params![
                entry.date,
                entry.title,
                entry.holiday_group,
                entry.day_label,
                entry.hebrew,
                entry.cancel_messages as i64,
                entry.open_time,
                entry.close_time,
                entry.hebrew_month,
                entry.hebrew_day,
            ],
        )?;
        Ok(())
    }

        pub fn ensure_custom_recurrence(&self, entry: &HolidayEntry) -> Result<()> {
        if let Some(existing) = self.get_holiday_day(&entry.date)? {
            if !existing.is_custom {
                return Ok(());
            }
            if existing.hebrew_month.is_some() && existing.hebrew_day.is_some() {
                return Ok(());
            }
            let conn = self.lock_conn()?;
            conn.execute(
                "UPDATE holiday_days
                 SET hebrew_month = COALESCE(hebrew_month, ?1),
                     hebrew_day = COALESCE(hebrew_day, ?2)
                 WHERE date = ?3 AND is_custom = 1",
                params![entry.hebrew_month, entry.hebrew_day, entry.date],
            )?;
            return Ok(());
        }

        self.add_custom_holiday_day(entry)
    }

    pub fn remove_stale_hebcal_holidays(&self, year: i32, valid_dates: &[String]) -> Result<()> {
        let conn = self.lock_conn()?;
        let prefix = format!("{year}-%");
        let mut stmt = conn.prepare(
            "SELECT date FROM holiday_days WHERE date LIKE ?1 AND is_custom = 0",
        )?;
        let dates: Vec<String> = stmt
            .query_map(params![prefix], |row| row.get(0))?
            .collect::<Result<Vec<_>>>()?;

        let valid: std::collections::HashSet<&str> =
            valid_dates.iter().map(String::as_str).collect();
        for date in dates {
            if !valid.contains(date.as_str()) {
                conn.execute("DELETE FROM holiday_days WHERE date = ?1", params![date])?;
            }
        }
        Ok(())
    }

    pub fn is_calendar_synced_on(&self, date: &str) -> Result<bool> {
        Ok(self.get_setting(HOLIDAYS_SYNCED_ON_KEY)?.as_deref() == Some(date))
    }

    pub fn mark_calendar_synced_on(&self, date: &str) -> Result<()> {
        self.set_setting(HOLIDAYS_SYNCED_ON_KEY, date)
    }

    pub fn delete_holiday_day(&self, date: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let deleted = conn.execute(
            "DELETE FROM holiday_days WHERE date = ?1 AND is_custom = 1",
            params![date],
        )?;
        if deleted == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

        pub fn delete_custom_holiday_recurring(&self, date: &str) -> Result<()> {
        let Some(existing) = self.get_holiday_day(date)? else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        };
        if !existing.is_custom {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        let conn = self.lock_conn()?;
        if let (Some(month), Some(day)) = (existing.hebrew_month, existing.hebrew_day) {
            let deleted = conn.execute(
                "DELETE FROM holiday_days
                 WHERE is_custom = 1
                   AND hebrew_month = ?1
                   AND hebrew_day = ?2
                   AND holiday_group = ?3",
                params![month, day, existing.holiday_group],
            )?;
            if deleted == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            return Ok(());
        }

        drop(conn);
        self.delete_holiday_day(date)
    }

    pub fn get_all_schedules(&self) -> Result<Vec<Schedule>> {
        let conn = self.lock_conn()?;
        let mut stmt =
            conn.prepare("SELECT id, name, is_active FROM schedules ORDER BY name")?;

        let schedules = stmt
            .query_map([], |row| {
                Ok(Schedule {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    is_active: row.get::<_, i64>(2)? != 0,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(schedules)
    }

    pub fn get_all_tasks(&self) -> Result<Vec<Task>> {
        let conn = self.lock_conn()?;
        let sql = format!("{} ORDER BY t.scheduled_time, t.title", Self::task_query_sql());
        let mut stmt = conn.prepare(&sql)?;

        let tasks = stmt
            .query_map([], Self::map_task_row)?
            .collect::<Result<Vec<_>>>()?;

        Ok(tasks)
    }

    fn schedule_is_active(&self, schedule_id: Option<i64>) -> Result<bool> {
        let Some(schedule_id) = schedule_id else {
            return Ok(true);
        };

        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT is_active FROM schedules WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![schedule_id], |row| row.get::<_, i64>(0))?;

        match rows.next() {
            Some(value) => Ok(value? != 0),
            None => Ok(true),
        }
    }

    pub fn task_is_playable_today(
        &self,
        task: &Task,
        weekday: u8,
        treat_as_thursday: bool,
    ) -> Result<bool> {
        if !task.is_active {
            return Ok(false);
        }

        if !crate::system_message_schedule::matches_weekdays(
            &task.days_of_week,
            weekday,
            treat_as_thursday,
        ) {
            return Ok(false);
        }

        self.schedule_is_active(task.schedule_id)
    }

    pub fn get_due_tasks(
        &self,
        lookback_minutes: i64,
        today: &str,
        weekday: u8,
    ) -> Result<Vec<Task>> {
        let now = Local::now();
        let window_start = now - Duration::minutes(lookback_minutes);
        let Some(op_date) = crate::operational_day::parse_operational_date(today) else {
            return Ok(Vec::new());
        };
        let treat_as_thursday = crate::system_message_schedule::treat_day_as_thursday(
            self.get_day_before_erev_as_thursday().unwrap_or(false),
            crate::system_message_schedule::tomorrow_is_erev_chag(self, today),
        );
        let conn = self.lock_conn()?;
        let sql = format!(
            "{} WHERE t.is_active = 1
                  AND (t.last_played_date IS NULL OR t.last_played_date != ?1)
                  AND (s.is_active IS NULL OR s.is_active = 1)
             ORDER BY t.scheduled_time, t.id",
            Self::task_query_sql()
        );
        let mut stmt = conn.prepare(&sql)?;

        let tasks = stmt
            .query_map(params![today], Self::map_task_row)?
            .collect::<Result<Vec<_>>>()?;

        let mut due = Vec::new();
        for task in tasks {
            if !crate::system_message_schedule::matches_weekdays(
                &task.days_of_week,
                weekday,
                treat_as_thursday,
            ) {
                continue;
            }
            let Some(scheduled_dt) =
                crate::operational_day::scheduled_wall_datetime(&task.scheduled_time, op_date)
            else {
                continue;
            };
            if scheduled_dt >= window_start && scheduled_dt <= now {
                due.push(task);
            }
        }
        Ok(due)
    }

    pub fn get_missed_tasks(&self, window_minutes: i64) -> Result<Vec<Task>> {
        let now = Local::now();
        let operational_date = crate::operational_day::operational_date_string(now);
        let weekday = crate::operational_day::operational_weekday(now);
        let window_start = now - Duration::minutes(window_minutes);
        let Some(op_date) = crate::operational_day::parse_operational_date(&operational_date)
        else {
            return Ok(Vec::new());
        };
        let treat_as_thursday = crate::system_message_schedule::treat_day_as_thursday(
            self.get_day_before_erev_as_thursday().unwrap_or(false),
            crate::system_message_schedule::tomorrow_is_erev_chag(self, &operational_date),
        );

        let mut missed = Vec::new();

        for task in self.get_all_tasks()? {
            if !self.task_is_playable_today(&task, weekday, treat_as_thursday)? {
                continue;
            }

            if task.last_played_date.as_deref() == Some(operational_date.as_str()) {
                continue;
            }

            let Some(scheduled_dt) =
                crate::operational_day::scheduled_wall_datetime(&task.scheduled_time, op_date)
            else {
                continue;
            };
            if scheduled_dt > window_start && scheduled_dt < now {
                missed.push(task);
            }
        }

        Ok(missed)
    }

    pub fn mark_played(&self, task_id: i64, date: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE tasks SET last_played_date = ?1 WHERE id = ?2",
            params![date, task_id],
        )?;
        Ok(())
    }

    pub fn log_play(&self, task_id: Option<i64>, task_title: &str, status: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let played_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO play_log (task_id, task_title, played_at, status)
             VALUES (?1, ?2, ?3, ?4)",
            params![task_id, task_title, played_at, status],
        )?;
        Ok(())
    }

    pub fn get_play_log(&self, limit: i64) -> Result<Vec<PlayLogEntry>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, task_id, task_title, played_at, status
             FROM play_log ORDER BY id DESC LIMIT ?1",
        )?;

        let entries = stmt
            .query_map(params![limit], |row| {
                Ok(PlayLogEntry {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    task_title: row.get(2)?,
                    played_at: row.get(3)?,
                    status: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(entries)
    }

    pub fn count_emergency_plays_on_operational_date(&self) -> Result<u64> {
        let now = Local::now();
        let operational_date = crate::operational_day::operational_date_string(now);
        let next_calendar = {
            let date = crate::operational_day::parse_operational_date(&operational_date)
                .unwrap_or_else(|| now.date_naive());
            (date + Duration::days(1))
                .format("%Y-%m-%d")
                .to_string()
        };

        // Operational day runs until 02:00 next calendar morning.
        let window_start = format!("{operational_date} 02:00:00");
        let window_end = format!("{next_calendar} 02:00:00");

        let conn = self.lock_conn()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM play_log
             WHERE (
                    (status = 'emergency_ok' AND task_title LIKE 'Emergency:%')
                 OR (status = 'success' AND task_title LIKE 'Oref:%')
               )
               AND played_at >= ?1
               AND played_at < ?2",
            params![window_start, window_end],
            |row| row.get(0),
        )?;
        Ok(count.max(0) as u64)
    }

    pub fn is_day_disabled(&self, date: &str, schedule_id: i64) -> Result<bool> {
        let conn = self.lock_conn()?;
        for sid in [0_i64, schedule_id] {
            let mut stmt = conn.prepare(
                "SELECT is_disabled FROM schedule_day_overrides WHERE date = ?1 AND schedule_id = ?2",
            )?;
            let mut rows = stmt.query_map(params![date, sid], |row| row.get::<_, i64>(0))?;
            if let Some(value) = rows.next() {
                if value? != 0 {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn is_task_disabled_for_date(&self, date: &str, task_id: i64) -> Result<bool> {
        let conn = self.lock_conn()?;
        let value: Option<i64> = conn
            .query_row(
                "SELECT is_disabled FROM schedule_task_overrides WHERE date = ?1 AND task_id = ?2",
                params![date, task_id],
                |row| row.get(0),
            )
            .ok();
        Ok(value.unwrap_or(0) != 0)
    }

    pub fn get_schedule_overrides(
        &self,
        start: &str,
        end: &str,
    ) -> Result<ScheduleOverridesBundle> {
        let conn = self.lock_conn()?;
        let mut day_stmt = conn.prepare(
            "SELECT date, schedule_id, is_disabled FROM schedule_day_overrides
             WHERE date >= ?1 AND date <= ?2",
        )?;
        let day_overrides = day_stmt
            .query_map(params![start, end], |row| {
                Ok(DayOverrideEntry {
                    date: row.get(0)?,
                    schedule_id: row.get(1)?,
                    is_disabled: row.get::<_, i64>(2)? != 0,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        let mut task_stmt = conn.prepare(
            "SELECT date, task_id, is_disabled FROM schedule_task_overrides
             WHERE date >= ?1 AND date <= ?2",
        )?;
        let task_overrides = task_stmt
            .query_map(params![start, end], |row| {
                Ok(TaskOverrideEntry {
                    date: row.get(0)?,
                    task_id: row.get(1)?,
                    is_disabled: row.get::<_, i64>(2)? != 0,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(ScheduleOverridesBundle {
            day_overrides,
            task_overrides,
        })
    }
}
