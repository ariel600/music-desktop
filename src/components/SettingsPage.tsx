import { useState } from "react";
import EmergencyMessagesPage from "./EmergencyMessagesPage";
import MusicPage from "./MusicPage";
import OverviewPage from "./OverviewPage";
import SchedulesPage from "./SchedulesPage";
import SystemMessagesPage from "./SystemMessagesPage";
import SettingsTabsPage from "./settings/SettingsTabsPage";
import Sidebar, { type AppView } from "./layout/Sidebar";
import StatusBar from "./layout/StatusBar";
import TitleBar from "./layout/TitleBar";
import { useSystemActivity } from "./SystemActivityProvider";

const SETTINGS_LOCK_HINT = "לא ניתן לערוך הגדרות כשהמערכת כבויה";

export default function SettingsPage() {
  const [activeView, setActiveView] = useState<AppView>("overview");
  const { active } = useSystemActivity();
  const lockEditing = !active && activeView !== "overview";

  function renderContent() {
    switch (activeView) {
      case "overview":
        return <OverviewPage />;
      case "music":
        return <MusicPage />;
      case "schedules":
        return <SchedulesPage />;
      case "emergency-messages":
        return <EmergencyMessagesPage />;
      case "system-messages":
        return <SystemMessagesPage />;
      case "settings":
        return <SettingsTabsPage />;
    }
  }

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-[#e8f3f2]">
      <TitleBar />

      <div className="flex min-h-0 min-w-0 flex-1" dir="ltr">
        <Sidebar activeView={activeView} onViewChange={setActiveView} />

        <main
          className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden p-3 sm:p-4"
          dir="rtl"
        >
          <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg bg-white shadow-md">
            <div className="flex min-h-0 flex-1 flex-col overflow-hidden p-4 sm:p-5">
              {renderContent()}
            </div>

            {lockEditing ? (
              <div
                className="absolute inset-0 z-30 cursor-not-allowed bg-slate-200/25"
                title={SETTINGS_LOCK_HINT}
                aria-label={SETTINGS_LOCK_HINT}
                onClick={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                }}
                onContextMenu={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                }}
              />
            ) : null}
          </div>
        </main>
      </div>

      <StatusBar />
    </div>
  );
}
