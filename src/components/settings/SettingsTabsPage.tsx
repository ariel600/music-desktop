import { useState } from "react";
import HolidaysSettings from "./HolidaysSettings";
import LogsSettings from "./LogsSettings";
import OperatingHoursSettings from "./OperatingHoursSettings";
import SystemSettings from "./SystemSettings";

type SettingsTab = "system" | "operating-hours" | "holidays" | "logs";

const SETTINGS_TABS: { id: SettingsTab; label: string }[] = [
  { id: "system", label: "הגדרות מערכת" },
  { id: "operating-hours", label: "הגדרת שעות פעילות" },
  { id: "holidays", label: "הגדרת חגים" },
  { id: "logs", label: "לוגים" },
];

export default function SettingsTabsPage() {
  const [activeTab, setActiveTab] = useState<SettingsTab>("system");

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <div className="flex shrink-0 flex-wrap rounded-lg border border-teal-200 bg-teal-50 p-0.5">
        {SETTINGS_TABS.map((tab) => (
          <button
            key={tab.id}
            type="button"
            onClick={() => setActiveTab(tab.id)}
            className={`rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? "bg-teal-700 text-white shadow-sm"
                : "text-teal-700 hover:bg-teal-100"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        {activeTab === "system" && <SystemSettings />}
        {activeTab === "operating-hours" && <OperatingHoursSettings />}
        {activeTab === "holidays" && <HolidaysSettings />}
        {activeTab === "logs" && <LogsSettings />}
      </div>
    </div>
  );
}
