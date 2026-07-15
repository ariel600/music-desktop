import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { EmergencyAlertPayload } from "../types";

function formatReceivedAt(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString("he-IL", {
    timeZone: "Asia/Jerusalem",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function EmergencyAlertCard({
  alert,
  onDismiss,
}: {
  alert: EmergencyAlertPayload;
  onDismiss: () => void;
}) {
  const typeLabel =
    alert.message_type === "pre-alert"
      ? "התראה מקדימה"
      : alert.message_type === "red-alert"
        ? "צבע אדום"
        : alert.message_type === "hostile-aircraft"
          ? "חדירת כלי טיס עוין"
          : alert.message_type === "end"
            ? "סיום"
            : "הודעה לא מוגדרת";

  return (
    <article className="w-[min(28rem,calc(100vw-2rem))] rounded-xl border border-slate-300 bg-white shadow-2xl">
      <div className="flex items-start justify-between gap-3 border-b border-slate-200 bg-slate-100 px-4 py-3">
        <div>
          <p className="text-xs font-semibold uppercase tracking-wide text-slate-500">
            התראת פיקוד העורף
          </p>
          <h3 className="mt-0.5 text-base font-bold text-slate-900">{typeLabel}</h3>
        </div>
        <button
          type="button"
          onClick={onDismiss}
          className="rounded-md px-2 py-1 text-sm text-slate-500 transition-colors hover:bg-white hover:text-slate-800"
          aria-label="סגור התראה"
        >
          ✕
        </button>
      </div>

      <div className="space-y-3 px-4 py-4 text-right">
        <div>
          <p className="text-xs font-medium text-slate-500">סוג ההתרעה</p>
          <p className="mt-1 text-sm font-semibold text-slate-900">{alert.title}</p>
        </div>

        <div>
          <p className="text-xs font-medium text-slate-500">אזורים</p>
          <p className="mt-1 text-sm leading-relaxed text-slate-800">
            {alert.cities.join(" · ")}
          </p>
        </div>

        {alert.description && (
          <div>
            <p className="text-xs font-medium text-slate-500">תיאור</p>
            <p className="mt-1 text-sm leading-relaxed text-slate-800">{alert.description}</p>
          </div>
        )}

        <div>
          <p className="text-xs font-medium text-slate-500">זמן קבלה</p>
          <p className="mt-1 text-sm text-slate-700">{formatReceivedAt(alert.received_at)}</p>
        </div>
      </div>
    </article>
  );
}

export default function EmergencyAlertStack() {
  const [alerts, setAlerts] = useState<EmergencyAlertPayload[]>([]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listen<EmergencyAlertPayload>("emergency-alert", (event) => {
      if (disposed) {
        return;
      }

      setAlerts((current) => [event.payload, ...current]);
    }).then((cleanup) => {
      if (disposed) {
        cleanup();
        return;
      }

      unlisten = cleanup;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  if (alerts.length === 0) {
    return null;
  }

  return (
    <div
      className="pointer-events-none fixed inset-x-0 top-4 z-[200] flex flex-col items-center gap-3 px-4"
      dir="rtl"
    >
      {alerts.map((alert) => (
        <div key={alert.id} className="pointer-events-auto">
          <EmergencyAlertCard
            alert={alert}
            onDismiss={() =>
              setAlerts((current) => current.filter((item) => item.id !== alert.id))
            }
          />
        </div>
      ))}
    </div>
  );
}
