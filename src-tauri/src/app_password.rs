use rand::Rng;
use sha2::{Digest, Sha256};

pub fn hash_password(password: &str) -> String {
    let salt: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    let digest = sha256_hex(&salt, password);
    format!("{salt}:{digest}")
}

pub fn verify_password(password: &str, stored: &str) -> bool {
    let Some((salt, expected)) = stored.split_once(':') else {
        return false;
    };
    sha256_hex(salt, password) == expected
}

fn sha256_hex(salt: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}
