import { useEffect, useState } from "react";
import { ask } from "@tauri-apps/plugin-dialog";
import {
  clearAppPassword,
  clearSettingsPassword,
  getLockMusicAdd,
  hasAppPassword,
  hasSettingsPassword,
  setAppPassword,
  setLockMusicAdd,
  setSettingsPassword,
} from "../../api";
import { errMsg } from "../../lib/errors";
import { useSettingsAuth } from "../SettingsAuthProvider";

type PasswordDialogKind = "login" | "edit";
type PasswordDialogAction = "add" | "change" | "remove";

type PasswordDialogState = {
  kind: PasswordDialogKind;
  action: PasswordDialogAction;
};

const passwordInputClassName =
  "mt-1 w-full rounded-lg border border-teal-200 px-3 py-2 text-sm text-teal-900 outline-none focus:border-teal-500 focus:ring-1 focus:ring-teal-400 disabled:opacity-50";

function PasswordField({
  label,
  value,
  busy,
  onChange,
  autoFocus,
}: {
  label: string;
  value: string;
  busy: boolean;
  onChange: (value: string) => void;
  autoFocus?: boolean;
}) {
  return (
    <label className="text-sm font-medium text-teal-800">
      {label}
      <input
        type="password"
        value={value}
        disabled={busy}
        autoFocus={autoFocus}
        onChange={(event) => onChange(event.target.value)}
        className={passwordInputClassName}
      />
    </label>
  );
}

function emptyPasswordFields() {
  return { current: "", next: "", confirm: "" };
}

function validateNewPasswordPair(next: string, confirm: string): string | null {
  if (next.trim().length < 4) {
    return "הסיסמה חייבת להכיל לפחות 4 תווים.";
  }
  if (next !== confirm) {
    return "הסיסמה החדשה ואימות הסיסמה אינם תואמים.";
  }
  return null;
}

function passwordKindLabel(kind: PasswordDialogKind): string {
  return kind === "login" ? "סיסמת התחברות" : "סיסמת עריכה";
}

function PasswordDialog({
  dialog,
  fields,
  busy,
  error,
  onFieldsChange,
  onClose,
  onSubmit,
}: {
  dialog: PasswordDialogState;
  fields: { current: string; next: string; confirm: string };
  busy: boolean;
  error: string | null;
  onFieldsChange: (fields: {
    current: string;
    next: string;
    confirm: string;
  }) => void;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const title = passwordKindLabel(dialog.kind);
  const actionTitle =
    dialog.action === "add"
      ? `הגדרת ${title}`
      : dialog.action === "change"
        ? `שינוי ${title}`
        : `הסרת ${title}`;

  function setField(key: "current" | "next" | "confirm", value: string) {
    onFieldsChange({ ...fields, [key]: value });
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      dir="rtl"
      role="dialog"
      aria-modal="true"
      aria-labelledby="password-dialog-title"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-xl border border-teal-100 bg-white p-5 shadow-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <h3
          id="password-dialog-title"
          className="text-base font-semibold text-teal-900"
        >
          {actionTitle}
        </h3>
        <p className="mt-1 text-xs text-teal-600">
          {dialog.kind === "login"
            ? "סיסמה זו תוצג בכניסה לתוכנה."
            : "סיסמה זו תידרש בעת עריכת נתונים והגדרות במערכת."}
        </p>

        <div className="mt-4 flex flex-col gap-3">
          {dialog.action === "add" && (
            <>
              <PasswordField
                label="סיסמה"
                value={fields.next}
                busy={busy}
                onChange={(value) => setField("next", value)}
                autoFocus
              />
              <PasswordField
                label="הזנת סיסמה שוב"
                value={fields.confirm}
                busy={busy}
                onChange={(value) => setField("confirm", value)}
              />
            </>
          )}

          {dialog.action === "change" && (
            <>
              <PasswordField
                label="סיסמה ישנה"
                value={fields.current}
                busy={busy}
                onChange={(value) => setField("current", value)}
                autoFocus
              />
              <PasswordField
                label="סיסמה חדשה"
                value={fields.next}
                busy={busy}
                onChange={(value) => setField("next", value)}
              />
              <PasswordField
                label="הזנת סיסמה חדשה שוב"
                value={fields.confirm}
                busy={busy}
                onChange={(value) => setField("confirm", value)}
              />
            </>
          )}

          {dialog.action === "remove" && (
            <PasswordField
              label="סיסמה נוכחית לאישור ההסרה"
              value={fields.current}
              busy={busy}
              onChange={(value) => setField("current", value)}
              autoFocus
            />
          )}
        </div>

        {error && (
          <p className="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
            {error}
          </p>
        )}

        <div className="mt-4 flex flex-wrap justify-end gap-2">
          <button
            type="button"
            disabled={busy}
            onClick={onClose}
            className="rounded-lg border border-teal-200 bg-white px-3 py-1.5 text-sm font-semibold text-teal-800 hover:bg-teal-50 disabled:cursor-not-allowed disabled:opacity-60"
          >
            ביטול
          </button>
          <button
            type="button"
            disabled={busy}
            onClick={onSubmit}
            className={
              dialog.action === "remove"
                ? "rounded-lg border border-red-300 bg-white px-3 py-1.5 text-sm font-semibold text-red-700 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-60"
                : "rounded-lg bg-teal-700 px-3 py-1.5 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
            }
          >
            {dialog.action === "add"
              ? "שמירה"
              : dialog.action === "change"
                ? "שמירת סיסמה חדשה"
                : "אישור הסרה"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default function PasswordSettingsPanel() {
  const { refresh: refreshSettingsAuth } = useSettingsAuth();
  const [ready, setReady] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [dialogError, setDialogError] = useState<string | null>(null);

  const [loginEnabled, setLoginEnabled] = useState(false);
  const [editEnabled, setEditEnabled] = useState(false);
  const [lockMusicAdd, setLockMusicAddState] = useState(false);

  const [dialog, setDialog] = useState<PasswordDialogState | null>(null);
  const [fields, setFields] = useState(emptyPasswordFields);

  useEffect(() => {
    let cancelled = false;
    void Promise.all([hasAppPassword(), hasSettingsPassword(), getLockMusicAdd()])
      .then(([login, edit, lockMusic]) => {
        if (!cancelled) {
          setLoginEnabled(login);
          setEditEnabled(edit);
          setLockMusicAddState(lockMusic);
          setReady(true);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setReady(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function openDialog(kind: PasswordDialogKind, action: PasswordDialogAction) {
    setMessage(null);
    setDialogError(null);
    setFields(emptyPasswordFields());
    setDialog({ kind, action });
  }

  function closeDialog() {
    if (busy) {
      return;
    }
    setDialog(null);
    setDialogError(null);
    setFields(emptyPasswordFields());
  }

  async function handleDialogSubmit() {
    if (!dialog) {
      return;
    }
    setDialogError(null);

    if (dialog.action === "remove") {
      if (fields.current.trim().length === 0) {
        setDialogError("יש להזין את הסיסמה הנוכחית להסרה.");
        return;
      }
      const confirmed = await ask(
        dialog.kind === "login"
          ? "האם להסיר את סיסמת ההתחברות? אחרי ההסרה ניתן יהיה לפתוח את התוכנה ללא סיסמה."
          : "האם להסיר את סיסמת העריכה? אחרי ההסרה ניתן יהיה לערוך במערכת ללא סיסמה.",
        {
          title:
            dialog.kind === "login"
              ? "הסרת סיסמת התחברות"
              : "הסרת סיסמת עריכה",
          kind: "warning",
        },
      );
      if (!confirmed) {
        return;
      }
      setBusy(true);
      try {
        if (dialog.kind === "login") {
          await clearAppPassword(fields.current);
          setLoginEnabled(false);
          setMessage("סיסמת ההתחברות הוסרה.");
        } else {
          await clearSettingsPassword(fields.current);
          setEditEnabled(false);
          try {
            await setLockMusicAdd(false);
            setLockMusicAddState(false);
          } catch {
            setLockMusicAddState(false);
          }
          await refreshSettingsAuth();
          setMessage("סיסמת העריכה הוסרה.");
        }
        setDialog(null);
        setFields(emptyPasswordFields());
      } catch (err) {
        setDialogError(
          errMsg(err, "שגיאה בהסרת הסיסמה."),
        );
      } finally {
        setBusy(false);
      }
      return;
    }

    const changing = dialog.action === "change";
    if (changing && fields.current.trim().length === 0) {
      setDialogError("יש להזין את הסיסמה הישנה.");
      return;
    }
    const validation = validateNewPasswordPair(fields.next, fields.confirm);
    if (validation) {
      setDialogError(validation);
      return;
    }

    setBusy(true);
    try {
      if (dialog.kind === "login") {
        await setAppPassword(fields.next, changing ? fields.current : null);
        setLoginEnabled(true);
        setMessage(
          changing
            ? "סיסמת ההתחברות עודכנה בהצלחה."
            : "סיסמת ההתחברות הוגדרה בהצלחה.",
        );
      } else {
        await setSettingsPassword(
          fields.next,
          changing ? fields.current : null,
        );
        setEditEnabled(true);
        await refreshSettingsAuth();
        setMessage(
          changing
            ? "סיסמת העריכה עודכנה בהצלחה."
            : "סיסמת העריכה הוגדרה בהצלחה.",
        );
      }
      setDialog(null);
      setFields(emptyPasswordFields());
    } catch (err) {
      setDialogError(
        errMsg(err, "שגיאה בשמירת הסיסמה."),
      );
    } finally {
      setBusy(false);
    }
  }

  async function handleToggleLockMusic(next: boolean) {
    setMessage(null);
    setBusy(true);
    try {
      const saved = await setLockMusicAdd(next);
      setLockMusicAddState(saved);
      await refreshSettingsAuth();
      setMessage(
        saved
          ? "הוספת מוזיקה ננעלה — תידרש סיסמת עריכה."
          : "הוספת מוזיקה פתוחה ללא סיסמת עריכה.",
      );
    } catch (err) {
      setMessage(
        errMsg(err, "שגיאה בעדכון נעילת הוספת מוזיקה."),
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col rounded-lg border border-teal-100 bg-white p-4 shadow-sm">
      <h3 className="shrink-0 text-sm font-semibold text-teal-900">ניהול סיסמאות</h3>

      <div className="mt-3 flex min-h-0 flex-1 flex-col gap-3">
        <div className="flex min-h-0 flex-1 flex-col justify-center rounded-lg border border-teal-100 bg-teal-50/40 p-3">
          <h4 className="text-sm font-semibold text-teal-900">
            סיסמת התחברות
          </h4>
          <p className="mt-1 text-xs text-teal-600">
            נדרשת בכניסה לתוכנה.
            {ready && (
              <span className="mt-1 block font-medium text-teal-800">
                {loginEnabled ? "מוגדרת" : "לא מוגדרת"}
              </span>
            )}
          </p>
          <div className="mt-3 flex flex-col gap-2">
            {!loginEnabled ? (
              <button
                type="button"
                disabled={!ready || busy}
                onClick={() => openDialog("login", "add")}
                className="rounded-lg bg-teal-700 px-3 py-2 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
              >
                הגדרת סיסמת התחברות
              </button>
            ) : (
              <>
                <button
                  type="button"
                  disabled={!ready || busy}
                  onClick={() => openDialog("login", "change")}
                  className="rounded-lg bg-teal-700 px-3 py-2 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
                >
                  שינוי סיסמת התחברות
                </button>
                <button
                  type="button"
                  disabled={!ready || busy}
                  onClick={() => openDialog("login", "remove")}
                  className="rounded-lg border border-red-300 bg-white px-3 py-2 text-sm font-semibold text-red-700 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-60"
                >
                  הסרת סיסמת התחברות
                </button>
              </>
            )}
          </div>
        </div>

        <div className="flex min-h-0 flex-1 flex-col justify-center rounded-lg border border-teal-100 bg-teal-50/40 p-3">
          <h4 className="text-sm font-semibold text-teal-900">סיסמת עריכה</h4>
          <p className="mt-1 text-xs text-teal-600">
            נדרשת בעריכת הגדרות ונתונים.
            {ready && (
              <span className="mt-1 block font-medium text-teal-800">
                {editEnabled ? "מוגדרת" : "לא מוגדרת"}
              </span>
            )}
          </p>
          <div className="mt-3 flex flex-col gap-2">
            {!editEnabled ? (
              <button
                type="button"
                disabled={!ready || busy}
                onClick={() => openDialog("edit", "add")}
                className="rounded-lg bg-teal-700 px-3 py-2 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
              >
                הגדרת סיסמת עריכה
              </button>
            ) : (
              <>
                <button
                  type="button"
                  disabled={!ready || busy}
                  onClick={() => openDialog("edit", "change")}
                  className="rounded-lg bg-teal-700 px-3 py-2 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
                >
                  שינוי סיסמת עריכה
                </button>
                <button
                  type="button"
                  disabled={!ready || busy}
                  onClick={() => openDialog("edit", "remove")}
                  className="rounded-lg border border-red-300 bg-white px-3 py-2 text-sm font-semibold text-red-700 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-60"
                >
                  הסרת סיסמת עריכה
                </button>
              </>
            )}
          </div>

          <label className="mt-3 flex cursor-pointer items-center gap-2 text-sm text-teal-800">
            <input
              type="checkbox"
              checked={lockMusicAdd}
              disabled={!ready || busy || !editEnabled}
              onChange={(event) =>
                void handleToggleLockMusic(event.target.checked)
              }
              className="h-4 w-4 rounded border-teal-300 text-teal-700 focus:ring-teal-500 disabled:opacity-50"
            />
            נעילת הוספת מוזיקה
          </label>
          {!editEnabled && (
            <p className="mt-1 text-xs text-teal-500">
              ניתן לנעול רק אחרי הגדרת סיסמת עריכה.
            </p>
          )}
        </div>
      </div>

      {message && (
        <p className="mt-2 shrink-0 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          {message}
        </p>
      )}

      {dialog && (
        <PasswordDialog
          dialog={dialog}
          fields={fields}
          busy={busy}
          error={dialogError}
          onFieldsChange={setFields}
          onClose={closeDialog}
          onSubmit={() => void handleDialogSubmit()}
        />
      )}
    </div>
  );
}
