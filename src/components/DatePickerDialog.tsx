import { useMemo, useState } from "react";

interface DatePickerDialogProps {
  value: string;
  onChange: (date: string) => void;
  onClose: () => void;
}

const HEBREW_MONTHS = [
  "ינואר",
  "פברואר",
  "מרץ",
  "אפריל",
  "מאי",
  "יוני",
  "יולי",
  "אוגוסט",
  "ספטמבר",
  "אוקטובר",
  "נובמבר",
  "דצמבר",
];

const WEEKDAY_LABELS = ["א׳", "ב׳", "ג׳", "ד׳", "ה׳", "ו׳", "ש׳"];

function parseDate(value: string): Date {
  if (value) {
    const parsed = new Date(`${value}T12:00:00`);
    if (!Number.isNaN(parsed.getTime())) {
      return parsed;
    }
  }
  return new Date();
}

function toDateString(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export default function DatePickerDialog({
  value,
  onChange,
  onClose,
}: DatePickerDialogProps) {
  const initial = parseDate(value);
  const [viewYear, setViewYear] = useState(initial.getFullYear());
  const [viewMonth, setViewMonth] = useState(initial.getMonth());

  const cells = useMemo(() => {
    const firstDay = new Date(viewYear, viewMonth, 1);
    const startWeekday = firstDay.getDay();
    const daysInMonth = new Date(viewYear, viewMonth + 1, 0).getDate();

    const result: Array<{ date: string | null; label: number | null }> = [];
    for (let i = 0; i < startWeekday; i += 1) {
      result.push({ date: null, label: null });
    }
    for (let day = 1; day <= daysInMonth; day += 1) {
      const date = new Date(viewYear, viewMonth, day);
      result.push({ date: toDateString(date), label: day });
    }
    return result;
  }, [viewMonth, viewYear]);

  function shiftMonth(delta: number) {
    const next = new Date(viewYear, viewMonth + delta, 1);
    setViewYear(next.getFullYear());
    setViewMonth(next.getMonth());
  }

  function handleSelect(date: string) {
    onChange(date);
    onClose();
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-sm rounded-xl bg-white p-4 shadow-xl"
        dir="rtl"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="mb-4 flex items-center justify-between">
          <button
            type="button"
            onClick={() => shiftMonth(-1)}
            className="rounded-lg px-2 py-1 text-teal-700 hover:bg-teal-50"
          >
            ‹
          </button>
          <h3 className="font-bold text-teal-900">
            {HEBREW_MONTHS[viewMonth]} {viewYear}
          </h3>
          <button
            type="button"
            onClick={() => shiftMonth(1)}
            className="rounded-lg px-2 py-1 text-teal-700 hover:bg-teal-50"
          >
            ›
          </button>
        </div>

        <div className="mb-2 grid grid-cols-7 gap-1 text-center text-xs font-semibold text-teal-600">
          {WEEKDAY_LABELS.map((label) => (
            <div key={label}>{label}</div>
          ))}
        </div>

        <div className="grid grid-cols-7 gap-1">
          {cells.map((cell, index) =>
            cell.date ? (
              <button
                key={cell.date}
                type="button"
                onClick={() => handleSelect(cell.date!)}
                className={`rounded-lg py-2 text-sm transition-colors ${
                  cell.date === value
                    ? "bg-teal-700 font-bold text-white"
                    : "text-teal-800 hover:bg-teal-100"
                }`}
              >
                {cell.label}
              </button>
            ) : (
              <div key={`empty-${index}`} />
            ),
          )}
        </div>

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg bg-teal-100 px-3 py-1.5 text-sm text-teal-800 hover:bg-teal-200"
          >
            ביטול
          </button>
        </div>
      </div>
    </div>
  );
}
