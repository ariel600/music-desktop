import { useCallback, useDeferredValue, useEffect, useMemo, useRef, useState } from "react";
import { deleteCustomHoliday, getHolidays } from "../api";
import AddHolidayDialog from "./AddHolidayDialog";
import DeleteHolidayConfirmDialog from "./DeleteHolidayConfirmDialog";
import HolidayStatusDialog from "./HolidayStatusDialog";
import {
  HolidayKindIcon,
  holidayKindIconShellClass,
} from "./HolidayKindIcon";
import ToggleSwitch from "./ui/ToggleSwitch";
import type { HolidayEntry } from "../types";
import { errMsg } from "../lib/errors";
import {
  dayDisplayName,
  holidayIsOpen,
  isManualHoliday,
  isToday,
  normalizeHolidayStatusKind,
} from "../lib/holidays";
import {
  compareHolidayRulesByHebrewDate,
  compareHolidaysByHebrewDate,
  formatHolidayHebrewIdentity,
  holidayRecurrenceKey,
  isTechnicalHolidayAnchor,
  withHebrewIdentity,
} from "../lib/hebrewHolidays";
import { getOperationalDate } from "../lib/operationalDay";

interface HolidayListItem {
  holiday: HolidayEntry;
  name: string;
  hebrewDate: string;
  statusText: string;
  searchText: string;
}

function HolidayRowActionsMenu({
  canDelete,
  disabled,
  onStatus,
  onDelete,
}: {
  canDelete: boolean;
  disabled?: boolean;
  onStatus: () => void;
  onDelete: () => void;
}) {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menuOpen) {
      return;
    }

    function handleClickOutside(event: MouseEvent) {
      if (!menuRef.current?.contains(event.target as Node)) {
        setMenuOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [menuOpen]);

  return (
    <div ref={menuRef} className="relative shrink-0">
      <button
        type="button"
        disabled={disabled}
        onClick={() => setMenuOpen((current) => !current)}
        className="flex h-9 w-9 items-center justify-center rounded-lg text-teal-700 transition-colors hover:bg-white/80 disabled:cursor-not-allowed disabled:opacity-50"
        aria-label="אפשרויות"
        aria-expanded={menuOpen}
      >
        <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor" aria-hidden>
          <circle cx="12" cy="5" r="1.8" />
          <circle cx="12" cy="12" r="1.8" />
          <circle cx="12" cy="19" r="1.8" />
        </svg>
      </button>

      {menuOpen && (
        <div className="absolute bottom-full left-0 z-50 mb-1 min-w-[11rem] rounded-lg border border-teal-200 bg-white py-1 shadow-lg">
          <button
            type="button"
            className="w-full px-3 py-2 text-right text-sm text-teal-900 transition-colors hover:bg-teal-50"
            onClick={() => {
              setMenuOpen(false);
              onStatus();
            }}
          >
            סטטוס
          </button>
          {canDelete && (
            <button
              type="button"
              className="w-full px-3 py-2 text-right text-sm text-red-700 transition-colors hover:bg-red-50"
              onClick={() => {
                setMenuOpen(false);
                onDelete();
              }}
            >
              מחיקה
            </button>
          )}
        </div>
      )}
    </div>
  );
}

function buildHolidayListItem(holiday: HolidayEntry): HolidayListItem {
  const name = dayDisplayName(holiday);
  const hebrewDate = formatHolidayHebrewIdentity(holiday);
  const isOpen = holidayIsOpen(holiday);
  const statusText = isOpen
    ? holiday.open_time && holiday.close_time
      ? `פתוח · ${holiday.open_time}–${holiday.close_time}`
      : "פתוח"
    : "סגור";
  const searchText = [
    name,
    holiday.title,
    holiday.holiday_group,
    holiday.day_label,
    holiday.date,
    hebrewDate,
    holiday.open_time ?? "",
    holiday.close_time ?? "",
    isOpen ? "פתוח" : "סגור",
    isManualHoliday(holiday) ? "מותאם" : "",
  ]
    .join(" ")
    .toLowerCase();

  return {
    holiday,
    name,
    hebrewDate,
    statusText,
    searchText,
  };
}

export default function HolidaysPage() {
  const [holidays, setHolidays] = useState<HolidayEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [statusHoliday, setStatusHoliday] = useState<HolidayEntry | null>(null);
  const [holidayToDelete, setHolidayToDelete] = useState<HolidayEntry | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [busyDate, setBusyDate] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const deferredSearch = useDeferredValue(search);

  const loadHolidays = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await getHolidays();
      setHolidays([...data].sort(compareHolidaysByHebrewDate));
    } catch (err) {
      setError(errMsg(err, "שגיאה בטעינת החגים."));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadHolidays();
  }, [loadHolidays]);

  async function handleConfirmDelete() {
    if (!holidayToDelete) {
      return;
    }

    setIsDeleting(true);
    setBusyDate(holidayToDelete.date);
    setError(null);
    try {
      await deleteCustomHoliday(holidayToDelete.date);
      const updated = await getHolidays();
      setHolidays([...updated].sort(compareHolidaysByHebrewDate));
      setMessage("החג נמחק");
      setHolidayToDelete(null);
    } catch (err) {
      setError(errMsg(err, "שגיאה במחיקה."));
    } finally {
      setIsDeleting(false);
      setBusyDate(null);
    }
  }

  const holidayItems = useMemo(() => {
    const today = getOperationalDate();
    const rules = new Map<string, HolidayEntry>();

    for (const rawHoliday of holidays) {
      const holiday = withHebrewIdentity(rawHoliday);
      if (isTechnicalHolidayAnchor(holiday)) {
        continue;
      }
      const key = holidayRecurrenceKey(holiday);
      const current = rules.get(key);
      if (!current) {
        rules.set(key, holiday);
        continue;
      }

      const currentIsUpcoming = current.date >= today;
      const candidateIsUpcoming = holiday.date >= today;
      const candidateIsBetter =
        (candidateIsUpcoming && !currentIsUpcoming) ||
        (candidateIsUpcoming &&
          currentIsUpcoming &&
          holiday.date < current.date) ||
        (!candidateIsUpcoming &&
          !currentIsUpcoming &&
          holiday.date > current.date);
      if (candidateIsBetter) {
        rules.set(key, holiday);
      }
    }

    return [...rules.values()]
      .sort(compareHolidayRulesByHebrewDate)
      .map(buildHolidayListItem);
  }, [holidays]);

  const filteredHolidays = useMemo(() => {
    const query = deferredSearch.trim().toLowerCase();
    if (!query) {
      return holidayItems;
    }
    return holidayItems.filter((item) => item.searchText.includes(query));
  }, [holidayItems, deferredSearch]);

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <div className="flex shrink-0 flex-wrap items-center gap-3">
        <button
          type="button"
          onClick={() => setShowAddDialog(true)}
          className="rounded-lg bg-teal-700 px-3 py-1.5 text-sm font-medium text-white hover:bg-teal-800"
        >
          + הוספת חג
        </button>
        <input
          type="search"
          placeholder="חיפוש חגים..."
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          className="min-w-[12rem] flex-1 rounded-lg border border-teal-200 bg-white px-3 py-1.5 text-sm text-teal-900 shadow-sm focus:border-teal-500 focus:outline-none focus:ring-1 focus:ring-teal-500"
        />
        {holidayItems.length > 0 && (
          <span className="text-sm text-teal-600">
            {deferredSearch.trim()
              ? `${filteredHolidays.length} מתוך ${holidayItems.length}`
              : `${holidayItems.length} ${holidayItems.length === 1 ? "חג" : "חגים"}`}
          </span>
        )}
      </div>

      {message && (
        <p className="shrink-0 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          {message}
        </p>
      )}
      {error && (
        <p className="shrink-0 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}

      {isLoading ? (
        <p className="py-8 text-center text-sm text-teal-600">טוען חגים...</p>
      ) : holidayItems.length === 0 ? (
        <div className="flex min-h-0 flex-1 flex-col items-center justify-center rounded-lg border border-dashed border-teal-200 bg-teal-50/40 px-6 py-12 text-center">
          <p className="text-sm font-medium text-teal-900">אין חגים</p>
          <p className="mt-1 text-sm text-teal-600">
            לחץ על &quot;הוספת חג&quot; כדי להוסיף יום מותאם.
          </p>
        </div>
      ) : filteredHolidays.length === 0 ? (
        <p className="py-8 text-center text-sm text-teal-600">
          לא נמצאו חגים התואמים לחיפוש.
        </p>
      ) : (
        <ul className="min-h-0 flex-1 space-y-2 overflow-y-auto overflow-x-visible">
          {filteredHolidays.map((item) => {
            const { holiday } = item;
            const isOpen = holidayIsOpen(holiday);
            const busy = busyDate === holiday.date;
            const manual = isManualHoliday(holiday);

            return (
              <li
                key={holidayRecurrenceKey(holiday)}
                className={`flex items-center gap-3 rounded-lg border px-4 py-3 ${
                  isToday(holiday.date)
                    ? "border-teal-300 bg-teal-100/70"
                    : "border-teal-100 bg-teal-50/40"
                }`}
              >
                <span
                  className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-full ${holidayKindIconShellClass(holiday.day_label)}`}
                >
                  <HolidayKindIcon
                    kind={normalizeHolidayStatusKind(holiday.day_label)}
                    className="h-4 w-4"
                  />
                </span>

                <div className="min-w-0 flex-1">
                  <p className="truncate font-medium text-teal-900">
                    {item.name}
                    {manual ? (
                      <span className="ms-2 text-xs font-normal text-teal-500">
                        מותאם
                      </span>
                    ) : null}
                  </p>
                  <p className="truncate text-sm text-teal-600">
                    {item.hebrewDate}
                    {isToday(holiday.date) ? " · היום" : ""}
                    {" · "}
                    {holiday.day_label}
                  </p>
                  <p className="truncate text-xs text-teal-500">{item.statusText}</p>
                </div>

                <ToggleSwitch
                  id={`holiday-${holiday.date}`}
                  checked={isOpen}
                  offLabel="סגור"
                  onLabel="פתוח"
                  readOnly
                />

                <HolidayRowActionsMenu
                  canDelete={manual}
                  disabled={busy || isDeleting}
                  onStatus={() => setStatusHoliday(holiday)}
                  onDelete={() => setHolidayToDelete(holiday)}
                />
              </li>
            );
          })}
        </ul>
      )}

      {showAddDialog && (
        <AddHolidayDialog
          onClose={() => setShowAddDialog(false)}
          onSaved={() => {
            setMessage("החג נוסף בהצלחה");
            void loadHolidays();
          }}
        />
      )}

      {statusHoliday && (
        <HolidayStatusDialog
          holiday={statusHoliday}
          onClose={() => setStatusHoliday(null)}
          onSaved={(updated) => {
            setHolidays([...updated].sort(compareHolidaysByHebrewDate));
            setMessage("הסטטוס נשמר");
          }}
        />
      )}

      {holidayToDelete && (
        <DeleteHolidayConfirmDialog
          title={dayDisplayName(holidayToDelete)}
          isDeleting={isDeleting}
          onCancel={() => {
            if (!isDeleting) {
              setHolidayToDelete(null);
            }
          }}
          onConfirm={() => void handleConfirmDelete()}
        />
      )}
    </div>
  );
}
