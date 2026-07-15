import { useCallback, useEffect, useState } from "react";
import AddMusicDialog from "./AddMusicDialog";
import DeleteMusicConfirmDialog from "./DeleteMusicConfirmDialog";
import VocalOnlyWarningDialog from "./VocalOnlyWarningDialog";
import { deleteMusicFiles, listMusicFiles, playAudio } from "../api";
import type { MusicFileEntry } from "../types";
import { formatFileSize } from "../lib/formatFileSize";
import { errMsg } from "../lib/errors";
import {
  MUSIC_FOLDERS,
  musicIconPath,
  requiresVocalOnlyWarning,
  type MusicFolder,
} from "../lib/musicFolders";

function sortMusicFiles(files: MusicFileEntry[]): MusicFileEntry[] {
  return [...files].sort((a, b) => a.name.localeCompare(b.name, "he"));
}

function FolderGrid({ onSelect }: { onSelect: (folder: MusicFolder) => void }) {
  return (
    <div className="grid grid-cols-2 gap-x-6 gap-y-8 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
      {MUSIC_FOLDERS.map((folder) => (
        <button
          key={folder.icon}
          type="button"
          onClick={() => onSelect(folder)}
          className="group flex flex-col items-center gap-2 rounded-md p-3 text-center transition-colors hover:bg-teal-50/80"
        >
          <img
            src={musicIconPath(folder.icon)}
            alt=""
            className="h-24 w-24 object-contain drop-shadow-sm transition-transform group-hover:scale-105 sm:h-28 sm:w-28"
            draggable={false}
          />
          <span className="max-w-[8.5rem] text-sm leading-snug text-teal-900">
            {folder.label}
          </span>
        </button>
      ))}
    </div>
  );
}

function SongList({
  folder,
  onBack,
}: {
  folder: MusicFolder;
  onBack: () => void;
}) {
  const [files, setFiles] = useState<MusicFileEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [playError, setPlayError] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [playingPath, setPlayingPath] = useState<string | null>(null);
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showVocalWarning, setShowVocalWarning] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [vocalWarningAcknowledged, setVocalWarningAcknowledged] = useState(false);

  const selectedCount = selectedPaths.size;

  function handleOpenAddDialog() {
    if (requiresVocalOnlyWarning(folder.icon)) {
      setShowVocalWarning(true);
      return;
    }
    setVocalWarningAcknowledged(true);
    setShowAddDialog(true);
  }

  function handleVocalWarningConfirm() {
    setShowVocalWarning(false);
    setVocalWarningAcknowledged(true);
    setShowAddDialog(true);
  }

  function toggleSelection(path: string) {
    setSelectedPaths((current) => {
      const next = new Set(current);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  }

  const loadFiles = useCallback(async () => {
    setIsLoading(true);
    setLoadError(null);
    try {
      const fileList = await listMusicFiles(folder.icon);
      setFiles(sortMusicFiles(fileList));
      setSelectedPaths((current) => {
        const validPaths = new Set(fileList.map((file) => file.path));
        const next = new Set<string>();
        for (const path of current) {
          if (validPaths.has(path)) {
            next.add(path);
          }
        }
        return next;
      });
    } catch (err) {
      setLoadError(errMsg(err, "שגיאה בטעינת השירים."));
      setFiles([]);
      setSelectedPaths(new Set());
    } finally {
      setIsLoading(false);
    }
  }, [folder.icon]);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setIsLoading(true);
      setLoadError(null);
      try {
        const fileList = await listMusicFiles(folder.icon);
        if (!cancelled) {
          setFiles(sortMusicFiles(fileList));
          setSelectedPaths(new Set());
        }
      } catch (err) {
        if (!cancelled) {
          setLoadError(
            errMsg(err, "שגיאה בטעינת השירים."),
          );
          setFiles([]);
          setSelectedPaths(new Set());
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, [folder.icon]);

  async function handlePlay(file: MusicFileEntry) {
    if (playingPath) {
      return;
    }

    setPlayError(null);
    setPlayingPath(file.path);
    try {
      await playAudio(file.path, null, "music");
    } catch (err) {
      setPlayError(errMsg(err, "שגיאה בהשמעה."));
    } finally {
      setPlayingPath(null);
    }
  }

  async function handleConfirmDelete() {
    const paths = Array.from(selectedPaths);
    if (paths.length === 0) {
      return;
    }

    setIsDeleting(true);
    setDeleteError(null);
    try {
      await deleteMusicFiles(folder.icon, paths);
      setShowDeleteConfirm(false);
      setSelectedPaths(new Set());
      await loadFiles();
    } catch (err) {
      setDeleteError(
        errMsg(err, "שגיאה במחיקת השירים."),
      );
    } finally {
      setIsDeleting(false);
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onBack}
          className="rounded-lg bg-teal-100 px-3 py-1.5 text-sm text-teal-800 transition-colors hover:bg-teal-200"
        >
          ← חזרה
        </button>
        <img
          src={musicIconPath(folder.icon)}
          alt=""
          className="h-10 w-10 object-contain"
        />
        <h3 className="text-lg font-bold text-teal-900">{folder.label}</h3>

        {selectedCount > 0 && (
          <button
            type="button"
            onClick={() => setShowDeleteConfirm(true)}
            disabled={isDeleting}
            className="mr-auto flex items-center gap-1.5 rounded-lg bg-red-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
            title="מחק שירים נבחרים"
          >
            <svg
              viewBox="0 0 24 24"
              className="h-4 w-4"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              aria-hidden
            >
              <path d="M3 6h18M8 6V4h8v2M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
              <path d="M10 11v6M14 11v6" />
            </svg>
            מחק ({selectedCount})
          </button>
        )}

        <button
          type="button"
          onClick={handleOpenAddDialog}
          className={`rounded-lg bg-teal-700 px-3 py-1.5 text-sm font-medium text-white hover:bg-teal-800 ${
            selectedCount > 0 ? "" : "mr-auto"
          }`}
        >
          + הוסף שירים
        </button>
        <span className="text-sm text-teal-600">{files.length} שירים</span>
      </div>

      {loadError && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {loadError}
        </p>
      )}

      {playError && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {playError}
        </p>
      )}

      {deleteError && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {deleteError}
        </p>
      )}

      {isLoading ? (
        <p className="py-8 text-center text-sm text-teal-600">טוען שירים...</p>
      ) : files.length === 0 ? (
        <div className="rounded-lg border border-dashed border-teal-200 bg-teal-50/50 px-4 py-10 text-center">
          <p className="text-sm text-teal-800">אין שירים בתיקייה זו.</p>
          <p className="mt-2 text-xs text-teal-500">
            לחץ על &quot;הוסף שירים&quot; כדי לייבא קבצי מוזיקה.
          </p>
        </div>
      ) : (
        <ul className="min-h-0 flex-1 space-y-2 overflow-y-auto">
          {files.map((file, index) => {
            const isPlaying = playingPath === file.path;
            const isSelected = selectedPaths.has(file.path);
            return (
              <li key={file.path}>
                <div
                  role="button"
                  tabIndex={0}
                  onClick={() => toggleSelection(file.path)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      toggleSelection(file.path);
                    }
                  }}
                  className={`flex w-full cursor-pointer items-center gap-3 rounded-xl border px-4 py-3 text-right transition-colors ${
                    isSelected
                      ? "border-teal-600 bg-teal-50 ring-1 ring-teal-300"
                      : isPlaying
                        ? "border-teal-500 bg-teal-100"
                        : "border-teal-100 bg-white hover:border-teal-200 hover:bg-teal-50/60"
                  }`}
                >
                  <span
                    className={`flex h-5 w-5 shrink-0 items-center justify-center rounded border-2 transition-colors ${
                      isSelected
                        ? "border-teal-600 bg-teal-600 text-white"
                        : "border-teal-300 bg-white"
                    }`}
                    aria-hidden
                  >
                    {isSelected && (
                      <svg
                        viewBox="0 0 24 24"
                        className="h-3.5 w-3.5"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="3"
                      >
                        <path d="M5 13l4 4L19 7" />
                      </svg>
                    )}
                  </span>
                  <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-teal-100 text-sm font-bold text-teal-700">
                    {index + 1}
                  </span>
                  <button
                    type="button"
                    onClick={(event) => {
                      event.stopPropagation();
                      void handlePlay(file);
                    }}
                    disabled={playingPath !== null}
                    className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-teal-700 text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-50"
                    aria-label={`השמע ${file.name}`}
                  >
                    {isPlaying ? (
                      <span className="text-xs">▶</span>
                    ) : (
                      <svg
                        viewBox="0 0 24 24"
                        className="h-4 w-4"
                        fill="currentColor"
                        aria-hidden
                      >
                        <path d="M8 5v14l11-7z" />
                      </svg>
                    )}
                  </button>
                  <span className="min-w-0 flex-1">
                    <span className="block truncate font-medium text-teal-900">
                      {file.name}
                    </span>
                    <span className="text-xs text-teal-500">
                      {formatFileSize(file.size_bytes)}
                    </span>
                  </span>
                </div>
              </li>
            );
          })}
        </ul>
      )}

      {showVocalWarning && (
        <VocalOnlyWarningDialog
          onBack={() => setShowVocalWarning(false)}
          onConfirm={handleVocalWarningConfirm}
        />
      )}

      {showDeleteConfirm && (
        <DeleteMusicConfirmDialog
          count={selectedCount}
          isDeleting={isDeleting}
          onCancel={() => setShowDeleteConfirm(false)}
          onConfirm={() => void handleConfirmDelete()}
        />
      )}

      {showAddDialog && (
        <AddMusicDialog
          folder={folder}
          vocalWarningAcknowledged={vocalWarningAcknowledged}
          onClose={() => {
            setShowAddDialog(false);
            setVocalWarningAcknowledged(false);
          }}
          onSaved={() => void loadFiles()}
        />
      )}
    </div>
  );
}

export default function MusicPage() {
  const [selectedFolder, setSelectedFolder] = useState<MusicFolder | null>(null);

  if (selectedFolder) {
    return (
      <SongList
        folder={selectedFolder}
        onBack={() => setSelectedFolder(null)}
      />
    );
  }

  return <FolderGrid onSelect={setSelectedFolder} />;
}
