//! Ingest dxn-setup staged secrets into SecureStore, then remove staging (Phase G2).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use securestore::{KeySource, SecretsManager};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::system::keystore::SecureStoreKeystoreProvider;

#[derive(Debug, Deserialize)]
struct SeedManifest {
    format_version: u32,
    entry_count: usize,
    entries_sha256: String,
    entries_file: String,
}

#[derive(Debug)]
pub enum IngestError {
    Io(io::Error),
    InvalidStaging(String),
    SecureStore(String),
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IngestError::Io(e) => write!(f, "{}", e),
            IngestError::InvalidStaging(s) => write!(f, "{}", s),
            IngestError::SecureStore(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for IngestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IngestError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for IngestError {
    fn from(e: io::Error) -> Self {
        IngestError::Io(e)
    }
}

fn map_ss(e: securestore::Error) -> IngestError {
    IngestError::SecureStore(e.to_string())
}

fn valid_seed_key(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
}

/// Parse `secrets.seed`: `key=value`, skip blanks and `#` lines, reject duplicates and bad keys.
pub fn parse_secrets_seed(content: &str) -> Result<Vec<(String, String)>, IngestError> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            return Err(IngestError::InvalidStaging(format!(
                "invalid line (expected key=value): {}",
                line.chars().take(40).collect::<String>()
            )));
        };
        let k = k.trim();
        if !valid_seed_key(k) {
            return Err(IngestError::InvalidStaging(format!(
                "invalid secret key (allowed: ASCII alnum and ._-): {}",
                k
            )));
        }
        if !seen.insert(k.to_string()) {
            return Err(IngestError::InvalidStaging(format!("duplicate key: {}", k)));
        }
        out.push((k.to_string(), v.to_string()));
    }
    Ok(out)
}

fn verify_file_sha256(expected_hex: &str, file_bytes: &[u8]) -> Result<(), IngestError> {
    let mut h = Sha256::new();
    h.update(file_bytes);
    let got = hex::encode(h.finalize());
    let exp = expected_hex.trim().to_ascii_lowercase();
    let got = got.to_ascii_lowercase();
    let exp_b = exp.as_bytes();
    let got_b = got.as_bytes();
    if exp_b.len() != got_b.len() || exp_b.ct_eq(got_b).unwrap_u8() == 0 {
        return Err(IngestError::InvalidStaging(
            "manifest entries_sha256 does not match secrets file contents".to_string(),
        ));
    }
    Ok(())
}

/// True if `dir` looks like a dxn-setup seed directory (manifest + entries file).
pub fn staging_ready(seed_dir: &Path) -> bool {
    seed_dir.is_dir()
        && seed_dir.join("manifest.json").is_file()
        && seed_dir.join("secrets.seed").is_file()
}

/// Ingest staged secrets into the vault at `vault_abs` / `key_abs`, then delete `seed_dir`.
pub fn ingest_staged_secrets(
    seed_dir: &Path,
    vault_abs: &Path,
    key_abs: &Path,
) -> Result<SecureStoreKeystoreProvider, IngestError> {
    let manifest_path = seed_dir.join("manifest.json");
    let manifest_raw = fs::read_to_string(&manifest_path)?;
    let manifest: SeedManifest = serde_json::from_str(&manifest_raw).map_err(|e| {
        IngestError::InvalidStaging(format!("manifest.json: {}", e))
    })?;
    if manifest.format_version != 1 {
        return Err(IngestError::InvalidStaging(format!(
            "unsupported manifest format_version: {}",
            manifest.format_version
        )));
    }

    let entries_path = seed_dir.join(&manifest.entries_file);
    if !entries_path.is_file() {
        return Err(IngestError::InvalidStaging(format!(
            "entries file missing: {}",
            entries_path.display()
        )));
    }
    let entries_bytes = fs::read(&entries_path)?;
    verify_file_sha256(&manifest.entries_sha256, &entries_bytes)?;

    let entries = parse_secrets_seed(&String::from_utf8_lossy(&entries_bytes))?;
    if entries.len() != manifest.entry_count {
        return Err(IngestError::InvalidStaging(format!(
            "entry_count {} does not match parsed entries {}",
            manifest.entry_count,
            entries.len()
        )));
    }

    if let Some(parent) = vault_abs.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = key_abs.parent() {
        fs::create_dir_all(parent)?;
    }

    if vault_abs.is_file() && key_abs.is_file() {
        let mut sm = SecretsManager::load(vault_abs, KeySource::from_file(key_abs)).map_err(map_ss)?;
        for (k, v) in &entries {
            sm.set(k.as_str(), v.as_str());
        }
        sm.save_as(vault_abs).map_err(map_ss)?;
    } else if !vault_abs.exists() && !key_abs.exists() {
        let mut sm = SecretsManager::new(KeySource::Csprng).map_err(map_ss)?;
        for (k, v) in &entries {
            sm.set(k.as_str(), v.as_str());
        }
        sm.save_as(vault_abs).map_err(map_ss)?;
        sm.export_key(key_abs).map_err(map_ss)?;
    } else {
        return Err(IngestError::InvalidStaging(format!(
            "refusing mixed state: vault exists={} key exists={}; remove orphan file or both",
            vault_abs.is_file(),
            key_abs.is_file()
        )));
    }

    fs::remove_dir_all(seed_dir)?;

    SecureStoreKeystoreProvider::open(vault_abs, key_abs).map_err(|e| IngestError::SecureStore(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::keystore::KeystoreProvider;
    use tempfile::TempDir;

    #[test]
    fn parse_seed_skips_comments_and_blank() {
        let s = "# c\n\na.b=1\nx_y-z=2\n";
        let p = parse_secrets_seed(s).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[0], ("a.b".to_string(), "1".to_string()));
    }

    #[test]
    fn ingest_round_trip_removes_staging() {
        let tmp = TempDir::new().unwrap();
        let seed = tmp.path().join("seed");
        fs::create_dir_all(&seed).unwrap();
        let body = "k1=hello\n";
        let mut h = Sha256::new();
        h.update(body.as_bytes());
        let hex = hex::encode(h.finalize());
        fs::write(
            seed.join("manifest.json"),
            serde_json::json!({
                "format_version": 1,
                "entry_count": 1,
                "entries_sha256": hex,
                "entries_file": "secrets.seed"
            })
            .to_string(),
        )
        .unwrap();
        fs::write(seed.join("secrets.seed"), body).unwrap();

        let vault = tmp.path().join("v.json");
        let key = tmp.path().join("v.key");
        let prov = ingest_staged_secrets(&seed, &vault, &key).unwrap();
        assert!(!seed.exists());
        assert!(vault.is_file() && key.is_file());
        assert_eq!(prov.get("k1").unwrap(), "hello");
    }
}
