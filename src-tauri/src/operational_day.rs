use chrono::{
    DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Timelike, Utc, Weekday,
};

const JERUSALEM_LATITUDE: f64 = 31.7683;
const JERUSALEM_LONGITUDE: f64 = 35.2137;
const OFFICIAL_SUNSET_ZENITH: f64 = 90.833;

pub fn operational_date(now: DateTime<Local>) -> NaiveDate {
    let date = now.date_naive();
    if now.hour() < 2 {
        date - Duration::days(1)
    } else {
        date
    }
}

pub fn operational_date_string(now: DateTime<Local>) -> String {
    operational_date(now).format("%Y-%m-%d").to_string()
}

pub fn operational_weekday(now: DateTime<Local>) -> u8 {
    operational_date(now)
        .weekday()
        .num_days_from_sunday() as u8
}

fn normalize(value: f64, modulus: f64) -> f64 {
    ((value % modulus) + modulus) % modulus
}

fn jerusalem_sunset_utc(date: NaiveDate) -> Option<DateTime<Utc>> {
    let day = date.ordinal() as f64;
    let longitude_hour = JERUSALEM_LONGITUDE / 15.0;
    let approximate_time = day + (18.0 - longitude_hour) / 24.0;
    let mean_anomaly = 0.9856 * approximate_time - 3.289;
    let true_longitude = normalize(
        mean_anomaly
            + 1.916 * mean_anomaly.to_radians().sin()
            + 0.020 * (2.0 * mean_anomaly).to_radians().sin()
            + 282.634,
        360.0,
    );

    let mut right_ascension =
        (0.91764 * true_longitude.to_radians().tan()).atan().to_degrees();
    right_ascension = normalize(right_ascension, 360.0);
    right_ascension += (true_longitude / 90.0).floor() * 90.0
        - (right_ascension / 90.0).floor() * 90.0;
    right_ascension /= 15.0;

    let sin_declination = 0.39782 * true_longitude.to_radians().sin();
    let cos_declination = sin_declination.asin().cos();
    let cosine_hour_angle = (OFFICIAL_SUNSET_ZENITH.to_radians().cos()
        - sin_declination * JERUSALEM_LATITUDE.to_radians().sin())
        / (cos_declination * JERUSALEM_LATITUDE.to_radians().cos());
    if !(-1.0..=1.0).contains(&cosine_hour_angle) {
        return None;
    }

    let hour_angle = cosine_hour_angle.acos().to_degrees() / 15.0;
    let local_mean_time =
        hour_angle + right_ascension - 0.06571 * approximate_time - 6.622;
    let utc_hours = normalize(local_mean_time - longitude_hour, 24.0);
    let seconds = (utc_hours * 3600.0).round() as i64;
    let midnight = date.and_hms_opt(0, 0, 0)?;
    Some(Utc.from_utc_datetime(&(midnight + Duration::seconds(seconds))))
}

/// Gregorian row whose Hebrew date is in effect in Jerusalem. The Hebrew day
/// advances at astronomical sunset while the civil/operational date is kept
/// separate.
fn hebrew_date_for_instant(civil_date: NaiveDate, now_utc: DateTime<Utc>) -> NaiveDate {
    match jerusalem_sunset_utc(civil_date) {
        Some(sunset) if now_utc >= sunset => civil_date + Duration::days(1),
        _ => civil_date,
    }
}

pub fn hebrew_date(now: DateTime<Local>) -> NaiveDate {
    let civil_date = now.date_naive();
    hebrew_date_for_instant(civil_date, now.with_timezone(&Utc))
}

pub fn hebrew_date_string(now: DateTime<Local>) -> String {
    hebrew_date(now).format("%Y-%m-%d").to_string()
}

/// Map a scheduled `HH:MM` belonging to an operational day onto a wall-clock datetime.
/// Times before 02:00 fall on the calendar day after the operational date.
pub fn scheduled_wall_datetime(
    scheduled_time: &str,
    operational_date: NaiveDate,
) -> Option<DateTime<Local>> {
    let time = NaiveTime::parse_from_str(scheduled_time.trim(), "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(scheduled_time.trim(), "%H:%M:%S"))
        .ok()?;
    let calendar_date = if time.hour() < 2 {
        operational_date + Duration::days(1)
    } else {
        operational_date
    };
    calendar_date.and_time(time).and_local_timezone(Local).latest()
}

pub fn parse_operational_date(date_str: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

pub fn next_date_string(date_str: &str) -> Option<String> {
    let date = parse_operational_date(date_str)?;
    Some((date + Duration::days(1)).format("%Y-%m-%d").to_string())
}

/// Israel DST (IDT, UTC+3) from Friday before the last Sunday of March through
/// the last Sunday of October (exclusive end date at midnight).
pub fn is_israel_summer_on_date(date: NaiveDate) -> bool {
    let year = date.year();
    let start = israel_dst_start_date(year);
    let end = israel_dst_end_date(year);
    date >= start && date < end
}

fn israel_dst_start_date(year: i32) -> NaiveDate {
    let last_sunday_march = last_weekday_of_month(year, 3, Weekday::Sun);
    last_sunday_march - Duration::days(2) // Friday before
}

fn israel_dst_end_date(year: i32) -> NaiveDate {
    last_weekday_of_month(year, 10, Weekday::Sun)
}

fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    let first_of_next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    // `month` is always 1..=12 here, so the fallback is never expected to run;
    // it just avoids a panic instead of unwrapping.
    let last_day = first_of_next_month
        .and_then(|date| date.pred_opt())
        .unwrap_or_else(|| {
            NaiveDate::from_ymd_opt(year, month, 28).unwrap_or_default()
        });

    let mut date = last_day;
    while date.weekday() != weekday {
        date -= Duration::days(1);
    }
    date
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn before_two_am_belongs_to_previous_day() {
        let now = Local.with_ymd_and_hms(2026, 6, 24, 1, 30, 0).unwrap();
        assert_eq!(operational_date_string(now), "2026-06-23");
    }

    #[test]
    fn late_evening_uses_operational_calendar_date() {
        let op = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
        let dt = scheduled_wall_datetime("23:00", op).unwrap();
        assert_eq!(dt.date_naive(), op);
        assert_eq!(dt.hour(), 23);
    }

    #[test]
    fn early_morning_slot_is_next_calendar_day() {
        let op = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
        let dt = scheduled_wall_datetime("01:30", op).unwrap();
        assert_eq!(dt.date_naive(), NaiveDate::from_ymd_opt(2026, 6, 24).unwrap());
    }

    #[test]
    fn israel_summer_span_2026() {
        assert!(!is_israel_summer_on_date(
            NaiveDate::from_ymd_opt(2026, 3, 26).unwrap()
        ));
        assert!(is_israel_summer_on_date(
            NaiveDate::from_ymd_opt(2026, 3, 27).unwrap()
        )); // Friday before last Sunday (29th)
        assert!(is_israel_summer_on_date(
            NaiveDate::from_ymd_opt(2026, 10, 24).unwrap()
        ));
        assert!(!is_israel_summer_on_date(
            NaiveDate::from_ymd_opt(2026, 10, 25).unwrap()
        )); // last Sunday of October
    }

    #[test]
    fn jerusalem_sunset_is_in_expected_utc_range() {
        let summer = jerusalem_sunset_utc(
            NaiveDate::from_ymd_opt(2026, 7, 16).unwrap(),
        )
        .unwrap();
        assert_eq!(summer.date_naive(), NaiveDate::from_ymd_opt(2026, 7, 16).unwrap());
        assert!((16..=17).contains(&summer.hour()));

        let winter = jerusalem_sunset_utc(
            NaiveDate::from_ymd_opt(2026, 12, 16).unwrap(),
        )
        .unwrap();
        assert_eq!(winter.date_naive(), NaiveDate::from_ymd_opt(2026, 12, 16).unwrap());
        assert!((14..=15).contains(&winter.hour()));
    }

    #[test]
    fn hebrew_date_changes_exactly_at_jerusalem_sunset() {
        let civil = NaiveDate::from_ymd_opt(2026, 7, 16).unwrap();
        let sunset = jerusalem_sunset_utc(civil).unwrap();
        assert_eq!(
            hebrew_date_for_instant(civil, sunset - Duration::seconds(1)),
            civil
        );
        assert_eq!(
            hebrew_date_for_instant(civil, sunset),
            civil + Duration::days(1)
        );
    }
}
