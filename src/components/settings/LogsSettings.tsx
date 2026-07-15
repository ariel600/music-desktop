import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getPlayLog } from "../../api";
import type { PlayLogEntry } from "../../types";
import { errMsg } from "../../lib/errors";

const STATUS_LABELS: Record<string, string> = {
  success: "OK",
  error: "ERR",
  warn: "WARN",
  info: "INFO",
  app_start: "APP START",
  app_ready: "APP READY",
  app_exit: "APP EXIT",
  system_on: "SYSTEM ON",
  system_off: "SYSTEM OFF",
  settings: "SETTINGS",
  maintenance: "MAINTENANCE",
  system_ok: "SYSTEM OK",
  system_error: "SYSTEM ERR",
  emergency_ok: "EMERGENCY OK",
  emergency_error: "EMERGENCY ERR",
  emergency_skip: "EMERGENCY SKIP",
  oref_error: "OREF ERR",
  music_started: "MUSIC START",
  music_stopped: "MUSIC STOP",
  music_error: "MUSIC ERR",
  music_import: "MUSIC IMPORT",
  music_delete: "MUSIC DELETE",
  converter_error: "CONVERTER ERR",
  missed: "MISSED",
  skipped_holiday: "SKIP holiday",
  skipped_day: "SKIP day",
  skipped_override: "SKIP override",
};

type LogFilter = "all" | "problems";

function statusLabel(status: string): string {
  return STATUS_LABELS[status] ?? status.toUpperCase();
}

function isProblemStatus(status: string): boolean {
  return (
    status === "error" ||
    status === "warn" ||
    status === "system_error" ||
    status === "emergency_error" ||
    status === "emergency_skip" ||
    status === "oref_error" ||
    status === "music_error" ||
    status === "converter_error" ||
    status === "missed" ||
    status.startsWith("skipped_")
  );
}

function statusColorClass(status: string): string {
  if (
    status === "success" ||
    status === "system_ok" ||
    status === "emergency_ok" ||
    status === "music_started" ||
    status === "music_import" ||
    status === "app_start" ||
    status === "app_ready" ||
    status === "system_on"
  ) {
    return "text-emerald-400";
  }
  if (
    status === "music_stopped" ||
    status === "music_delete" ||
    status === "settings" ||
    status === "info" ||
    status === "maintenance" ||
    status === "app_exit" ||
    status === "system_off" ||
    status === "warn" ||
    status === "emergency_skip" ||
    status === "missed" ||
    status.startsWith("skipped_")
  ) {
    return "text-amber-300";
  }
  return "text-red-400";
}

export default function LogsSettings() {
  const [entries, setEntries] = useState<PlayLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<LogFilter>("all");
  const [query, setQuery] = useState("");
  const bodyRef = useRef<HTMLDivElement>(null);
  const stickToBottomRef = useRef(true);

  const loadLogs = useCallback(async () => {
    try {
      const log = await getPlayLog(500);
      setEntries([...log].reverse());
      setError(null);
    } catch (err) {
      setError(errMsg(err, "Failed to load logs."));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadLogs();
    const timer = window.setInterval(() => {
      void loadLogs();
    }, 5_000);
    return () => window.clearInterval(timer);
  }, [loadLogs]);

  const problemCount = useMemo(
    () => entries.filter((entry) => isProblemStatus(entry.status)).length,
    [entries],
  );

  const visibleEntries = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return entries.filter((entry) => {
      if (filter === "problems" && !isProblemStatus(entry.status)) {
        return false;
      }
      if (!needle) {
        return true;
      }
      const haystack = `${entry.status} ${entry.task_title} ${entry.played_at}`.toLowerCase();
      return haystack.includes(needle);
    });
  }, [entries, filter, query]);

  useEffect(() => {
    const body = bodyRef.current;
    if (!body || !stickToBottomRef.current) {
      return;
    }
    body.scrollTop = body.scrollHeight;
  }, [visibleEntries]);

  function handleScroll() {
    const body = bodyRef.current;
    if (!body) {
      return;
    }
    const distanceFromBottom =
      body.scrollHeight - body.scrollTop - body.clientHeight;
    stickToBottomRef.current = distanceFromBottom < 48;
  }

  return (
    <div className="flex h-full min-h-0 flex-1 flex-col" dir="ltr">
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-zinc-700 bg-[#0c0f12] shadow-lg shadow-black/20">
        <div className="flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-zinc-700/80 bg-[#161a20] px-3 py-2">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5" aria-hidden>
              <span className="h-2.5 w-2.5 rounded-full bg-[#ff5f56]" />
              <span className="h-2.5 w-2.5 rounded-full bg-[#ffbd2e]" />
              <span className="h-2.5 w-2.5 rounded-full bg-[#27c93f]" />
            </div>
            <span className="font-mono text-xs text-zinc-400">
              nusic — system.log
            </span>
            {problemCount > 0 ? (
              <span className="rounded border border-red-700/60 bg-red-950/50 px-1.5 py-0.5 font-mono text-[11px] text-red-300">
                {problemCount} issue{problemCount === 1 ? "" : "s"}
              </span>
            ) : null}
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <div className="flex overflow-hidden rounded border border-zinc-600">
              <button
                type="button"
                onClick={() => setFilter("all")}
                className={`px-2 py-1 font-mono text-xs ${
                  filter === "all"
                    ? "bg-zinc-700 text-zinc-100"
                    : "bg-zinc-900 text-zinc-400 hover:bg-zinc-800"
                }`}
              >
                all
              </button>
              <button
                type="button"
                onClick={() => setFilter("problems")}
                className={`px-2 py-1 font-mono text-xs ${
                  filter === "problems"
                    ? "bg-zinc-700 text-zinc-100"
                    : "bg-zinc-900 text-zinc-400 hover:bg-zinc-800"
                }`}
              >
                problems
              </button>
            </div>
            <input
              type="search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="search…"
              className="w-36 rounded border border-zinc-600 bg-zinc-900 px-2 py-1 font-mono text-xs text-zinc-200 placeholder:text-zinc-600 focus:border-zinc-400 focus:outline-none"
            />
            <button
              type="button"
              onClick={() => {
                setLoading(true);
                stickToBottomRef.current = true;
                void loadLogs();
              }}
              className="rounded border border-zinc-600 bg-zinc-800 px-2.5 py-1 font-mono text-xs text-zinc-200 hover:bg-zinc-700"
            >
              refresh
            </button>
          </div>
        </div>

        <div
          ref={bodyRef}
          onScroll={handleScroll}
          className="min-h-0 flex-1 overflow-y-auto bg-[#0c0f12] px-3 py-3 font-mono text-[13px] leading-6 text-zinc-300"
        >
          {error && (
            <p className="mb-2 text-red-400">
              <span className="text-zinc-500">$ </span>
              error: {error}
            </p>
          )}

          {loading && entries.length === 0 ? (
            <p className="text-zinc-500">
              <span className="text-emerald-500">$</span> loading logs…
            </p>
          ) : visibleEntries.length === 0 ? (
            <p className="text-zinc-500">
              <span className="text-emerald-500">$</span>{" "}
              {filter === "problems"
                ? "no problems in recent logs"
                : "no log entries yet"}
            </p>
          ) : (
            <ul className="space-y-0.5">
              {visibleEntries.map((entry) => (
                <li
                  key={entry.id}
                  className={`flex gap-2 whitespace-pre-wrap break-all ${
                    isProblemStatus(entry.status)
                      ? "rounded bg-red-950/25 px-1 -mx-1"
                      : ""
                  }`}
                >
                  <span className="shrink-0 select-none text-zinc-600">›</span>
                  <span>
                    <span className="text-zinc-500">[{entry.played_at}]</span>{" "}
                    <span className={statusColorClass(entry.status)}>
                      {statusLabel(entry.status)}
                    </span>{" "}
                    <span className="text-zinc-200">{entry.task_title}</span>
                  </span>
                </li>
              ))}
            </ul>
          )}

          <p className="mt-2 text-emerald-500">
            ${" "}
            <span className="inline-block h-4 w-2 animate-pulse bg-emerald-500/80 align-middle" />
          </p>
        </div>

        <div className="shrink-0 border-t border-zinc-800 bg-[#10141a] px-3 py-1.5 font-mono text-[11px] text-zinc-500">
          {visibleEntries.length}/{entries.length} lines · auto-refresh 5s ·
          filter={filter} · English only
        </div>
      </div>
    </div>
  );
}
