import { useState } from "react";
import DatePickerDialog from "../DatePickerDialog";
import Time24Input from "./Time24Input";
import { errMsg } from "../../lib/errors";
import {
  formatOperatingDateDisplay,
  mergeTemporaryHours,
  OPERATING_DAYS,
  temporaryHoursOnly,
  type OperatingDayId,
  type SeasonOperatingHours,
  type TemporaryOperatingHours,
} from "../../lib/operatingHours";

interface EditTemporaryOperatingHoursDialogProps {
  initial: TemporaryOperatingHours;
  onClose: () => void;
  onSave: (next: TemporaryOperatingHours) => Promise<void>;
}

const dateButtonClass =
  "w-full rounded-lg border border-teal-200 bg-white px-3 py-2.5 text-right text-sm text-teal-900 shadow-sm hover:bg-teal-50 focus:border-teal-500 focus:outline-none focus:ring-1 focus:ring-teal-500";

type DateField = "from" | "to";

export default function EditTemporaryOperatingHoursDialog({
  initial,
  onClose,
  onSave,
}: EditTemporaryOperatingHoursDialogProps) {
  const [hours, setHours] = useState<SeasonOperatingHours>(() =>
    temporaryHoursOnly(initial),
  );
  const [validFrom, setValidFrom] = useState(initial.valid_from ?? "");
  const [validTo, setValidTo] = useState(initial.valid_to ?? "");
  const [datePickerField, setDatePickerField] = useState<DateField | null>(
    null,
  );
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function updateDayTime(
    dayId: OperatingDayId,
    field: "open" | "close",
    value: string,
  ) {
    setHours((current) => ({
      ...current,
      [dayId]: {
        ...current[dayId],
        [field]: value,
      },
    }));
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    setError(null);

    if (!validFrom || !validTo) {
      setError("יש לבחור תאריך התחלה ותאריך סיום.");
      return;
    }

    if (validFrom > validTo) {
      setError("תאריך ההתחלה חייב להיות לפני תאריך הסיום או שווה לו.");
      return;
    }

    setIsSubmitting(true);
    try {
      await onSave(mergeTemporaryHours(hours, validFrom, validTo));
      onClose();
    } catch (err) {
      setError(
        errMsg(err, "שגיאה בשמירת שעות זמניות."),
      );
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <form
        className="flex max-h-[90vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl bg-white shadow-xl"
        onSubmit={(event) => void handleSubmit(event)}
        dir="rtl"
      >
        <div className="flex shrink-0 items-center border-b border-teal-100 px-5 py-4">
          <h2 className="text-lg font-bold text-teal-900">
            עריכת שעות פעילות זמניות
          </h2>
        </div>

        <div className="min-h-0 flex-1 space-y-5 overflow-y-auto px-5 py-4">
          {error && (
            <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
              {error}
            </p>
          )}

          <section className="space-y-2">
            <h3 className="text-sm font-semibold text-teal-900">תקופת תוקף</h3>
            <div className="grid gap-3 sm:grid-cols-2">
              <div className="space-y-1">
                <span className="text-xs font-medium text-teal-700">מ־תאריך</span>
                <button
                  type="button"
                  className={dateButtonClass}
                  onClick={() => setDatePickerField("from")}
                >
                  {formatOperatingDateDisplay(validFrom || null)}
                </button>
              </div>
              <div className="space-y-1">
                <span className="text-xs font-medium text-teal-700">עד תאריך</span>
                <button
                  type="button"
                  className={dateButtonClass}
                  onClick={() => setDatePickerField("to")}
                >
                  {formatOperatingDateDisplay(validTo || null)}
                </button>
              </div>
            </div>
          </section>

          <section className="space-y-3">
            <h3 className="text-sm font-semibold text-teal-900">שעות לפי יום</h3>
            <div className="grid grid-cols-[5.5rem_1fr_1fr] gap-3 px-1 text-sm font-semibold text-teal-700">
              <span>יום</span>
              <span>שעת פתיחה</span>
              <span>שעת סגירה</span>
            </div>
            <ul className="space-y-2">
              {OPERATING_DAYS.map((day) => {
                const dayValue = hours[day.id];
                return (
                  <li
                    key={day.id}
                    className="grid grid-cols-[5.5rem_1fr_1fr] items-center gap-3 rounded-xl bg-teal-50/70 px-3 py-2"
                  >
                    <span className="text-sm font-semibold text-teal-900">
                      {day.label}
                    </span>
                    <Time24Input
                      value={dayValue.open}
                      disabled={isSubmitting}
                      ariaLabel={`שעת פתיחה ${day.label}`}
                      onCommit={(nextValue) =>
                        updateDayTime(day.id, "open", nextValue)
                      }
                    />
                    <Time24Input
                      value={dayValue.close}
                      disabled={isSubmitting}
                      ariaLabel={`שעת סגירה ${day.label}`}
                      onCommit={(nextValue) =>
                        updateDayTime(day.id, "close", nextValue)
                      }
                    />
                  </li>
                );
              })}
            </ul>
          </section>
        </div>

        <div className="flex shrink-0 justify-start gap-2 border-t border-teal-100 px-5 py-4">
          <button
            type="submit"
            disabled={isSubmitting}
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

      {datePickerField && (
        <DatePickerDialog
          value={datePickerField === "from" ? validFrom : validTo}
          onChange={(date) => {
            if (datePickerField === "from") {
              setValidFrom(date);
            } else {
              setValidTo(date);
            }
          }}
          onClose={() => setDatePickerField(null)}
        />
      )}
    </div>
  );
}
