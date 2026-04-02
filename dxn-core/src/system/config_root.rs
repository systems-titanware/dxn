//! Full `config.json` envelope: `System` plus provisioning fields (`settings`, `keystore`, `vault`).

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::system::models::System;
use crate::system::setup_lock::SetupLock;

/// Top-level config.json shape (includes blocks ignored by `System` alone).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRoot {
    #[serde(default)]
    pub settings: Option<ConfigSettings>,
    #[serde(default)]
    pub keystore: Option<KeystoreConfig>,
    #[serde(default)]
    pub vault: Option<VaultConfig>,
    #[serde(flatten)]
    pub system: System,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSettings {
    #[serde(default)]
    pub project_root: Option<String>,
    #[serde(default)]
    pub auth_root: Option<String>,
    #[serde(default)]
    pub keystore_seed_path: Option<String>,
}

/// dxn-setup writes **snake_case** keys (`vault_path`, `key_path`). Aliases accept camelCase if templates use it.
#[derive(Debug, Deserialize)]
pub struct KeystoreConfig {
    #[serde(alias = "vaultPath")]
    pub vault_path: String,
    #[serde(alias = "keyPath")]
    pub key_path: String,
}

#[derive(Debug, Deserialize)]
pub struct VaultConfig {
    #[serde(default, alias = "saltB64")]
    pub salt_b64: Option<String>,
    #[serde(default, alias = "keyFingerprintB64")]
    pub key_fingerprint_b64: Option<String>,
}

/// Ensure `config.json` vault metadata matches the lock when both are present.
pub fn cross_check_vault_with_lock(
    lock: &SetupLock,
    vault: Option<&VaultConfig>,
) -> Result<(), String> {
    let Some(v) = vault else {
        return Ok(());
    };
    if let Some(ref salt) = v.salt_b64 {
        if salt.trim() != lock.keystore.kdf.salt.trim() {
            return Err(
                "config.json vault.salt_b64 does not match .dxn-setup-lock.json keystore.kdf.salt"
                    .to_string(),
            );
        }
    }
    if let Some(ref fp) = v.key_fingerprint_b64 {
        if fp.trim() != lock.keystore.key_fingerprint.trim() {
            return Err(
                "config.json vault.key_fingerprint_b64 does not match lock keystore.key_fingerprint"
                    .to_string(),
            );
        }
    }
    Ok(())
}

/// Prefer `config.json` keystore paths; fall back to lock.
pub fn keystore_paths<'a>(
    lock: &'a SetupLock,
    config: &'a ConfigRoot,
) -> (&'a str, &'a str) {
    if let Some(ref k) = config.keystore {
        return (k.vault_path.as_str(), k.key_path.as_str());
    }
    (
        lock.keystore.vault_path.as_str(),
        lock.keystore.key_path.as_str(),
    )
}

/// Staging directory for keystore seeds. **Lock wins** when non-empty; else `settings.keystoreSeedPath`.
/// Relative paths resolve against the directory containing `config.json`.
pub fn resolve_keystore_seed_path(
    lock: &SetupLock,
    config: &ConfigRoot,
    config_path: &str,
) -> Option<PathBuf> {
    let raw = if !lock.keystore_seed_path.trim().is_empty() {
        lock.keystore_seed_path.trim()
    } else {
        config
            .settings
            .as_ref()
            .and_then(|s| s.keystore_seed_path.as_deref())
            .unwrap_or("")
            .trim()
    };
    if raw.is_empty() {
        return None;
    }
    let p = PathBuf::from(raw);
    Some(if p.is_absolute() {
        p
    } else {
        Path::new(config_path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(raw.trim_start_matches("./"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_keystore_and_vault_snake_case() {
        let j = r#"{
            "data": { "public": [], "private": null },
            "server": { "public": null, "private": null },
            "integrations": { "public": null, "private": null },
            "functions": { "public": null, "private": null },
            "keystore": {
                "vault_path": "./dxn-files/_vault/keystore.json",
                "key_path": "./dxn-files/_vault/keystore.key"
            },
            "vault": {
                "salt_b64": "abcd",
                "key_fingerprint_b64": "efgh=="
            }
        }"#;
        let r: ConfigRoot = serde_json::from_str(j).expect("parse");
        let k = r.keystore.as_ref().expect("keystore");
        assert!(k.vault_path.ends_with("keystore.json"));
        assert!(k.key_path.ends_with("keystore.key"));
        let v = r.vault.as_ref().expect("vault");
        assert_eq!(v.salt_b64.as_deref(), Some("abcd"));
    }
}
