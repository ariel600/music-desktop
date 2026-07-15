type UnlockHandler = () => Promise<boolean>;

export type SettingsAuthKind = "settings" | "music";

let unlockHandler: UnlockHandler | null = null;
let settingsUnlocked = false;
let hasSettingsPasswordCache = false;
let lockMusicAddCache = false;

export function registerSettingsUnlockHandler(handler: UnlockHandler | null) {
  unlockHandler = handler;
}

export function markSettingsUnlocked(value: boolean) {
  settingsUnlocked = value;
}

export function setSettingsAuthCache(options: {
  hasSettingsPassword: boolean;
  lockMusicAdd: boolean;
}) {
  hasSettingsPasswordCache = options.hasSettingsPassword;
  lockMusicAddCache = options.lockMusicAdd;
  if (!options.hasSettingsPassword) {
    settingsUnlocked = true;
  }
}

export async function ensureSettingsAuth(
  kind: SettingsAuthKind = "settings",
): Promise<boolean> {
  if (kind === "music" && !lockMusicAddCache) {
    return true;
  }
  if (!hasSettingsPasswordCache) {
    return true;
  }
  if (settingsUnlocked) {
    return true;
  }
  if (!unlockHandler) {
    return false;
  }
  const ok = await unlockHandler();
  if (ok) {
    settingsUnlocked = true;
  }
  return ok;
}
