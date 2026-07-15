interface DeleteHolidayConfirmDialogProps {
  title: string;
  onCancel: () => void;
  onConfirm: () => void;
  isDeleting: boolean;
}

export default function DeleteHolidayConfirmDialog({
  title,
  onCancel,
  onConfirm,
  isDeleting,
}: DeleteHolidayConfirmDialogProps) {
  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 p-4">
      <div
        className="w-full max-w-md rounded-xl border-2 border-red-300 bg-red-50 p-5 shadow-xl"
        dir="rtl"
        role="alertdialog"
        aria-labelledby="delete-holiday-title"
        aria-describedby="delete-holiday-warning"
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
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M12 9v4m0 4h.01M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"
            />
          </svg>
          <h3 id="delete-holiday-title" className="text-lg font-bold">
            אזהרה — מחיקת חג
          </h3>
        </div>

        <p id="delete-holiday-warning" className="text-sm leading-relaxed text-red-900">
          האם למחוק את החג &quot;{title}&quot;?
        </p>
        <p className="mt-2 text-sm font-medium text-red-800">
          פעולה זו בלתי הפיכה — החג יימחק מלוח השנה ומכל השנים שבהן הוא מופיע לפי
          התאריך העברי.
        </p>

        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            disabled={isDeleting}
            className="rounded-lg border border-red-200 bg-white px-4 py-2 text-sm text-red-800 hover:bg-red-100 disabled:cursor-not-allowed disabled:opacity-50"
          >
            ביטול
          </button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={isDeleting}
            className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {isDeleting ? "מוחק..." : "מחק בכל זאת"}
          </button>
        </div>
      </div>
    </div>
  );
}
