import { useEffect, useState } from "react";
import { getOperatingHours, setHolidayStatus } from "../api";
import {
  getDefaultHolidayHours,
  HOLIDAY_STATUS_KINDS,
  holidayIsOpen,
  normalizeHolidayStatusKind,
  type HolidayStatusKind,
} from "../lib/holidays";
import {
  createDefaultOperatingHours,
  type OperatingHoursSettingsData,
} from "../lib/operatingHours";
import type { HolidayEntry } from "../types";
import { HolidayKindIcon } from "./HolidayKindIcon";
import Time24Input from "./settings/Time24Input";
import ToggleSwitch from "./ui/ToggleSwitch";
import { errMsg } from "../lib/errors";

interface HolidayStatusDialogProps {
  holiday: HolidayEntry;
  onClose: () => void;
  onSaved: (holidays: HolidayEntry[]) => void;
}

export default function HolidayStatusDialog({
  holiday,
  onClose,
  onSaved,
}: HolidayStatusDialogProps) {
  const [kind, setKind] = useState<HolidayStatusKind>(() =>
    normalizeHolidayStatusKind(holiday.day_label),
  );
  const [isOpen, setIsOpen] = useState(() => holidayIsOpen(holiday));
  const [openTime, setOpenTime] = useState(holiday.open_time ?? "00:00");
  const [closeTime, setCloseTime] = useState(holiday.close_time ?? "00:00");
  const [operatingHours, setOperatingHours] = useState<OperatingHoursSettingsData>(
    createDefaultOperatingHours,
  );
  const [hoursReady, setHoursReady] = useState(false);
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
        const defaults = getDefaultHolidayHours(
          normalizeHolidayStatusKind(holiday.day_label),
          holiday.date,
          data,
        );
        if (!holiday.open_time) {
          setOpenTime(defaults.open);
        }
        if (!holiday.close_time) {
          setCloseTime(defaults.close);
        }
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
  }, [holiday.date, holiday.day_label, holiday.open_time, holiday.close_time]);

  function applyKindDefaults(nextKind: HolidayStatusKind) {
    setKind(nextKind);
    const defaults = getDefaultHolidayHours(nextKind, holiday.date, operatingHours);
    setOpenTime(defaults.open);
    setCloseTime(defaults.close);
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    setError(null);
    setIsSubmitting(true);
    try {
      const updated = await setHolidayStatus(
        holiday.date,
        kind,
        !isOpen,
        isOpen ? openTime : null,
        isOpen ? closeTime : null,
      );
      onSaved(updated);
      onClose();
    } catch (err) {
      setError(errMsg(err, "שגיאה בשמירת סטטוס החג."));
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
        <h2 className="mb-1 text-lg font-bold text-teal-900">סטטוס חג</h2>
        <p className="mb-4 text-sm text-teal-600">{holiday.title}</p>

        {error && (
          <p className="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
            {error}
          </p>
        )}

        <div className="space-y-4">
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
                    onClick={() => applyKindDefaults(option.id)}
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
                    disabled={isSubmitting || !hoursReady}
                    ariaLabel="שעת פתיחה"
                    onCommit={setOpenTime}
                  />
                </div>
                <div className="space-y-1">
                  <span className="text-[11px] font-semibold text-teal-600">סגירה</span>
                  <Time24Input
                    value={closeTime}
                    disabled={isSubmitting || !hoursReady}
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
            {isSubmitting ? "שומר..." : "שמירה"}
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
    </div>
  );
}
