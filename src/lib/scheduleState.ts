import type {
  ScheduleOverridesBundle,
  Task,
} from "../types";
import { matchesWeekdays } from "./dayBeforeErev";

export function isDayDisabled(
  date: string,
  scheduleId: number,
  overrides: ScheduleOverridesBundle,
): boolean {
  return overrides.day_overrides.some(
    (entry) =>
      entry.date === date &&
      entry.is_disabled &&
      (entry.schedule_id === 0 || entry.schedule_id === scheduleId),
  );
}

export function countTasksForDay(
  tasks: Task[],
  scheduleId: number,
  _date: string,
  weekday: number,
  treatAsThursday = false,
): number {
  return tasks.filter(
    (task) =>
      task.schedule_id === scheduleId &&
      matchesWeekdays(task.days_of_week, weekday, treatAsThursday),
  ).length;
}
