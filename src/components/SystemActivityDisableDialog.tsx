interface SystemActivityDisableDialogProps {
  step: 1 | 2;
  onCancel: () => void;
  onConfirm: () => void;
}

export default function SystemActivityDisableDialog({
  step,
  onCancel,
  onConfirm,
}: SystemActivityDisableDialogProps) {
  const isEmergencyStep = step === 2;

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-black/50 p-4">
      <div
        className={`w-full max-w-md rounded-xl border-2 p-5 shadow-xl ${
          isEmergencyStep
            ? "border-red-400 bg-red-50"
            : "border-amber-300 bg-amber-50"
        }`}
        dir="rtl"
        role="alertdialog"
        aria-labelledby="system-activity-warning-title"
      >
        <div
          className={`mb-3 flex items-center gap-2 ${
            isEmergencyStep ? "text-red-700" : "text-amber-800"
          }`}
        >
          <svg
            viewBox="0 0 24 24"
            className="h-6 w-6 shrink-0"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden
          >
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            <path d="M12 9v4M12 17h.01" />
          </svg>
          <h3
            id="system-activity-warning-title"
            className="text-lg font-bold"
          >
            {isEmergencyStep ? "אזהרת חירום" : "כיבוי פעילות המערכת"}
          </h3>
        </div>

        <p
          className={`text-sm leading-relaxed ${
            isEmergencyStep ? "text-red-900" : "text-amber-950"
          }`}
        >
          {isEmergencyStep
            ? "שים לב: גם הודעות חירום לא יושמעו כל עוד המערכת כבויה."
            : "כיבוי פעילות המערכת יגרום לכך שלא תושמע מוזיקה ולא יושמעו הודעות מערכת או הודעות מתוזמנות."}
        </p>

        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className={`rounded-lg border bg-white px-4 py-2 text-sm ${
              isEmergencyStep
                ? "border-red-200 text-red-800 hover:bg-red-100"
                : "border-amber-200 text-amber-900 hover:bg-amber-100"
            }`}
          >
            ביטול
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className={`rounded-lg px-4 py-2 text-sm font-medium text-white ${
              isEmergencyStep
                ? "bg-red-600 hover:bg-red-700"
                : "bg-amber-600 hover:bg-amber-700"
            }`}
          >
            אישור
          </button>
        </div>
      </div>
    </div>
  );
}
