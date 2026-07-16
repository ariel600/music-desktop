use rodio::{Decoder, OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

static GENERAL_VOLUME: AtomicU32 = AtomicU32::new(1.0f32.to_bits());
static MUSIC_VOLUME: AtomicU32 = AtomicU32::new(0.20f32.to_bits());
static SYSTEM_VOLUME: AtomicU32 = AtomicU32::new(0.30f32.to_bits());
static EMERGENCY_VOLUME: AtomicU32 = AtomicU32::new(0.25f32.to_bits());
static MUSIC_DUCK_DEPTH: AtomicU32 = AtomicU32::new(0);
static ANNOUNCEMENT_ACTIVE: AtomicU32 = AtomicU32::new(0);
static ANNOUNCEMENT_LOCK: Mutex<()> = Mutex::new(());
static MUSIC_PLAY_LOCK: Mutex<()> = Mutex::new(());
static ACTIVE_MUSIC: Mutex<Option<ActiveMusicControl>> = Mutex::new(None);

pub const MUSIC_DUCK_FACTOR: f32 = 0.7;
const VOLUME_TEST_SOUND: &[u8] = include_bytes!("../resources/volume-test.wav");

#[derive(Debug, Clone)]
pub struct NowPlayingInfo {
    pub title: String,
    pub file_path: String,
    pub folder: Option<String>,
    pub artwork_data_url: Option<String>,
}

struct ActiveMusicControl {
    sink: Sink,
    base_volume: f32,
    info: NowPlayingInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeChannel {
    General,
    Music,
    System,
    Emergency,
}

impl VolumeChannel {
    pub fn parse(value: Option<&str>) -> Self {
        match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some("music") => Self::Music,
            Some("system") => Self::System,
            Some("emergency") => Self::Emergency,
            _ => Self::General,
        }
    }

    fn atomic(self) -> &'static AtomicU32 {
        match self {
            Self::General => &GENERAL_VOLUME,
            Self::Music => &MUSIC_VOLUME,
            Self::System => &SYSTEM_VOLUME,
            Self::Emergency => &EMERGENCY_VOLUME,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVolumes {
    pub general: f32,
    pub music: f32,
    pub system: f32,
    pub emergency: f32,
}

impl Default for AudioVolumes {
    fn default() -> Self {
        Self {
            general: 1.0,
            music: 0.20,
            system: 0.30,
            emergency: 0.25,
        }
    }
}

impl AudioVolumes {
    pub fn clamp_all(mut self) -> Self {
        self.general = self.general.clamp(0.0, 1.0);
        self.music = self.music.clamp(0.0, 1.0);
        self.system = self.system.clamp(0.0, 1.0);
        self.emergency = self.emergency.clamp(0.0, 1.0);
        self
    }

    pub fn with_channel(mut self, channel: VolumeChannel, volume: f32) -> Self {
        let clamped = volume.clamp(0.0, 1.0);
        match channel {
            VolumeChannel::General => self.general = clamped,
            VolumeChannel::Music => self.music = clamped,
            VolumeChannel::System => self.system = clamped,
            VolumeChannel::Emergency => self.emergency = clamped,
        }
        self
    }
}

fn store_bits(slot: &AtomicU32, volume: f32) {
    slot.store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

fn load_bits(slot: &AtomicU32) -> f32 {
    f32::from_bits(slot.load(Ordering::Relaxed)).clamp(0.0, 1.0)
}

pub fn init_volumes(volumes: &AudioVolumes) {
    let volumes = volumes.clone().clamp_all();
    store_bits(&GENERAL_VOLUME, volumes.general);
    store_bits(&MUSIC_VOLUME, volumes.music);
    store_bits(&SYSTEM_VOLUME, volumes.system);
    store_bits(&EMERGENCY_VOLUME, volumes.emergency);
    refresh_active_music_volume();
}

pub fn get_volumes() -> AudioVolumes {
    AudioVolumes {
        general: load_bits(&GENERAL_VOLUME),
        music: load_bits(&MUSIC_VOLUME),
        system: load_bits(&SYSTEM_VOLUME),
        emergency: load_bits(&EMERGENCY_VOLUME),
    }
}

pub fn get_channel_volume(channel: VolumeChannel) -> f32 {
    load_bits(channel.atomic())
}

pub fn get_volume() -> f32 {
    get_channel_volume(VolumeChannel::General)
}

pub fn resolve_volume(task_volume: Option<f32>) -> f32 {
    task_volume
        .map(|volume| volume.clamp(0.0, 1.0))
        .unwrap_or_else(get_volume)
}

pub fn resolve_playback_volume(channel: VolumeChannel, override_volume: Option<f32>) -> f32 {
    override_volume
        .map(|volume| volume.clamp(0.0, 1.0))
        .unwrap_or_else(|| get_channel_volume(channel))
}

#[allow(dead_code)]
pub fn is_audio_active() -> bool {
    if ANNOUNCEMENT_ACTIVE.load(Ordering::SeqCst) > 0 {
        return true;
    }
    is_music_active()
}

pub fn is_music_active() -> bool {
    lock_mutex(&ACTIVE_MUSIC, "active music")
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

fn is_music_ducked() -> bool {
    MUSIC_DUCK_DEPTH.load(Ordering::SeqCst) > 0
}

fn effective_music_volume(configured: f32) -> f32 {
    let configured = configured.clamp(0.0, 1.0);
    if is_music_ducked() {
        configured * MUSIC_DUCK_FACTOR
    } else {
        configured
    }
}

fn refresh_active_music_volume() {
    if let Ok(guard) = lock_mutex(&ACTIVE_MUSIC, "active music") {
        if let Some(music) = guard.as_ref() {
            music
                .sink
                .set_volume(effective_music_volume(music.base_volume));
        }
    }
}

struct MusicDuckGuard;

impl MusicDuckGuard {
    fn enter() -> Self {
        MUSIC_DUCK_DEPTH.fetch_add(1, Ordering::SeqCst);
        refresh_active_music_volume();
        Self
    }
}

impl Drop for MusicDuckGuard {
    fn drop(&mut self) {
        MUSIC_DUCK_DEPTH.fetch_sub(1, Ordering::SeqCst);
        refresh_active_music_volume();
    }
}

fn lock_mutex<'a, T>(mutex: &'a Mutex<T>, label: &str) -> Result<MutexGuard<'a, T>, String> {
    mutex.lock().or_else(|poisoned| {
        tracing::warn!(target: "audio", "recovering poisoned {label} mutex");
        Ok(poisoned.into_inner())
    })
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic לא ידוע בפענוח שמע".to_string()
    }
}

struct DecodedAudio {
    channels: u16,
    sample_rate: u32,
    samples: Vec<f32>,
}

fn decode_audio_bytes(bytes: &[u8]) -> Result<DecodedAudio, String> {
    if bytes.is_empty() {
        return Err("קובץ השמע ריק".to_string());
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        if looks_like_pcm_wav(bytes) {
            let (channels, sample_rate, samples) = pcm_wav_to_samples(bytes)?;
            if channels == 0 || sample_rate == 0 {
                return Err("פרמטרי שמע לא תקינים בקובץ".to_string());
            }
            let samples = samples
                .into_iter()
                .map(|sample| sample as f32 / i16::MAX as f32)
                .collect();
            return Ok(DecodedAudio {
                channels,
                sample_rate,
                samples,
            });
        }

        let decoder = Decoder::new(Cursor::new(bytes.to_vec())).map_err(|e| e.to_string())?;
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        if channels == 0 || sample_rate == 0 {
            return Err("פרמטרי שמע לא תקינים בקובץ".to_string());
        }
        let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();
        if samples.is_empty() {
            return Err("לא ניתן לפענח את קובץ השמע".to_string());
        }
        Ok(DecodedAudio {
            channels,
            sample_rate,
            samples,
        })
    }));

    match result {
        Ok(inner) => inner,
        Err(payload) => Err(format!(
            "קובץ שמע לא תקין ({})",
            panic_message(payload)
        )),
    }
}

fn play_announcement_internal(file_path: &str, volume: f32) -> Result<(), String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("קובץ לא נמצא: {file_path}"));
    }

    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    play_audio_bytes(&bytes, volume)
}

fn play_audio_bytes(bytes: &[u8], volume: f32) -> Result<(), String> {
    let decoded = decode_audio_bytes(bytes)?;
    let source = rodio::buffer::SamplesBuffer::new(
        decoded.channels,
        decoded.sample_rate,
        decoded.samples,
    );
    play_with_sink(source, volume)
}

fn looks_like_pcm_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

fn play_with_sink<S>(source: S, volume: f32) -> Result<(), String>
where
    S: rodio::Source + Send + 'static,
    f32: rodio::Sample + rodio::cpal::FromSample<S::Item>,
    S::Item: rodio::Sample + Send,
{
    crate::os_volume::force_system_output_full();
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("audio device open failed: {e}"))?;
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("audio sink start failed: {e}"))?;
    sink.set_volume(volume.clamp(0.0, 1.0));
    sink.append(source);
    // The OS session exists only after the stream opened; clear any remembered
    // per-app mute now so this playback is actually audible.
    crate::os_volume::ensure_self_audible();
    sink.sleep_until_end();
    Ok(())
}

fn pcm_wav_to_samples(bytes: &[u8]) -> Result<(u16, u32, Vec<i16>), String> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("קובץ השמע אינו WAV תקין".to_string());
    }

    let mut offset = 12usize;
    let mut channels = 1u16;
    let mut sample_rate = 44_100u32;
    let mut bits_per_sample = 16u16;
    let mut pcm: Option<&[u8]> = None;

    while offset + 8 <= bytes.len() {
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_size = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .map_err(|_| "כותרת WAV פגומה".to_string())?,
        ) as usize;
        let data_start = offset + 8;
        let data_end = data_start
            .checked_add(chunk_size)
            .ok_or_else(|| "כותרת WAV פגומה".to_string())?;
        if data_end > bytes.len() {
            return Err("קובץ השמע פגום או חתוך".to_string());
        }

        if chunk_id == b"fmt " {
            if chunk_size < 16 {
                return Err("כותרת fmt ב-WAV קצרה מדי".to_string());
            }
            let format_tag = u16::from_le_bytes(
                bytes[data_start..data_start + 2]
                    .try_into()
                    .map_err(|_| "כותרת WAV פגומה".to_string())?,
            );
            if format_tag != 1 {
                return Err("פורמט WAV זה אינו נתמך (נדרש PCM)".to_string());
            }
            channels = u16::from_le_bytes(
                bytes[data_start + 2..data_start + 4]
                    .try_into()
                    .map_err(|_| "כותרת WAV פגומה".to_string())?,
            );
            sample_rate = u32::from_le_bytes(
                bytes[data_start + 4..data_start + 8]
                    .try_into()
                    .map_err(|_| "כותרת WAV פגומה".to_string())?,
            );
            bits_per_sample = u16::from_le_bytes(
                bytes[data_start + 14..data_start + 16]
                    .try_into()
                    .map_err(|_| "כותרת WAV פגומה".to_string())?,
            );
        } else if chunk_id == b"data" {
            pcm = Some(&bytes[data_start..data_end]);
        }

        offset = data_end + (chunk_size % 2);
    }

    let pcm = pcm.ok_or_else(|| "לא נמצא מידע שמע בקובץ".to_string())?;
    let samples: Vec<i16> = match bits_per_sample {
        8 => pcm
            .iter()
            .map(|&sample| ((sample as i16) - 128) << 8)
            .collect(),
        16 => pcm
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect(),
        other => {
            return Err(format!("עומק ביט לא נתמך ב-WAV: {other}"));
        }
    };

    if samples.is_empty() {
        return Err("קובץ השמע ריק".to_string());
    }

    Ok((channels.max(1), sample_rate.max(1), samples))
}

fn play_announcement_blocking(file_path: &str, volume: f32) -> Result<(), String> {
    ANNOUNCEMENT_ACTIVE.fetch_add(1, Ordering::SeqCst);
    let result = (|| {
        let _guard = lock_mutex(&ANNOUNCEMENT_LOCK, "announcement")?;
        play_announcement_internal(file_path, volume)
    })();
    ANNOUNCEMENT_ACTIVE.fetch_sub(1, Ordering::SeqCst);
    result
}

fn play_bytes_blocking(bytes: &'static [u8], volume: f32) -> Result<(), String> {
    ANNOUNCEMENT_ACTIVE.fetch_add(1, Ordering::SeqCst);
    let result = (|| {
        let _guard = lock_mutex(&ANNOUNCEMENT_LOCK, "announcement")?;
        play_audio_bytes(bytes, volume)
    })();
    ANNOUNCEMENT_ACTIVE.fetch_sub(1, Ordering::SeqCst);
    result
}

pub fn stop_music() {
    if let Ok(mut active) = lock_mutex(&ACTIVE_MUSIC, "active music") {
        stop_active_music(&mut active);
    }
    crate::os_volume::on_music_stopped();
}

pub fn get_now_playing() -> Option<NowPlayingInfo> {
    lock_mutex(&ACTIVE_MUSIC, "active music")
        .ok()
        .and_then(|guard| guard.as_ref().map(|music| music.info.clone()))
}

fn stop_active_music(guard: &mut Option<ActiveMusicControl>) {
    if let Some(previous) = guard.take() {
        previous.sink.stop();
    }
}

fn play_music_blocking(file_path: &str, volume: f32) -> Result<(), String> {
    let _play_guard = lock_mutex(&MUSIC_PLAY_LOCK, "music play")?;

    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("קובץ לא נמצא: {file_path}"));
    }

    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let decoded = decode_audio_bytes(&bytes)?;
    let source = rodio::buffer::SamplesBuffer::new(
        decoded.channels,
        decoded.sample_rate,
        decoded.samples,
    );

    let title = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "שיר".to_string());
    let folder = crate::overview::folder_slug_from_music_path(path);
    let artwork_data_url = crate::overview::find_artwork_data_url(path);
    let info = NowPlayingInfo {
        title,
        file_path: file_path.to_string(),
        folder,
        artwork_data_url,
    };

    let base_volume = volume.clamp(0.0, 1.0);
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("audio device open failed: {e}"))?;
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("audio sink start failed: {e}"))?;
    sink.set_volume(effective_music_volume(base_volume));
    sink.append(source);

    {
        let mut active = lock_mutex(&ACTIVE_MUSIC, "active music")?;
        stop_active_music(&mut active);
        *active = Some(ActiveMusicControl {
            sink,
            base_volume,
            info,
        });
    }
    crate::os_volume::on_music_started();
    crate::os_volume::ensure_self_audible();

    loop {
        let finished = {
            let active = lock_mutex(&ACTIVE_MUSIC, "active music")?;
            match active.as_ref() {
                Some(music) => music.sink.empty(),
                None => true,
            }
        };
        if finished {
            break;
        }
        thread::sleep(Duration::from_millis(40));
    }

    let mut active = lock_mutex(&ACTIVE_MUSIC, "active music")?;
    stop_active_music(&mut active);
    crate::os_volume::on_music_stopped();

    Ok(())
}

pub fn play_audio_blocking_for_channel(
    file_path: &str,
    volume: f32,
    channel: VolumeChannel,
) -> Result<(), String> {
    match channel {
        VolumeChannel::Music => play_music_blocking(file_path, volume),
        VolumeChannel::General | VolumeChannel::System | VolumeChannel::Emergency => {
            let _duck = MusicDuckGuard::enter();
            play_announcement_blocking(file_path, volume)
        }
    }
}

pub fn play_volume_test(channel: VolumeChannel, volume: Option<f32>) -> Result<(), String> {
    let volume = volume
        .map(|value| value.clamp(0.0, 1.0))
        .unwrap_or_else(|| get_channel_volume(channel));
    let _duck = MusicDuckGuard::enter();
    play_bytes_blocking(VOLUME_TEST_SOUND, volume)
}

#[tauri::command]
pub async fn play_volume_test_channel(
    channel: String,
    volume: Option<f32>,
) -> Result<(), String> {
    let channel = VolumeChannel::parse(Some(&channel));
    tauri::async_runtime::spawn_blocking(move || play_volume_test(channel, volume))
        .await
        .map_err(|error| format!("שגיאה בהרצת בדיקת השמע: {error}"))?
}

#[tauri::command]
pub async fn play_audio(
    file_path: String,
    volume: Option<f32>,
    channel: Option<String>,
) -> Result<(), String> {
    let channel = VolumeChannel::parse(channel.as_deref());
    let resolved = resolve_playback_volume(channel, volume);
    tauri::async_runtime::spawn_blocking(move || {
        play_audio_blocking_for_channel(&file_path, resolved, channel)
    })
    .await
    .map_err(|error| format!("שגיאה בהרצת השמעה: {error}"))?
}

#[tauri::command]
pub fn get_audio_volumes() -> Result<AudioVolumes, String> {
    Ok(get_volumes())
}

#[tauri::command]
pub fn set_audio_volume_channel(
    channel: String,
    volume: f32,
    state: tauri::State<crate::db::DbState>,
) -> Result<AudioVolumes, String> {
    let volumes = state
        .set_audio_volume_channel(&channel, volume)
        .map_err(|error| error.to_string())?;
    init_volumes(&volumes);
    crate::app_log::settings(
        &state,
        &format!("volume {channel} = {:.0}%", volume.clamp(0.0, 1.0) * 100.0),
    );
    Ok(volumes)
}
