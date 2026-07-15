import { useState, type ReactNode } from "react";
import SystemActivityDisableDialog from "../SystemActivityDisableDialog";
import { useSystemActivity } from "../SystemActivityProvider";
import ToggleSwitch from "../ui/ToggleSwitch";

export type AppView =
  | "overview"
  | "system-messages"
  | "emergency-messages"
  | "music"
  | "schedules"
  | "settings";

interface SidebarProps {
  activeView: AppView;
  onViewChange: (view: AppView) => void;
}

interface NavItem {
  id: AppView;
  label: string;
  icon: ReactNode;
}

const navItems: NavItem[] = [
  {
    id: "overview",
    label: "סקירה",
    icon: (
      <svg viewBox="0 0 24 24" className="h-4 w-4 shrink-0" fill="none" stroke="currentColor" strokeWidth="2">
        <rect x="3" y="3" width="7" height="7" rx="1" />
        <rect x="14" y="3" width="7" height="7" rx="1" />
        <rect x="3" y="14" width="7" height="7" rx="1" />
        <rect x="14" y="14" width="7" height="7" rx="1" />
      </svg>
    ),
  },
  {
    id: "system-messages",
    label: "הודעות מערכת",
    icon: (
      <svg viewBox="0 0 24 24" className="h-4 w-4 shrink-0" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M9 11l3 3L22 4" />
        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
      </svg>
    ),
  },
  {
    id: "emergency-messages",
    label: "הודעות חירום",
    icon: (
      <svg viewBox="0 0 24 24" className="h-4 w-4 shrink-0" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
        <path d="M12 9v4M12 17h.01" />
      </svg>
    ),
  },
  {
    id: "music",
    label: "מוזיקה",
    icon: (
      <svg viewBox="0 0 24 24" className="h-4 w-4 shrink-0" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M9 18V5l12-2v13" />
        <circle cx="6" cy="18" r="3" />
        <circle cx="18" cy="16" r="3" />
      </svg>
    ),
  },
  {
    id: "schedules",
    label: "לוחות זמנים",
    icon: (
      <svg viewBox="0 0 24 24" className="h-4 w-4 shrink-0" fill="none" stroke="currentColor" strokeWidth="2">
        <rect x="3" y="4" width="18" height="18" rx="2" />
        <path d="M16 2v4M8 2v4M3 10h18" />
      </svg>
    ),
  },
  {
    id: "settings",
    label: "הגדרות",
    icon: (
      <svg viewBox="0 0 24 24" className="h-4 w-4 shrink-0" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
    ),
  },
];

export default function Sidebar({ activeView, onViewChange }: SidebarProps) {
  const { active, setActive } = useSystemActivity();
  const [disableStep, setDisableStep] = useState<1 | 2 | null>(null);
  const [busy, setBusy] = useState(false);

  async function applyActive(next: boolean) {
    setBusy(true);
    try {
      await setActive(next);
    } finally {
      setBusy(false);
      setDisableStep(null);
    }
  }

  function handleToggle(next: boolean) {
    if (busy) {
      return;
    }
    if (next) {
      void applyActive(true);
      return;
    }
    setDisableStep(1);
  }

  return (
    <>
      <aside className="flex h-full w-48 shrink-0 flex-col overflow-hidden bg-teal-800 text-teal-50">
        <nav className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 py-3">
          {navItems.map((item) => (
            <button
              key={item.id}
              type="button"
              onClick={() => onViewChange(item.id)}
              className={`flex w-full items-center gap-2 rounded-md px-2 py-2 text-xs transition-colors ${
                activeView === item.id
                  ? "bg-teal-700 text-white shadow-inner"
                  : "text-teal-100 hover:bg-teal-700/70"
              }`}
              title={item.label}
            >
              {item.icon}
              <span className="truncate">{item.label}</span>
            </button>
          ))}
        </nav>

        <div
          className={`shrink-0 border-t border-teal-700/80 px-2 py-3 ${
            active ? "bg-teal-900/40" : "bg-slate-900/50"
          }`}
          dir="rtl"
        >
          <p
            className={`mb-2 text-center text-xs font-extrabold tracking-wide ${
              active ? "text-emerald-200" : "text-slate-300"
            }`}
          >
            פעילות המערכת
          </p>
          <div className="flex scale-110 justify-center py-1 [&_span]:!text-teal-50">
            <ToggleSwitch
              checked={active}
              disabled={busy}
              onChange={handleToggle}
              offLabel="כבוי"
              onLabel="פעיל"
            />
          </div>
          {!active ? (
            <p className="mt-2 text-center text-[10px] font-medium text-amber-200">
              המערכת כבויה
            </p>
          ) : null}
        </div>
      </aside>

      {disableStep ? (
        <SystemActivityDisableDialog
          step={disableStep}
          onCancel={() => setDisableStep(null)}
          onConfirm={() => {
            if (disableStep === 1) {
              setDisableStep(2);
              return;
            }
            void applyActive(false);
          }}
        />
      ) : null}
    </>
  );
}
