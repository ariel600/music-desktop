import { useCallback, useEffect, useState } from "react";
import {
  getDayBeforeErevAsThursday,
  getHolidays,
  getOperatingHours,
  getSchedules,
  getTasks,
  setDayBeforeErevAsThursday,
} from "../api";
import type { HolidayEntry, Schedule, Task } from "../types";
import {
  createDefaultOperatingHours,
  type OperatingHoursSettingsData,
} from "../lib/operatingHours";
import { getOperationalDate } from "../lib/operationalDay";
import { errMsg } from "../lib/errors";
import ScheduleMonthlyView from "./schedule/ScheduleMonthlyView";
import ToggleSwitch from "./ui/ToggleSwitch";

export default function SchedulesPage() {
  const [schedules, setSchedules] = useState<Schedule[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [holidays, setHolidays] = useState<HolidayEntry[]>([]);
  const [operatingHours, setOperatingHours] = useState<OperatingHoursSettingsData>(
    createDefaultOperatingHours,
  );
  const [focusDate, setFocusDate] = useState(getOperationalDate);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [dayBeforeErevAsThursday, setDayBeforeErevAsThursdayState] =
    useState(false);
  const [savingRule, setSavingRule] = useState(false);

  const loadData = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [scheduleData, taskData, holidayData, hoursData, erevRule] =
        await Promise.all([
          getSchedules(),
          getTasks(),
          getHolidays().catch(() => [] as HolidayEntry[]),
          getOperatingHours().catch(() => createDefaultOperatingHours()),
          getDayBeforeErevAsThursday().catch(() => false),
        ]);
      setSchedules(scheduleData);
      setTasks(taskData);
      setHolidays(holidayData);
      setOperatingHours(hoursData);
      setDayBeforeErevAsThursdayState(erevRule);
    } catch (err) {
      setError(errMsg(err, "שגיאה בטעינת לוחות הזמנים."));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  async function handleDayBeforeErevToggle(enabled: boolean) {
    setSavingRule(true);
    setError(null);
    setMessage(null);
    try {
      const saved = await setDayBeforeErevAsThursday(enabled);
      setDayBeforeErevAsThursdayState(saved);
      setMessage(
        saved
          ? "היום שלפני ערב חג יפעיל גם הודעות של יום חמישי (לפי הגדרת החגים)."
          : "בוטלה התאמת היום שלפני ערב חג ליום חמישי.",
      );
    } catch (err) {
      setError(errMsg(err, "שגיאה בשמירת הגדרת ערב חג."));
    } finally {
      setSavingRule(false);
    }
  }

  const scheduleId =
    schedules.find((schedule) => schedule.is_active)?.id ?? schedules[0]?.id;

  if (isLoading) {
    return (
      <p className="py-8 text-center text-sm text-teal-600">טוען לוח שנה...</p>
    );
  }

  if (error && !scheduleId) {
    return (
      <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">{error}</p>
    );
  }

  if (!scheduleId) {
    return (
      <p className="py-8 text-center text-sm text-teal-600">
        אין מערכות שעות פעילות.
      </p>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-3">
      <div className="flex shrink-0 flex-col gap-2 rounded-xl border border-teal-100 bg-teal-50/50 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="min-w-0">
          <p className="text-sm font-semibold text-teal-950">
            יום שלפני ערב חג כמו יום חמישי
          </p>
          <p className="mt-0.5 text-xs text-teal-700">
            מסונכרן עם הגדרת החגים: ביום שלפני «ערב חג» מופיעה התווית בלוח.
            כשהאפשרות פעילה — יושמעו גם הודעות של יום חמישי ושעות הפעילות
            יוצגו כשל חמישי.
          </p>
        </div>
        <ToggleSwitch
          checked={dayBeforeErevAsThursday}
          disabled={savingRule}
          onChange={(checked) => {
            void handleDayBeforeErevToggle(checked);
          }}
        />
      </div>

      {message && (
        <p className="shrink-0 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          {message}
        </p>
      )}
      {error && (
        <p className="shrink-0 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}

      <div className="min-h-0 flex-1">
        <ScheduleMonthlyView
          scheduleId={scheduleId}
          date={focusDate}
          tasks={tasks}
          holidays={holidays}
          operatingHours={operatingHours}
          dayBeforeErevAsThursday={dayBeforeErevAsThursday}
          onDateChange={setFocusDate}
        />
      </div>
    </div>
  );
}
