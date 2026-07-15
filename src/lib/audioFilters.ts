/** File-dialog filter for audio-only pickers (system/emergency messages). */
export const AUDIO_FILTERS = [
  {
    name: "קבצי שמע",
    extensions: ["mp3", "wav", "ogg", "flac", "m4a", "aac", "wma"],
  },
];

/** File-dialog filter for the music library, which also accepts video (mp4). */
export const AUDIO_VIDEO_FILTERS = [
  {
    name: "קבצי שמע ווידאו",
    extensions: ["mp3", "wav", "ogg", "flac", "m4a", "aac", "wma", "mp4"],
  },
];
