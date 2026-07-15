export default function StatusBar() {
  return (
    <footer
      className="flex h-10 shrink-0 items-center bg-teal-900 px-4 text-xs text-teal-100"
      dir="rtl"
    >
      <div className="flex min-w-0 items-center gap-2">
        <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-emerald-500/20 text-emerald-400">
          <svg
            viewBox="0 0 24 24"
            className="h-3.5 w-3.5"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
          >
            <path d="M20 6L9 17l-5-5" />
          </svg>
        </span>
        <div className="min-w-0">
          <span className="font-semibold text-emerald-300">פעיל</span>
          <span className="mx-2 text-teal-600">|</span>
          <span className="text-teal-200">מנוע תזמון רץ ברקע</span>
        </div>
      </div>
    </footer>
  );
}
