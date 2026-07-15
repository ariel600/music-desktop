interface VocalOnlyWarningDialogProps {
  onBack: () => void;
  onConfirm: () => void;
}

export default function VocalOnlyWarningDialog({
  onBack,
  onConfirm,
}: VocalOnlyWarningDialogProps) {
  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4">
      <div
        className="w-full max-w-md rounded-xl border-2 border-red-300 bg-red-50 p-5 shadow-xl"
        dir="rtl"
        role="alertdialog"
        aria-labelledby="vocal-warning-title"
      >
        <div className="mb-3 flex items-center gap-2 text-red-700">
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
          <h3 id="vocal-warning-title" className="text-lg font-bold">
            אזהרה
          </h3>
        </div>

        <p className="text-sm leading-relaxed text-red-900">
          שים לב, בתיקיה זו ניתן להוסיף שירים ווקאליים בלבד
        </p>

        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onBack}
            className="rounded-lg border border-red-200 bg-white px-4 py-2 text-sm text-red-800 hover:bg-red-100"
          >
            חזרה
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
          >
            הבנתי
          </button>
        </div>
      </div>
    </div>
  );
}
