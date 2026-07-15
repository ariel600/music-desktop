export interface MusicFolder {
  label: string;
  icon: string;
}

export const MUSIC_FOLDERS: MusicFolder[] = [
  { label: "כללי", icon: "general" },
  { label: "ווקאלי כללי", icon: "general-vocal" },
  { label: "שבת", icon: "shabbat" },
  { label: "ראש השנה", icon: "rosh-hashana" },
  { label: "סוכות", icon: "sukkot" },
  { label: "חנוכה", icon: "chanukah" },
  { label: "טו בשבט", icon: "tu-bishvat" },
  { label: "פורים", icon: "purim" },
  { label: "פסח", icon: "pesach" },
  { label: "ספירת העומר", icon: "sefirat-haomer" },
  { label: 'ל"ג בעומר ווקאלי', icon: "lag-baomer-vocal" },
  { label: 'ל"ג בעומר', icon: "lag-baomer" },
  { label: "שבועות", icon: "shavuot" },
  { label: "בין המיצרים", icon: "bein-hametzarim" },
];

export function musicIconPath(icon: string): string {
  return `/music-icons/${icon}.ico`;
}

const VOCAL_ONLY_WARNING_FOLDERS = new Set([
  "general-vocal",
  "sefirat-haomer",
  "lag-baomer-vocal",
  "bein-hametzarim",
]);

export function requiresVocalOnlyWarning(folderIcon: string): boolean {
  return VOCAL_ONLY_WARNING_FOLDERS.has(folderIcon);
}
