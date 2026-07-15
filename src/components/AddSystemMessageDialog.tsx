import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { addSystemMessage } from "../api";
import type {
  SystemMessageOffsetDirection,
  SystemMessageOperatingAnchor,
  SystemMessageScheduleMode,
} from "../lib/systemMessageSchedule";
import { DEFAULT_SCHOOL_DAYS, SYSTEM_MESSAGE_DAYS } from "../types";
import { errMsg } from "../lib/errors";

const inputClass =
  "w-full rounded-lg border border-teal-200 bg-white px-3 py-2 text-sm text-teal-900 shadow-sm focus:border-teal-500 focus:outline-none focus:ring-1 focus:ring-teal-500";

import { AUDIO_FILTERS } from "../lib/audioFilters";

interface AddSystemMessageDialogProps {
  onClose: () => void;
  onSaved: () => void;
}

export default function AddSystemMessageDialog({
  onClose,
  onSaved,
}: AddSystemMessageDialogProps) {
  const [title, setTitle] = useState("");
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [selectedDays, setSelectedDays] = useState<number[]>(DEFAULT_SCHOOL_DAYS);
  const [scheduleMode, setScheduleMode] =
    useState<SystemMessageScheduleMode>("fixed_time");
  const [scheduledTime, setScheduledTime] = useState("08:00");
  const [operatingAnchor, setOperatingAnchor] =
    useState<SystemMessageOperatingAnchor>("open");
  const [offsetDirection, setOffsetDirection] =
    useState<SystemMessageOffsetDirection>("before");
  const [offsetMinutes, setOffsetMinutes] = useState("5");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function toggleDay(day: number) {
    setSelectedDays((current) =>
      current.includes(day)
        ? current.filter((value) => value !== day)
        : [...current, day].sort((a, b) => a - b),
    );
  }

  async function handleSelectFile() {
    setError(null);

    const selected = await open({
      multiple: false,
      filters: AUDIO_FILTERS,
    });

    if (typeof selected === "string") {
      setSelectedFile(selected);
    }
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    setError(null);

    if (!title.trim()) {
      setError("יש להזין כותרת להודעה.");
      return;
    }

    if (!selectedFile) {
      setError("יש לבחור קובץ שמע.");
      return;
    }

    if (selectedDays.length === 0) {
      setError("יש לבחור לפחות יום אחד.");
      return;
    }

    const parsedOffsetMinutes = Number(offsetMinutes);
    if (scheduleMode === "relative_operating_hours") {
      if (!Number.isFinite(parsedOffsetMinutes) || parsedOffsetMinutes <= 0) {
        setError("יש להזין מספר דקות חיובי.");
        return;
      }
    }

    setIsSubmitting(true);

    try {
      await addSystemMessage(title.trim(), selectedFile, {
        daysOfWeek: selectedDays,
        scheduleMode,
        scheduledTime: scheduleMode === "fixed_time" ? scheduledTime : null,
        operatingAnchor:
          scheduleMode === "relative_operating_hours" ? operatingAnchor : null,
        offsetDirection:
          scheduleMode === "relative_operating_hours" ? offsetDirection : null,
        offsetMinutes:
          scheduleMode === "relative_operating_hours"
            ? parsedOffsetMinutes
            : null,
      });
      onSaved();
      onClose();
    } catch (err) {
      setError(errMsg(err, "שגיאה בשמירת ההודעה."));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="flex max-h-[85vh] w-full max-w-2xl flex-col rounded-xl bg-white shadow-xl"
        dir="rtl"
        onClick={(event) => event.stopPropagation()}
      >
        <form className="flex min-h-0 flex-1 flex-col" onSubmit={handleSubmit}>
          <header className="border-b border-teal-100 px-5 py-4">
            <h3 className="text-lg font-bold text-teal-900">הוספת הודעת מערכת</h3>
          </header>

          <div className="min-h-0 flex-1 space-y-4 overflow-y-auto p-5">
            <label className="block space-y-1.5">
              <span className="text-sm font-semibold text-teal-800">כותרת ההודעה</span>
              <input
                type="text"
                className={inputClass}
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                placeholder="לדוגמה: הודעת פתיחה"
                autoFocus
              />
            </label>

            <div className="space-y-1.5">
              <span className="text-sm font-semibold text-teal-800">קובץ שמע</span>
              <div className="flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  className="rounded-lg bg-teal-100 px-3 py-2 text-sm font-medium text-teal-800 hover:bg-teal-200"
                  onClick={() => void handleSelectFile()}
                >
                  בחר קובץ
                </button>
                <span className="text-sm text-teal-600">
                  {selectedFile
                    ? selectedFile.split(/[/\\]/).pop()
                    : "לא נבחר קובץ"}
                </span>
              </div>
            </div>

            <fieldset className="space-y-2">
              <legend className="text-sm font-semibold text-teal-800">ימים</legend>
              <div className="flex flex-wrap gap-2">
                {SYSTEM_MESSAGE_DAYS.map((day) => (
                  <label
                    key={day.value}
                    className={`flex cursor-pointer items-center gap-1.5 rounded-full border px-3 py-1.5 text-sm ${
                      selectedDays.includes(day.value)
                        ? "border-teal-600 bg-teal-700 text-white"
                        : "border-teal-200 bg-teal-50 text-teal-800"
                    }`}
                  >
                    <input
                      type="checkbox"
                      className="sr-only"
                      checked={selectedDays.includes(day.value)}
                      onChange={() => toggleDay(day.value)}
                    />
                    <span>{day.label}</span>
                  </label>
                ))}
              </div>
            </fieldset>

            <fieldset className="space-y-3 rounded-lg border border-teal-200 bg-teal-50/60 p-4">
              <legend className="px-1 text-sm font-semibold text-teal-800">
                מתי להשמיע
              </legend>

              <label className="flex cursor-pointer items-start gap-2">
                <input
                  type="radio"
                  name="schedule-mode"
                  className="mt-1"
                  checked={scheduleMode === "fixed_time"}
                  onChange={() => setScheduleMode("fixed_time")}
                />
                <span className="flex-1 space-y-2">
                  <span className="block text-sm font-medium text-teal-900">בשעה קבועה</span>
                  <input
                    type="time"
                    dir="ltr"
                    className={inputClass}
                    value={scheduledTime}
                    disabled={scheduleMode !== "fixed_time"}
                    onChange={(event) => setScheduledTime(event.target.value)}
                  />
                </span>
              </label>

              <label className="flex cursor-pointer items-start gap-2">
                <input
                  type="radio"
                  name="schedule-mode"
                  className="mt-1"
                  checked={scheduleMode === "relative_operating_hours"}
                  onChange={() => setScheduleMode("relative_operating_hours")}
                />
                <span className="flex-1 space-y-2">
                  <span className="block text-sm font-medium text-teal-900">
                    זמן יחסי לשעות פעילות
                  </span>
                  <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
                    <input
                      type="number"
                      min={1}
                      className={inputClass}
                      value={offsetMinutes}
                      disabled={scheduleMode !== "relative_operating_hours"}
                      onChange={(event) => setOffsetMinutes(event.target.value)}
                      placeholder="דקות"
                    />
                    <select
                      className={inputClass}
                      value={offsetDirection}
                      disabled={scheduleMode !== "relative_operating_hours"}
                      onChange={(event) =>
                        setOffsetDirection(
                          event.target.value as SystemMessageOffsetDirection,
                        )
                      }
                    >
                      <option value="before">לפני</option>
                      <option value="after">אחרי</option>
                    </select>
                    <select
                      className={inputClass}
                      value={operatingAnchor}
                      disabled={scheduleMode !== "relative_operating_hours"}
                      onChange={(event) =>
                        setOperatingAnchor(
                          event.target.value as SystemMessageOperatingAnchor,
                        )
                      }
                    >
                      <option value="open">שעת פתיחה</option>
                      <option value="close">שעת סגירה</option>
                    </select>
                  </div>
                </span>
              </label>
            </fieldset>

            {error && <p className="text-sm text-red-600">{error}</p>}
          </div>

          <footer className="flex justify-end gap-2 border-t border-teal-100 px-5 py-4">
            <button
              type="button"
              onClick={onClose}
              disabled={isSubmitting}
              className="rounded-lg bg-teal-100 px-4 py-2 text-sm font-medium text-teal-800 hover:bg-teal-200 disabled:opacity-60"
            >
              ביטול
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="rounded-lg bg-teal-700 px-4 py-2 text-sm font-semibold text-white hover:bg-teal-800 disabled:opacity-60"
            >
              {isSubmitting ? "שומר..." : "שמור הודעה"}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}
