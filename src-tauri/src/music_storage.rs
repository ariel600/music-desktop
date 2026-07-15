use crate::audio_convert;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const MUSIC_LIBRARY_DIR: &str = ".music";

/// Canonicalize a path and strip the Windows verbatim prefix (`\\?\`) so that
/// `starts_with` comparisons work regardless of which side produced the
/// extended-length form.
fn normalized_canonical(path: &Path) -> std::io::Result<PathBuf> {
    let canonical = path.canonicalize()?;
    #[cfg(windows)]
    {
        if let Some(stripped) = canonical
            .to_str()
            .and_then(|value| value.strip_prefix(r"\\?\"))
        {
            return Ok(PathBuf::from(stripped));
        }
    }
    Ok(canonical)
}

pub const VOCAL_ONLY_FOLDERS: &[&str] = &[
    "general-vocal",
    "sefirat-haomer",
    "lag-baomer-vocal",
    "bein-hametzarim",
];

pub fn requires_vocal_warning(folder: &str) -> bool {
    VOCAL_ONLY_FOLDERS.contains(&folder)
}

pub const MUSIC_FOLDER_SLUGS: &[&str] = &[
    "general",
    "general-vocal",
    "shabbat",
    "rosh-hashana",
    "sukkot",
    "chanukah",
    "tu-bishvat",
    "purim",
    "pesach",
    "sefirat-haomer",
    "lag-baomer-vocal",
    "lag-baomer",
    "shavuot",
    "bein-hametzarim",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MusicFileEntry {
    pub name: String,
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
}

pub fn music_library_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(MUSIC_LIBRARY_DIR)
}

pub fn init_music_library(app_data_dir: &Path) -> Result<(), String> {
    let root = music_library_root(app_data_dir);
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;

    for slug in MUSIC_FOLDER_SLUGS {
        fs::create_dir_all(root.join(slug)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn music_folder_path(app_data_dir: &Path, folder: &str) -> Result<PathBuf, String> {
    if !MUSIC_FOLDER_SLUGS.contains(&folder) {
        return Err("תיקייה לא תקינה".to_string());
    }

    let path = music_library_root(app_data_dir).join(folder);
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn list_music_files(app_data_dir: &Path, folder: &str) -> Result<Vec<MusicFileEntry>, String> {
    let dir = music_folder_path(app_data_dir, folder)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if !file_type.is_file() {
            continue;
        }

        let path = entry.path();
        if !is_stored_audio_path(&path) {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
        let display_name = path
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| file_name.clone());

        files.push(MusicFileEntry {
            name: display_name,
            file_name,
            path: path.to_string_lossy().to_string(),
            size_bytes: metadata.len(),
        });
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

pub fn count_all_music_files(app_data_dir: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    for slug in MUSIC_FOLDER_SLUGS {
        total += list_music_files(app_data_dir, slug)?.len() as u64;
    }
    Ok(total)
}

fn is_audio_extension(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" | "wma"
    )
}

fn is_video_extension(extension: &str) -> bool {
    extension.eq_ignore_ascii_case("mp4")
}

fn is_importable_extension(extension: &str) -> bool {
    is_audio_extension(extension) || is_video_extension(extension)
}

fn is_stored_audio_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(is_audio_extension)
        .unwrap_or(false)
}

pub fn is_importable_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(is_importable_extension)
        .unwrap_or(false)
}

pub fn is_video_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(is_video_extension)
        .unwrap_or(false)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScannedMusicFile {
    pub name: String,
    pub source_path: String,
    pub size_bytes: u64,
    pub will_convert_to_mp3: bool,
}

pub fn scan_music_sources(source_paths: &[String]) -> Result<Vec<ScannedMusicFile>, String> {
    let mut collected = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for source_path in source_paths {
        let path = PathBuf::from(source_path);
        if !path.exists() {
            continue;
        }
        collect_importable_files(&path, &mut collected)?;
    }

    if collected.is_empty() {
        return Err("לא נמצאו קבצי שמע בנתיבים שנבחרו.".to_string());
    }

    collected.sort_by(|a, b| a.0.cmp(&b.0));

    let mut files = Vec::new();
    for (source_path, size_bytes) in collected {
        if !seen.insert(source_path.clone()) {
            continue;
        }

        let path = PathBuf::from(&source_path);
        let name = path
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| source_path.clone());

        files.push(ScannedMusicFile {
            name,
            source_path,
            size_bytes,
            will_convert_to_mp3: is_video_path(&path),
        });
    }

    Ok(files)
}

pub fn import_music_files(
    app_data_dir: &Path,
    folder: &str,
    source_paths: &[String],
    vocal_warning_acknowledged: bool,
) -> Result<Vec<MusicFileEntry>, String> {
    if requires_vocal_warning(folder) && !vocal_warning_acknowledged {
        return Err(
            "יש לאשר את אזהרת השירים הווקאליים לפני הוספה לתיקייה זו.".to_string(),
        );
    }

    let dest_dir = music_folder_path(app_data_dir, folder)?;
    let scanned = scan_music_sources(source_paths)?;
    tracing::info!(
        target: "music_storage",
        "[music.import] - started - folder={folder}, files={}",
        scanned.len()
    );
    let mut imported = Vec::new();
    let mut imported_paths: Vec<PathBuf> = Vec::new();

    for file in scanned {
        let import_result =
            import_single_file(app_data_dir, &dest_dir, &file.source_path);

        match import_result {
            Ok((entry, dest_path)) => {
                imported_paths.push(dest_path);
                imported.push(entry);
            }
            Err(error) => {
                for path in imported_paths {
                    let _ = fs::remove_file(path);
                }
                tracing::warn!(
                    target: "music_storage",
                    "[music.import] - failed - folder={folder}, rolled_back=true, error={error}"
                );
                return Err(error);
            }
        }
    }

    imported.sort_by(|a, b| a.name.cmp(&b.name));
    tracing::info!(
        target: "music_storage",
        "[music.import] - success - folder={folder}, imported={}",
        imported.len()
    );
    Ok(imported)
}

pub fn delete_music_files(
    app_data_dir: &Path,
    folder: &str,
    file_paths: &[String],
) -> Result<usize, String> {
    if file_paths.is_empty() {
        return Ok(0);
    }

    let dir = music_folder_path(app_data_dir, folder)?;
    let dir_canonical = normalized_canonical(&dir)
        .map_err(|error| format!("שגיאה בגישה לתיקיית המוזיקה: {error}"))?;

    let mut deleted = 0usize;

    for file_path in file_paths {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            return Err(format!("הקובץ לא נמצא: {file_path}"));
        }

        let canonical = normalized_canonical(&path)
            .map_err(|error| format!("שגיאה בגישה לקובץ: {error}"))?;

        if !canonical.starts_with(&dir_canonical) {
            tracing::warn!(
                target: "music_storage",
                "[music.delete] - rejected - reason=path_outside_library, folder={folder}, path={file_path}"
            );
            return Err("נתיב קובץ לא תקין".to_string());
        }

        if !is_stored_audio_path(&canonical) {
            tracing::warn!(
                target: "music_storage",
                "[music.delete] - rejected - reason=unsupported_type, folder={folder}, path={file_path}"
            );
            return Err("סוג קובץ לא נתמך למחיקה".to_string());
        }

        fs::remove_file(&canonical).map_err(|error| error.to_string())?;
        deleted += 1;
    }

    tracing::info!(
        target: "music_storage",
        "[music.delete] - success - folder={folder}, deleted={deleted}"
    );
    Ok(deleted)
}

pub fn import_audio_only_into_directory(
    dest_dir: &Path,
    source_path: &str,
) -> Result<MusicFileEntry, String> {
    let source = Path::new(source_path);

    if is_video_path(source) {
        return Err("ניתן להעלות רק קבצי שמע".to_string());
    }

    if !is_importable_path(source) || !is_stored_audio_path(source) {
        return Err("סוג קובץ לא נתמך. ניתן להעלות רק קבצי שמע.".to_string());
    }

    fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    crate::backup::validate_audio_file(source_path)?;

    let original_name = source
        .file_name()
        .ok_or_else(|| "שם קובץ לא תקין".to_string())?;
    let dest_name = unique_dest_name(dest_dir, original_name);
    let dest_path = dest_dir.join(&dest_name);
    fs::copy(source, &dest_path).map_err(|e| e.to_string())?;

    let metadata = fs::metadata(&dest_path).map_err(|e| e.to_string())?;
    let display_name = dest_path
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| dest_name.to_string_lossy().to_string());

    Ok(MusicFileEntry {
        name: display_name,
        file_name: dest_name.to_string_lossy().to_string(),
        path: dest_path.to_string_lossy().to_string(),
        size_bytes: metadata.len(),
    })
}

fn import_single_file(
    app_data_dir: &Path,
    dest_dir: &Path,
    source_path: &str,
) -> Result<(MusicFileEntry, PathBuf), String> {
    crate::backup::validate_audio_file(source_path)?;

    let source = PathBuf::from(source_path);
    let stem = source
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "song".to_string());

    let (dest_name, dest_path) = if is_video_path(&source) {
        let dest_name = unique_dest_mp3_name(dest_dir, &stem);
        let dest_path = dest_dir.join(&dest_name);
        audio_convert::convert_to_mp3(app_data_dir, &source, &dest_path)?;
        (dest_name, dest_path)
    } else {
        let original_name = source
            .file_name()
            .ok_or_else(|| "שם קובץ לא תקין".to_string())?;
        let dest_name = unique_dest_name(dest_dir, original_name);
        let dest_path = dest_dir.join(&dest_name);
        fs::copy(&source, &dest_path).map_err(|e| e.to_string())?;
        (dest_name.to_string_lossy().to_string(), dest_path)
    };

    let metadata = fs::metadata(&dest_path).map_err(|e| e.to_string())?;
    let entry = MusicFileEntry {
        name: Path::new(&dest_name)
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or(stem),
        file_name: dest_name,
        path: dest_path.to_string_lossy().to_string(),
        size_bytes: metadata.len(),
    };

    Ok((entry, dest_path))
}

fn collect_importable_files(path: &Path, out: &mut Vec<(String, u64)>) -> Result<(), String> {
    if path.is_file() {
        if !is_importable_path(path) {
            return Ok(());
        }

        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
        out.push((path.to_string_lossy().to_string(), metadata.len()));
        return Ok(());
    }

    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            collect_importable_files(&entry.path(), out)?;
        }
    }

    Ok(())
}

fn unique_dest_name(dest_dir: &Path, original_name: &std::ffi::OsStr) -> std::ffi::OsString {
    let original = original_name.to_string_lossy();
    let path = Path::new(original.as_ref());
    let extension = path
        .extension()
        .map(|value| value.to_string_lossy().to_string());
    let stem = path
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| original.to_string());

    let mut counter = 0;
    loop {
        let candidate = if counter == 0 {
            original.to_string()
        } else if let Some(ref ext) = extension {
            format!("{stem} ({counter}).{ext}")
        } else {
            format!("{stem} ({counter})")
        };

        if !dest_dir.join(&candidate).exists() {
            return std::ffi::OsString::from(candidate);
        }

        counter += 1;
    }
}

fn unique_dest_mp3_name(dest_dir: &Path, stem: &str) -> String {
    let mut counter = 0;
    loop {
        let candidate = if counter == 0 {
            format!("{stem}.mp3")
        } else {
            format!("{stem} ({counter}).mp3")
        };

        if !dest_dir.join(&candidate).exists() {
            return candidate;
        }

        counter += 1;
    }
}
