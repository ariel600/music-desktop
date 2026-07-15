import {
  FormEvent,
  ReactNode,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { listen } from "@tauri-apps/api/event";
import {
  getLockMusicAdd,
  hasSettingsPassword,
  verifySettingsPassword,
} from "../api";
import {
  markSettingsUnlocked,
  registerSettingsUnlockHandler,
  setSettingsAuthCache,
} from "../lib/settingsAuth";
import { errMsg } from "../lib/errors";

interface SettingsAuthContextValue {
  hasSettingsPassword: boolean;
  lockMusicAdd: boolean;
  settingsUnlocked: boolean;
  refresh: () => Promise<void>;
}

const SettingsAuthContext = createContext<SettingsAuthContextValue | null>(
  null,
);

export function useSettingsAuth() {
  const value = useContext(SettingsAuthContext);
  if (!value) {
    throw new Error("useSettingsAuth must be used within SettingsAuthProvider");
  }
  return value;
}

export function SettingsAuthProvider({ children }: { children: ReactNode }) {
  const [hasPassword, setHasPassword] = useState(false);
  const [lockMusicAdd, setLockMusicAdd] = useState(false);
  const [settingsUnlocked, setSettingsUnlocked] = useState(true);
  const [promptOpen, setPromptOpen] = useState(false);
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [resolver, setResolver] = useState<
    ((value: boolean) => void) | null
  >(null);

  const refresh = useCallback(async () => {
    try {
      const [enabled, lockMusic] = await Promise.all([
        hasSettingsPassword(),
        getLockMusicAdd(),
      ]);
      setHasPassword(enabled);
      setLockMusicAdd(lockMusic);
      setSettingsAuthCache({
        hasSettingsPassword: enabled,
        lockMusicAdd: lockMusic,
      });
      if (!enabled) {
        setSettingsUnlocked(true);
        markSettingsUnlocked(true);
      }
    } catch {
      setHasPassword(false);
      setLockMusicAdd(false);
      setSettingsUnlocked(true);
      markSettingsUnlocked(true);
      setSettingsAuthCache({
        hasSettingsPassword: false,
        lockMusicAdd: false,
      });
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listen("app-lock-required", () => {
      if (cancelled) {
        return;
      }
      setSettingsUnlocked(false);
      markSettingsUnlocked(false);
      void refresh();
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [refresh]);

  const requestUnlock = useCallback(() => {
    return new Promise<boolean>((resolve) => {
      setPassword("");
      setError(null);
      setPromptOpen(true);
      setResolver(() => resolve);
    });
  }, []);

  useEffect(() => {
    registerSettingsUnlockHandler(requestUnlock);
    return () => registerSettingsUnlockHandler(null);
  }, [requestUnlock]);

  function settle(value: boolean) {
    resolver?.(value);
    setResolver(null);
    setPromptOpen(false);
    setPassword("");
    setError(null);
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const ok = await verifySettingsPassword(password);
      if (!ok) {
        setError("סיסמת הגדרות שגויה.");
        return;
      }
      setSettingsUnlocked(true);
      markSettingsUnlocked(true);
      settle(true);
    } catch (err) {
      setError(errMsg(err, "שגיאה באימות הסיסמה."));
    } finally {
      setSubmitting(false);
    }
  }

  const value = useMemo(
    () => ({
      hasSettingsPassword: hasPassword,
      lockMusicAdd,
      settingsUnlocked,
      refresh,
    }),
    [hasPassword, lockMusicAdd, settingsUnlocked, refresh],
  );

  return (
    <SettingsAuthContext.Provider value={value}>
      {children}

      {promptOpen && (
        <div
          className="fixed inset-0 z-[80] flex items-center justify-center bg-black/40 p-4"
          dir="rtl"
        >
          <form
            onSubmit={(event) => void handleSubmit(event)}
            className="w-full max-w-sm rounded-xl border border-teal-200 bg-white p-6 shadow-xl"
          >
            <h2 className="text-lg font-bold text-teal-900">סיסמת הגדרות</h2>
            <p className="mt-1 text-sm text-teal-600">
              יש להזין את סיסמת ההגדרות כדי לבצע שינויים.
            </p>

            <label className="mt-5 block text-sm font-medium text-teal-800">
              סיסמה
              <input
                type="password"
                autoFocus
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                disabled={submitting}
                className="mt-1.5 w-full rounded-lg border border-teal-200 px-3 py-2 text-teal-900 outline-none focus:border-teal-500 focus:ring-1 focus:ring-teal-400"
              />
            </label>

            {error && (
              <p className="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
                {error}
              </p>
            )}

            <div className="mt-5 flex gap-2">
              <button
                type="submit"
                disabled={submitting || password.trim().length === 0}
                className="flex-1 rounded-lg bg-teal-700 px-4 py-2.5 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
              >
                {submitting ? "בודק..." : "אישור"}
              </button>
              <button
                type="button"
                disabled={submitting}
                onClick={() => settle(false)}
                className="rounded-lg border border-teal-200 bg-white px-4 py-2.5 text-sm font-semibold text-teal-800 hover:bg-teal-50"
              >
                ביטול
              </button>
            </div>
          </form>
        </div>
      )}
    </SettingsAuthContext.Provider>
  );
}
