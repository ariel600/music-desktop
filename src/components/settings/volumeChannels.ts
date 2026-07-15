import type { AudioVolumeChannel, AudioVolumes } from "../../api";

export type VolumeChannelId = Exclude<AudioVolumeChannel, "general">;

export const DEFAULT_VOLUMES: AudioVolumes = {
  general: 1,
  music: 0.2,
  system: 0.3,
  emergency: 0.25,
};

export const VOLUME_CHANNELS: {
  id: VolumeChannelId;
  label: string;
  shortLabel: string;
}[] = [
  { id: "system", label: "עוצמת שמע להודעות מערכת", shortLabel: "הודעות מערכת" },
  { id: "emergency", label: "עוצמת שמע להודעות חירום", shortLabel: "הודעות חירום" },
  { id: "music", label: "עוצמת שמע למוזיקה", shortLabel: "מוזיקה" },
];
