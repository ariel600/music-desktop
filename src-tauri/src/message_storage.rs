use crate::db;
use crate::music_storage::{self, MusicFileEntry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const EMERGENCY_MESSAGES_DIR: &str = ".emergency-messages";
pub const SYSTEM_MESSAGES_DIR: &str = ".system-messages";

pub const MESSAGE_LIBRARY_DIRS: &[&str] = &[EMERGENCY_MESSAGES_DIR, SYSTEM_MESSAGES_DIR];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmergencyMessageAudioEntry {
    pub message_type: String,
    pub name: String,
    pub path: String,
}

pub fn emergency_messages_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(EMERGENCY_MESSAGES_DIR)
}

pub fn system_messages_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(SYSTEM_MESSAGES_DIR)
}

pub fn init_message_libraries(app_data_dir: &Path) -> Result<(), String> {
    for dir in MESSAGE_LIBRARY_DIRS {
        fs::create_dir_all(app_data_dir.join(dir)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn import_emergency_message_audio(
    app_data_dir: &Path,
    message_type: &str,
    source_path: &str,
) -> Result<EmergencyMessageAudioEntry, String> {
    if !db::EMERGENCY_MESSAGE_TYPE_IDS.contains(&message_type) {
        return Err("סוג הודעה לא תקין".to_string());
    }

    let dest_dir = emergency_messages_root(app_data_dir);
    let imported: MusicFileEntry =
        music_storage::import_audio_only_into_directory(&dest_dir, source_path)?;

    Ok(EmergencyMessageAudioEntry {
        message_type: message_type.to_string(),
        name: imported.name,
        path: imported.path,
    })
}

pub fn remove_emergency_message_audio_file(path: &str) {
    remove_message_audio_file(path);
}

pub fn remove_message_audio_file(path: &str) {
    let file_path = PathBuf::from(path);
    if file_path.exists() {
        let _ = fs::remove_file(file_path);
    }
}

pub fn import_system_message_audio(
    app_data_dir: &Path,
    source_path: &str,
) -> Result<MusicFileEntry, String> {
    let dest_dir = system_messages_root(app_data_dir);
    music_storage::import_audio_only_into_directory(&dest_dir, source_path)
}
