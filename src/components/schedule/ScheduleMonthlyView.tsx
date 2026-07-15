import { useCallback, useEffect, useMemo, useState } from "react";
import { getScheduleOverrides } from "../../api";
import type { HolidayEntry, ScheduleOverridesBundle, Task } from "../../types";
import {
  formatHebrewDayInCell,
  formatHebrewMonthYear,
  getHebrewMonthGrid,
  getHebrewMonthRange,
  shiftHebrewMonth,
} from "../../lib/hebrewCalendar";
import { formatGregorianDayMonth, getOperationalDate, parseDateString } from "../../lib/operationalDay";
import {
  getCalendarDayAppearance,
  getCalendarDayCellClass,
  getCalendarDayLabel,
} from "../../lib/hebrewHolidays";
import {
  formatCalendarDayHours,
  resolveCalendarDayHours,
  type OperatingHoursSettingsData,
} from "../../lib/operatingHours";
import { treatDayAsThursday } from "../../lib/dayBeforeErev";
import { countTasksForDay, isDayDisabled } from "../../lib/scheduleState";

const WEEKDAY_LABELS = [
  "ראשון",
  "שני",
  "שלישי",
  "רביעי",
  "חמישי",
  "שישי",
  "שבת",
];

interface ScheduleMonthlyViewProps {
  scheduleId: number;
  date: string;
  tasks: Task[];
  holidays: HolidayEntry[];
  operatingHours: OperatingHoursSettingsData;
  dayBeforeErevAsThursday?: boolean;
  onDateChange: (date: string) => void;
}

export default function ScheduleMonthlyView({
  scheduleId,
  date,
  tasks,
  holidays,
  operatingHours,
  dayBeforeErevAsThursday = false,
  onDateChange,
}: ScheduleMonthlyViewProps) {
  const { start: monthStart, end: monthEnd } = useMemo(
    () => getHebrewMonthRange(date),
    [date],
  );
  const cells = useMemo(() => getHebrewMonthGrid(date), [date]);
  const monthTitle = useMemo(
    () => formatHebrewMonthYear(parseDateString(date)),
    [date],
  );
  const today = getOperationalDate();
  const holidaysByDate = useMemo(() => {
    const map = new Map<string, HolidayEntry>();
    for (const holiday of holidays) {
      map.set(holiday.date, holiday);
    }
    return map;
  }, [holidays]);

  const [overrides, setOverrides] = useState<ScheduleOverridesBundle>({
    day_overrides: [],
    task_overrides: [],
  });

  const loadOverrides = useCallback(async () => {
    const data = await getScheduleOverrides(monthStart, monthEnd);
    setOverrides(data);
  }, [monthStart, monthEnd]);

  useEffect(() => {
    void loadOverrides();
  }, [loadOverrides]);

  function shiftMonth(delta: number) {
    onDateChange(shiftHebrewMonth(date, delta));
  }

  const weekCount = Math.ceil(cells.length / 7);

  return (
    <div className="flex h-full min-h-0 flex-col gap-3">
      <div className="flex shrink-0 items-center justify-center">
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => shiftMonth(-1)}
            className="flex h-9 w-9 items-center justify-center rounded-lg bg-teal-100 text-lg text-teal-800 transition-colors hover:bg-teal-200"
            aria-label="חודש קודם"
          >
            ‹
          </button>
          <h3 className="min-w-[10rem] text-center text-lg font-bold text-teal-900">
            {monthTitle}
          </h3>
          <button
            type="button"
            onClick={() => shiftMonth(1)}
            className="flex h-9 w-9 items-center justify-center rounded-lg bg-teal-100 text-lg text-teal-800 transition-colors hover:bg-teal-200"
            aria-label="חודש הבא"
          >
            ›
          </button>
        </div>
      </div>

      <div className="grid shrink-0 grid-cols-7 gap-1 text-center text-[11px] font-semibold text-teal-600 sm:text-xs">
        {WEEKDAY_LABELS.map((label) => (
          <div key={label}>{label}</div>
        ))}
      </div>

      <div
        className="grid min-h-0 flex-1 grid-cols-7 gap-1"
        style={{ gridTemplateRows: `repeat(${weekCount}, minmax(0, 1fr))` }}
      >
        {cells.map((cell, index) => {
          if (!cell) {
            return <div key={`empty-${index}`} className="h-full" />;
          }

          const weekday = parseDateString(cell.dateStr).getDay();
          const asThursday = treatDayAsThursday(
            cell.dateStr,
            dayBeforeErevAsThursday,
            holidaysByDate,
          );
          const taskCount = countTasksForDay(
            tasks,
            scheduleId,
            cell.dateStr,
            weekday,
            asThursday,
          );
          const disabled = isDayDisabled(cell.dateStr, scheduleId, overrides);
          const appearance = getCalendarDayAppearance(
            cell.dateStr,
            today,
            holidays,
          );
          const dayLabel = getCalendarDayLabel(cell.dateStr, holidays);
          const holiday = holidaysByDate.get(cell.dateStr);
          const hoursDisplay = resolveCalendarDayHours(
            cell.dateStr,
            operatingHours,
            holiday,
            asThursday,
          );
          const hoursText = formatCalendarDayHours(hoursDisplay);
          const closed = hoursDisplay.status === "closed";

          return (
            <div
              key={cell.dateStr}
              className={`flex h-full w-full flex-col items-start rounded-lg border p-1.5 text-right ${getCalendarDayCellClass(appearance)} ${disabled ? "opacity-60" : ""}`}
            >
              <span className="block text-sm font-bold text-teal-900">
                {formatHebrewDayInCell(cell.hebrewDay)}
              </span>
              <span
                className={`block text-[10px] ${
                  appearance === "today" ? "text-sky-800" : "text-teal-500"
                }`}
              >
                {formatGregorianDayMonth(cell.dateStr)}
              </span>
              <span
                className={`mt-0.5 block text-[10px] font-semibold tabular-nums leading-tight ${
                  closed ? "text-rose-700" : "text-teal-800"
                }`}
              >
                {hoursText}
              </span>
              {taskCount > 0 && (
                <span className="mt-1 block text-[10px] text-teal-600">
                  {taskCount} הודעות
                </span>
              )}
              {dayLabel && (
                <span className="mt-auto self-end text-[10px] font-medium leading-tight text-teal-900">
                  {dayLabel}
                </span>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
