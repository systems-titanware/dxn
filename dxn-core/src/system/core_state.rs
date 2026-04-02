//! Persistent one-time init marker: `.dxn-core-state.json` next to `config.json`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CORE_STATE_FILENAME: &str = ".dxn-core-state.json";

/// On-disk state written after a successful init pass (checksum, instance binding).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CoreState {
    pub format_version: u32,
    pub initialized: bool,
    pub instance_id: String,
    /// Same form as `.dxn-setup-lock.json` `config_checksum` (`sha256:` + hex).
    pub config_checksum: String,
    /// True after keystore seed staging was absent or successfully ingested and removed.
    #[serde(default)]
    pub keystore_seed_ingested: bool,
    pub migration_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CoreState {
    pub fn new_first_write(
        instance_id: String,
        config_checksum: String,
        keystore_seed_ingested: bool,
    ) -> Self {
        Self {
            format_version: 1,
            initialized: true,
            instance_id,
            config_checksum,
            keystore_seed_ingested,
            migration_version: 1,
            updated_at: Some(chrono::Utc::now()),
        }
    }
}

/// Path to `.dxn-core-state.json` beside `config.json`.
pub fn core_state_path(config_path: &str) -> PathBuf {
    Path::new(config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(CORE_STATE_FILENAME)
}

/// `sha256:` + lowercase hex of raw file bytes (matches dxn-setup lock checksum).
pub fn compute_config_checksum(config_path: &str) -> io::Result<String> {
    let bytes = fs::read(config_path)?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(format!("sha256:{}", hex::encode(h.finalize())))
}

pub fn load_optional(path: &Path) -> io::Result<Option<CoreState>> {
    if !path.is_file() {
        return Ok(None);
    }
    let s = fs::read_to_string(path)?;
    let state: CoreState = serde_json::from_str(&s).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid {}: {}", path.display(), e),
        )
    })?;
    Ok(Some(state))
}

pub fn save(path: &Path, state: &CoreState) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("serialize core state: {}", e),
        )
    })?;
    fs::write(path, json)
}
