import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import SettingsPage from "./components/SettingsPage";
import EmergencyAlertStack from "./components/EmergencyAlertStack";
import AppPasswordGate from "./components/AppPasswordGate";
import { SettingsAuthProvider } from "./components/SettingsAuthProvider";
import { SystemActivityProvider } from "./components/SystemActivityProvider";
import { hasAppPassword } from "./api";

async function ensureNotificationPermission() {
  try {
    let granted = await isPermissionGranted();
    if (!granted) {
      granted = (await requestPermission()) === "granted";
    }
    return granted;
  } catch {
    return false;
  }
}

function App() {
  const [unlocked, setUnlocked] = useState(false);
  const [passwordEnabled, setPasswordEnabled] = useState(false);
  const [ready, setReady] = useState(false);

  const refreshPasswordStatus = useCallback(async () => {
    try {
      const enabled = await hasAppPassword();
      setPasswordEnabled(enabled);
      if (!enabled) {
        setUnlocked(true);
      }
    } catch {
      // Fail closed: if we cannot determine whether a password is set, keep the
      // app locked rather than granting access.
      setPasswordEnabled(true);
      setUnlocked(false);
    } finally {
      setReady(true);
    }
  }, []);

  useEffect(() => {
    void refreshPasswordStatus();
    void ensureNotificationPermission();
  }, [refreshPasswordStatus]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listen("app-lock-required", () => {
      void hasAppPassword()
        .then((enabled) => {
          if (cancelled) {
            return;
          }
          setPasswordEnabled(enabled);
          setUnlocked(!enabled);
        })
        .catch(() => {
          if (!cancelled) {
            setUnlocked(true);
          }
        });
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  if (!ready) {
    return (
      <div className="app-shell flex items-center justify-center">
        <p className="text-sm text-teal-700">טוען...</p>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <EmergencyAlertStack />
      {passwordEnabled && !unlocked ? (
        <AppPasswordGate
          onUnlocked={() => {
            setUnlocked(true);
            void refreshPasswordStatus();
          }}
        />
      ) : (
        <SettingsAuthProvider>
          <SystemActivityProvider>
            <SettingsPage />
          </SystemActivityProvider>
        </SettingsAuthProvider>
      )}
    </div>
  );
}

export default App;
