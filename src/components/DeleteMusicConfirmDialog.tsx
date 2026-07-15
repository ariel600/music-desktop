interface DeleteMusicConfirmDialogProps {
  count: number;
  onCancel: () => void;
  onConfirm: () => void;
  isDeleting: boolean;
}

export default function DeleteMusicConfirmDialog({
  count,
  onCancel,
  onConfirm,
  isDeleting,
}: DeleteMusicConfirmDialogProps) {
  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4">
      <div
        className="w-full max-w-md rounded-xl border-2 border-red-300 bg-red-50 p-5 shadow-xl"
        dir="rtl"
        role="alertdialog"
        aria-labelledby="delete-music-title"
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
            <path d="M3 6h18M8 6V4h8v2M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
            <path d="M10 11v6M14 11v6" />
          </svg>
          <h3 id="delete-music-title" className="text-lg font-bold">
            מחיקת שירים
          </h3>
        </div>

        <p className="text-sm leading-relaxed text-red-900">
          {count === 1
            ? "האם למחוק את השיר שנבחר?"
            : `האם למחוק ${count} שירים שנבחרו?`}
        </p>
        <p className="mt-2 text-sm font-medium text-red-800">
          פעולה זו בלתי הפיכה — אין דרך לשחזר שירים שנמחקו.
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
            {isDeleting ? "מוחק..." : "מחק"}
          </button>
        </div>
      </div>
    </div>
  );
}
