import { useEffect, useState } from "react";
import { addCustomHoliday, getOperatingHours } from "../api";
import {
  formatHolidayDate,
  getDefaultHolidayHours,
  HOLIDAY_STATUS_KINDS,
  type HolidayStatusKind,
} from "../lib/holidays";
import {
  createDefaultOperatingHours,
  type OperatingHoursSettingsData,
} from "../lib/operatingHours";
import DatePickerDialog from "./DatePickerDialog";
import { HolidayKindIcon } from "./HolidayKindIcon";
import Time24Input from "./settings/Time24Input";
import ToggleSwitch from "./ui/ToggleSwitch";
import { errMsg } from "../lib/errors";

const inputClass =
  "w-full rounded-lg border border-teal-200 bg-white px-3 py-2 text-sm text-teal-900 shadow-sm focus:border-teal-500 focus:outline-none focus:ring-1 focus:ring-teal-500";

interface AddHolidayDialogProps {
  onClose: () => void;
  onSaved: () => void;
}

export default function AddHolidayDialog({
  onClose,
  onSaved,
}: AddHolidayDialogProps) {
  const [date, setDate] = useState("");
  const [title, setTitle] = useState("");
  const [kind, setKind] = useState<HolidayStatusKind>("חג");
  const [isOpen, setIsOpen] = useState(false);
  const [openTime, setOpenTime] = useState("00:00");
  const [closeTime, setCloseTime] = useState("00:00");
  const [operatingHours, setOperatingHours] = useState<OperatingHoursSettingsData>(
    createDefaultOperatingHours,
  );
  const [hoursReady, setHoursReady] = useState(false);
  const [showDatePicker, setShowDatePicker] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void getOperatingHours()
      .then((data) => {
        if (cancelled) {
          return;
        }
        setOperatingHours(data);
        setHoursReady(true);
      })
      .catch(() => {
        if (!cancelled) {
          setHoursReady(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function applyDefaults(nextKind: HolidayStatusKind, nextDate: string) {
    if (!nextDate) {
      setOpenTime("00:00");
      setCloseTime("00:00");
      return;
    }
    const defaults = getDefaultHolidayHours(nextKind, nextDate, operatingHours);
    setOpenTime(defaults.open);
    setCloseTime(defaults.close);
  }

  function handleKindChange(nextKind: HolidayStatusKind) {
    setKind(nextKind);
    applyDefaults(nextKind, date);
  }

  function handleDateChange(nextDate: string) {
    setDate(nextDate);
    applyDefaults(kind, nextDate);
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    setError(null);

    if (!date) {
      setError("יש לבחור תאריך.");
      return;
    }

    if (!title.trim()) {
      setError("יש להזין שם לחג.");
      return;
    }

    setIsSubmitting(true);
    try {
      await addCustomHoliday(
        date,
        title.trim(),
        !isOpen,
        kind,
        isOpen ? openTime : null,
        isOpen ? closeTime : null,
      );
      onSaved();
      onClose();
    } catch (err) {
      setError(errMsg(err, "שגיאה בהוספת חג."));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <form
        className="w-full max-w-md rounded-xl bg-white p-5 shadow-xl"
        onSubmit={(event) => void handleSubmit(event)}
        dir="rtl"
      >
        <h2 className="mb-4 text-lg font-bold text-teal-900">הוספת חג</h2>

        {error && (
          <p className="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
            {error}
          </p>
        )}

        <div className="space-y-4">
          <div className="space-y-1">
            <span className="text-xs font-medium text-teal-700">תאריך</span>
            <button
              type="button"
              onClick={() => setShowDatePicker(true)}
              className={`${inputClass} text-right`}
            >
              {date ? formatHolidayDate(date) : "בחר תאריך"}
            </button>
          </div>

          <label className="block space-y-1">
            <span className="text-xs font-medium text-teal-700">שם החג</span>
            <input
              type="text"
              className={inputClass}
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              placeholder="הזן את שם החג"
            />
          </label>

          <div className="space-y-2">
            <span className="text-xs font-medium text-teal-700">סוג יום</span>
            <div className="grid grid-cols-3 gap-2">
              {HOLIDAY_STATUS_KINDS.map((option) => {
                const selected = kind === option.id;
                return (
                  <button
                    key={option.id}
                    type="button"
                    disabled={isSubmitting || !hoursReady}
                    onClick={() => handleKindChange(option.id)}
                    className={`flex flex-col items-center gap-1 rounded-lg border px-2 py-2 text-sm font-semibold transition-colors ${
                      selected
                        ? "border-teal-700 bg-teal-700 text-white"
                        : "border-teal-200 bg-white text-teal-800 hover:bg-teal-50"
                    }`}
                  >
                    <HolidayKindIcon kind={option.id} className="h-5 w-5" />
                    {option.label}
                  </button>
                );
              })}
            </div>
          </div>

          <div className="flex items-center justify-between gap-3 rounded-lg bg-teal-50 px-3 py-2">
            <span className="text-sm font-medium text-teal-800">מצב</span>
            <ToggleSwitch
              checked={isOpen}
              offLabel="סגור"
              onLabel="פתוח"
              disabled={isSubmitting}
              onChange={setIsOpen}
            />
          </div>

          {isOpen && (
            <div className="space-y-2 rounded-lg border border-teal-100 p-3">
              <span className="text-xs font-medium text-teal-700">שעות פעילות</span>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <span className="text-[11px] font-semibold text-teal-600">פתיחה</span>
                  <Time24Input
                    value={openTime}
                    disabled={isSubmitting || !hoursReady || !date}
                    ariaLabel="שעת פתיחה"
                    onCommit={setOpenTime}
                  />
                </div>
                <div className="space-y-1">
                  <span className="text-[11px] font-semibold text-teal-600">סגירה</span>
                  <Time24Input
                    value={closeTime}
                    disabled={isSubmitting || !hoursReady || !date}
                    ariaLabel="שעת סגירה"
                    onCommit={setCloseTime}
                  />
                </div>
              </div>
            </div>
          )}
        </div>

        <div className="mt-5 flex justify-start gap-2">
          <button
            type="submit"
            disabled={isSubmitting || !hoursReady}
            className="rounded-lg bg-teal-700 px-4 py-2 text-sm font-semibold text-white hover:bg-teal-800 disabled:opacity-60"
          >
            {isSubmitting ? "מוסיף..." : "הוספה"}
          </button>
          <button
            type="button"
            disabled={isSubmitting}
            onClick={onClose}
            className="rounded-lg border border-teal-200 px-4 py-2 text-sm font-semibold text-teal-800 hover:bg-teal-50 disabled:opacity-60"
          >
            ביטול
          </button>
        </div>
      </form>

      {showDatePicker && (
        <DatePickerDialog
          value={date}
          onChange={handleDateChange}
          onClose={() => setShowDatePicker(false)}
        />
      )}
    </div>
  );
}
