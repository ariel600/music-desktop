import { FormEvent, useEffect, useState } from "react";
import { hasAppPassword, verifyAppPassword } from "../api";
import { errMsg } from "../lib/errors";

interface AppPasswordGateProps {
  onUnlocked: () => void;
}

export default function AppPasswordGate({ onUnlocked }: AppPasswordGateProps) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [checking, setChecking] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void hasAppPassword()
      .then((enabled) => {
        if (cancelled) {
          return;
        }
        if (!enabled) {
          onUnlocked();
        } else {
          setChecking(false);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setChecking(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [onUnlocked]);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const ok = await verifyAppPassword(password);
      if (!ok) {
        setError("סיסמה שגויה.");
        return;
      }
      setPassword("");
      onUnlocked();
    } catch (err) {
      setError(errMsg(err, "שגיאה באימות הסיסמה."));
    } finally {
      setSubmitting(false);
    }
  }

  if (checking) {
    return (
      <div className="flex h-full items-center justify-center bg-[#e8f3f2]">
        <p className="text-sm text-teal-700">טוען...</p>
      </div>
    );
  }

  return (
    <div
      className="flex h-full items-center justify-center bg-[#e8f3f2] p-6"
      dir="rtl"
    >
      <form
        onSubmit={(event) => void handleSubmit(event)}
        className="w-full max-w-sm rounded-xl border border-teal-200 bg-white p-6 shadow-md"
      >
        <h2 className="text-lg font-bold text-teal-900">כניסה לתוכנה</h2>
        <p className="mt-1 text-sm text-teal-600">
          יש להזין את סיסמת התוכנה כדי להמשיך.
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

        <button
          type="submit"
          disabled={submitting || password.trim().length === 0}
          className="mt-5 w-full rounded-lg bg-teal-700 px-4 py-2.5 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
        >
          {submitting ? "בודק..." : "כניסה"}
        </button>
      </form>
    </div>
  );
}
