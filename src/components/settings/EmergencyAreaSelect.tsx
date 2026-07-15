import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import {
  getEmergencyMonitoredCities,
  getOrefCities,
  setEmergencyMonitoredCities,
  type OrefCity,
} from "../../api";
import { errMsg } from "../../lib/errors";

const DEFAULT_CITY = "ירושלים - מרכז";

export default function EmergencyAreaSelect() {
  const [cities, setCities] = useState<OrefCity[]>([]);
  const [selected, setSelected] = useState(DEFAULT_CITY);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [menuBox, setMenuBox] = useState<{
    top: number;
    left: number;
    width: number;
  } | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    void Promise.all([getEmergencyMonitoredCities(), getOrefCities()])
      .then(([monitored, list]) => {
        if (cancelled) {
          return;
        }
        setSelected(monitored[0] ?? DEFAULT_CITY);
        setCities(list);
      })
      .catch((err) => {
        if (!cancelled) {
          setError(
            errMsg(err, "שגיאה בטעינת הגדרת האזור."),
          );
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useLayoutEffect(() => {
    if (!open || !buttonRef.current) {
      setMenuBox(null);
      return;
    }

    function updateMenuBox() {
      const button = buttonRef.current;
      if (!button) {
        return;
      }
      const rect = button.getBoundingClientRect();
      setMenuBox({
        top: rect.bottom + 4,
        left: rect.left,
        width: rect.width,
      });
    }

    updateMenuBox();
    window.addEventListener("resize", updateMenuBox);
    window.addEventListener("scroll", updateMenuBox, true);
    return () => {
      window.removeEventListener("resize", updateMenuBox);
      window.removeEventListener("scroll", updateMenuBox, true);
    };
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    function handleClickOutside(event: MouseEvent) {
      const target = event.target as Node;
      if (
        rootRef.current?.contains(target) ||
        menuRef.current?.contains(target)
      ) {
        return;
      }
      setOpen(false);
      setQuery("");
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open]);

  useEffect(() => {
    if (open) {
      searchRef.current?.focus();
    }
  }, [open]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) {
      return cities;
    }
    return cities.filter(
      (city) =>
        city.name.toLowerCase().includes(q) ||
        city.name_en.toLowerCase().includes(q) ||
        city.zone.toLowerCase().includes(q),
    );
  }, [cities, query]);

  async function selectCity(name: string) {
    if (saving || name === selected) {
      setOpen(false);
      setQuery("");
      return;
    }

    const previous = selected;
    setSelected(name);
    setOpen(false);
    setQuery("");
    setSaving(true);
    setError(null);
    try {
      const saved = await setEmergencyMonitoredCities([name]);
      setSelected(saved[0] ?? DEFAULT_CITY);
    } catch (err) {
      setSelected(previous);
      setError(errMsg(err, "שגיאה בשמירת האזור."));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="flex flex-col rounded-lg border border-teal-100 bg-white p-4 shadow-sm">
      <h3 className="mb-3 text-sm font-semibold text-teal-900">
        אזור להתראות חירום
      </h3>

      <div ref={rootRef} className="relative">
        <button
          ref={buttonRef}
          type="button"
          disabled={loading || saving}
          onClick={() => setOpen((current) => !current)}
          className="flex w-full items-center justify-between gap-2 rounded-lg border border-teal-200 bg-teal-50/40 px-3 py-2.5 text-right text-sm text-teal-950 transition-colors hover:bg-teal-50 disabled:cursor-not-allowed disabled:opacity-50"
          aria-haspopup="listbox"
          aria-expanded={open}
        >
          <span className="min-w-0 truncate font-medium">
            {loading ? "טוען..." : selected}
          </span>
          <svg
            viewBox="0 0 24 24"
            className={`h-4 w-4 shrink-0 text-teal-600 transition-transform ${open ? "rotate-180" : ""}`}
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden
          >
            <path d="m6 9 6 6 6-6" />
          </svg>
        </button>

        {open &&
          menuBox &&
          createPortal(
            <div
              ref={menuRef}
              style={{
                position: "fixed",
                top: menuBox.top,
                left: menuBox.left,
                width: menuBox.width,
              }}
              className="z-50 overflow-hidden rounded-lg border border-teal-200 bg-white shadow-lg"
            >
              <div className="border-b border-teal-100 p-2">
                <input
                  ref={searchRef}
                  type="search"
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  placeholder="חיפוש אזור..."
                  className="w-full rounded-md border border-teal-200 px-2.5 py-1.5 text-sm text-teal-950 outline-none placeholder:text-teal-400 focus:border-teal-500 focus:ring-1 focus:ring-teal-400"
                />
              </div>
              <ul className="max-h-44 overflow-y-auto py-1" role="listbox">
                {filtered.length === 0 ? (
                  <li className="px-3 py-4 text-center text-sm text-teal-600">
                    לא נמצאו אזורים
                  </li>
                ) : (
                  filtered.slice(0, 120).map((city) => {
                    const isSelected = city.name === selected;
                    return (
                      <li key={city.name}>
                        <button
                          type="button"
                          role="option"
                          aria-selected={isSelected}
                          disabled={saving}
                          onClick={() => void selectCity(city.name)}
                          className={`flex w-full flex-col gap-0.5 px-3 py-2 text-right transition-colors hover:bg-teal-50 ${
                            isSelected
                              ? "bg-teal-50 font-medium text-teal-900"
                              : "text-teal-800"
                          }`}
                        >
                          <span className="truncate text-sm">{city.name}</span>
                          {city.zone ? (
                            <span className="truncate text-xs text-teal-500">
                              {city.zone}
                            </span>
                          ) : null}
                        </button>
                      </li>
                    );
                  })
                )}
                {filtered.length > 120 && (
                  <li className="px-3 py-2 text-center text-xs text-teal-500">
                    מציג 120 מתוך {filtered.length} — צמצמו את החיפוש
                  </li>
                )}
              </ul>
            </div>,
            document.body,
          )}
      </div>

      {error && (
        <p className="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}
    </div>
  );
}
