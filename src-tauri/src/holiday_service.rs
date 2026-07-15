use crate::db::{DbState, HolidayEntry, Task};
use chrono::{Datelike, NaiveTime};

pub fn should_skip_task_for_date_with_holiday(
    task: &Task,
    operational_date: &str,
    db: &DbState,
    holiday: Option<&HolidayEntry>,
) -> Result<Option<&'static str>, String> {
    let schedule_id = task.schedule_id.unwrap_or(1);
    if db
        .is_day_disabled(operational_date, schedule_id)
        .map_err(|e| e.to_string())?
    {
        return Ok(Some("skipped_day"));
    }

    if db
        .is_task_disabled_for_date(operational_date, task.id)
        .map_err(|e| e.to_string())?
    {
        return Ok(Some("skipped_override"));
    }

    if task.cancel_on_holiday {
        if let Some(holiday) = holiday {
            if holiday.cancel_messages {
                return Ok(Some("skipped_holiday"));
            }
        }
    }

    Ok(None)
}

fn nearby_years() -> [i32; 3] {
    let year = chrono::Local::now().year();
    [year - 1, year, year + 1]
}

pub fn get_holidays_list(db: &DbState) -> Result<Vec<HolidayEntry>, String> {
    db.get_holiday_days_for_years(&nearby_years())
        .map_err(|e| e.to_string())
}

pub fn sync_calendar_holidays(
    db: &DbState,
    entries: &[HolidayEntry],
) -> Result<Vec<HolidayEntry>, String> {
    let today = crate::operational_day::operational_date_string(chrono::Local::now());
    if db
        .is_calendar_synced_on(&today)
        .map_err(|e| e.to_string())?
    {
        return get_holidays_list(db);
    }

    let sync_years = nearby_years();
    let mut valid_dates = Vec::new();

    for entry in entries {
        let mut synced = entry.clone();
        synced.is_custom = false;
        db.upsert_hebcal_holiday(&synced)
            .map_err(|e| e.to_string())?;
        valid_dates.push(synced.date.clone());
    }

    for sync_year in sync_years {
        let year_dates: Vec<String> = valid_dates
            .iter()
            .filter(|date| date.starts_with(&format!("{sync_year}-")))
            .cloned()
            .collect();
        db.remove_stale_hebcal_holidays(sync_year, &year_dates)
            .map_err(|e| e.to_string())?;
    }

    db.mark_calendar_synced_on(&today)
        .map_err(|e| e.to_string())?;

    get_holidays_list(db)
}

pub fn ensure_custom_recurrences(
    db: &DbState,
    entries: &[HolidayEntry],
) -> Result<Vec<HolidayEntry>, String> {
    for entry in entries {
        let mut occurrence = entry.clone();
        occurrence.is_custom = true;
        db.ensure_custom_recurrence(&occurrence)
            .map_err(|e| e.to_string())?;
    }
    get_holidays_list(db)
}

pub fn set_holiday_status(
    db: &DbState,
    date: &str,
    day_label: &str,
    cancel_messages: bool,
    open_time: Option<String>,
    close_time: Option<String>,
) -> Result<Vec<HolidayEntry>, String> {
    let label = day_label.trim();
    if !matches!(label, "חג" | "ערב חג" | "אחר") {
        return Err("סוג יום לא תקין".to_string());
    }

    let open = open_time
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let close = close_time
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if !cancel_messages {
        let Some(open_value) = open.as_deref() else {
            return Err("יש להגדיר שעת פתיחה".to_string());
        };
        let Some(close_value) = close.as_deref() else {
            return Err("יש להגדיר שעת סגירה".to_string());
        };
        if NaiveTime::parse_from_str(open_value, "%H:%M").is_err() {
            return Err("שעת פתיחה אינה תקינה".to_string());
        }
        if NaiveTime::parse_from_str(close_value, "%H:%M").is_err() {
            return Err("שעת סגירה אינה תקינה".to_string());
        }
    }

    db.set_holiday_status(
        date,
        label,
        cancel_messages,
        if cancel_messages {
            None
        } else {
            open.as_deref()
        },
        if cancel_messages {
            None
        } else {
            close.as_deref()
        },
    )
    .map_err(|e| e.to_string())?;

    get_holidays_list(db)
}

pub fn add_custom_holiday(
    db: &DbState,
    date: &str,
    title: &str,
    cancel_messages: bool,
    day_label: Option<String>,
    open_time: Option<String>,
    close_time: Option<String>,
    hebrew_month: Option<String>,
    hebrew_day: Option<i32>,
) -> Result<Vec<HolidayEntry>, String> {
    let trimmed_title = title.trim();
    if trimmed_title.is_empty() {
        return Err("יש להזין שם לחג".to_string());
    }

    let label = day_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("חג");
    if !matches!(label, "חג" | "ערב חג" | "אחר") {
        return Err("סוג יום לא תקין".to_string());
    }

    let open = open_time
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let close = close_time
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if !cancel_messages {
        let Some(open_value) = open.as_deref() else {
            return Err("יש להגדיר שעת פתיחה".to_string());
        };
        let Some(close_value) = close.as_deref() else {
            return Err("יש להגדיר שעת סגירה".to_string());
        };
        if NaiveTime::parse_from_str(open_value, "%H:%M").is_err() {
            return Err("שעת פתיחה אינה תקינה".to_string());
        }
        if NaiveTime::parse_from_str(close_value, "%H:%M").is_err() {
            return Err("שעת סגירה אינה תקינה".to_string());
        }
    }

    let month = hebrew_month
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let day = hebrew_day.filter(|value| *value > 0);

    let entry = HolidayEntry {
        date: date.to_string(),
        title: trimmed_title.to_string(),
        holiday_group: trimmed_title.to_string(),
        day_label: label.to_string(),
        hebrew: Some(trimmed_title.to_string()),
        cancel_messages,
        is_custom: true,
        open_time: if cancel_messages { None } else { open },
        close_time: if cancel_messages { None } else { close },
        hebrew_month: month,
        hebrew_day: day,
    };

    db.add_custom_holiday_day(&entry)
        .map_err(|e| e.to_string())?;

    get_holidays_list(db)
}

pub fn delete_custom_holiday(db: &DbState, date: &str) -> Result<Vec<HolidayEntry>, String> {
    db.delete_custom_holiday_recurring(date)
        .map_err(|e| e.to_string())?;
    get_holidays_list(db)
}
