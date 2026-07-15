import { useEffect, useRef, useState } from "react";
import EmergencyAreaSelect from "./EmergencyAreaSelect";
import PasswordSettingsPanel from "./PasswordSettingsPanel";
import SystemActionsPanel from "./SystemActionsPanel";
import VolumeSettingsPanel from "./VolumeSettingsPanel";

export default function SystemSettings() {
  const actionsRef = useRef<HTMLDivElement>(null);
  const [actionsHeight, setActionsHeight] = useState<number | null>(null);

  useEffect(() => {
    const element = actionsRef.current;
    if (!element) {
      return;
    }

    const updateHeight = () => {
      setActionsHeight(element.getBoundingClientRect().height);
    };

    updateHeight();
    const observer = new ResizeObserver(updateHeight);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  return (
    <div
      className="flex h-full min-h-0 flex-1 gap-4 overflow-hidden"
      dir="rtl"
    >
      <div className="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-hidden">
        <div className="grid shrink-0 grid-cols-2 items-start gap-4">
          <div
            className="flex min-h-0 min-w-0 flex-col gap-4"
            style={
              actionsHeight != null ? { height: actionsHeight } : undefined
            }
          >
            <div className="min-h-0 min-w-0 flex-1">
              <VolumeSettingsPanel />
            </div>
            <div className="shrink-0">
              <EmergencyAreaSelect />
            </div>
          </div>

          <div
            className="min-h-0 min-w-0"
            style={
              actionsHeight != null ? { height: actionsHeight } : undefined
            }
          >
            <PasswordSettingsPanel />
          </div>
        </div>

        <div className="min-h-20 flex-1" aria-hidden="true" />
      </div>

      <div className="flex w-72 shrink-0 flex-col overflow-hidden">
        <div ref={actionsRef} className="shrink-0">
          <SystemActionsPanel />
        </div>
        <div className="min-h-20 flex-1" aria-hidden="true" />
      </div>
    </div>
  );
}
