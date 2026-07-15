import { useEffect, useState } from "react";
import {
  getOverviewSnapshot,
  type OverviewSnapshot,
} from "../api";
import { MUSIC_FOLDERS, musicIconPath } from "../lib/musicFolders";
import { errMsg } from "../lib/errors";
import { useSystemActivity } from "./SystemActivityProvider";

function folderMeta(slug: string | null | undefined) {
  if (!slug) {
    return { label: "לא זמין", icon: "general" };
  }
  const match = MUSIC_FOLDERS.find((folder) => folder.icon === slug);
  return {
    label: match?.label ?? slug,
    icon: match?.icon ?? "general",
  };
}

function formatUptime(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  if (hours <= 0) {
    return `${minutes} דק׳`;
  }
  if (minutes <= 0) {
    return `${hours} שע׳`;
  }
  return `${hours} שע׳ ${minutes} דק׳`;
}

function formatHours(hours: OverviewSnapshot["operatingHours"]): string {
  if (hours.closed || !hours.open || !hours.close) {
    return "סגור";
  }
  return `${hours.open} – ${hours.close}`;
}

function MetricTile({
  label,
  value,
  hint,
  iconSrc,
}: {
  label: string;
  value: string;
  hint?: string;
  iconSrc?: string;
}) {
  return (
    <div className="flex min-h-[7.5rem] flex-col justify-between rounded-2xl border border-teal-100/80 bg-gradient-to-br from-white to-teal-50/50 px-4 py-4 shadow-[0_1px_0_rgba(13,148,136,0.06)]">
      <div className="flex items-start justify-between gap-3">
        <p className="text-xs font-medium tracking-wide text-teal-600/90">
          {label}
        </p>
        {iconSrc ? (
          <img
            src={iconSrc}
            alt=""
            className="h-8 w-8 rounded-lg object-cover opacity-90"
          />
        ) : null}
      </div>
      <div>
        <p className="text-2xl font-semibold tabular-nums text-teal-950">
          {value}
        </p>
        {hint ? (
          <p className="mt-1 text-xs text-teal-600/80">{hint}</p>
        ) : null}
      </div>
    </div>
  );
}

export default function OverviewPage() {
  const { active: systemActive } = useSystemActivity();
  const [snapshot, setSnapshot] = useState<OverviewSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const next = await getOverviewSnapshot();
        if (!cancelled) {
          setSnapshot(next);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            errMsg(err, "שגיאה בטעינת הסקירה."),
          );
        }
      }
    }

    void load();
    const timer = window.setInterval(() => {
      void load();
    }, 2000);

    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  const folder = folderMeta(snapshot?.musicFolder);
  const nowPlaying = snapshot?.nowPlaying ?? null;
  const artSrc =
    nowPlaying?.artworkDataUrl ?? musicIconPath("general");

  return (
    <div className="relative flex h-full min-h-0 flex-1 flex-col" dir="rtl">
      {!systemActive ? (
        <div className="mb-4 flex items-center justify-center gap-2 rounded-xl border border-slate-300 bg-slate-100 px-3 py-2 text-sm font-semibold text-slate-600">
          <span
            className="inline-flex h-5 w-5 items-center justify-center rounded-full bg-slate-400 text-[11px] text-white"
            aria-hidden
          >
            &#10005;
          </span>
          המערכת כבויה — אין השמעה פעילה
        </div>
      ) : null}

      <div
        className={`flex min-h-0 flex-1 flex-col gap-5 overflow-auto transition-[filter,opacity] ${
          systemActive
            ? ""
            : "pointer-events-none grayscale opacity-55"
        }`}
      >
        <section className="relative overflow-hidden rounded-3xl border border-teal-100 bg-[radial-gradient(120%_120%_at_100%_0%,#ccfbf1_0%,#ffffff_45%,#f0fdfa_100%)] px-5 py-5 shadow-sm sm:px-7 sm:py-6">
          <div className="pointer-events-none absolute -left-10 top-0 h-40 w-40 rounded-full bg-teal-200/30 blur-3xl" />
          <div className="relative flex flex-col gap-5 sm:flex-row sm:items-center">
            <div className="relative mx-auto h-36 w-36 shrink-0 overflow-hidden rounded-2xl border border-teal-100 bg-white shadow-md sm:mx-0">
              <img
                src={artSrc}
                alt=""
                className="h-full w-full object-cover"
              />
              <div
                className={`absolute bottom-2 left-2 rounded-full px-2 py-0.5 text-[10px] font-semibold ${
                  !systemActive
                    ? "bg-slate-500 text-white"
                    : nowPlaying
                      ? "bg-emerald-500 text-white"
                      : "bg-teal-800/70 text-teal-50"
                }`}
              >
                {!systemActive ? "כבוי" : nowPlaying ? "מתנגן" : "מושהה"}
              </div>
            </div>

            <div className="min-w-0 flex-1 text-center sm:text-right">
              <p className="text-xs font-semibold tracking-wide text-teal-600">
                מוזיקה עכשיו
              </p>
              <h2 className="mt-1 truncate text-2xl font-bold text-teal-950 sm:text-3xl">
                {!systemActive
                  ? "המערכת כבויה"
                  : (nowPlaying?.title ?? "אין השמעה כרגע")}
              </h2>
              <div className="mt-3 inline-flex items-center gap-2 rounded-full border border-teal-200/80 bg-white/80 px-3 py-1.5 text-sm text-teal-800 backdrop-blur">
                <img
                  src={musicIconPath(folder.icon)}
                  alt=""
                  className="h-5 w-5 rounded object-cover"
                />
                <span className="font-medium">{folder.label}</span>
              </div>
            </div>
          </div>
        </section>

        {error ? (
          <p className="rounded-xl bg-red-50 px-3 py-2 text-sm text-red-600">
            {error}
          </p>
        ) : null}

        <section className="grid grid-cols-2 gap-3 lg:grid-cols-4">
          <MetricTile
            label="שירים בספרייה"
            value={snapshot ? String(snapshot.totalSongs) : "—"}
          />
          <MetricTile
            label="הודעות מערכת"
            value={snapshot ? String(snapshot.systemMessagesTotal) : "—"}
            hint={
              snapshot
                ? `${snapshot.systemMessagesToday} מתוזמנות להיום`
                : undefined
            }
          />
          <MetricTile
            label="התראות חירום היום"
            value={snapshot ? String(snapshot.emergencyPlaysToday) : "—"}
          />
          <MetricTile
            label="תיקיית מוזיקה"
            value={folder.label}
            iconSrc={musicIconPath(folder.icon)}
          />
        </section>

        <section className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <MetricTile
            label="שעות פעילות היום"
            value={snapshot ? formatHours(snapshot.operatingHours) : "—"}
          />
          <MetricTile
            label="זמן פעילות בסשן"
            value={
              snapshot ? formatUptime(snapshot.sessionUptimeSeconds) : "—"
            }
          />
        </section>
      </div>
    </div>
  );
}
