use crate::app_password::{hash_password, verify_password};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const RESET_PASSWORD: &str = "1234";

const STORE_DIR: &str = ".ndata";
const FILE_EXTENSION: &str = "bix";
const MAGIC: [u8; 4] = [0x51, 0x37, 0x58, 0x91];
const VERSION: u8 = 1;
const ROLE_APP: u8 = 1;
const ROLE_SETTINGS: u8 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PasswordKind {
    App,
    Settings,
}

impl PasswordKind {
    fn role_byte(self) -> u8 {
        match self {
            Self::App => ROLE_APP,
            Self::Settings => ROLE_SETTINGS,
        }
    }

    fn from_role_byte(value: u8) -> Option<Self> {
        match value {
            ROLE_APP => Some(Self::App),
            ROLE_SETTINGS => Some(Self::Settings),
            _ => None,
        }
    }
}

pub struct PasswordVault {
    root: PathBuf,
}

impl PasswordVault {
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let root = app_data_dir.join(STORE_DIR);
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&root, fs::Permissions::from_mode(0o700));
        }
        Ok(Self { root })
    }

    pub fn store_dir_name() -> &'static str {
        STORE_DIR
    }

    pub fn clear_all(&self) -> Result<(), String> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).map_err(|e| e.to_string())?;
        }
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.root, fs::Permissions::from_mode(0o700));
        }
        Ok(())
    }

    pub fn migrate_from_db(&self, db: &crate::db::DbState) -> Result<(), String> {
        self.migrate_kind_from_db(PasswordKind::App, crate::db::APP_PASSWORD_HASH_KEY, db)?;
        self.migrate_kind_from_db(
            PasswordKind::Settings,
            crate::db::SETTINGS_PASSWORD_HASH_KEY,
            db,
        )?;
        Ok(())
    }

    fn migrate_kind_from_db(
        &self,
        kind: PasswordKind,
        key: &str,
        db: &crate::db::DbState,
    ) -> Result<(), String> {
        if self.find_store_path(kind)?.is_some() {
            let _ = db.clear_setting_key(key);
            return Ok(());
        }
        let Some(hash) = db
            .get_setting(key)
            .map_err(|e| e.to_string())?
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };
        self.write_store(kind, &hash)?;
        let _ = db.clear_setting_key(key);
        Ok(())
    }

    pub fn has_password(&self, kind: PasswordKind) -> Result<bool, String> {
        self.apply_reset_if_needed(kind)?;
        Ok(self.find_store_path(kind)?.is_some())
    }

    pub fn verify(&self, kind: PasswordKind, password: &str) -> Result<bool, String> {
        self.apply_reset_if_needed(kind)?;
        let Some(path) = self.find_store_path(kind)? else {
            return Ok(false);
        };
        let record = read_store(&path)?;
        Ok(verify_password(password, &record.password_hash))
    }

    pub fn set_password(&self, kind: PasswordKind, password: &str) -> Result<(), String> {
        let trimmed = password.trim();
        if trimmed.is_empty() {
            return Err("הסיסמה לא יכולה להיות ריקה".to_string());
        }
        if trimmed.chars().count() < 4 {
            return Err("הסיסמה חייבת להכיל לפחות 4 תווים".to_string());
        }
        let _ = self.apply_reset_if_needed(kind);
        self.remove_kind(kind)?;
        let hash = hash_password(trimmed);
        self.write_store(kind, &hash)
    }

    pub fn clear_password(&self, kind: PasswordKind) -> Result<(), String> {
        self.remove_kind(kind)
    }

    fn apply_reset_if_needed(&self, kind: PasswordKind) -> Result<(), String> {
        let Some(store_path) = self.find_store_path(kind)? else {
            return Ok(());
        };
        let record = match read_store(&store_path) {
            Ok(record) => record,
            Err(_) => {
                self.reset_to_default(kind)?;
                return Ok(());
            }
        };
        let sentinel_path = sentinel_path_for(&store_path);
        let needs_reset = match fs::read(&sentinel_path) {
            Ok(bytes) => sha256_bytes(&bytes) != record.sentinel_hash,
            Err(_) => true,
        };
        if needs_reset {
            self.reset_to_default(kind)?;
        }
        Ok(())
    }

    fn reset_to_default(&self, kind: PasswordKind) -> Result<(), String> {
        self.remove_kind(kind)?;
        let hash = hash_password(RESET_PASSWORD);
        self.write_store(kind, &hash)
    }

    fn write_store(&self, kind: PasswordKind, password_hash: &str) -> Result<(), String> {
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let basename = random_basename();
        let store_path = self.root.join(format!("{basename}.{FILE_EXTENSION}"));
        let sentinel_path = sentinel_path_for(&store_path);

        let mut sentinel_bytes = vec![0u8; 64];
        rand::thread_rng().fill_bytes(&mut sentinel_bytes);
        let sentinel_hash = sha256_bytes(&sentinel_bytes);

        let blob = encode_store(kind, password_hash, &sentinel_hash)?;
        fs::write(&store_path, blob).map_err(|e| e.to_string())?;
        fs::write(&sentinel_path, sentinel_bytes).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn remove_kind(&self, kind: PasswordKind) -> Result<(), String> {
        if let Some(path) = self.find_store_path(kind)? {
            let sentinel = sentinel_path_for(&path);
            let _ = fs::remove_file(&path);
            let _ = fs::remove_file(&sentinel);
        }
        Ok(())
    }

    fn find_store_path(&self, kind: PasswordKind) -> Result<Option<PathBuf>, String> {
        if !self.root.exists() {
            return Ok(None);
        }
        let entries = fs::read_dir(&self.root).map_err(|e| e.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = match path.file_name().and_then(|value| value.to_str()) {
                Some(name) => name,
                None => continue,
            };
            if name.ends_with("++") {
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some(FILE_EXTENSION) {
                continue;
            }
            if let Ok(record) = read_store(&path) {
                if record.kind == kind {
                    return Ok(Some(path));
                }
            }
        }
        Ok(None)
    }
}

struct StoreRecord {
    kind: PasswordKind,
    password_hash: String,
    sentinel_hash: [u8; 32],
}

fn sentinel_path_for(store_path: &Path) -> PathBuf {
    let name = store_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("store.bix");
    store_path.with_file_name(format!("{name}++"))
}

fn random_basename() -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    let mut out = String::with_capacity(12);
    for _ in 0..12 {
        let idx = (rng.next_u32() as usize) % ALPHABET.len();
        out.push(ALPHABET[idx] as char);
    }
    out
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn encode_store(
    kind: PasswordKind,
    password_hash: &str,
    sentinel_hash: &[u8; 32],
) -> Result<Vec<u8>, String> {
    let hash_bytes = password_hash.as_bytes();
    if hash_bytes.len() > u16::MAX as usize {
        return Err("נתוני סיסמה ארוכים מדי".to_string());
    }

    let mut plain = Vec::with_capacity(2 + hash_bytes.len() + 32);
    plain.extend_from_slice(&(hash_bytes.len() as u16).to_le_bytes());
    plain.extend_from_slice(hash_bytes);
    plain.extend_from_slice(sentinel_hash);

    let mut key = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut key);
    let encrypted = xor_bytes(&plain, &key);

    let mut out = Vec::with_capacity(4 + 1 + 1 + 16 + 2 + encrypted.len());
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.push(kind.role_byte());
    out.extend_from_slice(&key);
    out.extend_from_slice(&(encrypted.len() as u16).to_le_bytes());
    out.extend_from_slice(&encrypted);
    Ok(out)
}

fn read_store(path: &Path) -> Result<StoreRecord, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() < 4 + 1 + 1 + 16 + 2 {
        return Err("קובץ נתונים פגום".to_string());
    }
    if bytes[0..4] != MAGIC {
        return Err("קובץ נתונים לא מזוהה".to_string());
    }
    if bytes[4] != VERSION {
        return Err("גרסת קובץ נתונים לא נתמכת".to_string());
    }
    let kind = PasswordKind::from_role_byte(bytes[5])
        .ok_or_else(|| "סוג קובץ נתונים לא תקין".to_string())?;
    let key: [u8; 16] = bytes[6..22]
        .try_into()
        .map_err(|_| "מפתח קובץ פגום".to_string())?;
    let enc_len = u16::from_le_bytes([bytes[22], bytes[23]]) as usize;
    if bytes.len() < 24 + enc_len {
        return Err("קובץ נתונים חתוך".to_string());
    }
    let plain = xor_bytes(&bytes[24..24 + enc_len], &key);
    if plain.len() < 2 + 32 {
        return Err("מטען קובץ פגום".to_string());
    }
    let hash_len = u16::from_le_bytes([plain[0], plain[1]]) as usize;
    if plain.len() < 2 + hash_len + 32 {
        return Err("מטען קובץ חתוך".to_string());
    }
    let password_hash = String::from_utf8(plain[2..2 + hash_len].to_vec())
        .map_err(|_| "מטען קובץ לא תקין".to_string())?;
    let mut sentinel_hash = [0u8; 32];
    sentinel_hash.copy_from_slice(&plain[2 + hash_len..2 + hash_len + 32]);
    Ok(StoreRecord {
        kind,
        password_hash,
        sentinel_hash,
    })
}

fn xor_bytes(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(index, byte)| byte ^ key[index % key.len()])
        .collect()
}
