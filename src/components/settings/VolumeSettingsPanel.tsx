import { useEffect, useState } from "react";
import {
  getAudioVolumes,
  setAudioVolumeChannel,
  type AudioVolumes,
} from "../../api";
import { errMsg } from "../../lib/errors";
import { Slider } from "../ui/slider";
import {
  DEFAULT_VOLUMES,
  VOLUME_CHANNELS,
  type VolumeChannelId,
} from "./volumeChannels";

export default function VolumeSettingsPanel() {
  const [volumes, setVolumesState] = useState<AudioVolumes>(DEFAULT_VOLUMES);
  const [draftPercents, setDraftPercents] = useState<
    Partial<Record<VolumeChannelId, string>>
  >({});
  const [volumeReady, setVolumeReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void getAudioVolumes()
      .then((value) => {
        if (!cancelled) {
          setVolumesState(value);
          setVolumeReady(true);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setVolumeReady(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function saveVolume(channel: VolumeChannelId, nextVolume: number) {
    const clamped = Math.min(1, Math.max(0, nextVolume));
    setVolumesState((current) => ({ ...current, [channel]: clamped }));
    setDraftPercents((current) => {
      const next = { ...current };
      delete next[channel];
      return next;
    });
    setError(null);
    try {
      const saved = await setAudioVolumeChannel(channel, clamped);
      setVolumesState(saved);
    } catch (err) {
      setError(errMsg(err, "שגיאה בשמירת עוצמת השמע."));
    }
  }

  function percentDisplay(channel: VolumeChannelId) {
    return (
      draftPercents[channel] ?? String(Math.round(volumes[channel] * 100))
    );
  }

  function commitPercentInput(channel: VolumeChannelId) {
    const raw = draftPercents[channel];
    if (raw === undefined) {
      return;
    }
    const parsed = Number.parseInt(raw.replace(/%/g, "").trim(), 10);
    if (Number.isNaN(parsed)) {
      setDraftPercents((current) => {
        const next = { ...current };
        delete next[channel];
        return next;
      });
      return;
    }
    void saveVolume(channel, Math.min(100, Math.max(0, parsed)) / 100);
  }

  return (
    <div className="flex h-full min-h-0 flex-col rounded-lg border border-teal-100 bg-white p-4 shadow-sm">
      <h3 className="mb-3 shrink-0 text-sm font-semibold text-teal-900">
        ניהול עוצמת שמע
      </h3>
      <div className="flex min-h-0 flex-1 flex-col gap-3">
        {VOLUME_CHANNELS.map((channel) => (
          <div
            key={channel.id}
            className="flex min-h-0 flex-1 flex-col justify-center rounded-lg border border-teal-100 bg-teal-50/40 px-3 py-3"
          >
            <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
              <p className="text-sm font-medium text-teal-900">{channel.label}</p>
              <label className="flex shrink-0 items-center gap-1 text-sm font-bold text-teal-700">
                <input
                  type="number"
                  min={0}
                  max={100}
                  step={1}
                  dir="ltr"
                  disabled={!volumeReady}
                  value={percentDisplay(channel.id)}
                  onChange={(event) => {
                    setDraftPercents((current) => ({
                      ...current,
                      [channel.id]: event.target.value,
                    }));
                  }}
                  onBlur={() => commitPercentInput(channel.id)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      event.currentTarget.blur();
                    }
                  }}
                  className="w-14 rounded-md border border-teal-200 bg-white px-2 py-1 text-center text-sm font-bold tabular-nums text-teal-800 outline-none focus:border-teal-500 focus:ring-1 focus:ring-teal-400 disabled:opacity-50"
                  aria-label={`${channel.label} באחוזים`}
                />
                <span>%</span>
              </label>
            </div>
            <Slider
              dir="rtl"
              min={0}
              max={1}
              step={0.01}
              value={[volumes[channel.id]]}
              disabled={!volumeReady}
              onValueChange={(values) => {
                const nextVolume = values[0] ?? 0;
                void saveVolume(channel.id, nextVolume);
              }}
              aria-label={channel.label}
            />
          </div>
        ))}
      </div>

      {error && (
        <p className="mt-2 shrink-0 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}
    </div>
  );
}
