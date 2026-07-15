import { useCallback, useEffect, useState, type ReactNode } from "react";
import { getOperatingHours, setOperatingHours } from "../../api";
import {
  getActiveIsraelClockSeason,
  type IsraelClockSeason,
} from "../../lib/israelClock";
import {
  createDefaultOperatingHours,
  formatOperatingDateDots,
  getSeasonDayHours,
  isTemporaryOperatingHoursActive,
  normalizeOperatingHours,
  OPERATING_DAYS,
  type OperatingDayId,
  type OperatingHoursSettingsData,
  type TemporaryOperatingHours,
} from "../../lib/operatingHours";
import EditTemporaryOperatingHoursDialog from "./EditTemporaryOperatingHoursDialog";
import { errMsg } from "../../lib/errors";
import Time24Input from "./Time24Input";

type SeasonalSectionId = IsraelClockSeason;

const SEASONAL_HOURS_ROWS: {
  id: IsraelClockSeason;
  label: string;
  appliesWhen: string;
  icon: ReactNode;
}[] = [
  {
    id: "winter",
    label: "שעות פעילות חורף",
    appliesWhen: "פועלות בשעון חורף",
    icon: (
      <svg
        viewBox="0 0 24 24"
        className="h-6 w-6"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        aria-hidden
      >
        <path d="M12 2v20M2 12h20M4.9 4.9l14.2 14.2M19.1 4.9 4.9 19.1" />
      </svg>
    ),
  },
  {
    id: "summer",
    label: "שעות פעילות קיץ",
    appliesWhen: "פועלות בשעון קיץ",
    icon: (
      <svg
        viewBox="0 0 24 24"
        className="h-6 w-6"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        aria-hidden
      >
        <circle cx="12" cy="12" r="4" />
        <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
      </svg>
    ),
  },
];

function EditSectionButton({
  editing,
  active,
  onToggle,
}: {
  editing: boolean;
  active: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onToggle}
      className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-lg transition-colors ${
        editing
          ? "bg-teal-700 text-white hover:bg-teal-800"
          : active
            ? "bg-white text-teal-800 hover:bg-teal-50"
            : "bg-slate-200 text-slate-600 hover:bg-slate-300"
      }`}
      title={editing ? "סיום עריכה" : "עריכה"}
      aria-label={editing ? "סיום עריכה" : "עריכה"}
      aria-pressed={editing}
    >
      {editing ? (
        <svg
          viewBox="0 0 24 24"
          className="h-4 w-4"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.5"
          aria-hidden
        >
          <path d="M20 6 9 17l-5-5" />
        </svg>
      ) : (
        <svg
          viewBox="0 0 24 24"
          className="h-4 w-4"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden
        >
          <path d="M12 20h9" />
          <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
        </svg>
      )}
    </button>
  );
}

export default function OperatingHoursSettings() {
  const [activeSeason, setActiveSeason] = useState<IsraelClockSeason>(() =>
    getActiveIsraelClockSeason(),
  );
  const [hours, setHours] = useState<OperatingHoursSettingsData>(
    createDefaultOperatingHours,
  );
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingSection, setEditingSection] = useState<SeasonalSectionId | null>(
    null,
  );
  const [showTemporaryDialog, setShowTemporaryDialog] = useState(false);
  const [nowTick, setNowTick] = useState(() => Date.now());

  const isTemporaryActive = isTemporaryOperatingHoursActive(
    hours.temporary,
    new Date(nowTick),
  );

  const loadHours = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await getOperatingHours();
      setHours(normalizeOperatingHours(data));
    } catch (err) {
      setError(
        errMsg(err, "שגיאה בטעינת שעות הפעילות."),
      );
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadHours();
  }, [loadHours]);

  useEffect(() => {
    function refresh() {
      setActiveSeason(getActiveIsraelClockSeason());
      setNowTick(Date.now());
    }

    refresh();
    const timer = window.setInterval(refresh, 60_000);
    return () => window.clearInterval(timer);
  }, []);

  function toggleEditing(section: SeasonalSectionId) {
    setEditingSection((current) => (current === section ? null : section));
  }

  async function updateDayTime(
    seasonId: IsraelClockSeason,
    dayId: OperatingDayId,
    field: "open" | "close",
    value: string,
  ) {
    const previous = hours;
    const next: OperatingHoursSettingsData = {
      ...hours,
      [seasonId]: {
        ...hours[seasonId],
        [dayId]: {
          ...getSeasonDayHours(hours[seasonId], dayId),
          [field]: value,
        },
      },
    };

    setHours(next);
    setIsSaving(true);
    setError(null);

    try {
      const saved = await setOperatingHours(next);
      setHours(normalizeOperatingHours(saved));
    } catch (err) {
      setHours(previous);
      setError(errMsg(err, "שגיאה בשמירת שעות הפעילות."));
    } finally {
      setIsSaving(false);
    }
  }

  async function saveTemporary(temporary: TemporaryOperatingHours) {
    const previous = hours;
    const next: OperatingHoursSettingsData = {
      ...hours,
      temporary,
    };
    setHours(next);
    setError(null);
    try {
      const saved = await setOperatingHours(next);
      setHours(normalizeOperatingHours(saved));
    } catch (err) {
      setHours(previous);
      throw err instanceof Error
        ? err
        : new Error("שגיאה בשמירת שעות זמניות.");
    }
  }

  const temporaryFromLabel = formatOperatingDateDots(hours.temporary.valid_from);
  const temporaryToLabel = formatOperatingDateDots(hours.temporary.valid_to);

  return (
    <div className="flex h-full min-h-0 flex-col gap-3">
      {error && (
        <p className="shrink-0 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}

      <div
        className={`flex shrink-0 items-center gap-3 rounded-xl border-2 px-4 py-4 transition-colors ${
          isTemporaryActive
            ? "border-teal-700 bg-teal-100 shadow-md ring-2 ring-teal-700/20"
            : "border-slate-300 bg-slate-100"
        }`}
      >
        <div
          className={`flex h-12 w-12 shrink-0 items-center justify-center rounded-lg shadow-sm ${
            isTemporaryActive
              ? "bg-white text-teal-800"
              : "bg-slate-200 text-slate-500"
          }`}
        >
          <svg
            viewBox="0 0 24 24"
            className="h-6 w-6"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden
          >
            <circle cx="12" cy="12" r="9" />
            <path d="M12 7v5l3 2" />
          </svg>
        </div>
        <div className="shrink-0">
          <div className="flex flex-nowrap items-center gap-2">
            <h3
              className={`whitespace-nowrap font-bold ${
                isTemporaryActive ? "text-teal-950" : "text-slate-600"
              }`}
            >
              שעות פעילות זמניות
            </h3>
            {isTemporaryActive && (
              <span className="rounded-full bg-teal-700 px-2.5 py-0.5 text-xs font-semibold text-white">
                מוגדר כרגע
              </span>
            )}
          </div>
          <p
            className={`mt-0.5 whitespace-nowrap text-xs ${
              isTemporaryActive ? "text-teal-800" : "text-slate-500"
            }`}
          >
            דורסות זמנית את שעות החורף או הקיץ
          </p>
        </div>

        {!isLoading && (
          <ul className="flex shrink-0 items-center gap-2 overflow-x-auto">
            {OPERATING_DAYS.map((day) => {
              const dayValue = getSeasonDayHours(hours.temporary, day.id);
              const labelClass = isTemporaryActive
                ? "text-teal-900"
                : "text-slate-500";
              const timeClass = isTemporaryActive
                ? "text-teal-800"
                : "text-slate-500";

              return (
                <li
                  key={day.id}
                  className={`flex shrink-0 flex-col gap-1.5 rounded-lg px-2.5 py-1.5 ${
                    isTemporaryActive ? "bg-white/80" : "bg-slate-200/70"
                  }`}
                >
                  <span
                    className={`text-center text-sm font-semibold leading-none ${labelClass}`}
                  >
                    {day.label}
                  </span>
                  <p
                    className={`whitespace-nowrap text-center text-sm font-medium tabular-nums leading-none ${timeClass}`}
                    dir="rtl"
                  >
                    מ- {dayValue.open} עד - {dayValue.close}
                  </p>
                </li>
              );
            })}
          </ul>
        )}

        <div className="flex min-w-[5.5rem] flex-1 items-center justify-center px-4">
          <div
            className={`whitespace-nowrap text-center text-xs leading-snug ${
              isTemporaryActive ? "text-teal-800" : "text-slate-500"
            }`}
          >
            <p className="font-semibold">תוקף</p>
            <p>מ- {temporaryFromLabel}</p>
            <p>עד- {temporaryToLabel}</p>
          </div>
        </div>

        <EditSectionButton
          editing={showTemporaryDialog}
          active={isTemporaryActive}
          onToggle={() => setShowTemporaryDialog(true)}
        />
      </div>

      {isLoading ? (
        <p className="py-8 text-center text-sm text-teal-600">טוען שעות פעילות...</p>
      ) : (
        <ul className="grid h-full min-h-0 flex-1 grid-cols-1 gap-3 overflow-hidden sm:grid-cols-2 sm:grid-rows-1">
          {SEASONAL_HOURS_ROWS.map((row) => {
            const isActive = !isTemporaryActive && row.id === activeSeason;
            const isEditing = editingSection === row.id;
            const seasonHours = hours[row.id];

            return (
              <li
                key={row.id}
                className={`flex h-full min-h-0 flex-col rounded-xl border-2 transition-colors ${
                  isActive
                    ? "border-teal-700 bg-teal-100 shadow-md ring-2 ring-teal-700/20"
                    : "border-slate-300 bg-slate-100"
                }`}
              >
                <header
                  className={`flex shrink-0 items-center gap-3 border-b px-4 py-3 ${
                    isActive ? "border-teal-200/70" : "border-slate-300"
                  }`}
                >
                  <div
                    className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-lg shadow-sm ${
                      isActive
                        ? "bg-white text-teal-800"
                        : "bg-slate-200 text-slate-500"
                    }`}
                  >
                    {row.icon}
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <h3
                        className={`truncate font-bold ${
                          isActive ? "text-teal-950" : "text-slate-600"
                        }`}
                      >
                        {row.label}
                      </h3>
                      {isActive && (
                        <span className="shrink-0 rounded-full bg-teal-700 px-2.5 py-0.5 text-xs font-semibold text-white">
                          מוגדר כרגע
                        </span>
                      )}
                    </div>
                    <p
                      className={`text-xs ${
                        isActive ? "text-teal-800" : "text-slate-500"
                      }`}
                    >
                      {isEditing ? "מצב עריכה" : row.appliesWhen}
                    </p>
                  </div>
                  <EditSectionButton
                    editing={isEditing}
                    active={isActive}
                    onToggle={() => toggleEditing(row.id)}
                  />
                </header>

                <div className="flex min-h-0 flex-1 flex-col overflow-hidden px-4 py-4">
                  <div
                    className={`mb-3 grid shrink-0 grid-cols-[5.5rem_1fr_1fr] gap-3 px-1 text-sm font-semibold ${
                      isActive ? "text-teal-700" : "text-slate-500"
                    }`}
                  >
                    <span>יום</span>
                    <span>שעת פתיחה</span>
                    <span>שעת סגירה</span>
                  </div>

                  <ul className="flex min-h-0 flex-1 flex-col gap-3">
                    {OPERATING_DAYS.map((day) => {
                      const dayValue = getSeasonDayHours(seasonHours, day.id);

                      return (
                        <li
                          key={day.id}
                          className={`grid min-h-0 flex-1 grid-cols-[5.5rem_1fr_1fr] items-center gap-3 rounded-xl px-3 py-2 ${
                            isActive ? "bg-white/80" : "bg-slate-200/70"
                          }`}
                        >
                          <span
                            className={`text-sm font-semibold ${
                              isActive ? "text-teal-900" : "text-slate-500"
                            }`}
                          >
                            {day.label}
                          </span>
                          <Time24Input
                            value={dayValue.open}
                            disabled={!isEditing || isSaving}
                            muted={!isEditing || !isActive}
                            ariaLabel={`שעת פתיחה ${day.label} ${row.label}`}
                            onCommit={(nextValue) =>
                              void updateDayTime(row.id, day.id, "open", nextValue)
                            }
                          />
                          <Time24Input
                            value={dayValue.close}
                            disabled={!isEditing || isSaving}
                            muted={!isEditing || !isActive}
                            ariaLabel={`שעת סגירה ${day.label} ${row.label}`}
                            onCommit={(nextValue) =>
                              void updateDayTime(
                                row.id,
                                day.id,
                                "close",
                                nextValue,
                              )
                            }
                          />
                        </li>
                      );
                    })}
                  </ul>
                </div>
              </li>
            );
          })}
        </ul>
      )}

      {showTemporaryDialog && (
        <EditTemporaryOperatingHoursDialog
          initial={hours.temporary}
          onClose={() => setShowTemporaryDialog(false)}
          onSave={saveTemporary}
        />
      )}
    </div>
  );
}
