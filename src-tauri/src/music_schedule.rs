use crate::db::{HolidayEntry, OperatingHoursSettings};
use crate::operational_day;
use crate::system_message_schedule;
use chrono::{Duration, Local, NaiveDate, NaiveTime, Timelike};

pub const FOLDER_GENERAL: &str = "general";
pub const FOLDER_GENERAL_VOCAL: &str = "general-vocal";
pub const FOLDER_SHABBAT: &str = "shabbat";
pub const FOLDER_ROSH_HASHANA: &str = "rosh-hashana";
pub const FOLDER_SUKKOT: &str = "sukkot";
pub const FOLDER_CHANUKAH: &str = "chanukah";
pub const FOLDER_TU_BISHVAT: &str = "tu-bishvat";
pub const FOLDER_PURIM: &str = "purim";
pub const FOLDER_PESACH: &str = "pesach";
pub const FOLDER_SEFIRAT: &str = "sefirat-haomer";
pub const FOLDER_LAG_VOCAL: &str = "lag-baomer-vocal";
pub const FOLDER_LAG: &str = "lag-baomer";
pub const FOLDER_SHAVUOT: &str = "shavuot";
pub const FOLDER_BEIN_HAMETZARIM: &str = "bein-hametzarim";

const MUSIC_BEFORE_OPEN_MINUTES: i64 = 10;
const MUSIC_AFTER_CLOSE_MINUTES: i64 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicFolderDecision {
    Silence,
    Folder { slug: &'static str, vocal: bool },
}

impl MusicFolderDecision {
    pub fn slug(self) -> Option<&'static str> {
        match self {
            Self::Silence => None,
            Self::Folder { slug, .. } => Some(slug),
        }
    }
}

fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !matches!(ch, '\u{0591}'..='\u{05C7}' | '\u{05F3}' | '\u{05F4}' | '"' | '\''))
        .collect::<String>()
        .replace(['״', '׳', '"', '\''], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn name_matches(value: &str, candidates: &[&str]) -> bool {
    let normalized = normalize_name(value);
    candidates
        .iter()
        .any(|candidate| normalize_name(candidate) == normalized)
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn dates_for_groups(holidays: &[HolidayEntry], groups: &[&str]) -> Vec<NaiveDate> {
    let mut dates = holidays
        .iter()
        .filter(|h| name_matches(&h.holiday_group, groups) || name_matches(&h.title, groups))
        .filter_map(|h| parse_date(&h.date))
        .collect::<Vec<_>>();
    dates.sort();
    dates.dedup();
    dates
}

fn ranges_by_year(dates: &[NaiveDate]) -> Vec<(NaiveDate, NaiveDate)> {
    if dates.is_empty() {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut start = dates[0];
    let mut prev = dates[0];
    for &date in &dates[1..] {
        if date == prev + Duration::days(1) {
            prev = date;
            continue;
        }
        ranges.push((start, prev));
        start = date;
        prev = date;
    }
    ranges.push((start, prev));
    ranges
}

fn in_inclusive(date: NaiveDate, start: NaiveDate, end: NaiveDate) -> bool {
    date >= start && date <= end
}

fn holiday_group_folder(group: &str) -> Option<&'static str> {
    if name_matches(group, &["ראש השנה"]) {
        Some(FOLDER_ROSH_HASHANA)
    } else if name_matches(group, &["סוכות"]) {
        Some(FOLDER_SUKKOT)
    } else if name_matches(group, &["חנוכה"]) {
        Some(FOLDER_CHANUKAH)
    } else if name_matches(group, &["טו בשבט", "ט״ו בשבט", "ט\"ו בשבט"]) {
        Some(FOLDER_TU_BISHVAT)
    } else if name_matches(group, &["פורים", "שושן פורים"]) {
        Some(FOLDER_PURIM)
    } else if name_matches(group, &["פסח"]) {
        Some(FOLDER_PESACH)
    } else if name_matches(group, &["לג בעומר", "ל״ג בעומר", "ל\"ג בעומר"]) {
        Some(FOLDER_LAG)
    } else if name_matches(group, &["שבועות"]) {
        Some(FOLDER_SHAVUOT)
    } else {
        None
    }
}

fn resolve_seasonal(date: NaiveDate, holidays: &[HolidayEntry]) -> &'static str {
    let rh_starts = dates_for_groups(holidays, &["ראש השנה"]);
    for rh in rh_starts {
        if in_inclusive(date, rh - Duration::days(20), rh - Duration::days(1)) {
            return FOLDER_ROSH_HASHANA;
        }
    }

    let yk_dates = dates_for_groups(holidays, &["יום כיפור"]);
    for rh in dates_for_groups(holidays, &["ראש השנה"]) {
        if let Some(&yk) = yk_dates.iter().find(|&&yk| yk >= rh) {
            if in_inclusive(date, rh, yk - Duration::days(1)) {
                return FOLDER_GENERAL;
            }
        }
    }

    let sukkot_ranges = ranges_by_year(&dates_for_groups(holidays, &["סוכות"]));
    for yk in &yk_dates {
        if let Some((_, sukkot_end)) = sukkot_ranges.iter().find(|(start, end)| {
            *end >= *yk && *start <= *yk + Duration::days(30)
        }) {
            if in_inclusive(date, *yk, *sukkot_end) {
                return FOLDER_SUKKOT;
            }
        }
    }

    for (start, end) in ranges_by_year(&dates_for_groups(holidays, &["חנוכה"])) {
        if in_inclusive(date, start - Duration::days(20), end) {
            return FOLDER_CHANUKAH;
        }
    }

    for tu in dates_for_groups(holidays, &["טו בשבט", "ט״ו בשבט", "ט\"ו בשבט"]) {
        if in_inclusive(date, tu - Duration::days(10), tu) {
            return FOLDER_TU_BISHVAT;
        }
    }

    for purim in dates_for_groups(holidays, &["פורים"]) {
        if in_inclusive(date, purim - Duration::days(25), purim) {
            return FOLDER_PURIM;
        }
    }

    for (start, end) in ranges_by_year(&dates_for_groups(holidays, &["פסח"])) {
        if in_inclusive(date, start - Duration::days(20), end) {
            return FOLDER_PESACH;
        }
    }

    let sivan_1_dates = {
        let mut dates = dates_for_groups(holidays, &["א סיוון", "א׳ סיוון", "א' סיוון", "א בסיוון"]);
        if dates.is_empty() {
            for lag in dates_for_groups(holidays, &["לג בעומר", "ל״ג בעומר", "ל\"ג בעומר"]) {
                dates.push(lag + Duration::days(12));
            }
        }
        dates.sort();
        dates.dedup();
        dates
    };
    let shavuot_dates = dates_for_groups(holidays, &["שבועות"]);
    for sivan_1 in sivan_1_dates {
        if let Some(&shavuot) = shavuot_dates.iter().find(|&&s| s >= sivan_1) {
            if in_inclusive(date, sivan_1, shavuot) {
                return FOLDER_SHAVUOT;
            }
        }
    }

    FOLDER_GENERAL
}

/// תשעה באב = ט׳ באב. ב׳ באב = 7 ימים לפני, י׳ באב = יום אחרי.
fn tisha_bav_dates(holidays: &[HolidayEntry]) -> Vec<NaiveDate> {
    dates_for_groups(holidays, &["תשעה באב"])
}

fn av10_dates(holidays: &[HolidayEntry]) -> Vec<NaiveDate> {
    let mut dates = dates_for_groups(holidays, &["י אב", "י׳ אב", "י' אב"]);
    for tisha in tisha_bav_dates(holidays) {
        dates.push(tisha + Duration::days(1));
    }
    dates.sort();
    dates.dedup();
    dates
}

/// No music from ב׳ באב through the end of י׳ באב (inclusive).
fn is_av_silence_period(
    date: NaiveDate,
    holiday: Option<&HolidayEntry>,
    holidays: &[HolidayEntry],
) -> bool {
    if holiday.is_some_and(|h| {
        name_matches(&h.holiday_group, &["תשעה באב"])
            || name_matches(&h.title, &["תשעה באב"])
            || name_matches(&h.holiday_group, &["י אב", "י׳ אב", "י' אב"])
            || name_matches(&h.title, &["י אב", "י׳ אב", "י' אב"])
    }) {
        return true;
    }
    if av10_dates(holidays).contains(&date) {
        return true;
    }
    for tisha in tisha_bav_dates(holidays) {
        let av2 = tisha - Duration::days(7);
        let av10 = tisha + Duration::days(1);
        if in_inclusive(date, av2, av10) {
            return true;
        }
    }
    false
}

fn resolve_vocal(date: NaiveDate, holidays: &[HolidayEntry]) -> Option<&'static str> {
    let tisha_dates = tisha_bav_dates(holidays);

    // בין המצרים ווקאלי: מי״ז בתמוז עד א׳ באב (יום לפני ב׳ באב).
    // מב׳ באב עד י׳ באב — שתיקה מלאה (ראה is_av_silence_period).
    let mut tammuz_17 = dates_for_groups(holidays, &["יז בתמוז", "י״ז בתמוז", "י\"ז בתמוז"]);
    if tammuz_17.is_empty() {
        for tisha in &tisha_dates {
            tammuz_17.push(*tisha - Duration::days(21));
        }
    }
    for start in tammuz_17 {
        if let Some(&tisha) = tisha_dates.iter().find(|&&t| t >= start) {
            let av1 = tisha - Duration::days(8); // ט׳ באב − 8 = א׳ באב
            if in_inclusive(date, start, av1) {
                return Some(FOLDER_BEIN_HAMETZARIM);
            }
        }
    }

    let lag_dates = dates_for_groups(holidays, &["לג בעומר", "ל״ג בעומר", "ל\"ג בעומר"]);
    for lag in &lag_dates {
        if in_inclusive(date, *lag - Duration::days(10), *lag - Duration::days(1)) {
            return Some(FOLDER_LAG_VOCAL);
        }
    }

    let pesach_ranges = ranges_by_year(&dates_for_groups(holidays, &["פסח"]));
    let mut sivan_1_dates = dates_for_groups(holidays, &["א סיוון", "א׳ סיוון", "א' סיוון", "א בסיוון"]);
    if sivan_1_dates.is_empty() {
        for lag in &lag_dates {
            sivan_1_dates.push(*lag + Duration::days(12));
        }
    }
    for (_, pesach_end) in pesach_ranges {
        let sefirat_start = pesach_end + Duration::days(1);
        if let Some(&sivan_1) = sivan_1_dates.iter().find(|&&s| s > pesach_end) {
            if in_inclusive(date, sefirat_start, sivan_1 - Duration::days(1)) {
                return Some(FOLDER_SEFIRAT);
            }
        }
    }

    None
}

pub fn resolve_music_folder(
    date: NaiveDate,
    weekday: u8,
    holiday: Option<&HolidayEntry>,
    holidays: &[HolidayEntry],
) -> MusicFolderDecision {
    if is_av_silence_period(date, holiday, holidays) {
        return MusicFolderDecision::Silence;
    }

    let lag_dates = dates_for_groups(holidays, &["לג בעומר", "ל״ג בעומר", "ל\"ג בעומר"]);
    if lag_dates.contains(&date) {
        return MusicFolderDecision::Folder {
            slug: FOLDER_LAG,
            vocal: false,
        };
    }

    // Vocal periods always win over the Thursday/Friday Shabbat folder.
    if let Some(vocal) = resolve_vocal(date, holidays) {
        return MusicFolderDecision::Folder {
            slug: vocal,
            vocal: true,
        };
    }

    let seasonal = resolve_seasonal(date, holidays);
    let is_erev_chag = holiday.is_some_and(|h| h.day_label.trim() == "ערב חג");

    if is_erev_chag {
        if let Some(holiday) = holiday {
            if let Some(folder) = holiday_group_folder(&holiday.holiday_group) {
                return MusicFolderDecision::Folder {
                    slug: folder,
                    vocal: false,
                };
            }
        }
        return MusicFolderDecision::Folder {
            slug: seasonal,
            vocal: false,
        };
    }

    // Erev Shabbat / Shabbat playlist — only outside vocal periods (handled above).
    if weekday == 4 || weekday == 5 {
        return MusicFolderDecision::Folder {
            slug: FOLDER_SHABBAT,
            vocal: false,
        };
    }

    MusicFolderDecision::Folder {
        slug: seasonal,
        vocal: false,
    }
}

pub fn fallback_folder(decision: MusicFolderDecision) -> Option<&'static str> {
    match decision {
        MusicFolderDecision::Silence => None,
        MusicFolderDecision::Folder { vocal: true, .. } => Some(FOLDER_GENERAL_VOCAL),
        MusicFolderDecision::Folder { vocal: false, .. } => Some(FOLDER_GENERAL),
    }
}

fn is_in_hours_window(
    now: chrono::DateTime<Local>,
    operational_date: &str,
    weekday: u8,
    settings: &OperatingHoursSettings,
    holiday: Option<&HolidayEntry>,
    before_open_minutes: i64,
    after_close_minutes: i64,
) -> bool {
    let Some(hours) = system_message_schedule::resolve_day_hours(
        operational_date,
        weekday,
        settings,
        holiday,
    ) else {
        return false;
    };
    if system_message_schedule::is_closed(&hours) {
        return false;
    }

    let Some(open) = system_message_schedule::parse_hhmm(&hours.open) else {
        return false;
    };
    let Some(close) = system_message_schedule::parse_hhmm(&hours.close) else {
        return false;
    };
    let Some(op_date) = operational_day::parse_operational_date(operational_date) else {
        return false;
    };

    let Some(open_dt) = wall_datetime(op_date, open) else {
        return false;
    };
    let Some(close_dt) = wall_datetime(op_date, close) else {
        return false;
    };

    let window_start = open_dt - Duration::minutes(before_open_minutes);
    let window_end = close_dt + Duration::minutes(after_close_minutes);

    if close_dt >= open_dt {
        now >= window_start && now <= window_end
    } else {
        now >= window_start || now <= window_end
    }
}

/// Strict open→close window for the current operational day (no music padding).
#[allow(dead_code)]
pub fn is_within_operating_hours(
    now: chrono::DateTime<Local>,
    operational_date: &str,
    weekday: u8,
    settings: &OperatingHoursSettings,
    holiday: Option<&HolidayEntry>,
) -> bool {
    is_in_hours_window(now, operational_date, weekday, settings, holiday, 0, 0)
}

pub fn is_in_music_window(
    now: chrono::DateTime<Local>,
    operational_date: &str,
    weekday: u8,
    settings: &OperatingHoursSettings,
    holiday: Option<&HolidayEntry>,
) -> bool {
    is_in_hours_window(
        now,
        operational_date,
        weekday,
        settings,
        holiday,
        MUSIC_BEFORE_OPEN_MINUTES,
        MUSIC_AFTER_CLOSE_MINUTES,
    )
}

fn wall_datetime(op_date: NaiveDate, time: NaiveTime) -> Option<chrono::DateTime<Local>> {
    let calendar = if time.hour() < 2 {
        op_date + Duration::days(1)
    } else {
        op_date
    };
    calendar.and_time(time).and_local_timezone(Local).latest()
}

pub fn resolve_today(
    holidays: &[HolidayEntry],
    settings: &OperatingHoursSettings,
    holiday: Option<&HolidayEntry>,
) -> (bool, MusicFolderDecision) {
    let now = Local::now();
    let operational_date = operational_day::operational_date_string(now);
    let weekday = operational_day::operational_weekday(now);
    let date = operational_day::hebrew_date(now);

    let in_window = is_in_music_window(now, &operational_date, weekday, settings, holiday);
    let decision = resolve_music_folder(date, weekday, holiday, holidays);
    (in_window, decision)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holiday(date: &str, group: &str, day_label: &str) -> HolidayEntry {
        HolidayEntry {
            date: date.into(),
            title: group.into(),
            holiday_group: group.into(),
            day_label: day_label.into(),
            hebrew: None,
            cancel_messages: false,
            is_custom: false,
            open_time: None,
            close_time: None,
            hebrew_month: None,
            hebrew_day: None,
        }
    }

    #[test]
    fn thursday_uses_shabbat() {
        // Ordinary Thursday outside any vocal/silence window.
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let decision = resolve_music_folder(date, 4, None, &[]);
        assert_eq!(decision.slug(), Some(FOLDER_SHABBAT));
    }

    #[test]
    fn vocal_overrides_thursday() {
        let holidays = vec![
            holiday("2026-04-02", "פסח", "חג"),
            holiday("2026-04-09", "פסח", "חג"),
            holiday("2026-05-05", "ל״ג בעומר", "חג"),
            holiday("2026-05-22", "שבועות", "חג"),
        ];
        let date = NaiveDate::from_ymd_opt(2026, 4, 16).unwrap();
        let decision = resolve_music_folder(date, 4, None, &holidays);
        assert_eq!(decision.slug(), Some(FOLDER_SEFIRAT));
        assert!(matches!(
            decision,
            MusicFolderDecision::Folder { vocal: true, .. }
        ));
    }

    #[test]
    fn bein_hametzarim_vocal_overrides_friday() {
        // תשעה באב 2026-07-23 → י״ז בתמוז ≈ 2026-07-02, א׳ באב = 2026-07-15
        let holidays = vec![holiday("2026-07-23", "תשעה באב", "חג")];
        let friday = NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(); // still before ב׳ באב
        let decision = resolve_music_folder(friday, 5, None, &holidays);
        assert_eq!(decision.slug(), Some(FOLDER_BEIN_HAMETZARIM));
        assert!(matches!(
            decision,
            MusicFolderDecision::Folder { vocal: true, .. }
        ));
    }

    #[test]
    fn lag_day_overrides_vocal() {
        let holidays = vec![
            holiday("2026-04-02", "פסח", "חג"),
            holiday("2026-04-09", "פסח", "חג"),
            holiday("2026-05-05", "ל״ג בעומר", "חג"),
            holiday("2026-05-22", "שבועות", "חג"),
        ];
        let date = NaiveDate::from_ymd_opt(2026, 5, 5).unwrap();
        let decision = resolve_music_folder(date, 2, None, &holidays);
        assert_eq!(decision.slug(), Some(FOLDER_LAG));
        assert!(matches!(
            decision,
            MusicFolderDecision::Folder { vocal: false, .. }
        ));
    }

    #[test]
    fn tisha_bav_silence() {
        let holidays = vec![holiday("2026-07-23", "תשעה באב", "חג")];
        let date = NaiveDate::from_ymd_opt(2026, 7, 23).unwrap();
        let decision = resolve_music_folder(date, 4, None, &holidays);
        assert_eq!(decision, MusicFolderDecision::Silence);
    }

    #[test]
    fn av2_through_av10_silence() {
        // תשעה באב = 2026-07-23 → ב׳ באב = 2026-07-16, י׳ באב = 2026-07-24
        let holidays = vec![holiday("2026-07-23", "תשעה באב", "חג")];
        let av2 = NaiveDate::from_ymd_opt(2026, 7, 16).unwrap();
        let av8 = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let av10 = NaiveDate::from_ymd_opt(2026, 7, 24).unwrap();
        assert_eq!(
            resolve_music_folder(av2, 4, None, &holidays),
            MusicFolderDecision::Silence
        );
        assert_eq!(
            resolve_music_folder(av8, 3, None, &holidays),
            MusicFolderDecision::Silence
        );
        assert_eq!(
            resolve_music_folder(av10, 5, None, &holidays),
            MusicFolderDecision::Silence
        );
    }

    #[test]
    fn av1_still_bein_hametzarim_vocal() {
        let holidays = vec![holiday("2026-07-23", "תשעה באב", "חג")];
        let av1 = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
        let decision = resolve_music_folder(av1, 2, None, &holidays);
        assert_eq!(decision.slug(), Some(FOLDER_BEIN_HAMETZARIM));
        assert!(matches!(
            decision,
            MusicFolderDecision::Folder { vocal: true, .. }
        ));
    }

    #[test]
    fn erev_chag_not_shabbat_folder() {
        let holidays = vec![
            holiday("2026-09-11", "ראש השנה", "חג"),
            holiday("2026-09-10", "ראש השנה", "ערב חג"),
        ];
        let date = NaiveDate::from_ymd_opt(2026, 9, 10).unwrap();
        let eve = holiday("2026-09-10", "ראש השנה", "ערב חג");
        let decision = resolve_music_folder(date, 4, Some(&eve), &holidays);
        assert_eq!(decision.slug(), Some(FOLDER_ROSH_HASHANA));
    }
}
