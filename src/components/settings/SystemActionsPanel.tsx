import { useState } from "react";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import {
  exportBackup,
  importBackup,
  playVolumeTest,
  resetSystem,
} from "../../api";
import { errMsg } from "../../lib/errors";
import { VOLUME_CHANNELS, type VolumeChannelId } from "./volumeChannels";

const BACKUP_EXTENSION = "mshbak";
const BACKUP_FILE_NAME = `גיבוי למערכת הודעות חכמה.${BACKUP_EXTENSION}`;
const BACKUP_FILTERS = [
  {
    name: "גיבוי מערכת הודעות חכמה",
    extensions: [BACKUP_EXTENSION],
  },
];

export default function SystemActionsPanel() {
  const [testChannel, setTestChannel] = useState<VolumeChannelId>("system");
  const [testing, setTesting] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const [keepMusic, setKeepMusic] = useState(true);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const busy = testing || isExporting || isRestoring || isResetting;

  async function handleTestAudio() {
    setMessage(null);
    setError(null);
    setTesting(true);
    try {
      await playVolumeTest(testChannel);
    } catch (err) {
      setError(errMsg(err, "שגיאה בבדיקת השמע."));
    } finally {
      setTesting(false);
    }
  }

  async function handleCreateBackup() {
    setMessage(null);
    setError(null);

    const dest = await save({
      title: "שמירת גיבוי הגדרות",
      defaultPath: BACKUP_FILE_NAME,
      filters: BACKUP_FILTERS,
    });

    if (typeof dest !== "string") {
      return;
    }

    setIsExporting(true);
    try {
      const path = await exportBackup(dest);
      setMessage(`הגיבוי נשמר בהצלחה ב: ${path}`);
    } catch (err) {
      setError(errMsg(err, "שגיאה ביצירת גיבוי."));
    } finally {
      setIsExporting(false);
    }
  }

  async function handleRestoreBackup() {
    setMessage(null);
    setError(null);

    const confirmed = await ask(
      "שחזור גיבוי ידרוס את ההגדרות והקבצים הנוכחיים. להמשיך?",
      { title: "שחזור גיבוי", kind: "warning" },
    );
    if (!confirmed) {
      return;
    }

    const source = await open({
      multiple: false,
      title: "בחרו קובץ גיבוי",
      filters: BACKUP_FILTERS,
    });

    if (typeof source !== "string") {
      return;
    }

    setIsRestoring(true);
    try {
      await importBackup(source);
      setMessage("הגיבוי שוחזר בהצלחה. מומלץ להפעיל מחדש את האפליקציה.");
    } catch (err) {
      setError(errMsg(err, "שגיאה בשחזור גיבוי."));
    } finally {
      setIsRestoring(false);
    }
  }

  async function handleResetSystem() {
    setMessage(null);
    setError(null);

    const firstConfirm = await ask(
      keepMusic
        ? "איפוס המערכת ימחק את כל ההגדרות, ההודעות והנתונים השמורים.\n\nתיקיות המוזיקה יישמרו כפי שבחרת.\n\nמומלץ מאוד לבצע גיבוי לפני האיפוס.\n\nהאם להמשיך?"
        : "איפוס המערכת ימחק את כל ההגדרות, ההודעות, המוזיקה והנתונים השמורים.\n\nמומלץ מאוד לבצע גיבוי לפני האיפוס.\n\nהאם להמשיך?",
      { title: "איפוס מערכת — אזהרה", kind: "warning" },
    );
    if (!firstConfirm) {
      return;
    }

    const secondConfirm = await ask(
      keepMusic
        ? "זו פעולה בלתי הפיכה ליתר נתוני המערכת.\n\nאחרי האיפוס לא ניתן לשחזר הגדרות והודעות אלא מקובץ גיבוי.\nתיקיות המוזיקה יישארו במערכת.\n\nלאשר סופית את האיפוס?"
        : "זו פעולה בלתי הפיכה.\n\nאחרי האיפוס לא ניתן לשחזר את הנתונים אלא מקובץ גיבוי.\n\nלאשר סופית את מחיקת כל נתוני המערכת?",
      { title: "איפוס מערכת — אישור סופי", kind: "warning" },
    );
    if (!secondConfirm) {
      return;
    }

    setIsResetting(true);
    try {
      await resetSystem(keepMusic);
      setMessage(
        keepMusic
          ? "המערכת אופסה בהצלחה (תיקיות המוזיקה נשמרו). מומלץ להפעיל מחדש את האפליקציה."
          : "המערכת אופסה בהצלחה. מומלץ להפעיל מחדש את האפליקציה.",
      );
    } catch (err) {
      setError(errMsg(err, "שגיאה באיפוס המערכת."));
    } finally {
      setIsResetting(false);
    }
  }

  return (
    <div className="flex flex-col rounded-lg border border-teal-100 bg-white p-4 shadow-sm">
      <div className="flex flex-col gap-4">
        <section className="rounded-lg border border-teal-100 bg-teal-50/40 p-3">
          <h3 className="text-sm font-semibold text-teal-900">בדיקת שמע</h3>
          <label className="mt-2 block text-xs font-medium text-teal-700">
            ערוץ
            <select
              value={testChannel}
              disabled={busy}
              onChange={(event) =>
                setTestChannel(event.target.value as VolumeChannelId)
              }
              className="mt-1 w-full rounded-lg border border-teal-200 bg-white px-2 py-1.5 text-sm text-teal-900 outline-none focus:border-teal-500 focus:ring-1 focus:ring-teal-400 disabled:opacity-60"
            >
              {VOLUME_CHANNELS.map((channel) => (
                <option key={channel.id} value={channel.id}>
                  {channel.shortLabel}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            disabled={busy}
            onClick={() => void handleTestAudio()}
            className="mt-2 w-full rounded-lg bg-teal-700 px-3 py-2 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {testing ? "משמיע..." : "בדיקת שמע"}
          </button>
        </section>

        <section className="rounded-lg border border-teal-100 bg-teal-50/40 p-3">
          <h3 className="text-sm font-semibold text-teal-900">גיבוי</h3>
          <div className="mt-2 flex flex-col gap-2">
            <button
              type="button"
              disabled={busy}
              onClick={() => void handleCreateBackup()}
              className="w-full rounded-lg bg-teal-700 px-3 py-2 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {isExporting ? "יוצר גיבוי..." : "יצירת גיבוי"}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => void handleRestoreBackup()}
              className="w-full rounded-lg border border-teal-300 bg-white px-3 py-2 text-sm font-semibold text-teal-800 hover:bg-teal-50 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {isRestoring ? "משחזר..." : "שחזור גיבוי"}
            </button>
          </div>
        </section>

        <section className="rounded-lg border border-red-100 bg-red-50/30 p-3">
          <h3 className="text-sm font-semibold text-red-800">איפוס</h3>
          <label className="mt-2 flex cursor-pointer items-center gap-2 text-sm text-teal-800">
            <input
              type="checkbox"
              checked={keepMusic}
              disabled={busy}
              onChange={(event) => setKeepMusic(event.target.checked)}
              className="h-4 w-4 rounded border-teal-300 text-teal-700 focus:ring-teal-500"
            />
            השארת תיקיות המוזיקה
          </label>
          <button
            type="button"
            disabled={busy}
            onClick={() => void handleResetSystem()}
            className="mt-2 w-full rounded-lg bg-red-600 px-3 py-2 text-sm font-semibold text-white hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {isResetting ? "מאפס..." : "איפוס המערכת"}
          </button>
        </section>
      </div>

      {message && (
        <p className="mt-3 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          {message}
        </p>
      )}
      {error && (
        <p className="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}
    </div>
  );
}
