import { useCallback, useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { importMusicFiles, scanMusicSources } from "../api";
import type { ScannedMusicFile } from "../types";
import type { MusicFolder } from "../lib/musicFolders";
import { formatFileSize } from "../lib/formatFileSize";

import { AUDIO_VIDEO_FILTERS } from "../lib/audioFilters";
import { errMsg } from "../lib/errors";

interface AddMusicDialogProps {
  folder: MusicFolder;
  vocalWarningAcknowledged: boolean;
  onClose: () => void;
  onSaved: () => void;
}

export default function AddMusicDialog({
  folder,
  vocalWarningAcknowledged,
  onClose,
  onSaved,
}: AddMusicDialogProps) {
  const [pendingFiles, setPendingFiles] = useState<ScannedMusicFile[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const isScanningRef = useRef(false);

  const addSources = useCallback(async (sourcePaths: string[]) => {
    if (sourcePaths.length === 0 || isScanningRef.current) {
      return;
    }

    isScanningRef.current = true;
    setIsScanning(true);
    setError(null);
    try {
      const scanned = await scanMusicSources(sourcePaths);
      if (scanned.length === 0) {
        setError("לא נמצאו קבצי שמע בנתיבים שנבחרו.");
        return;
      }

      setPendingFiles((current) => {
        const existing = new Set(current.map((file) => file.source_path));
        const merged = [...current];
        for (const file of scanned) {
          if (!existing.has(file.source_path)) {
            merged.push(file);
            existing.add(file.source_path);
          }
        }
        return merged.sort((a, b) => a.name.localeCompare(b.name, "he"));
      });
    } catch (err) {
      setError(errMsg(err, "שגיאה בסריקת הקבצים."));
    } finally {
      isScanningRef.current = false;
      setIsScanning(false);
    }
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          setIsDragging(true);
          return;
        }

        if (event.payload.type === "drop") {
          setIsDragging(false);
          void addSources(event.payload.paths);
          return;
        }

        setIsDragging(false);
      })
      .then((dispose) => {
        unlisten = dispose;
      });

    return () => {
      unlisten?.();
    };
  }, [addSources]);

  async function handlePickFromComputer() {
    try {
      const paths: string[] = [];

      const files = await open({
        multiple: true,
        filters: AUDIO_VIDEO_FILTERS,
        title: "בחר קבצים",
      });
      if (files) {
        paths.push(...(Array.isArray(files) ? files : [files]));
      }

      const folders = await open({
        directory: true,
        multiple: true,
        title: "בחר תיקיות",
      });
      if (folders) {
        paths.push(...(Array.isArray(folders) ? folders : [folders]));
      }

      if (paths.length === 0) {
        return;
      }

      await addSources(paths);
    } catch (err) {
      setError(errMsg(err, "שגיאה בבחירה מהמחשב."));
    }
  }

  function removeFile(sourcePath: string) {
    setPendingFiles((current) =>
      current.filter((file) => file.source_path !== sourcePath),
    );
  }

  function handleClose() {
    if (isSaving || isScanning) {
      return;
    }
    onClose();
  }

  async function handleSave() {
    if (pendingFiles.length === 0) {
      return;
    }

    setIsSaving(true);
    setError(null);
    try {
      await importMusicFiles(
        folder.icon,
        pendingFiles.map((file) => file.source_path),
        vocalWarningAcknowledged,
      );
      onSaved();
      onClose();
    } catch (err) {
      setError(errMsg(err, "שגיאה בשמירת השירים."));
    } finally {
      setIsSaving(false);
    }
  }

  const isBusy = isSaving || isScanning;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={handleClose}
    >
      <div
        className="flex max-h-[85vh] w-full max-w-2xl flex-col rounded-xl bg-white shadow-xl"
        dir="rtl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="border-b border-teal-100 px-5 py-4">
          <h3 className="text-lg font-bold text-teal-900">
            הוספת שירים — {folder.label}
          </h3>
        </header>

        <div className="min-h-0 flex-1 overflow-y-auto p-5">
          <div
            className={`mb-4 rounded-xl border-2 border-dashed px-4 py-10 text-center transition-colors ${
              isDragging
                ? "border-teal-500 bg-teal-50"
                : "border-teal-200 bg-teal-50/40"
            }`}
          >
            <p className="text-sm font-medium text-teal-900">
              גרור לכאן שירים או תיקיות מוזיקה
            </p>
            <p className="mt-1 text-xs text-teal-600">
              mp3, wav, ogg, flac, m4a, aac, wma, mp4 (יומר ל-mp3)
            </p>
            <div className="mt-4 flex flex-wrap justify-center gap-2">
              <button
                type="button"
                onClick={() => void handlePickFromComputer()}
                disabled={isBusy}
                className="rounded-lg bg-teal-700 px-3 py-1.5 text-sm text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-50"
              >
                בחר מהמחשב
              </button>
            </div>
          </div>

          {error && (
            <p className="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
              {error}
            </p>
          )}

          {isScanning ? (
            <p className="py-4 text-center text-sm text-teal-600">סורק קבצים...</p>
          ) : pendingFiles.length === 0 ? (
            <p className="py-4 text-center text-sm text-teal-500">
              עדיין לא נוספו שירים לרשימה.
            </p>
          ) : (
            <div className="space-y-2">
              <p className="text-sm font-medium text-teal-800">
                {pendingFiles.length} שירים לייבוא
              </p>
              <ul className="max-h-64 space-y-1 overflow-y-auto rounded-lg border border-teal-100">
                {pendingFiles.map((file) => (
                  <li
                    key={file.source_path}
                    className="flex items-center gap-3 border-b border-teal-50 px-3 py-2 last:border-b-0"
                  >
                    <span className="min-w-0 flex-1 truncate text-sm text-teal-900">
                      {file.name}
                      {file.will_convert_to_mp3 && (
                        <span className="mr-1 text-xs text-amber-600">
                          (יומר ל-MP3)
                        </span>
                      )}
                    </span>
                    <span className="shrink-0 text-xs text-teal-500">
                      {formatFileSize(file.size_bytes)}
                    </span>
                    <button
                      type="button"
                      onClick={() => removeFile(file.source_path)}
                      disabled={isBusy}
                      className="shrink-0 rounded px-2 py-1 text-xs text-red-600 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      הסר
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>

        <footer className="flex justify-end gap-2 border-t border-teal-100 px-5 py-4">
          <button
            type="button"
            onClick={handleClose}
            disabled={isBusy}
            className="rounded-lg bg-teal-100 px-4 py-2 text-sm text-teal-800 hover:bg-teal-200 disabled:cursor-not-allowed disabled:opacity-50"
          >
            ביטול
          </button>
          <button
            type="button"
            onClick={() => void handleSave()}
            disabled={pendingFiles.length === 0 || isBusy}
            className="rounded-lg bg-teal-700 px-4 py-2 text-sm font-medium text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {isSaving ? "שומר..." : "שמירה"}
          </button>
        </footer>
      </div>
    </div>
  );
}
