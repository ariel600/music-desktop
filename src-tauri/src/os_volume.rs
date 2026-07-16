use crate::db::DbState;
use chrono::Local;
use std::sync::atomic::AtomicU64;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

const LOCKED_INTERVAL: Duration = Duration::from_secs(1);
const IDLE_POLL_INTERVAL: Duration = Duration::from_millis(750);
/// OS master volume after the protection lock is released (user can change freely).
const RELEASED_SYSTEM_VOLUME: f32 = 0.20;

static OS_VOLUME_UI_LOG: AtomicU64 = AtomicU64::new(0);

/// Forces the OS master output volume to 100% and unmutes it.
pub fn force_system_output_full() {
    if let Err(error) = set_system_output_level_inner(1.0) {
        tracing::warn!(target: "os_volume", "לא ניתן לדרוס את עוצמת המחשב: {error}");
    }
}

fn log_os_volume_error(app: Option<&AppHandle>, detail: &str) {
    tracing::warn!(target: "os_volume", "{detail}");
    if let Some(app) = app {
        crate::app_log::from_app_rate_limited(app, &OS_VOLUME_UI_LOG, 60, detail, "error");
    }
}

pub fn on_music_started() {
    if let Err(error) = lock_external_audio() {
        tracing::warn!(target: "os_volume", "נעילת שמע חיצוני נכשלה: {error}");
    }
}

pub fn on_music_stopped() {
    // Unlock is handled by the operating-hours / music-window guard.
}

/// Called when the operator turns system activity off — release protection
/// immediately and drop OS volume to the idle default.
pub fn on_protection_released() {
    if let Err(error) = unlock_external_audio() {
        tracing::warn!(target: "os_volume", "שחרור שמע חיצוני נכשל: {error}");
    }
}

fn should_lock_volume(app: &AppHandle) -> bool {
    if !crate::system_activity::is_active() {
        return false;
    }
    if crate::audio::is_music_active() {
        return true;
    }
    // Lock for the full music window (open−10 … close+15), not only open→close,
    // so after-close playback stays protected even between tracks.
    is_within_music_window_now(app)
}

fn is_within_music_window_now(app: &AppHandle) -> bool {
    let Some(db) = app.try_state::<DbState>() else {
        return false;
    };
    let Ok(settings) = db.get_operating_hours() else {
        return false;
    };
    let now = Local::now();
    let operational_date = crate::operational_day::operational_date_string(now);
    let weekday = crate::operational_day::operational_weekday(now);
    let holiday = db.get_holiday_day(&operational_date).ok().flatten();
    crate::music_schedule::is_in_music_window(
        now,
        &operational_date,
        weekday,
        &settings,
        holiday.as_ref(),
    )
}

/// During the music/operating window (and while music plays): keep OS volume
/// at 100% and mute every other application's audio — even when folders are empty.
/// Outside those windows (or when system activity is off): unmute others and
/// set OS volume to 20% so the user can change it freely.
pub fn start_system_volume_guard(app: AppHandle) {
    thread::spawn(move || {
        let mut locked = false;
        loop {
            let should_lock = should_lock_volume(&app);
            if should_lock {
                if let Err(error) = lock_external_audio() {
                    log_os_volume_error(
                        Some(&app),
                        &format!("OS volume lock failed: {error}"),
                    );
                }
                locked = true;
                thread::sleep(LOCKED_INTERVAL);
                continue;
            }

            if locked {
                if let Err(error) = unlock_external_audio() {
                    log_os_volume_error(
                        Some(&app),
                        &format!("OS volume unlock failed: {error}"),
                    );
                }
                locked = false;
            }

            thread::sleep(IDLE_POLL_INTERVAL);
        }
    });
}

fn lock_external_audio() -> Result<(), String> {
    set_system_output_level_inner(1.0)?;
    mute_external_sessions(true)?;
    Ok(())
}

fn unlock_external_audio() -> Result<(), String> {
    mute_external_sessions(false)?;
    set_system_output_level_inner(RELEASED_SYSTEM_VOLUME)?;
    Ok(())
}

#[cfg(target_os = "windows")]
mod win {
    use std::ptr;
    use windows::core::Interface;
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{
        eConsole, eMultimedia, eRender, ERole, IAudioSessionControl2, IAudioSessionManager2,
        IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
    };
    use windows::Win32::System::Threading::GetCurrentProcessId;

    fn ensure_com() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    fn enumerator() -> Result<IMMDeviceEnumerator, String> {
        ensure_com();
        unsafe {
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| e.to_string())
        }
    }

    fn endpoint(role: ERole) -> Result<IMMDevice, String> {
        let enumerator = enumerator()?;
        unsafe {
            enumerator
                .GetDefaultAudioEndpoint(eRender, role)
                .map_err(|e| e.to_string())
        }
    }

    fn set_endpoint_volume(device: &IMMDevice, level: f32) -> Result<(), String> {
        let level = level.clamp(0.0, 1.0);
        unsafe {
            let volume = device
                .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                .map_err(|e| e.to_string())?;
            volume
                .SetMasterVolumeLevelScalar(level, ptr::null())
                .map_err(|e| e.to_string())?;
            volume
                .SetMute(false, ptr::null())
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Set master volume on both Console and Multimedia default render devices.
    pub fn set_system_output_level(level: f32) -> Result<(), String> {
        let mut last_error = None;
        let mut ok = false;
        for role in [eConsole, eMultimedia] {
            match endpoint(role).and_then(|device| set_endpoint_volume(&device, level)) {
                Ok(()) => ok = true,
                Err(error) => last_error = Some(error),
            }
        }
        if ok {
            Ok(())
        } else {
            Err(last_error.unwrap_or_else(|| "לא ניתן לשנות את עוצמת המערכת".into()))
        }
    }

    fn apply_session_policy(device: &IMMDevice, mute_others: bool, our_pid: u32) {
        unsafe {
            let Ok(session_manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)
            else {
                return;
            };
            let Ok(session_enumerator) = session_manager.GetSessionEnumerator() else {
                return;
            };
            let Ok(count) = session_enumerator.GetCount() else {
                return;
            };

            for index in 0..count {
                let Ok(session) = session_enumerator.GetSession(index) else {
                    continue;
                };
                let Ok(session2) = session.cast::<IAudioSessionControl2>() else {
                    continue;
                };
                let pid = session2.GetProcessId().unwrap_or(0);
                let Ok(volume) = session.cast::<ISimpleAudioVolume>() else {
                    continue;
                };

                if pid == our_pid {
                    // Keep our own streams audible at full session volume.
                    let _ = volume.SetMute(false, ptr::null());
                    let _ = volume.SetMasterVolume(1.0, ptr::null());
                    continue;
                }

                if mute_others {
                    let _ = volume.SetMute(true, ptr::null());
                    let _ = volume.SetMasterVolume(0.0, ptr::null());
                } else {
                    let _ = volume.SetMute(false, ptr::null());
                    // Restore a usable session level; apps keep their own mixer prefs.
                    let _ = volume.SetMasterVolume(1.0, ptr::null());
                }
            }
        }
    }

    pub fn mute_external_sessions(mute: bool) -> Result<(), String> {
        ensure_com();
        let our_pid = unsafe { GetCurrentProcessId() };
        let mut touched = false;
        let mut last_error = None;

        for role in [eConsole, eMultimedia] {
            match endpoint(role) {
                Ok(device) => {
                    apply_session_policy(&device, mute, our_pid);
                    touched = true;
                }
                Err(error) => last_error = Some(error),
            }
        }

        if touched {
            Ok(())
        } else {
            Err(last_error.unwrap_or_else(|| "לא ניתן לגשת לסשני השמע".into()))
        }
    }
}

#[cfg(target_os = "windows")]
fn mute_external_sessions(mute: bool) -> Result<(), String> {
    win::mute_external_sessions(mute)
}

#[cfg(target_os = "windows")]
fn set_system_output_level_inner(level: f32) -> Result<(), String> {
    win::set_system_output_level(level)
}

#[cfg(target_os = "linux")]
fn mute_external_sessions(mute: bool) -> Result<(), String> {
    let our_pid = std::process::id().to_string();
    let inputs = list_pulse_sink_inputs().unwrap_or_default();

    for input in inputs {
        let is_ours = input_belongs_to_us(&input, &our_pid);
        if mute {
            // PipeWire/ALSA streams from rodio often omit application.process.id.
            // Never mute unknown/own streams — only confirmed foreign PIDs.
            if is_ours || input.process_id.is_none() {
                ensure_sink_input_audible(&input.index);
                continue;
            }
            let _ = run_ok("pactl", &["set-sink-input-mute", &input.index, "1"]);
        } else {
            let _ = run_ok("pactl", &["set-sink-input-mute", &input.index, "0"]);
            if is_ours {
                ensure_sink_input_audible(&input.index);
            }
        }
    }

    // Always re-assert that our own streams are unmuted after a lock pass.
    for input in list_pulse_sink_inputs().unwrap_or_default() {
        if input_belongs_to_us(&input, &our_pid) {
            ensure_sink_input_audible(&input.index);
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn input_belongs_to_us(input: &SinkInput, our_pid: &str) -> bool {
    if input.process_id.as_deref() == Some(our_pid) {
        return true;
    }
    let name = input.application_name.as_deref().unwrap_or("");
    let node = input.node_name.as_deref().unwrap_or("");
    // cpal/rodio on PipeWire: "PipeWire ALSA [nusic]" / "alsa_playback.nusic"
    name.to_ascii_lowercase().contains("nusic") || node.to_ascii_lowercase().contains("nusic")
}

#[cfg(target_os = "linux")]
fn ensure_sink_input_audible(index: &str) {
    let _ = run_ok("pactl", &["set-sink-input-mute", index, "0"]);
    let _ = run_ok("pactl", &["set-sink-input-volume", index, "100%"]);
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct SinkInput {
    index: String,
    process_id: Option<String>,
    application_name: Option<String>,
    node_name: Option<String>,
}

#[cfg(target_os = "linux")]
fn list_pulse_sink_inputs() -> Result<Vec<SinkInput>, String> {
    use std::process::Command;

    let output = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
        .map_err(|error| format!("pactl: {error}"))?;
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut inputs = Vec::new();
    let mut current_index: Option<String> = None;
    let mut current_pid: Option<String> = None;
    let mut current_app: Option<String> = None;
    let mut current_node: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Sink Input #") {
            if let Some(index) = current_index.take() {
                inputs.push(SinkInput {
                    index,
                    process_id: current_pid.take(),
                    application_name: current_app.take(),
                    node_name: current_node.take(),
                });
            }
            current_index = Some(rest.trim().to_string());
            current_pid = None;
            current_app = None;
            current_node = None;
        } else if let Some(rest) = trimmed.strip_prefix("application.process.id = ") {
            let pid = rest.trim().trim_matches('"');
            if !pid.is_empty() {
                current_pid = Some(pid.to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("application.name = ") {
            let name = rest.trim().trim_matches('"');
            if !name.is_empty() {
                current_app = Some(name.to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("node.name = ") {
            let name = rest.trim().trim_matches('"');
            if !name.is_empty() {
                current_node = Some(name.to_string());
            }
        }
    }

    if let Some(index) = current_index {
        inputs.push(SinkInput {
            index,
            process_id: current_pid,
            application_name: current_app,
            node_name: current_node,
        });
    }

    Ok(inputs)
}

#[cfg(target_os = "linux")]
fn set_system_output_level_inner(level: f32) -> Result<(), String> {
    let level = level.clamp(0.0, 1.0);
    let percent = ((level * 100.0).round() as u32).min(100);
    let fraction = format!("{level:.2}");
    let percent_arg = format!("{percent}%");

    if run_ok(
        "wpctl",
        &["set-volume", "@DEFAULT_AUDIO_SINK@", &fraction],
    ) {
        let _ = run_ok("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "0"]);
        return Ok(());
    }

    if run_ok(
        "pactl",
        &["set-sink-volume", "@DEFAULT_SINK@", &percent_arg],
    ) {
        let _ = run_ok("pactl", &["set-sink-mute", "@DEFAULT_SINK@", "0"]);
        return Ok(());
    }

    if run_ok(
        "amixer",
        &["-q", "set", "Master", &percent_arg, "unmute"],
    ) {
        return Ok(());
    }

    Err("לא נמצא כלי לשליטה בעוצמת המערכת (wpctl/pactl/amixer)".to_string())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn mute_external_sessions(_mute: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn set_system_output_level_inner(_level: f32) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_ok(program: &str, args: &[&str]) -> bool {
    use std::process::Command;

    Command::new(program)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
