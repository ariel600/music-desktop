import type { HolidayEntry } from "../types";
import { getActiveIsraelClockSeason } from "./israelClock";
import {
  type OperatingDayHours,
  type OperatingHoursSettingsData,
  weekdayToOperatingDayId,
} from "./operatingHours";
import { parseDateString } from "./operationalDay";
import { getJerusalemHebrewDateString } from "./jerusalemDate";

export type HolidayStatusKind = "חג" | "ערב חג" | "אחר";

export const HOLIDAY_STATUS_KINDS: { id: HolidayStatusKind; label: string }[] = [
  { id: "חג", label: "חג" },
  { id: "ערב חג", label: "ערב חג" },
  { id: "אחר", label: "אחר" },
];

export function formatHolidayDate(date: string): string {
  const parsed = new Date(`${date}T12:00:00`);
  if (Number.isNaN(parsed.getTime())) {
    return date;
  }
  return parsed.toLocaleDateString("he-IL", {
    weekday: "short",
    day: "numeric",
    month: "short",
  });
}

export function isToday(date: string): boolean {
  return date === getJerusalemHebrewDateString();
}

export function dayDisplayName(holiday: HolidayEntry): string {
  if (holiday.day_label === "ערב חג") {
    return `ערב ${holiday.holiday_group}`;
  }
  if (holiday.day_label === "חג" || holiday.is_custom) {
    return holiday.title;
  }
  return `${holiday.holiday_group} · ${holiday.day_label}`;
}

export function normalizeHolidayStatusKind(dayLabel: string): HolidayStatusKind {
  if (dayLabel === "ערב חג") {
    return "ערב חג";
  }
  if (dayLabel === "אחר") {
    return "אחר";
  }
  return "חג";
}

export function getDefaultHolidayHours(
  kind: HolidayStatusKind,
  dateStr: string,
  operatingHours: OperatingHoursSettingsData,
): OperatingDayHours {
  const season = operatingHours[getActiveIsraelClockSeason(parseDateString(dateStr))];
  if (kind === "ערב חג") {
    return { ...season.friday };
  }
  const dayId = weekdayToOperatingDayId(dateStr);
  return { ...season[dayId] };
}

export function holidayIsOpen(holiday: HolidayEntry): boolean {
  return !holiday.cancel_messages;
}

export function isManualHoliday(holiday: HolidayEntry): boolean {
  return Boolean(holiday.is_custom);
}
