//! Default keystore provider using the [securestore](https://docs.rs/securestore) crate.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use securestore::{ErrorKind, KeySource, SecretsManager};

use super::{validate_key, KeystoreError, KeystoreProvider};

/// Encrypted JSON vault backed by `SecretsManager`. All mutations call `save_as` to the vault path.
pub struct SecureStoreKeystoreProvider {
    vault_path: PathBuf,
    inner: Mutex<SecretsManager>,
}

impl SecureStoreKeystoreProvider {
    /// Open an existing vault file with a key file (e.g. from `ssclient` or `export_key`).
    pub fn open(
        vault_path: impl AsRef<Path>,
        key_file: impl AsRef<Path>,
    ) -> Result<Self, KeystoreError> {
        let vault_path = vault_path.as_ref().to_path_buf();
        let sm = SecretsManager::load(&vault_path, KeySource::from_file(key_file.as_ref()))
            .map_err(map_securestore_err)?;
        Ok(Self {
            vault_path,
            inner: Mutex::new(sm),
        })
    }

    /// Open an existing vault using a password-derived key.
    pub fn open_with_password(
        vault_path: impl AsRef<Path>,
        password: &str,
    ) -> Result<Self, KeystoreError> {
        let vault_path = vault_path.as_ref().to_path_buf();
        let sm = SecretsManager::load(&vault_path, KeySource::Password(password))
            .map_err(map_securestore_err)?;
        Ok(Self {
            vault_path,
            inner: Mutex::new(sm),
        })
    }

    /// Create a new empty vault with a password and persist to `vault_path`. For tests and first-run tooling.
    pub fn create_with_password(
        vault_path: impl AsRef<Path>,
        password: &str,
    ) -> Result<Self, KeystoreError> {
        let vault_path = vault_path.as_ref().to_path_buf();
        let sm = SecretsManager::new(KeySource::Password(password)).map_err(map_securestore_err)?;
        sm.save_as(&vault_path).map_err(map_securestore_err)?;
        Ok(Self {
            vault_path,
            inner: Mutex::new(sm),
        })
    }

    fn with_mut<T>(&self, f: impl FnOnce(&mut SecretsManager) -> Result<T, KeystoreError>) -> Result<T, KeystoreError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| KeystoreError::LockPoisoned)?;
        f(&mut *guard)
    }

    fn persist(&self, sm: &mut SecretsManager) -> Result<(), KeystoreError> {
        sm.save_as(&self.vault_path).map_err(map_securestore_err)
    }
}

fn map_securestore_err(e: securestore::Error) -> KeystoreError {
    match e.kind() {
        ErrorKind::SecretNotFound => KeystoreError::NotFound(e.to_string()),
        _ => KeystoreError::Store(e.to_string()),
    }
}

impl KeystoreProvider for SecureStoreKeystoreProvider {
    fn get(&self, key: &str) -> Result<String, KeystoreError> {
        validate_key(key)?;
        self.with_mut(|sm| {
            sm.get(key).map_err(map_securestore_err)
        })
    }

    fn get_bytes(&self, key: &str) -> Result<Vec<u8>, KeystoreError> {
        validate_key(key)?;
        self.with_mut(|sm| {
            sm.get_as::<Vec<u8>>(key).map_err(map_securestore_err)
        })
    }

    fn set(&self, key: &str, value: &str) -> Result<(), KeystoreError> {
        validate_key(key)?;
        self.with_mut(|sm| {
            sm.set(key, value);
            self.persist(sm)
        })
    }

    fn set_bytes(&self, key: &str, value: &[u8]) -> Result<(), KeystoreError> {
        validate_key(key)?;
        self.with_mut(|sm| {
            sm.set(key, value);
            self.persist(sm)
        })
    }

    fn delete(&self, key: &str) -> Result<(), KeystoreError> {
        validate_key(key)?;
        self.with_mut(|sm| {
            match sm.remove(key) {
                Ok(()) => self.persist(sm),
                Err(e) if matches!(e.kind(), ErrorKind::SecretNotFound) => Ok(()),
                Err(e) => Err(map_securestore_err(e)),
            }
        })
    }

    fn exists(&self, key: &str) -> Result<bool, KeystoreError> {
        validate_key(key)?;
        self.with_mut(|sm| Ok(sm.keys().any(|k| k == key)))
    }

    fn list_keys(&self) -> Result<Vec<String>, KeystoreError> {
        self.with_mut(|sm| Ok(sm.keys().map(|k| k.to_string()).collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn password_vault_round_trip() {
        let dir = TempDir::new().unwrap();
        let vault = dir.path().join("keystore.json");
        let prov =
            SecureStoreKeystoreProvider::create_with_password(&vault, "test-vault-pass").unwrap();
        assert!(vault.is_file());

        prov.set("db_password", "secret123").unwrap();
        assert_eq!(prov.get("db_password").unwrap(), "secret123");
        assert!(prov.exists("db_password").unwrap());
        assert!(prov.list_keys().unwrap().contains(&"db_password".to_string()));

        prov.set_bytes("blob", b"\0\xff").unwrap();
        assert_eq!(prov.get_bytes("blob").unwrap(), vec![0u8, 255u8]);

        prov.delete("db_password").unwrap();
        assert!(!prov.exists("db_password").unwrap());
        prov.delete("db_password").unwrap();

        let loaded = SecureStoreKeystoreProvider::open_with_password(&vault, "test-vault-pass").unwrap();
        assert!(loaded.exists("blob").unwrap());
        assert_eq!(loaded.get_bytes("blob").unwrap(), vec![0u8, 255u8]);
    }

    #[test]
    fn csprng_keyfile_round_trip() {
        let dir = TempDir::new().unwrap();
        let vault = dir.path().join("vault.json");
        let key_path = dir.path().join("vault.key");

        let mut sm = SecretsManager::new(KeySource::Csprng).expect("new vault");
        sm.set("k", "v");
        sm.save_as(&vault).expect("save_as");
        sm.export_key(&key_path).expect("export_key");
        drop(sm);

        let prov = SecureStoreKeystoreProvider::open(&vault, &key_path).expect("open");
        assert_eq!(prov.get("k").unwrap(), "v");

        prov.set("k", "v2").unwrap();
        let prov2 = SecureStoreKeystoreProvider::open(&vault, &key_path).expect("reopen");
        assert_eq!(prov2.get("k").unwrap(), "v2");
    }
}
