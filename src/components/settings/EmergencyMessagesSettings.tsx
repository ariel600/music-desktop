import { useCallback, useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  getEmergencyMessageAudioFiles,
  getEmergencyMessageSettings,
  importEmergencyMessageAudio,
  setEmergencyMessageEnabled,
} from "../../api";
import {
  EMERGENCY_MESSAGE_TYPES,
  type EmergencyAlertCategoryId,
  type EmergencyMessageType,
} from "../../lib/emergencyAlertTypes";
import ToggleSwitch from "../ui/ToggleSwitch";

const NO_AUDIO_TOOLTIP = "אין אפשרות להפעיל את ההודעה בלי קובץ שמע";
const NO_AUDIO_LABEL = "אין קובץ להשמעה";

import { AUDIO_FILTERS } from "../../lib/audioFilters";
import { errMsg } from "../../lib/errors";

function MessageTypeIcon({
  typeId,
  className,
}: {
  typeId: EmergencyAlertCategoryId;
  className?: string;
}) {
  switch (typeId) {
    case "pre-alert":
      return (
        <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M12 9v4M12 17h.01" />
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
        </svg>
      );
    case "red-alert":
      return (
        <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="9" />
          <path d="M12 7v6M12 17h.01" />
        </svg>
      );
    case "hostile-aircraft":
      return (
        <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z" />
        </svg>
      );
    case "end":
      return (
        <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M20 6 9 17l-5-5" />
        </svg>
      );
    case "unconfigured":
      return (
        <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="9" />
          <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3M12 17h.01" />
        </svg>
      );
  }
}

function RowActionsMenu({
  hasFile,
  enabled,
  disabled,
  requiresAudioToEnable,
  onSelectFile,
  onToggleEnabled,
}: {
  hasFile: boolean;
  enabled: boolean;
  disabled?: boolean;
  requiresAudioToEnable: boolean;
  onSelectFile: () => void;
  onToggleEnabled: () => void;
}) {
  const canEnable = hasFile || !requiresAudioToEnable;
  const [open, setOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    function handleClickOutside(event: MouseEvent) {
      if (!menuRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open]);

  return (
    <div ref={menuRef} className="relative shrink-0">
      <button
        type="button"
        disabled={disabled}
        onClick={() => setOpen((current) => !current)}
        className="flex h-9 w-9 items-center justify-center rounded-lg text-teal-700 transition-colors hover:bg-white/80 disabled:cursor-not-allowed disabled:opacity-50"
        aria-label="אפשרויות"
        aria-expanded={open}
      >
        <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor" aria-hidden>
          <circle cx="12" cy="5" r="1.8" />
          <circle cx="12" cy="12" r="1.8" />
          <circle cx="12" cy="19" r="1.8" />
        </svg>
      </button>

      {open && (
        <div className="absolute left-0 top-full z-20 mt-1 min-w-[11rem] rounded-lg border border-teal-200 bg-white py-1 shadow-lg">
          {enabled ? (
            <button
              type="button"
              className="w-full px-3 py-2 text-right text-sm text-teal-900 transition-colors hover:bg-teal-50"
              onClick={() => {
                setOpen(false);
                onToggleEnabled();
              }}
            >
              כיבוי
            </button>
          ) : (
            <span
              className="block w-full"
              title={!canEnable ? NO_AUDIO_TOOLTIP : undefined}
            >
              <button
                type="button"
                disabled={!canEnable}
                className="w-full px-3 py-2 text-right text-sm text-teal-900 transition-colors hover:bg-teal-50 disabled:cursor-not-allowed disabled:text-teal-400"
                onClick={() => {
                  if (!canEnable) {
                    return;
                  }
                  setOpen(false);
                  onToggleEnabled();
                }}
              >
                הפעלה
              </button>
            </span>
          )}
          <button
            type="button"
            className="w-full px-3 py-2 text-right text-sm text-teal-900 transition-colors hover:bg-teal-50"
            onClick={() => {
              setOpen(false);
              onSelectFile();
            }}
          >
            {hasFile ? "שינוי קובץ שמע" : "הוספת קובץ שמע"}
          </button>
        </div>
      )}
    </div>
  );
}

function MessageTypeRow({
  messageType,
  enabled,
  audioName,
  saving,
  importing,
  onToggle,
  onSelectFile,
}: {
  messageType: EmergencyMessageType;
  enabled: boolean;
  audioName: string | null;
  saving: boolean;
  importing: boolean;
  onToggle: (enabled: boolean) => void;
  onSelectFile: () => void;
}) {
  const hasFile = audioName != null;
  const busy = saving || importing;
  const isUnconfigured = messageType.id === "unconfigured";

  return (
    <li
      className={`flex items-center gap-4 rounded-xl border ${messageType.accent.border} ${messageType.accent.bg} px-4 py-4`}
    >
      <div
        className={`flex h-11 w-11 shrink-0 items-center justify-center rounded-lg bg-white shadow-sm ${messageType.accent.icon}`}
      >
        <MessageTypeIcon typeId={messageType.id} className="h-5 w-5" />
      </div>

      <div className="min-w-0 flex-1">
        <h3 className="text-base font-bold text-teal-950">
          {messageType.label}
          {isUnconfigured && (
            <span className="mr-2 text-sm font-normal text-teal-700">
              {enabled
                ? "(תוצג חלונית על המסך עם פרטי ההודעה)"
                : "(בהפעלה תוצג חלונית על המסך עם פרטי ההודעה)"}
            </span>
          )}
        </h3>
        <p className="mt-0.5 text-sm text-teal-700">{messageType.description}</p>
        <p
          className={`mt-2 truncate text-sm ${
            audioName ? "font-medium text-teal-900" : "text-teal-400"
          }`}
          title={audioName ?? undefined}
        >
          {audioName ?? NO_AUDIO_LABEL}
        </p>
      </div>

      <div
        title={
          !hasFile && !enabled && messageType.id !== "unconfigured"
            ? NO_AUDIO_TOOLTIP
            : undefined
        }
        className="shrink-0"
      >
        <ToggleSwitch
          id={`emergency-${messageType.id}`}
          checked={enabled}
          readOnly
        />
      </div>

      <RowActionsMenu
        hasFile={hasFile}
        enabled={enabled}
        disabled={busy}
        requiresAudioToEnable={!isUnconfigured}
        onSelectFile={onSelectFile}
        onToggleEnabled={() => onToggle(!enabled)}
      />
    </li>
  );
}

export default function EmergencyMessagesSettings() {
  const [enabledByType, setEnabledByType] = useState<Record<string, boolean>>({});
  const [audioByType, setAudioByType] = useState<
    Record<string, { name: string | null; path: string | null }>
  >({});
  const [savingType, setSavingType] = useState<string | null>(null);
  const [importingType, setImportingType] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadSettings = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [settings, audioFiles] = await Promise.all([
        getEmergencyMessageSettings(),
        getEmergencyMessageAudioFiles(),
      ]);
      setEnabledByType(settings);
      setAudioByType(
        Object.fromEntries(
          audioFiles.map((file) => [
            file.message_type,
            { name: file.name ?? null, path: file.path ?? null },
          ]),
        ),
      );
    } catch (err) {
      setError(
        errMsg(err, "שגיאה בטעינת הגדרות הודעות חירום."),
      );
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  async function handleToggle(messageType: EmergencyMessageType, enabled: boolean) {
    const hasAudio = Boolean(audioByType[messageType.id]?.path);
    if (enabled && !hasAudio && messageType.id !== "unconfigured") {
      setError(NO_AUDIO_TOOLTIP);
      return;
    }

    setSavingType(messageType.id);
    setError(null);

    const previous = enabledByType[messageType.id] ?? false;
    setEnabledByType((current) => ({ ...current, [messageType.id]: enabled }));

    try {
      const settings = await setEmergencyMessageEnabled(messageType.id, enabled);
      setEnabledByType(settings);
    } catch (err) {
      setEnabledByType((current) => ({ ...current, [messageType.id]: previous }));
      setError(errMsg(err, "שגיאה בשמירת ההגדרה."));
    } finally {
      setSavingType(null);
    }
  }

  async function handleSelectFile(messageType: EmergencyMessageType) {
    setError(null);

    const selected = await open({
      multiple: false,
      filters: AUDIO_FILTERS,
    });

    if (typeof selected !== "string") {
      return;
    }

    setImportingType(messageType.id);

    try {
      const imported = await importEmergencyMessageAudio(messageType.id, selected);
      setAudioByType((current) => ({
        ...current,
        [messageType.id]: {
          name: imported.name ?? null,
          path: imported.path ?? null,
        },
      }));
    } catch (err) {
      setError(errMsg(err, "שגיאה בייבוא קובץ השמע."));
    } finally {
      setImportingType(null);
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      {error && (
        <p className="shrink-0 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}

      {isLoading ? (
        <p className="py-8 text-center text-sm text-teal-600">טוען הגדרות...</p>
      ) : (
        <ul className="min-h-0 flex-1 space-y-3 overflow-y-auto pb-1">
          {EMERGENCY_MESSAGE_TYPES.map((messageType) => {
            const audio = audioByType[messageType.id] ?? { name: null, path: null };

            return (
              <MessageTypeRow
                key={messageType.id}
                messageType={messageType}
                enabled={enabledByType[messageType.id] ?? false}
                audioName={audio.path ? audio.name : null}
                saving={savingType === messageType.id}
                importing={importingType === messageType.id}
                onToggle={(enabled) => void handleToggle(messageType, enabled)}
                onSelectFile={() => void handleSelectFile(messageType)}
              />
            );
          })}
        </ul>
      )}
    </div>
  );
}
