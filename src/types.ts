export interface Schedule {
  id: number;
  name: string;
  is_active: boolean;
}

export interface Task {
  id: number;
  title: string;
  file_path: string;
  scheduled_time: string;
  is_active: boolean;
  last_played_date?: string | null;
  schedule_id?: number | null;
  schedule_name?: string | null;
  days_of_week: number[];
  volume?: number | null;
  cancel_on_holiday: boolean;
}

export interface PlayLogEntry {
  id: number;
  task_id?: number | null;
  task_title: string;
  played_at: string;
  status: string;
}

export interface HolidayEntry {
  date: string;
  title: string;
  holiday_group: string;
  day_label: string;
  hebrew?: string | null;
  cancel_messages: boolean;
  is_custom: boolean;
  open_time?: string | null;
  close_time?: string | null;
  hebrew_month?: string | null;
  hebrew_day?: number | null;
}

interface DayOverrideEntry {
  date: string;
  schedule_id: number;
  is_disabled: boolean;
}

interface TaskOverrideEntry {
  date: string;
  task_id: number;
  is_disabled: boolean;
}

export interface ScheduleOverridesBundle {
  day_overrides: DayOverrideEntry[];
  task_overrides: TaskOverrideEntry[];
}

export interface MusicFileEntry {
  name: string;
  file_name: string;
  path: string;
  size_bytes: number;
}

export interface EmergencyMessageAudioFile {
  message_type: string;
  name?: string | null;
  path?: string | null;
}

export interface SystemMessage {
  id: number;
  title: string;
  file_path: string;
  audio_name?: string | null;
  is_active: boolean;
  days_of_week: number[];
  schedule_mode: "fixed_time" | "relative_operating_hours";
  scheduled_time?: string | null;
  operating_anchor?: "open" | "close" | null;
  offset_direction?: "before" | "after" | null;
  offset_minutes?: number | null;
}

export interface EmergencyAlertPayload {
  id: string;
  message_type: string;
  title: string;
  description?: string | null;
  cities: string[];
  received_at: string;
}

export interface ScannedMusicFile {
  name: string;
  source_path: string;
  size_bytes: number;
  will_convert_to_mp3: boolean;
}

const WEEKDAYS = [
  { value: 0, label: "א׳" },
  { value: 1, label: "ב׳" },
  { value: 2, label: "ג׳" },
  { value: 3, label: "ד׳" },
  { value: 4, label: "ה׳" },
  { value: 5, label: "ו׳" },
  { value: 6, label: "ש׳" },
] as const;

export const DEFAULT_SCHOOL_DAYS = [0, 1, 2, 3, 4];

const SYSTEM_MESSAGE_DAY_HOLIDAY_EVE = 7;
const SYSTEM_MESSAGE_DAY_HOLIDAY = 8;

export const SYSTEM_MESSAGE_DAYS = [
  ...WEEKDAYS,
  { value: SYSTEM_MESSAGE_DAY_HOLIDAY_EVE, label: "ערב חג" },
  { value: SYSTEM_MESSAGE_DAY_HOLIDAY, label: "חג" },
] as const;

export function formatSystemMessageDays(days: number[]): string {
  return days
    .slice()
    .sort((a, b) => a - b)
    .map(
      (day) =>
        SYSTEM_MESSAGE_DAYS.find((item) => item.value === day)?.label ??
        String(day),
    )
    .join(", ");
}
