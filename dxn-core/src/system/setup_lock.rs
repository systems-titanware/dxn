//! Parsed `.dxn-setup-lock.json` (provisioning metadata from dxn-setup).

use std::fs;
use std::path::{Path, PathBuf};

/// Full lock file content required for vault fingerprint verification and keystore paths.
#[derive(Debug, serde::Deserialize)]
pub struct SetupLock {
    pub instance_id: String,
    pub project_root: String,
    pub keystore_seed_path: String,
    pub keystore: KeystoreLockSection,
}

#[derive(Debug, serde::Deserialize)]
pub struct KeystoreLockSection {
    pub vault_path: String,
    pub key_path: String,
    pub kdf: LockKdf,
    pub key_fingerprint: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct LockKdf {
    pub algorithm: String,
    #[serde(default)]
    pub profile: Option<String>,
    pub salt: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

#[derive(Debug)]
pub enum SetupLockError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
}

impl std::fmt::Display for SetupLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupLockError::Io(e) => write!(f, "{}", e),
            SetupLockError::Json(e) => write!(f, "{}", e),
            SetupLockError::Invalid(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for SetupLockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SetupLockError::Io(e) => Some(e),
            SetupLockError::Json(e) => Some(e),
            SetupLockError::Invalid(_) => None,
        }
    }
}

impl From<std::io::Error> for SetupLockError {
    fn from(e: std::io::Error) -> Self {
        SetupLockError::Io(e)
    }
}

impl From<serde_json::Error> for SetupLockError {
    fn from(e: serde_json::Error) -> Self {
        SetupLockError::Json(e)
    }
}

/// Read and validate structure for supported vault KDF (argon2id only for now).
pub fn load_setup_lock(path: &Path) -> Result<SetupLock, SetupLockError> {
    let bytes = fs::read_to_string(path)?;
    let lock: SetupLock = serde_json::from_str(&bytes)?;
    validate_setup_lock(&lock)?;
    Ok(lock)
}

fn validate_setup_lock(lock: &SetupLock) -> Result<(), SetupLockError> {
    if lock.keystore.kdf.algorithm.to_ascii_lowercase() != "argon2id" {
        return Err(SetupLockError::Invalid(format!(
            "unsupported KDF algorithm in lock: {}",
            lock.keystore.kdf.algorithm
        )));
    }
    if lock.instance_id.trim().is_empty() {
        return Err(SetupLockError::Invalid(
            "lock instance_id is empty".to_string(),
        ));
    }
    Ok(())
}

/// Lock file path next to `config.json`.
pub fn setup_lock_path_for_config(config_path: &str) -> PathBuf {
    Path::new(config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".dxn-setup-lock.json")
}
