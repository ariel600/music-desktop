import {
  normalizeHolidayStatusKind,
  type HolidayStatusKind,
} from "../lib/holidays";

export function HolidayKindIcon({
  kind,
  className = "h-4 w-4",
}: {
  kind: HolidayStatusKind | string;
  className?: string;
}) {
  const normalized = normalizeHolidayStatusKind(kind);

  switch (normalized) {
    case "חג":
      return (
        <svg
          viewBox="0 0 24 24"
          className={className}
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden
        >
          <path d="m12 3 2.4 4.9 5.4.8-3.9 3.8.9 5.4L12 15.9 7.2 17.9l.9-5.4-3.9-3.8 5.4-.8Z" />
        </svg>
      );
    case "ערב חג":
      return (
        <svg
          viewBox="0 0 24 24"
          className={className}
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden
        >
          <path d="M21 14.5A8.5 8.5 0 1 1 9.5 3 7 7 0 0 0 21 14.5Z" />
        </svg>
      );
    case "אחר":
      return (
        <svg
          viewBox="0 0 24 24"
          className={className}
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden
        >
          <path d="M5 3v18M5 4h10l-1.5 3.5L15 11H5" />
        </svg>
      );
  }
}

export function holidayKindIconShellClass(kind: HolidayStatusKind | string): string {
  const normalized = normalizeHolidayStatusKind(kind);
  switch (normalized) {
    case "חג":
      return "bg-amber-100 text-amber-800";
    case "ערב חג":
      return "bg-indigo-100 text-indigo-800";
    case "אחר":
      return "bg-slate-200 text-slate-700";
  }
}
