import type { HolidayEntry } from "../types";
import { nextDateString } from "./operationalDay";

export const WEEKDAY_THURSDAY = 4;

export function isErevChagLabel(dayLabel: string | null | undefined): boolean {
  return (dayLabel ?? "").trim() === "ערב חג";
}

/** True when the day after `dateStr` is marked "ערב חג" in holiday settings. */
export function tomorrowIsErevChag(
  dateStr: string,
  holidaysByDate: Map<string, HolidayEntry> | HolidayEntry[],
): boolean {
  const tomorrow = nextDateString(dateStr);
  if (holidaysByDate instanceof Map) {
    return isErevChagLabel(holidaysByDate.get(tomorrow)?.day_label);
  }
  const holiday = holidaysByDate.find((entry) => entry.date === tomorrow);
  return isErevChagLabel(holiday?.day_label);
}

export function treatDayAsThursday(
  dateStr: string,
  dayBeforeErevAsThursday: boolean,
  holidaysByDate: Map<string, HolidayEntry> | HolidayEntry[],
): boolean {
  return (
    dayBeforeErevAsThursday && tomorrowIsErevChag(dateStr, holidaysByDate)
  );
}

export function matchesWeekdays(
  daysOfWeek: number[],
  weekday: number,
  treatAsThursday: boolean,
): boolean {
  if (daysOfWeek.includes(weekday)) {
    return true;
  }
  return treatAsThursday && daysOfWeek.includes(WEEKDAY_THURSDAY);
}
