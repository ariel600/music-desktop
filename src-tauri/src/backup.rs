use std::fs::{self, File};
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const BACKUP_EXTENSION: &str = "mshbak";
pub const BACKUP_MAGIC: &[u8] = b"MSHBAK01";

pub fn export_backup(app_data_dir: &Path, dest_file: &Path) -> Result<String, String> {
    let dest_file = ensure_backup_extension(dest_file);
    tracing::info!(target: "backup", "[backup.export] - started - dest={}", dest_file.display());

    if let Some(parent) = dest_file.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let staging = staging_dir("export")?;
    let staging_guard = TempDirGuard(staging.clone());

    let db_path = app_data_dir.join("nusic.db");
    if db_path.exists() {
        fs::copy(&db_path, staging.join("nusic.db")).map_err(|e| e.to_string())?;
    }

    let audio_dir = app_data_dir.join("audio");
    if audio_dir.exists() {
        copy_dir_recursive(&audio_dir, &staging.join("audio"))?;
    }

    let music_dir = app_data_dir.join(crate::music_storage::MUSIC_LIBRARY_DIR);
    if music_dir.exists() {
        copy_dir_recursive(
            &music_dir,
            &staging.join(crate::music_storage::MUSIC_LIBRARY_DIR),
        )?;
    }

    for dir in crate::message_storage::MESSAGE_LIBRARY_DIRS {
        let source = app_data_dir.join(dir);
        if source.exists() {
            copy_dir_recursive(&source, &staging.join(dir))?;
        }
    }

    let password_dir = app_data_dir.join(crate::password_vault::PasswordVault::store_dir_name());
    if password_dir.exists() {
        copy_dir_recursive(
            &password_dir,
            &staging.join(crate::password_vault::PasswordVault::store_dir_name()),
        )?;
    }

    write_backup_archive(&staging, &dest_file)?;
    drop(staging_guard);

    tracing::info!(target: "backup", "[backup.export] - success - dest={}", dest_file.display());
    Ok(dest_file.to_string_lossy().to_string())
}

pub fn import_backup(app_data_dir: &Path, source_file: &Path) -> Result<(), String> {
    tracing::info!(target: "backup", "[backup.import] - started - source={}", source_file.display());
    if !source_file.exists() {
        tracing::warn!(target: "backup", "[backup.import] - failed - reason=source_missing");
        return Err("קובץ הגיבוי לא נמצא".to_string());
    }

    let extension = source_file
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if extension != BACKUP_EXTENSION {
        tracing::warn!(target: "backup", "[backup.import] - failed - reason=bad_extension, got={extension}");
        return Err(format!(
            "סוג קובץ לא נתמך. יש לבחור קובץ .{BACKUP_EXTENSION}"
        ));
    }

    let staging = staging_dir("import")?;
    let staging_guard = TempDirGuard(staging.clone());
    extract_backup_archive(source_file, &staging)?;

    let db_source = staging.join("nusic.db");
    if !db_source.exists() {
        return Err("קובץ הגיבוי פגום או אינו מכיל נתוני מערכת".to_string());
    }

    fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;

    let db_dest = app_data_dir.join("nusic.db");
    fs::copy(&db_source, &db_dest).map_err(|e| e.to_string())?;

    let audio_source = staging.join("audio");
    if audio_source.exists() {
        let audio_dest = app_data_dir.join("audio");
        if audio_dest.exists() {
            fs::remove_dir_all(&audio_dest).map_err(|e| e.to_string())?;
        }
        copy_dir_recursive(&audio_source, &audio_dest)?;
    }

    let music_source = staging.join(crate::music_storage::MUSIC_LIBRARY_DIR);
    if music_source.exists() {
        let music_dest = app_data_dir.join(crate::music_storage::MUSIC_LIBRARY_DIR);
        if music_dest.exists() {
            fs::remove_dir_all(&music_dest).map_err(|e| e.to_string())?;
        }
        copy_dir_recursive(&music_source, &music_dest)?;
        crate::music_storage::init_music_library(app_data_dir)?;
    }

    for dir in crate::message_storage::MESSAGE_LIBRARY_DIRS {
        let source = staging.join(dir);
        if source.exists() {
            let dest = app_data_dir.join(dir);
            if dest.exists() {
                fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
            }
            copy_dir_recursive(&source, &dest)?;
        }
    }

    let password_source =
        staging.join(crate::password_vault::PasswordVault::store_dir_name());
    let password_dest =
        app_data_dir.join(crate::password_vault::PasswordVault::store_dir_name());
    if password_dest.exists() {
        fs::remove_dir_all(&password_dest).map_err(|e| e.to_string())?;
    }
    if password_source.exists() {
        copy_dir_recursive(&password_source, &password_dest)?;
    }

    crate::message_storage::init_message_libraries(app_data_dir)?;
    drop(staging_guard);

    tracing::info!(target: "backup", "[backup.import] - success - restored into {}", app_data_dir.display());
    Ok(())
}

pub fn reset_system(
    app_data_dir: &Path,
    db: &crate::db::DbState,
    keep_music: bool,
) -> Result<(), String> {
    tracing::warn!(target: "backup", "[backup.reset] - started - keep_music={keep_music}");
    let audio_dir = app_data_dir.join("audio");
    if audio_dir.exists() {
        fs::remove_dir_all(&audio_dir).map_err(|e| e.to_string())?;
    }

    if !keep_music {
        let music_dir = app_data_dir.join(crate::music_storage::MUSIC_LIBRARY_DIR);
        if music_dir.exists() {
            fs::remove_dir_all(&music_dir).map_err(|e| e.to_string())?;
        }
    }

    for dir in crate::message_storage::MESSAGE_LIBRARY_DIRS {
        let path = app_data_dir.join(dir);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        }
    }

    let volume_test = app_data_dir.join("volume-test.wav");
    if volume_test.exists() {
        let _ = fs::remove_file(&volume_test);
    }

    let password_dir = app_data_dir.join(crate::password_vault::PasswordVault::store_dir_name());
    if password_dir.exists() {
        fs::remove_dir_all(&password_dir).map_err(|e| e.to_string())?;
    }

    db.factory_reset().map_err(|e| e.to_string())?;
    crate::music_storage::init_music_library(app_data_dir)?;
    crate::message_storage::init_message_libraries(app_data_dir)?;
    crate::audio::init_volumes(&crate::audio::AudioVolumes::default());

    tracing::info!(target: "backup", "[backup.reset] - success - keep_music={keep_music}");
    Ok(())
}

fn ensure_backup_extension(path: &Path) -> PathBuf {
    match path.extension().and_then(|value| value.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case(BACKUP_EXTENSION) => path.to_path_buf(),
        _ => path.with_extension(BACKUP_EXTENSION),
    }
}

fn staging_dir(kind: &str) -> Result<PathBuf, String> {
    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%f").to_string();
    let dir = std::env::temp_dir().join(format!("nusic_{kind}_{stamp}"));
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_backup_archive(source_dir: &Path, dest_file: &Path) -> Result<(), String> {
    let mut output = File::create(dest_file).map_err(|e| e.to_string())?;
    output
        .write_all(BACKUP_MAGIC)
        .map_err(|e| e.to_string())?;

    {
        let mut zip = ZipWriter::new(&mut output);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        add_dir_to_zip(&mut zip, source_dir, Path::new(""), options)?;
        zip.finish().map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn add_dir_to_zip<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    source_dir: &Path,
    prefix: &Path,
    options: SimpleFileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(source_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = prefix.join(entry.file_name());
        let name_str = name
            .to_str()
            .ok_or_else(|| "שם קובץ לא תקין בגיבוי".to_string())?
            .replace('\\', "/");

        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            zip.add_directory(format!("{name_str}/"), options)
                .map_err(|e| e.to_string())?;
            add_dir_to_zip(zip, &path, &name, options)?;
        } else {
            zip.start_file(&name_str, options)
                .map_err(|e| e.to_string())?;
            let mut input = File::open(&path).map_err(|e| e.to_string())?;
            std::io::copy(&mut input, zip).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn extract_backup_archive(source_file: &Path, dest_dir: &Path) -> Result<(), String> {
    let mut file = File::open(source_file).map_err(|e| e.to_string())?;
    let mut magic = [0_u8; 8];
    file.read_exact(&mut magic)
        .map_err(|_| "קובץ הגיבוי פגום או אינו בפורמט של המערכת".to_string())?;
    if magic != BACKUP_MAGIC {
        return Err("קובץ הגיבוי פגום או אינו בפורמט של המערכת".to_string());
    }

    let mut archive = ZipArchive::new(file).map_err(|_| {
        "קובץ הגיבוי פגום או אינו בפורמט של המערכת".to_string()
    })?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let entry_name = entry
            .enclosed_name()
            .ok_or_else(|| "שם קובץ לא תקין בגיבוי".to_string())?;
        let out_path = dest_dir.join(entry_name);

        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut out_file = File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;

    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let target = dest.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub fn validate_audio_file(path: &str) -> Result<(), String> {
    let file_path = PathBuf::from(path);
    if !file_path.exists() {
        return Err("קובץ השמע לא נמצא".to_string());
    }

    let metadata = fs::metadata(&file_path).map_err(|e| e.to_string())?;
    if metadata.len() == 0 {
        return Err("קובץ השמע ריק".to_string());
    }

    Ok(())
}
