import { invoke } from "@tauri-apps/api/core";
import type {
  EmergencyMessageAudioFile,
  HolidayEntry,
  MusicFileEntry,
  PlayLogEntry,
  Schedule,
  ScannedMusicFile,
  SystemMessage,
  Task,
} from "./types";
import type { SystemMessageScheduleInput } from "./lib/systemMessageSchedule";
import type { OperatingHoursSettingsData } from "./lib/operatingHours";
import { listJewishCalendarHolidaysAroundNow, findGregorianDatesForHebrewDay, getHebrewDateParts, withHebrewIdentity } from "./lib/hebrewHolidays";
import {
  ensureSettingsAuth,
  type SettingsAuthKind,
} from "./lib/settingsAuth";

async function guardedInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
  kind: SettingsAuthKind = "settings",
): Promise<T> {
  const ok = await ensureSettingsAuth(kind);
  if (!ok) {
    throw new Error("הפעולה בוטלה — נדרשת סיסמת הגדרות.");
  }
  return invoke<T>(cmd, args);
}

export async function getTasks(): Promise<Task[]> {
  return invoke<Task[]>("get_tasks");
}

export async function getSchedules(): Promise<Schedule[]> {
  return invoke<Schedule[]>("get_schedules");
}

export type AudioVolumeChannel =
  | "general"
  | "music"
  | "system"
  | "emergency";

export interface AudioVolumes {
  general: number;
  music: number;
  system: number;
  emergency: number;
}

export async function playAudio(
  filePath: string,
  volume?: number | null,
  channel?: AudioVolumeChannel,
): Promise<void> {
  return invoke("play_audio", {
    filePath,
    volume: volume ?? undefined,
    channel: channel ?? undefined,
  });
}

export async function getAudioVolumes(): Promise<AudioVolumes> {
  return invoke<AudioVolumes>("get_audio_volumes");
}

export async function setAudioVolumeChannel(
  channel: AudioVolumeChannel,
  volume: number,
): Promise<AudioVolumes> {
  return guardedInvoke<AudioVolumes>("set_audio_volume_channel", {
    channel,
    volume,
  });
}

export async function playVolumeTest(
  channel: AudioVolumeChannel,
  volume?: number | null,
): Promise<void> {
  return invoke("play_volume_test_channel", {
    channel,
    volume: volume ?? null,
  });
}

export async function getPlayLog(limit = 50): Promise<PlayLogEntry[]> {
  return invoke<PlayLogEntry[]>("get_play_log", { limit });
}

export async function exportBackup(destPath: string): Promise<string> {
  return guardedInvoke<string>("export_backup", { destPath });
}

export async function importBackup(sourcePath: string): Promise<void> {
  return guardedInvoke("import_backup", { sourcePath });
}

export async function resetSystem(keepMusic = false): Promise<void> {
  return guardedInvoke("reset_system", { keepMusic });
}

export async function hasAppPassword(): Promise<boolean> {
  return invoke<boolean>("has_app_password");
}

export async function verifyAppPassword(password: string): Promise<boolean> {
  return invoke<boolean>("verify_app_password", { password });
}

export async function setAppPassword(
  newPassword: string,
  currentPassword?: string | null,
): Promise<void> {
  return invoke("set_app_password", {
    newPassword,
    currentPassword: currentPassword ?? null,
  });
}

export async function clearAppPassword(currentPassword: string): Promise<void> {
  return invoke("clear_app_password", { currentPassword });
}

export async function hasSettingsPassword(): Promise<boolean> {
  return invoke<boolean>("has_settings_password");
}

export async function verifySettingsPassword(
  password: string,
): Promise<boolean> {
  return invoke<boolean>("verify_settings_password", { password });
}

export async function setSettingsPassword(
  newPassword: string,
  currentPassword?: string | null,
): Promise<void> {
  return invoke("set_settings_password", {
    newPassword,
    currentPassword: currentPassword ?? null,
  });
}

export async function clearSettingsPassword(
  currentPassword: string,
): Promise<void> {
  return invoke("clear_settings_password", { currentPassword });
}

export async function getLockMusicAdd(): Promise<boolean> {
  return invoke<boolean>("get_lock_music_add");
}

export async function setLockMusicAdd(enabled: boolean): Promise<boolean> {
  return guardedInvoke<boolean>("set_lock_music_add", { enabled });
}

export async function getSystemActive(): Promise<boolean> {
  return invoke<boolean>("get_system_active");
}

export async function setSystemActive(active: boolean): Promise<boolean> {
  return invoke<boolean>("set_system_active", { active });
}

export async function getDayBeforeErevAsThursday(): Promise<boolean> {
  return invoke<boolean>("get_day_before_erev_as_thursday");
}

export async function setDayBeforeErevAsThursday(
  enabled: boolean,
): Promise<boolean> {
  return guardedInvoke<boolean>("set_day_before_erev_as_thursday", { enabled });
}

export async function getOperatingHours(): Promise<OperatingHoursSettingsData> {
  return invoke<OperatingHoursSettingsData>("get_operating_hours");
}

export async function setOperatingHours(
  settings: OperatingHoursSettingsData,
): Promise<OperatingHoursSettingsData> {
  return guardedInvoke<OperatingHoursSettingsData>("set_operating_hours", {
    settings,
  });
}

type EmergencyMessageSettings = Record<string, boolean>;

export async function getEmergencyMessageSettings(): Promise<EmergencyMessageSettings> {
  return invoke<EmergencyMessageSettings>("get_emergency_message_settings");
}

export async function setEmergencyMessageEnabled(
  messageType: string,
  enabled: boolean,
): Promise<EmergencyMessageSettings> {
  return guardedInvoke<EmergencyMessageSettings>("set_emergency_message_enabled", {
    messageType,
    enabled,
  });
}

export async function getEmergencyMessageAudioFiles(): Promise<EmergencyMessageAudioFile[]> {
  return invoke<EmergencyMessageAudioFile[]>("get_emergency_message_audio_files");
}

export async function importEmergencyMessageAudio(
  messageType: string,
  sourcePath: string,
): Promise<EmergencyMessageAudioFile> {
  return guardedInvoke<EmergencyMessageAudioFile>("import_emergency_message_audio", {
    messageType,
    sourcePath,
  });
}

export type OrefCity = {
  name: string;
  name_en: string;
  zone: string;
};

export async function getOrefCities(): Promise<OrefCity[]> {
  return invoke<OrefCity[]>("get_oref_cities");
}

export async function getEmergencyMonitoredCities(): Promise<string[]> {
  return invoke<string[]>("get_emergency_monitored_cities");
}

export async function setEmergencyMonitoredCities(cities: string[]): Promise<string[]> {
  return guardedInvoke<string[]>("set_emergency_monitored_cities", { cities });
}

export async function getSystemMessages(): Promise<SystemMessage[]> {
  return invoke<SystemMessage[]>("get_system_messages");
}

export async function addSystemMessage(
  title: string,
  sourcePath: string,
  schedule: SystemMessageScheduleInput,
): Promise<SystemMessage> {
  return guardedInvoke<SystemMessage>("add_system_message", {
    title,
    sourcePath,
    daysOfWeek: schedule.daysOfWeek,
    scheduleMode: schedule.scheduleMode,
    scheduledTime: schedule.scheduledTime ?? null,
    operatingAnchor: schedule.operatingAnchor ?? null,
    offsetDirection: schedule.offsetDirection ?? null,
    offsetMinutes: schedule.offsetMinutes ?? null,
  });
}

export async function updateSystemMessage(
  id: number,
  title: string,
  schedule: SystemMessageScheduleInput,
): Promise<SystemMessage> {
  return guardedInvoke<SystemMessage>("update_system_message", {
    id,
    title,
    daysOfWeek: schedule.daysOfWeek,
    scheduleMode: schedule.scheduleMode,
    scheduledTime: schedule.scheduledTime ?? null,
    operatingAnchor: schedule.operatingAnchor ?? null,
    offsetDirection: schedule.offsetDirection ?? null,
    offsetMinutes: schedule.offsetMinutes ?? null,
  });
}

export async function setSystemMessageEnabled(
  id: number,
  enabled: boolean,
): Promise<SystemMessage> {
  return guardedInvoke<SystemMessage>("set_system_message_enabled", {
    id,
    enabled,
  });
}

export async function updateSystemMessageAudio(
  id: number,
  sourcePath: string,
): Promise<SystemMessage> {
  return guardedInvoke<SystemMessage>("update_system_message_audio", {
    id,
    sourcePath,
  });
}

export async function deleteSystemMessage(id: number): Promise<void> {
  return guardedInvoke("delete_system_message", { id });
}

function customRecurrenceKey(holiday: HolidayEntry): string | null {
  if (!holiday.is_custom || !holiday.hebrew_month || holiday.hebrew_day == null) {
    return null;
  }
  return `${holiday.hebrew_month}|${holiday.hebrew_day}|${holiday.holiday_group}`;
}

function buildCustomRecurrenceEntries(list: HolidayEntry[]): HolidayEntry[] {
  const year = new Date().getFullYear();
  const templates = new Map<string, HolidayEntry>();

  for (const holiday of list) {
    const keyed = withHebrewIdentity(holiday);
    const key = customRecurrenceKey(keyed);
    if (!key) {
      continue;
    }
    if (!templates.has(key)) {
      templates.set(key, keyed);
    }
  }

  const expansions: HolidayEntry[] = [];
  for (const template of templates.values()) {
    const month = template.hebrew_month!;
    const day = template.hebrew_day!;
    for (const y of [year - 1, year, year + 1]) {
      for (const date of findGregorianDatesForHebrewDay(month, day, y)) {
        expansions.push({
          ...template,
          date,
          is_custom: true,
          hebrew_month: month,
          hebrew_day: day,
        });
      }
    }
  }
  return expansions;
}

export async function getHolidays(): Promise<HolidayEntry[]> {
  const syncedToday = await invoke<boolean>("is_calendar_synced_today").catch(() => false);
  let list: HolidayEntry[];
  if (syncedToday) {
    list = await invoke<HolidayEntry[]>("get_holidays_list");
  } else {
    const entries = listJewishCalendarHolidaysAroundNow();
    list = await invoke<HolidayEntry[]>("sync_calendar_holidays", { entries });
  }
  const expansions = buildCustomRecurrenceEntries(list);
  if (expansions.length > 0) {
    const existing = new Set(list.map((holiday) => holiday.date));
    const missing = expansions.some((entry) => !existing.has(entry.date));
    if (missing) {
      list = await invoke<HolidayEntry[]>("ensure_custom_recurrences", {
        entries: expansions,
      });
    }
  }
  return list;
}

export async function setHolidayStatus(
  date: string,
  dayLabel: string,
  cancelMessages: boolean,
  openTime: string | null,
  closeTime: string | null,
): Promise<HolidayEntry[]> {
  return guardedInvoke<HolidayEntry[]>("set_holiday_status", {
    date,
    dayLabel,
    cancelMessages,
    openTime,
    closeTime,
  });
}

export async function addCustomHoliday(
  date: string,
  title: string,
  cancelMessages = true,
  dayLabel: string | null = "חג",
  openTime: string | null = null,
  closeTime: string | null = null,
): Promise<HolidayEntry[]> {
  const { day, month } = getHebrewDateParts(date);
  const year = new Date().getFullYear();

  let list = await guardedInvoke<HolidayEntry[]>("add_custom_holiday", {
    date,
    title,
    cancelMessages,
    dayLabel,
    openTime,
    closeTime,
    hebrewMonth: month,
    hebrewDay: day,
  });

  const expansions: HolidayEntry[] = [];
  const template = list.find((entry) => entry.date === date) ?? {
    date,
    title,
    holiday_group: title,
    day_label: dayLabel ?? "חג",
    hebrew: title,
    cancel_messages: cancelMessages,
    is_custom: true,
    open_time: openTime,
    close_time: closeTime,
    hebrew_month: month,
    hebrew_day: day,
  };

  for (const y of [year - 1, year, year + 1]) {
    for (const occurrence of findGregorianDatesForHebrewDay(month, day, y)) {
      expansions.push({
        ...template,
        date: occurrence,
        is_custom: true,
        hebrew_month: month,
        hebrew_day: day,
      });
    }
  }

  if (expansions.length > 0) {
    list = await guardedInvoke<HolidayEntry[]>("ensure_custom_recurrences", {
      entries: expansions,
    });
  }

  return list;
}

export async function deleteCustomHoliday(date: string): Promise<HolidayEntry[]> {
  return guardedInvoke<HolidayEntry[]>("delete_custom_holiday", { date });
}

export async function getScheduleOverrides(
  start: string,
  end: string,
): Promise<import("./types").ScheduleOverridesBundle> {
  return invoke("get_schedule_overrides", { start, end });
}

export async function listMusicFiles(folder: string): Promise<MusicFileEntry[]> {
  return invoke<MusicFileEntry[]>("list_music_files", { folder });
}

export interface OverviewNowPlaying {
  title: string;
  filePath: string;
  folder: string | null;
  artworkDataUrl: string | null;
}

export interface OverviewHours {
  closed: boolean;
  open: string | null;
  close: string | null;
}

export interface OverviewSnapshot {
  totalSongs: number;
  nowPlaying: OverviewNowPlaying | null;
  systemMessagesTotal: number;
  systemMessagesToday: number;
  musicFolder: string | null;
  emergencyPlaysToday: number;
  operatingHours: OverviewHours;
  sessionUptimeSeconds: number;
}

export async function getOverviewSnapshot(): Promise<OverviewSnapshot> {
  return invoke<OverviewSnapshot>("get_overview_snapshot");
}

export async function scanMusicSources(
  sourcePaths: string[],
): Promise<ScannedMusicFile[]> {
  return invoke<ScannedMusicFile[]>("scan_music_sources", { sourcePaths });
}

export async function importMusicFiles(
  folder: string,
  sourcePaths: string[],
  vocalWarningAcknowledged = true,
): Promise<MusicFileEntry[]> {
  return guardedInvoke<MusicFileEntry[]>(
    "import_music_files",
    {
      folder,
      sourcePaths,
      vocalWarningAcknowledged,
    },
    "music",
  );
}

export async function deleteMusicFiles(
  folder: string,
  filePaths: string[],
): Promise<number> {
  return guardedInvoke<number>(
    "delete_music_files",
    { folder, filePaths },
    "music",
  );
}
