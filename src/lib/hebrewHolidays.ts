import { getHebrewMonthParts } from "./hebrewCalendar";
import { formatHebrewDay } from "./hebrewDate";
import { dayDisplayName } from "./holidays";
import { isErevChagLabel } from "./dayBeforeErev";
import { nextDateString, parseDateString, toDateString } from "./operationalDay";
import type { HolidayEntry } from "../types";

const hebrewMonthNameFormatter = new Intl.DateTimeFormat("he-u-ca-hebrew", {
  month: "long",
});

type CalendarDayAppearance = "today" | "holiday" | "erev" | "normal";

function normalizeHebrewMonth(month: string): string {
  const normalized = month
    .replace(/[\u0591-\u05C7]/g, "")
    .replace(/\u05F3|\u05F4/g, "")
    .trim();
  if (normalized === "חשון") return "חשוון";
  if (normalized === "איר") return "אייר";
  if (normalized === "סיון") return "סיוון";
  return normalized;
}

function getHebrewMonthName(date: Date): string {
  return normalizeHebrewMonth(hebrewMonthNameFormatter.format(date));
}

export function getHebrewDateParts(dateStr: string): {
  day: number;
  month: string;
  year: number;
} {
  const date = parseDateString(dateStr);
  const parts = getHebrewMonthParts(date);
  return {
    day: parts.day,
    month: getHebrewMonthName(date),
    year: parts.year,
  };
}

export function formatHolidayHebrewDate(dateStr: string): string {
  const { day, month } = getHebrewDateParts(dateStr);
  return `${formatHebrewDay(day)} ${month}`;
}

export function formatHolidayHebrewIdentity(holiday: HolidayEntry): string {
  const keyed = withHebrewIdentity(holiday);
  return `${formatHebrewDay(keyed.hebrew_day!)} ${keyed.hebrew_month}`;
}

function normalizeHolidayName(value: string): string {
  return value
    .replace(/[\u0591-\u05C7]/g, "")
    .replace(/[\u05F3\u05F4"']/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

const TECHNICAL_HOLIDAY_ANCHORS = new Set([
  normalizeHolidayName("א׳ בסיוון"),
  normalizeHolidayName("י״ז בתמוז"),
  normalizeHolidayName("י׳ באב"),
]);

/** Internal calendar markers needed by music rules, not user-facing holidays. */
export function isTechnicalHolidayAnchor(holiday: HolidayEntry): boolean {
  if (holiday.is_custom) {
    return false;
  }
  return (
    TECHNICAL_HOLIDAY_ANCHORS.has(normalizeHolidayName(holiday.title)) ||
    TECHNICAL_HOLIDAY_ANCHORS.has(
      normalizeHolidayName(holiday.holiday_group),
    )
  );
}

/** Stable identity for a recurring holiday rule, independent of Gregorian year. */
export function holidayRecurrenceKey(holiday: HolidayEntry): string {
  const keyed = withHebrewIdentity(holiday);
  return [
    normalizeHebrewMonth(keyed.hebrew_month ?? ""),
    keyed.hebrew_day ?? "",
    normalizeHolidayName(keyed.holiday_group),
    normalizeHolidayName(keyed.day_label),
    keyed.is_custom ? "custom" : "system",
  ].join("|");
}

const HEBREW_MONTH_SORT_KEYS: Record<string, number> = {
  תשרי: 1,
  חשון: 2,
  חשוון: 2,
  כסלו: 3,
  טבת: 4,
  שבט: 5,
  אדר: 6,
  "אדר א": 6,
  "אדר ב": 7,
  ניסן: 8,
  אייר: 9,
  איר: 9,
  סיוון: 10,
  סיון: 10,
  תמוז: 11,
  אב: 12,
  אלול: 13,
};

function hebrewMonthSortKey(month: string): number {
  const normalized = normalizeHebrewMonth(month);
  if (HEBREW_MONTH_SORT_KEYS[normalized] != null) {
    return HEBREW_MONTH_SORT_KEYS[normalized];
  }
  if (normalized.startsWith("אדר א")) {
    return 6;
  }
  if (normalized.startsWith("אדר ב")) {
    return 7;
  }
  if (normalized.startsWith("אדר")) {
    return 6;
  }
  return 99;
}

export function compareHolidaysByHebrewDate(
  a: { date: string },
  b: { date: string },
): number {
  const pa = getHebrewDateParts(a.date);
  const pb = getHebrewDateParts(b.date);
  if (pa.year !== pb.year) {
    return pa.year - pb.year;
  }
  const monthDiff = hebrewMonthSortKey(pa.month) - hebrewMonthSortKey(pb.month);
  if (monthDiff !== 0) {
    return monthDiff;
  }
  if (pa.day !== pb.day) {
    return pa.day - pb.day;
  }
  return a.date.localeCompare(b.date);
}

export function compareHolidayRulesByHebrewDate(
  a: HolidayEntry,
  b: HolidayEntry,
): number {
  const keyedA = withHebrewIdentity(a);
  const keyedB = withHebrewIdentity(b);
  const monthDiff =
    hebrewMonthSortKey(keyedA.hebrew_month ?? "") -
    hebrewMonthSortKey(keyedB.hebrew_month ?? "");
  if (monthDiff !== 0) {
    return monthDiff;
  }
  const dayDiff = (keyedA.hebrew_day ?? 0) - (keyedB.hebrew_day ?? 0);
  if (dayDiff !== 0) {
    return dayDiff;
  }
  return dayDisplayName(a).localeCompare(dayDisplayName(b), "he");
}

function monthIs(month: string, ...candidates: string[]): boolean {
  const normalized = normalizeHebrewMonth(month);
  return candidates.some((candidate) => normalizeHebrewMonth(candidate) === normalized);
}

function isPurimAdarMonth(month: string): boolean {
  const normalized = normalizeHebrewMonth(month);
  if (normalized === "אדר א" || normalized.startsWith("אדר א ")) {
    return false;
  }
  return (
    normalized === "אדר" ||
    normalized === "אדר ב" ||
    normalized.startsWith("אדר ב")
  );
}

function isOpenByDefaultHoliday(title: string): boolean {
  return (
    title === "חנוכה" ||
    title === "ט״ו בשבט" ||
    title === 'ט"ו בשבט' ||
    title === "פורים" ||
    title === "שושן פורים" ||
    title === "ל״ג בעומר" ||
    title === 'ל"ג בעומר' ||
    title === "י״ז בתמוז" ||
    title === 'י"ז בתמוז' ||
    title === "י׳ באב" ||
    title === "י' אב" ||
    title === "א׳ בסיוון" ||
    title === "א' בסיוון"
  );
}

function isChanukahDate(dateStr: string): boolean {
  const date = parseDateString(dateStr);
  for (let offset = 0; offset < 8; offset++) {
    const cursor = new Date(date);
    cursor.setDate(cursor.getDate() - offset);
    const parts = getHebrewDateParts(toDateString(cursor));
    if (monthIs(parts.month, "כסלו") && parts.day === 25) {
      return true;
    }
  }
  return false;
}

function getJewishCalendarHoliday(dateStr: string): string | null {
  const { day, month } = getHebrewDateParts(dateStr);

  if (monthIs(month, "תשרי")) {
    if (day === 1 || day === 2) {
      return "ראש השנה";
    }
    if (day === 10) {
      return "יום כיפור";
    }
    if (day >= 15 && day <= 22) {
      return "סוכות";
    }
  }

  if (isChanukahDate(dateStr)) {
    return "חנוכה";
  }

  if (monthIs(month, "שבט") && day === 15) {
    return "ט״ו בשבט";
  }

  if (isPurimAdarMonth(month) && day === 14) {
    return "פורים";
  }

  if (isPurimAdarMonth(month) && day === 15) {
    return "שושן פורים";
  }

  if (monthIs(month, "ניסן") && day >= 14 && day <= 21) {
    return "פסח";
  }

  if (monthIs(month, "אייר", "איר") && day === 18) {
    return "ל״ג בעומר";
  }

  if (monthIs(month, "סיוון", "סיון") && day === 6) {
    return "שבועות";
  }

  if (monthIs(month, "אב") && day === 9) {
    return "תשעה באב";
  }

  if (monthIs(month, "אב") && day === 10) {
    return "י׳ באב";
  }

  if (monthIs(month, "תמוז") && day === 17) {
    return "י״ז בתמוז";
  }

  if (monthIs(month, "סיוון", "סיון") && day === 1) {
    return "א׳ בסיוון";
  }

  return null;
}

function isShabbat(dateStr: string): boolean {
  return parseDateString(dateStr).getDay() === 6;
}

function isErevShabbat(dateStr: string): boolean {
  return parseDateString(dateStr).getDay() === 5;
}

function getJewishCalendarHolidayEve(dateStr: string): string | null {
  const { day, month } = getHebrewDateParts(dateStr);

  if (monthIs(month, "אלול") && day === 29) {
    return "ערב ראש השנה";
  }

  if (monthIs(month, "תשרי") && day === 9) {
    return "ערב יום כיפור";
  }

  if (monthIs(month, "תשרי") && day === 14) {
    return "ערב סוכות";
  }

  if (monthIs(month, "ניסן") && day === 13) {
    return "ערב פסח";
  }

  if (monthIs(month, "סיוון", "סיון") && day === 5) {
    return "ערב שבועות";
  }

  return null;
}

export function getCalendarDayLabel(
  dateStr: string,
  holidays?: HolidayEntry[],
): string | null {
  const entry = holidays?.find((holiday) => holiday.date === dateStr);
  if (entry) {
    // Same display names as Holidays settings (`dayDisplayName`).
    return dayDisplayName(entry);
  }

  if (holidays) {
    // Synced with Holidays settings: day before a DB "ערב חג".
    const tomorrow = nextDateString(dateStr);
    const tomorrowEntry = holidays.find((holiday) => holiday.date === tomorrow);
    if (isErevChagLabel(tomorrowEntry?.day_label)) {
      return "יום שלפני ערב חג";
    }

    // Weekly rhythm only — holiday names come solely from settings/DB.
    return (
      (isShabbat(dateStr) ? "שבת" : null) ??
      (isErevShabbat(dateStr) ? "ערב שבת" : null)
    );
  }

  return (
    getJewishCalendarHoliday(dateStr) ??
    getJewishCalendarHolidayEve(dateStr) ??
    (isShabbat(dateStr) ? "שבת" : null) ??
    (isErevShabbat(dateStr) ? "ערב שבת" : null)
  );
}

export function getCalendarDayAppearance(
  dateStr: string,
  today: string,
  holidays?: HolidayEntry[],
): CalendarDayAppearance {
  if (dateStr === today) {
    return "today";
  }

  const entry = holidays?.find((holiday) => holiday.date === dateStr);
  if (entry) {
    return entry.day_label === "ערב חג" ? "erev" : "holiday";
  }

  if (holidays) {
    const tomorrow = nextDateString(dateStr);
    const tomorrowEntry = holidays.find((holiday) => holiday.date === tomorrow);
    if (isErevChagLabel(tomorrowEntry?.day_label)) {
      return "erev";
    }
    if (isShabbat(dateStr)) {
      return "holiday";
    }
    if (isErevShabbat(dateStr)) {
      return "erev";
    }
    return "normal";
  }

  if (getJewishCalendarHoliday(dateStr) || isShabbat(dateStr)) {
    return "holiday";
  }

  if (getJewishCalendarHolidayEve(dateStr) || isErevShabbat(dateStr)) {
    return "erev";
  }

  return "normal";
}

export function getCalendarDayCellClass(appearance: CalendarDayAppearance): string {
  switch (appearance) {
    case "today":
      return "border-sky-500 bg-sky-100";
    case "holiday":
      return "border-teal-500 bg-teal-200";
    case "erev":
      return "border-teal-300 bg-teal-100";
    default:
      return "border-teal-100 bg-white";
  }
}

function holidayEntryFromDate(date: string): HolidayEntry | null {
  const { day, month } = getHebrewDateParts(date);
  const holiday = getJewishCalendarHoliday(date);
  if (holiday) {
    const openByDefault = isOpenByDefaultHoliday(holiday);
    const dayLabel =
      holiday === "י״ז בתמוז" ||
      holiday === 'י"ז בתמוז' ||
      holiday === "י׳ באב" ||
      holiday === "י' אב" ||
      holiday === "א׳ בסיוון" ||
      holiday === "א' בסיוון"
        ? "אחר"
        : "חג";
    return {
      date,
      title: holiday,
      holiday_group: holiday,
      day_label: dayLabel,
      hebrew: holiday,
      cancel_messages: !openByDefault,
      is_custom: false,
      open_time: null,
      close_time: null,
      hebrew_month: month,
      hebrew_day: day,
    };
  }

  const eve = getJewishCalendarHolidayEve(date);
  if (eve) {
    const group = eve.replace(/^ערב\s+/, "");
    return {
      date,
      title: eve,
      holiday_group: group,
      day_label: "ערב חג",
      hebrew: eve,
      cancel_messages: true,
      is_custom: false,
      hebrew_month: month,
      hebrew_day: day,
    };
  }

  return null;
}

function listJewishCalendarHolidaysForYear(year: number): HolidayEntry[] {
  const entries: HolidayEntry[] = [];
  const cursor = new Date(year, 0, 1);
  const end = new Date(year, 11, 31);

  while (cursor <= end) {
    const date = toDateString(cursor);
    const entry = holidayEntryFromDate(date);
    if (entry) {
      entries.push(entry);
    }
    cursor.setDate(cursor.getDate() + 1);
  }

  return entries;
}

export function listJewishCalendarHolidaysAroundNow(
  now: Date = new Date(),
): HolidayEntry[] {
  const year = now.getFullYear();
  const byDate = new Map<string, HolidayEntry>();

  for (const y of [year - 1, year, year + 1]) {
    for (const entry of listJewishCalendarHolidaysForYear(y)) {
      byDate.set(entry.date, entry);
    }
  }

  return [...byDate.values()].sort(compareHolidaysByHebrewDate);
}

export function findGregorianDatesForHebrewDay(
  hebrewMonth: string,
  hebrewDay: number,
  gregorianYear: number,
): string[] {
  const month = normalizeHebrewMonth(hebrewMonth);
  const matches: string[] = [];
  const cursor = new Date(gregorianYear, 0, 1);
  const end = new Date(gregorianYear, 11, 31);

  while (cursor <= end) {
    const date = toDateString(cursor);
    const parts = getHebrewDateParts(date);
    if (parts.day === hebrewDay && normalizeHebrewMonth(parts.month) === month) {
      matches.push(date);
    }
    cursor.setDate(cursor.getDate() + 1);
  }

  return matches;
}

export function withHebrewIdentity(entry: HolidayEntry): HolidayEntry {
  if (entry.hebrew_month && entry.hebrew_day != null) {
    return entry;
  }
  const { day, month } = getHebrewDateParts(entry.date);
  return {
    ...entry,
    hebrew_month: month,
    hebrew_day: day,
  };
}
