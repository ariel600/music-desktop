use crate::db::{
    HolidayEntry, OperatingDayHours, OperatingHoursSettings, SeasonOperatingHours, SystemMessage,
    SYSTEM_MESSAGE_DAY_HOLIDAY, SYSTEM_MESSAGE_DAY_HOLIDAY_EVE, SYSTEM_MESSAGE_SCHEDULE_FIXED,
    SYSTEM_MESSAGE_SCHEDULE_RELATIVE, WEEKDAY_THURSDAY,
};
use chrono::{Duration, NaiveDate, NaiveTime};

pub fn is_erev_chag(holiday: Option<&HolidayEntry>) -> bool {
    holiday.is_some_and(|entry| entry.day_label.trim() == "ערב חג")
}

/// True when the calendar day *after* `operational_date` is marked "ערב חג".
/// Fails closed (returns false) on any DB error.
pub fn tomorrow_is_erev_chag(db: &crate::db::DbState, operational_date: &str) -> bool {
    crate::operational_day::next_date_string(operational_date)
        .and_then(|tomorrow| db.get_holiday_day(&tomorrow).ok().flatten())
        .as_ref()
        .map(|entry| is_erev_chag(Some(entry)))
        .unwrap_or(false)
}

/// When enabled, the day before "ערב חג" also matches Thursday schedules.
pub fn treat_day_as_thursday(
    day_before_erev_as_thursday: bool,
    tomorrow_is_erev_chag: bool,
) -> bool {
    day_before_erev_as_thursday && tomorrow_is_erev_chag
}

/// Weekday match for schedule tasks / shared day-of-week lists (0–6).
pub fn matches_weekdays(
    days_of_week: &[u8],
    weekday: u8,
    treat_as_thursday: bool,
) -> bool {
    if days_of_week.contains(&weekday) {
        return true;
    }
    treat_as_thursday && days_of_week.contains(&WEEKDAY_THURSDAY)
}

/// Weekday used for relative open/close hours: Thursday when the item runs
/// only via the day-before-erev rule.
pub fn hours_weekday_for_days(
    days_of_week: &[u8],
    weekday: u8,
    treat_as_thursday: bool,
) -> u8 {
    if treat_as_thursday
        && !days_of_week.contains(&weekday)
        && days_of_week.contains(&WEEKDAY_THURSDAY)
    {
        WEEKDAY_THURSDAY
    } else {
        weekday
    }
}

pub fn matches_operational_day(
    message: &SystemMessage,
    weekday: u8,
    holiday: Option<&HolidayEntry>,
    day_before_erev_as_thursday: bool,
    tomorrow_is_erev_chag: bool,
) -> bool {
    let treat_as_thursday =
        treat_day_as_thursday(day_before_erev_as_thursday, tomorrow_is_erev_chag);
    if matches_weekdays(&message.days_of_week, weekday, treat_as_thursday) {
        return true;
    }

    if let Some(holiday) = holiday {
        let label = holiday.day_label.trim();
        if message.days_of_week.contains(&SYSTEM_MESSAGE_DAY_HOLIDAY_EVE) && label == "ערב חג"
        {
            return true;
        }
        if message.days_of_week.contains(&SYSTEM_MESSAGE_DAY_HOLIDAY) && label == "חג" {
            return true;
        }
    }

    false
}

pub fn resolve_play_time(
    message: &SystemMessage,
    date_str: &str,
    weekday: u8,
    operating_hours: &OperatingHoursSettings,
    holiday: Option<&HolidayEntry>,
    treat_as_thursday: bool,
) -> Option<String> {
    let hours_weekday = hours_weekday_for_days(&message.days_of_week, weekday, treat_as_thursday);
    match message.schedule_mode.as_str() {
        SYSTEM_MESSAGE_SCHEDULE_FIXED => message
            .scheduled_time
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string()),
        SYSTEM_MESSAGE_SCHEDULE_RELATIVE => {
            let hours = resolve_day_hours(date_str, hours_weekday, operating_hours, holiday)?;
            if is_closed(&hours) {
                return None;
            }

            let anchor = message.operating_anchor.as_deref()?.trim();
            let direction = message.offset_direction.as_deref()?.trim();
            let minutes = message.offset_minutes.filter(|value| *value > 0)?;

            let base = match anchor {
                "open" => hours.open.as_str(),
                "close" => hours.close.as_str(),
                _ => return None,
            };
            let base_time = parse_hhmm(base)?;
            let offset = Duration::minutes(minutes);
            let noon = NaiveDate::from_ymd_opt(2000, 1, 1)?.and_time(base_time);
            let shifted = if direction == "before" {
                noon.checked_sub_signed(offset)?
            } else if direction == "after" {
                noon.checked_add_signed(offset)?
            } else {
                return None;
            };
            Some(shifted.format("%H:%M").to_string())
        }
        _ => None,
    }
}

pub(crate) fn resolve_day_hours(
    date_str: &str,
    weekday: u8,
    settings: &OperatingHoursSettings,
    holiday: Option<&HolidayEntry>,
) -> Option<OperatingDayHours> {
    if let Some(holiday) = holiday {
        let open = holiday.open_time.as_deref().map(str::trim).unwrap_or("");
        let close = holiday.close_time.as_deref().map(str::trim).unwrap_or("");
        if !open.is_empty() && !close.is_empty() {
            return Some(OperatingDayHours {
                open: open.to_string(),
                close: close.to_string(),
            });
        }
        if holiday.cancel_messages {
            return Some(OperatingDayHours {
                open: "00:00".into(),
                close: "00:00".into(),
            });
        }
    }

    let season = if temporary_active(&settings.temporary, date_str) {
        &settings.temporary.hours
    } else if is_israel_summer_on_date(date_str) {
        &settings.summer
    } else {
        &settings.winter
    };

    Some(day_hours_for_weekday(season, weekday).clone())
}

fn temporary_active(
    temporary: &crate::db::TemporaryOperatingHours,
    date_str: &str,
) -> bool {
    let from = temporary.valid_from.as_deref().map(str::trim).unwrap_or("");
    let to = temporary.valid_to.as_deref().map(str::trim).unwrap_or("");
    if from.len() != 10 || to.len() != 10 || from > to {
        return false;
    }
    date_str >= from && date_str <= to
}

fn day_hours_for_weekday(season: &SeasonOperatingHours, weekday: u8) -> &OperatingDayHours {
    match weekday {
        0 => &season.sunday,
        1 => &season.monday,
        2 => &season.tuesday,
        3 => &season.wednesday,
        4 => &season.thursday,
        5 => &season.friday,
        _ => &season.motzei_shabbat,
    }
}

pub(crate) fn is_closed(hours: &OperatingDayHours) -> bool {
    hours.open.trim() == "00:00" && hours.close.trim() == "00:00"
}

pub(crate) fn parse_hhmm(value: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(value.trim(), "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(value.trim(), "%H:%M:%S"))
        .ok()
}

fn is_israel_summer_on_date(date_str: &str) -> bool {
    let Some(date) = crate::operational_day::parse_operational_date(date_str) else {
        return false;
    };
    crate::operational_day::is_israel_summer_on_date(date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::SystemMessage;

    fn sample_message(days: Vec<u8>, mode: &str) -> SystemMessage {
        SystemMessage {
            id: 1,
            title: "t".into(),
            file_path: "/tmp/a.mp3".into(),
            audio_name: None,
            is_active: true,
            days_of_week: days,
            schedule_mode: mode.into(),
            scheduled_time: Some("08:45".into()),
            operating_anchor: Some("open".into()),
            offset_direction: Some("before".into()),
            offset_minutes: Some(5),
            last_played_date: None,
        }
    }

    #[test]
    fn fixed_time_passthrough() {
        let message = sample_message(vec![0], SYSTEM_MESSAGE_SCHEDULE_FIXED);
        let hours = crate::db::OperatingHoursSettings {
            winter: crate::db::SeasonOperatingHours {
                sunday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                monday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                tuesday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                wednesday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                thursday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                friday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                motzei_shabbat: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
            },
            summer: crate::db::SeasonOperatingHours {
                sunday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                monday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                tuesday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                wednesday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                thursday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                friday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                motzei_shabbat: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
            },
            temporary: crate::db::TemporaryOperatingHours {
                hours: crate::db::SeasonOperatingHours {
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
                },
                valid_from: None,
                valid_to: None,
            },
        };
        assert_eq!(
            resolve_play_time(&message, "2026-07-14", 2, &hours, None, false).as_deref(),
            Some("08:45")
        );
    }

    #[test]
    fn relative_before_open() {
        let message = sample_message(vec![0], SYSTEM_MESSAGE_SCHEDULE_RELATIVE);
        let hours = crate::db::OperatingHoursSettings {
            winter: crate::db::SeasonOperatingHours {
                sunday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                monday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                tuesday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                wednesday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                thursday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                friday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                motzei_shabbat: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
            },
            summer: crate::db::SeasonOperatingHours {
                sunday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                monday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                tuesday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                wednesday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                thursday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                friday: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
                motzei_shabbat: OperatingDayHours {
                    open: "08:00".into(),
                    close: "16:00".into(),
                },
            },
            temporary: crate::db::TemporaryOperatingHours {
                hours: crate::db::SeasonOperatingHours {
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
                },
                valid_from: None,
                valid_to: None,
            },
        };
        assert_eq!(
            resolve_play_time(&message, "2026-01-14", 3, &hours, None, false).as_deref(),
            Some("07:55")
        );
    }

    #[test]
    fn holiday_day_match() {
        let message = sample_message(vec![SYSTEM_MESSAGE_DAY_HOLIDAY], SYSTEM_MESSAGE_SCHEDULE_FIXED);
        let holiday = HolidayEntry {
            date: "2026-07-14".into(),
            title: "t".into(),
            holiday_group: "g".into(),
            day_label: "חג".into(),
            hebrew: None,
            cancel_messages: false,
            is_custom: false,
            open_time: None,
            close_time: None,
            hebrew_month: None,
            hebrew_day: None,
        };
        assert!(matches_operational_day(
            &message,
            2,
            Some(&holiday),
            false,
            false
        ));
        assert!(!matches_operational_day(&message, 2, None, false, false));
    }

    #[test]
    fn day_before_erev_matches_thursday_messages() {
        let thursday_message = sample_message(vec![4], SYSTEM_MESSAGE_SCHEDULE_FIXED);
        assert!(matches_operational_day(
            &thursday_message,
            2, // Tuesday
            None,
            true,
            true
        ));
        assert!(!matches_operational_day(
            &thursday_message,
            2,
            None,
            false,
            true
        ));
        assert!(!matches_operational_day(
            &thursday_message,
            2,
            None,
            true,
            false
        ));
    }

    #[test]
    fn matches_weekdays_treats_thursday_on_day_before_erev() {
        assert!(matches_weekdays(&[4], 2, true));
        assert!(!matches_weekdays(&[4], 2, false));
        assert!(matches_weekdays(&[2], 2, false));
        assert_eq!(hours_weekday_for_days(&[4], 2, true), 4);
        assert_eq!(hours_weekday_for_days(&[2, 4], 2, true), 2);
    }
}
