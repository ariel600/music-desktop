import { useEffect, useState } from "react";
import {
  formatTimeDigits,
  normalizeTime24h,
} from "../../lib/operatingHours";

const timeInputClass =
  "w-full rounded-lg border text-center font-mono text-sm tabular-nums shadow-sm focus:outline-none focus:ring-1 disabled:cursor-default px-3 py-2.5";

export default function Time24Input({
  value,
  disabled,
  ariaLabel,
  muted,
  onCommit,
}: {
  value: string;
  disabled?: boolean;
  ariaLabel: string;
  muted?: boolean;
  onCommit: (value: string) => void;
}) {
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  function commit(raw: string = draft) {
    const normalized = normalizeTime24h(raw);
    if (!normalized) {
      setDraft(value);
      return;
    }
    setDraft(normalized);
    if (normalized !== value) {
      onCommit(normalized);
    }
  }

  return (
    <input
      type="text"
      inputMode="numeric"
      placeholder="00:00"
      maxLength={5}
      dir="ltr"
      readOnly={disabled}
      className={`${timeInputClass} ${
        muted || disabled
          ? "border-slate-200 bg-slate-50 text-slate-500 focus:border-slate-300 focus:ring-slate-300"
          : "border-teal-200 bg-white text-teal-900 focus:border-teal-500 focus:ring-teal-500"
      } ${disabled ? "pointer-events-none" : ""}`}
      value={draft}
      aria-label={ariaLabel}
      onFocus={(event) => {
        if (disabled) {
          return;
        }
        event.currentTarget.select();
      }}
      onClick={(event) => {
        if (disabled) {
          return;
        }
        event.currentTarget.select();
      }}
      onMouseUp={(event) => {
        if (!disabled) {
          event.preventDefault();
        }
      }}
      onChange={(event) => {
        if (disabled) {
          return;
        }
        const formatted = formatTimeDigits(event.target.value);
        setDraft(formatted);

        const digits = formatted.replace(/\D/g, "");
        if (digits.length === 4) {
          commit(formatted);
        }
      }}
      onBlur={() => {
        if (!disabled) {
          commit();
        }
      }}
      onKeyDown={(event) => {
        if (disabled) {
          return;
        }
        if (event.key === "Enter") {
          event.currentTarget.blur();
        }
      }}
    />
  );
}
