use ffmpeg_sidecar::download::{
    download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg_without_extras,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static FFMPEG_SETUP_LOCK: Mutex<()> = Mutex::new(());
static CONVERTER_READY: AtomicBool = AtomicBool::new(false);

pub const CONVERTER_INTERNET_REQUIRED_MSG: &str =
    "ממיר השמע לא מותקן. התקנת התוכנה דורשת חיבור לאינטרנט — חבר את המחשב לרשת והפעל מחדש את התוכנה.";

fn ffmpeg_exe_name() -> &'static str {
    if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    }
}

fn app_bundled_sidecar_name() -> String {
    let target = env!("BUILD_TARGET");
    if cfg!(windows) {
        format!("ffmpeg-{target}.exe")
    } else {
        format!("ffmpeg-{target}")
    }
}

fn app_bundled_ffmpeg_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let parent = exe.parent()?;
    let path = parent.join(app_bundled_sidecar_name());
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn local_ffmpeg_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("bin").join(ffmpeg_exe_name())
}

pub fn install_converter(app_data_dir: &Path) -> Result<PathBuf, String> {
    if let Some(path) = app_bundled_ffmpeg_path() {
        CONVERTER_READY.store(true, Ordering::Relaxed);
        return Ok(path);
    }

    let local_path = local_ffmpeg_path(app_data_dir);
    if local_path.exists() {
        CONVERTER_READY.store(true, Ordering::Relaxed);
        return Ok(local_path);
    }

    let _guard = FFMPEG_SETUP_LOCK
        .lock()
        .map_err(|_| "שגיאה בנעילת התקנת ממיר השמע".to_string())?;

    if let Some(path) = app_bundled_ffmpeg_path() {
        CONVERTER_READY.store(true, Ordering::Relaxed);
        return Ok(path);
    }

    if local_path.exists() {
        CONVERTER_READY.store(true, Ordering::Relaxed);
        return Ok(local_path);
    }

    match download_ffmpeg_to(app_data_dir) {
        Ok(path) => {
            CONVERTER_READY.store(true, Ordering::Relaxed);
            Ok(path)
        }
        Err(error) => {
            CONVERTER_READY.store(false, Ordering::Relaxed);
            Err(error)
        }
    }
}

fn download_ffmpeg_to(app_data_dir: &Path) -> Result<PathBuf, String> {
    let bin_dir = app_data_dir.join("bin");
    fs::create_dir_all(&bin_dir).map_err(|error| error.to_string())?;

    let ffmpeg_path = local_ffmpeg_path(app_data_dir);

    tracing::info!(target: "audio_convert", "[ffmpeg.download] - started - dest={}", bin_dir.display());

    let url = ffmpeg_download_url().map_err(|error| {
        tracing::warn!(target: "audio_convert", "[ffmpeg.download] - failed - stage=resolve_url, error={error}");
        CONVERTER_INTERNET_REQUIRED_MSG.to_string()
    })?;

    let archive = download_ffmpeg_package(url, &bin_dir).map_err(|error| {
        tracing::warn!(target: "audio_convert", "[ffmpeg.download] - failed - stage=download, error={error}");
        CONVERTER_INTERNET_REQUIRED_MSG.to_string()
    })?;

    unpack_ffmpeg_without_extras(&archive, &bin_dir).map_err(|error| {
        tracing::warn!(target: "audio_convert", "[ffmpeg.download] - failed - stage=unpack, error={error}");
        CONVERTER_INTERNET_REQUIRED_MSG.to_string()
    })?;

    if archive.exists() {
        fs::remove_file(&archive).ok();
    }

    if !ffmpeg_path.exists() {
        tracing::warn!(target: "audio_convert", "[ffmpeg.download] - failed - stage=verify, reason=binary_missing");
        return Err(CONVERTER_INTERNET_REQUIRED_MSG.to_string());
    }

    tracing::info!(target: "audio_convert", "[ffmpeg.download] - success - path={}", ffmpeg_path.display());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&ffmpeg_path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&ffmpeg_path, permissions).map_err(|error| error.to_string())?;
    }

    Ok(ffmpeg_path)
}

fn resolve_ffmpeg(app_data_dir: &Path) -> Result<PathBuf, String> {
    if let Some(path) = app_bundled_ffmpeg_path() {
        return Ok(path);
    }

    let local_path = local_ffmpeg_path(app_data_dir);
    if local_path.exists() {
        return Ok(local_path);
    }

    Err(CONVERTER_INTERNET_REQUIRED_MSG.to_string())
}

pub fn convert_to_mp3(
    app_data_dir: &Path,
    source: &Path,
    dest: &Path,
) -> Result<(), String> {
    let ffmpeg = resolve_ffmpeg(app_data_dir)?;

    let mut command = Command::new(&ffmpeg);
    command.args([
        "-nostdin",
        "-y",
        "-i",
        &source.to_string_lossy(),
        "-vn",
        "-acodec",
        "libmp3lame",
        "-q:a",
        "2",
        &dest.to_string_lossy(),
    ]);

    // Prevent a console window from flashing when the GUI (windows_subsystem =
    // "windows") spawns ffmpeg on Windows.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    tracing::info!(
        target: "audio_convert",
        "[ffmpeg.convert] - started - source={}, dest={}",
        source.display(),
        dest.display()
    );

    let output = command.output().map_err(|error| {
        tracing::warn!(target: "audio_convert", "[ffmpeg.convert] - failed - stage=spawn, error={error}");
        format!("הפעלת ממיר השמע נכשלה: {error}")
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(
            target: "audio_convert",
            "[ffmpeg.convert] - failed - stage=transcode, status={}, details={stderr}",
            output.status
        );
        return Err(format!("המרת MP4 ל-MP3 נכשלה: {stderr}"));
    }

    tracing::info!(
        target: "audio_convert",
        "[ffmpeg.convert] - success - dest={}",
        dest.display()
    );
    Ok(())
}
