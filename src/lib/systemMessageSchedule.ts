import { formatSystemMessageDays } from "../types";

export type SystemMessageScheduleMode = "fixed_time" | "relative_operating_hours";
export type SystemMessageOperatingAnchor = "open" | "close";
export type SystemMessageOffsetDirection = "before" | "after";

export interface SystemMessageScheduleInput {
  daysOfWeek: number[];
  scheduleMode: SystemMessageScheduleMode;
  scheduledTime?: string | null;
  operatingAnchor?: SystemMessageOperatingAnchor | null;
  offsetDirection?: SystemMessageOffsetDirection | null;
  offsetMinutes?: number | null;
}

const ANCHOR_LABELS: Record<SystemMessageOperatingAnchor, string> = {
  open: "שעת פתיחה",
  close: "שעת סגירה",
};

const DIRECTION_LABELS: Record<SystemMessageOffsetDirection, string> = {
  before: "לפני",
  after: "אחרי",
};

export function formatSystemMessageSchedule(message: {
  schedule_mode: SystemMessageScheduleMode;
  scheduled_time?: string | null;
  operating_anchor?: SystemMessageOperatingAnchor | null;
  offset_direction?: SystemMessageOffsetDirection | null;
  offset_minutes?: number | null;
  days_of_week: number[];
}): string {
  const days = formatSystemMessageDays(message.days_of_week);

  if (message.schedule_mode === "fixed_time" && message.scheduled_time) {
    return `${days} · בשעה ${message.scheduled_time}`;
  }

  if (
    message.schedule_mode === "relative_operating_hours" &&
    message.operating_anchor &&
    message.offset_direction &&
    message.offset_minutes
  ) {
    return `${days} · ${message.offset_minutes} דקות ${DIRECTION_LABELS[message.offset_direction]} ${ANCHOR_LABELS[message.operating_anchor]}`;
  }

  return days;
}
