import { getActiveIsraelClockSeason } from "./israelClock";

type OperatingHoursSeason = "winter" | "summer";

type OperatingHoursSection = OperatingHoursSeason | "temporary";

export type OperatingDayId =
  | "sunday"
  | "monday"
  | "tuesday"
  | "wednesday"
  | "thursday"
  | "friday"
  | "motzei-shabbat";

export interface OperatingDayHours {
  open: string;
  close: string;
}

export type SeasonOperatingHours = Record<OperatingDayId, OperatingDayHours>;

export type TemporaryOperatingHours = SeasonOperatingHours & {
  valid_from?: string | null;
  valid_to?: string | null;
};

export interface OperatingHoursSettingsData {
  winter: SeasonOperatingHours;
  summer: SeasonOperatingHours;
  temporary: TemporaryOperatingHours;
}

export const OPERATING_DAYS: { id: OperatingDayId; label: string }[] = [
  { id: "sunday", label: "ראשון" },
  { id: "monday", label: "שני" },
  { id: "tuesday", label: "שלישי" },
  { id: "wednesday", label: "רביעי" },
  { id: "thursday", label: "חמישי" },
  { id: "friday", label: "שישי" },
  { id: "motzei-shabbat", label: "מוצאי שבת" },
];

const DEFAULT_OPERATING_DAY_HOURS: OperatingDayHours = {
  open: "00:00",
  close: "00:00",
};

const JERUSALEM_TIME_ZONE = "Asia/Jerusalem";

function createDefaultSeasonHours(): SeasonOperatingHours {
  return {
    sunday: { ...DEFAULT_OPERATING_DAY_HOURS },
    monday: { ...DEFAULT_OPERATING_DAY_HOURS },
    tuesday: { ...DEFAULT_OPERATING_DAY_HOURS },
    wednesday: { ...DEFAULT_OPERATING_DAY_HOURS },
    thursday: { ...DEFAULT_OPERATING_DAY_HOURS },
    friday: { ...DEFAULT_OPERATING_DAY_HOURS },
    "motzei-shabbat": { ...DEFAULT_OPERATING_DAY_HOURS },
  };
}

function createDefaultTemporaryHours(): TemporaryOperatingHours {
  return {
    ...createDefaultSeasonHours(),
    valid_from: null,
    valid_to: null,
  };
}

export function createDefaultOperatingHours(): OperatingHoursSettingsData {
  return {
    winter: createDefaultSeasonHours(),
    summer: createDefaultSeasonHours(),
    temporary: createDefaultTemporaryHours(),
  };
}

export function getSeasonDayHours(
  season: SeasonOperatingHours,
  dayId: OperatingDayId,
): OperatingDayHours {
  return season[dayId];
}

export function temporaryHoursOnly(
  temporary: TemporaryOperatingHours,
): SeasonOperatingHours {
  return {
    sunday: { ...temporary.sunday },
    monday: { ...temporary.monday },
    tuesday: { ...temporary.tuesday },
    wednesday: { ...temporary.wednesday },
    thursday: { ...temporary.thursday },
    friday: { ...temporary.friday },
    "motzei-shabbat": { ...temporary["motzei-shabbat"] },
  };
}

export function mergeTemporaryHours(
  hours: SeasonOperatingHours,
  validFrom: string | null,
  validTo: string | null,
): TemporaryOperatingHours {
  return {
    ...hours,
    valid_from: validFrom,
    valid_to: validTo,
  };
}

export function normalizeOperatingHours(
  data: OperatingHoursSettingsData,
): OperatingHoursSettingsData {
  const temporaryRaw = {
    ...createDefaultTemporaryHours(),
    ...data.temporary,
  };

  return {
    winter: data.winter ?? createDefaultSeasonHours(),
    summer: data.summer ?? createDefaultSeasonHours(),
    temporary: {
      ...createDefaultSeasonHours(),
      ...temporaryHoursOnly(temporaryRaw),
      valid_from: temporaryRaw.valid_from ?? null,
      valid_to: temporaryRaw.valid_to ?? null,
    },
  };
}

function getIsraelDateString(date: Date = new Date()): string {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone: JERUSALEM_TIME_ZONE,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(date);
}

export function isTemporaryOperatingHoursActive(
  temporary: TemporaryOperatingHours,
  date: Date = new Date(),
): boolean {
  return isTemporaryOperatingHoursActiveOnDate(
    temporary,
    getIsraelDateString(date),
  );
}

export function formatOperatingDateDisplay(value: string | null | undefined): string {
  if (!value || !/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return "בחרו תאריך";
  }
  const [year, month, day] = value.split("-");
  return `${day}/${month}/${year}`;
}

export function formatOperatingDateDots(
  value: string | null | undefined,
  emptyLabel = "לא הוגדר",
): string {
  if (!value || !/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return emptyLabel;
  }
  const [year, month, day] = value.split("-");
  return `${day}.${month}.${year}`;
}

export function normalizeTime24h(value: string): string | null {
  const trimmed = value.trim();
  const digits = trimmed.replace(/\D/g, "");

  let hours: string;
  let minutes: string;

  if (/^([01]\d|2[0-3]):([0-5]\d)$/.test(trimmed)) {
    [, hours, minutes] = trimmed.match(/^([01]\d|2[0-3]):([0-5]\d)$/)!;
  } else if (digits.length === 4) {
    hours = digits.slice(0, 2);
    minutes = digits.slice(2, 4);
    if (Number(hours) > 23 || Number(minutes) > 59) {
      return null;
    }
  } else {
    return null;
  }

  return `${hours.padStart(2, "0")}:${minutes.padStart(2, "0")}`;
}

export function formatTimeDigits(raw: string): string {
  const digits = raw.replace(/\D/g, "").slice(0, 4);
  if (digits.length <= 2) {
    return digits;
  }
  return `${digits.slice(0, 2)}:${digits.slice(2)}`;
}

function isOperatingDayClosed(hours: OperatingDayHours): boolean {
  return hours.open === "00:00" && hours.close === "00:00";
}

function isTemporaryOperatingHoursActiveOnDate(
  temporary: TemporaryOperatingHours,
  dateStr: string,
): boolean {
  const from = temporary.valid_from?.trim() ?? "";
  const to = temporary.valid_to?.trim() ?? "";
  if (!/^\d{4}-\d{2}-\d{2}$/.test(from) || !/^\d{4}-\d{2}-\d{2}$/.test(to)) {
    return false;
  }
  if (from > to) {
    return false;
  }
  return dateStr >= from && dateStr <= to;
}

export function weekdayToOperatingDayId(dateStr: string): OperatingDayId {
  const weekday = new Date(`${dateStr}T12:00:00`).getDay();
  return weekdayIndexToOperatingDayId(weekday);
}

export function weekdayIndexToOperatingDayId(weekday: number): OperatingDayId {
  const map: OperatingDayId[] = [
    "sunday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "motzei-shabbat",
  ];
  return map[Number.isNaN(weekday) ? 0 : weekday] ?? "sunday";
}

function getOperatingHoursForDate(
  settings: OperatingHoursSettingsData,
  dateStr: string,
  weekdayOverride?: number,
): { hours: OperatingDayHours; source: OperatingHoursSection } {
  const dayId =
    weekdayOverride === undefined
      ? weekdayToOperatingDayId(dateStr)
      : weekdayIndexToOperatingDayId(weekdayOverride);
  if (isTemporaryOperatingHoursActiveOnDate(settings.temporary, dateStr)) {
    return {
      hours: { ...settings.temporary[dayId] },
      source: "temporary",
    };
  }

  const season = getActiveIsraelClockSeason(new Date(`${dateStr}T12:00:00`));
  return {
    hours: { ...settings[season][dayId] },
    source: season,
  };
}

type CalendarDayHoursDisplay =
  | { status: "closed" }
  | { status: "open"; open: string; close: string };

export function resolveCalendarDayHours(
  dateStr: string,
  settings: OperatingHoursSettingsData,
  holiday?: { cancel_messages: boolean; open_time?: string | null; close_time?: string | null } | null,
  /** When true (day before ערב חג as Thursday), use Thursday operating hours. */
  treatAsThursday = false,
): CalendarDayHoursDisplay {
  if (holiday) {
    if (holiday.cancel_messages) {
      return { status: "closed" };
    }
    const open = holiday.open_time?.trim() ?? "";
    const close = holiday.close_time?.trim() ?? "";
    if (open && close) {
      if (open === "00:00" && close === "00:00") {
        return { status: "closed" };
      }
      return { status: "open", open, close };
    }
  }

  const { hours } = getOperatingHoursForDate(
    settings,
    dateStr,
    treatAsThursday ? 4 : undefined,
  );
  if (isOperatingDayClosed(hours)) {
    return { status: "closed" };
  }
  return { status: "open", open: hours.open, close: hours.close };
}

export function formatCalendarDayHours(
  display: CalendarDayHoursDisplay,
): string {
  if (display.status === "closed") {
    return "סגור";
  }
  return `${display.open}–${display.close}`;
}
