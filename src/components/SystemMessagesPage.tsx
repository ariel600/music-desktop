import { useCallback, useDeferredValue, useEffect, useMemo, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  deleteSystemMessage,
  getSystemMessages,
  setSystemMessageEnabled,
  updateSystemMessageAudio,
} from "../api";
import { formatSystemMessageSchedule } from "../lib/systemMessageSchedule";
import { errMsg } from "../lib/errors";
import type { SystemMessage } from "../types";
import AddSystemMessageDialog from "./AddSystemMessageDialog";
import DeleteSystemMessageConfirmDialog from "./DeleteSystemMessageConfirmDialog";
import EditSystemMessageDialog from "./EditSystemMessageDialog";
import ToggleSwitch from "./ui/ToggleSwitch";

import { AUDIO_FILTERS } from "../lib/audioFilters";

function RowActionsMenu({
  enabled,
  disabled,
  onEdit,
  onToggleEnabled,
  onChangeFile,
  onDelete,
}: {
  enabled: boolean;
  disabled?: boolean;
  onEdit: () => void;
  onToggleEnabled: () => void;
  onChangeFile: () => void;
  onDelete: () => void;
}) {
  const [openMenu, setOpenMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!openMenu) {
      return;
    }

    function handleClickOutside(event: MouseEvent) {
      if (!menuRef.current?.contains(event.target as Node)) {
        setOpenMenu(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [openMenu]);

  return (
    <div ref={menuRef} className="relative shrink-0">
      <button
        type="button"
        disabled={disabled}
        onClick={() => setOpenMenu((current) => !current)}
        className="flex h-9 w-9 items-center justify-center rounded-lg text-teal-700 transition-colors hover:bg-white/80 disabled:cursor-not-allowed disabled:opacity-50"
        aria-label="אפשרויות"
        aria-expanded={openMenu}
      >
        <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor" aria-hidden>
          <circle cx="12" cy="5" r="1.8" />
          <circle cx="12" cy="12" r="1.8" />
          <circle cx="12" cy="19" r="1.8" />
        </svg>
      </button>

      {openMenu && (
        <div className="absolute left-0 top-full z-20 mt-1 min-w-[11rem] rounded-lg border border-teal-200 bg-white py-1 shadow-lg">
          <button
            type="button"
            className="w-full px-3 py-2 text-right text-sm text-teal-900 transition-colors hover:bg-teal-50"
            onClick={() => {
              setOpenMenu(false);
              onEdit();
            }}
          >
            עריכה
          </button>
          <button
            type="button"
            className="w-full px-3 py-2 text-right text-sm text-teal-900 transition-colors hover:bg-teal-50"
            onClick={() => {
              setOpenMenu(false);
              onToggleEnabled();
            }}
          >
            {enabled ? "כיבוי" : "הפעלה"}
          </button>
          <button
            type="button"
            className="w-full px-3 py-2 text-right text-sm text-teal-900 transition-colors hover:bg-teal-50"
            onClick={() => {
              setOpenMenu(false);
              onChangeFile();
            }}
          >
            שינוי קובץ שמע
          </button>
          <button
            type="button"
            className="w-full px-3 py-2 text-right text-sm text-red-700 transition-colors hover:bg-red-50"
            onClick={() => {
              setOpenMenu(false);
              onDelete();
            }}
          >
            מחיקה
          </button>
        </div>
      )}
    </div>
  );
}

interface SystemMessageListItem {
  message: SystemMessage;
  scheduleText: string;
  searchText: string;
}

function buildSystemMessageListItem(message: SystemMessage): SystemMessageListItem {
  const scheduleText = formatSystemMessageSchedule(message);
  const searchText = [
    message.title,
    message.audio_name ?? "",
    message.file_path,
    scheduleText,
    message.scheduled_time ?? "",
    message.schedule_mode === "fixed_time" ? "שעה קבועה" : "שעות פעילות",
    message.is_active ? "פעיל" : "כבוי",
  ]
    .join(" ")
    .toLowerCase();

  return { message, scheduleText, searchText };
}

export default function SystemMessagesPage() {
  const [messages, setMessages] = useState<SystemMessage[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [busyId, setBusyId] = useState<number | null>(null);
  const [messageToDelete, setMessageToDelete] = useState<SystemMessage | null>(null);
  const [messageToEdit, setMessageToEdit] = useState<SystemMessage | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [search, setSearch] = useState("");
  const deferredSearch = useDeferredValue(search);

  const loadMessages = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await getSystemMessages();
      setMessages(data);
    } catch (err) {
      setError(
        errMsg(err, "שגיאה בטעינת הודעות המערכת."),
      );
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadMessages();
  }, [loadMessages]);

  async function handleToggle(message: SystemMessage) {
    const nextEnabled = !message.is_active;
    setBusyId(message.id);
    setError(null);

    const previous = message.is_active;
    setMessages((current) =>
      current.map((item) =>
        item.id === message.id ? { ...item, is_active: nextEnabled } : item,
      ),
    );

    try {
      const updated = await setSystemMessageEnabled(message.id, nextEnabled);
      setMessages((current) =>
        current.map((item) => (item.id === updated.id ? updated : item)),
      );
    } catch (err) {
      setMessages((current) =>
        current.map((item) =>
          item.id === message.id ? { ...item, is_active: previous } : item,
        ),
      );
      setError(errMsg(err, "שגיאה בשמירת ההגדרה."));
    } finally {
      setBusyId(null);
    }
  }

  async function handleChangeFile(message: SystemMessage) {
    setError(null);

    const selected = await open({
      multiple: false,
      filters: AUDIO_FILTERS,
    });

    if (typeof selected !== "string") {
      return;
    }

    setBusyId(message.id);
    try {
      const updated = await updateSystemMessageAudio(message.id, selected);
      setMessages((current) =>
        current.map((item) => (item.id === updated.id ? updated : item)),
      );
    } catch (err) {
      setError(errMsg(err, "שגיאה בשינוי קובץ השמע."));
    } finally {
      setBusyId(null);
    }
  }

  async function handleConfirmDelete() {
    if (!messageToDelete) {
      return;
    }

    setIsDeleting(true);
    setError(null);
    try {
      await deleteSystemMessage(messageToDelete.id);
      setMessages((current) =>
        current.filter((item) => item.id !== messageToDelete.id),
      );
      setMessageToDelete(null);
    } catch (err) {
      setError(errMsg(err, "שגיאה במחיקת ההודעה."));
    } finally {
      setIsDeleting(false);
    }
  }

  const messageItems = useMemo(
    () => messages.map(buildSystemMessageListItem),
    [messages],
  );

  const filteredMessages = useMemo(() => {
    const query = deferredSearch.trim().toLowerCase();
    if (!query) {
      return messageItems;
    }
    return messageItems.filter((item) => item.searchText.includes(query));
  }, [messageItems, deferredSearch]);

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <div className="flex shrink-0 flex-wrap items-center gap-3">
        <button
          type="button"
          onClick={() => setShowAddDialog(true)}
          className="rounded-lg bg-teal-700 px-3 py-1.5 text-sm font-medium text-white hover:bg-teal-800"
        >
          + הוספת הודעת מערכת
        </button>
        <input
          type="search"
          placeholder="חיפוש הודעות מערכת..."
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          className="min-w-[12rem] flex-1 rounded-lg border border-teal-200 bg-white px-3 py-1.5 text-sm text-teal-900 shadow-sm focus:border-teal-500 focus:outline-none focus:ring-1 focus:ring-teal-500"
        />
        {messageItems.length > 0 && (
          <span className="text-sm text-teal-600">
            {deferredSearch.trim()
              ? `${filteredMessages.length} מתוך ${messageItems.length}`
              : `${messageItems.length} ${messageItems.length === 1 ? "הודעה" : "הודעות"}`}
          </span>
        )}
      </div>

      {error && (
        <p className="shrink-0 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}

      {isLoading ? (
        <p className="py-8 text-center text-sm text-teal-600">טוען הודעות...</p>
      ) : messageItems.length === 0 ? (
        <div className="flex min-h-0 flex-1 flex-col items-center justify-center rounded-lg border border-dashed border-teal-200 bg-teal-50/40 px-6 py-12 text-center">
          <p className="text-sm font-medium text-teal-900">אין הודעות מערכת</p>
          <p className="mt-1 text-sm text-teal-600">
            לחץ על &quot;הוספת הודעת מערכת&quot; כדי ליצור הודעה חדשה.
          </p>
        </div>
      ) : filteredMessages.length === 0 ? (
        <p className="py-8 text-center text-sm text-teal-600">
          לא נמצאו הודעות התואמות לחיפוש.
        </p>
      ) : (
        <ul className="min-h-0 flex-1 space-y-2 overflow-y-auto">
          {filteredMessages.map((item) => {
            const { message, scheduleText } = item;
            const busy = busyId === message.id;

            return (
              <li
                key={message.id}
                className="flex items-center gap-3 rounded-lg border border-teal-100 bg-teal-50/40 px-4 py-3"
              >
                <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-teal-100 text-teal-700">
                  <svg
                    viewBox="0 0 24 24"
                    className="h-4 w-4"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    aria-hidden
                  >
                    <path d="M9 11l3 3L22 4" />
                    <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
                  </svg>
                </span>
                <div className="min-w-0 flex-1">
                  <p className="truncate font-medium text-teal-900">{message.title}</p>
                  <p className="truncate text-sm text-teal-600">
                    {message.audio_name ?? "קובץ שמע"}
                  </p>
                  <p className="truncate text-xs text-teal-500">{scheduleText}</p>
                </div>

                <ToggleSwitch
                  id={`system-message-${message.id}`}
                  checked={message.is_active}
                  readOnly
                />

                <RowActionsMenu
                  enabled={message.is_active}
                  disabled={busy || isDeleting}
                  onEdit={() => setMessageToEdit(message)}
                  onToggleEnabled={() => void handleToggle(message)}
                  onChangeFile={() => void handleChangeFile(message)}
                  onDelete={() => setMessageToDelete(message)}
                />
              </li>
            );
          })}
        </ul>
      )}

      {showAddDialog && (
        <AddSystemMessageDialog
          onClose={() => setShowAddDialog(false)}
          onSaved={() => void loadMessages()}
        />
      )}

      {messageToEdit && (
        <EditSystemMessageDialog
          message={messageToEdit}
          onClose={() => setMessageToEdit(null)}
          onSaved={(updated) => {
            setMessages((current) =>
              current.map((item) => (item.id === updated.id ? updated : item)),
            );
          }}
        />
      )}

      {messageToDelete && (
        <DeleteSystemMessageConfirmDialog
          title={messageToDelete.title}
          isDeleting={isDeleting}
          onCancel={() => {
            if (!isDeleting) {
              setMessageToDelete(null);
            }
          }}
          onConfirm={() => void handleConfirmDelete()}
        />
      )}
    </div>
  );
}
