import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { getSystemActive, setSystemActive as setSystemActiveApi } from "../api";

interface SystemActivityContextValue {
  active: boolean;
  ready: boolean;
  setActive: (active: boolean) => Promise<void>;
}

const SystemActivityContext = createContext<SystemActivityContextValue | null>(
  null,
);

export function SystemActivityProvider({ children }: { children: ReactNode }) {
  const [active, setActiveState] = useState(true);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void getSystemActive()
      .then((value) => {
        if (!cancelled) {
          setActiveState(value);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setActiveState(true);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setReady(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const setActive = useCallback(async (next: boolean) => {
    const saved = await setSystemActiveApi(next);
    setActiveState(saved);
  }, []);

  const value = useMemo(
    () => ({ active, ready, setActive }),
    [active, ready, setActive],
  );

  return (
    <SystemActivityContext.Provider value={value}>
      {children}
    </SystemActivityContext.Provider>
  );
}

export function useSystemActivity() {
  const ctx = useContext(SystemActivityContext);
  if (!ctx) {
    throw new Error("useSystemActivity must be used within SystemActivityProvider");
  }
  return ctx;
}
