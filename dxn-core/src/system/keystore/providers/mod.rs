//! Keystore providers — pluggable backends for encrypted key-value storage.

pub mod securestore;

use std::fmt;

// ============================================================================
// ERRORS
// ============================================================================

/// Errors from keystore operations (never include secret values in messages).
#[derive(Debug)]
pub enum KeystoreError {
    /// Secret key does not exist.
    NotFound(String),
    /// Mutex poisoned on the backing store.
    LockPoisoned,
    /// Underlying securestore / crypto error.
    Store(String),
    /// I/O error persisting the vault.
    Io(std::io::Error),
    /// Invalid or empty key name.
    InvalidKey(String),
}

impl fmt::Display for KeystoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeystoreError::NotFound(k) => write!(f, "Keystore key not found: {}", k),
            KeystoreError::LockPoisoned => write!(f, "Keystore lock poisoned"),
            KeystoreError::Store(s) => write!(f, "Keystore error: {}", s),
            KeystoreError::Io(e) => write!(f, "Keystore I/O error: {}", e),
            KeystoreError::InvalidKey(s) => write!(f, "Invalid keystore key: {}", s),
        }
    }
}

impl std::error::Error for KeystoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KeystoreError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for KeystoreError {
    fn from(err: std::io::Error) -> Self {
        KeystoreError::Io(err)
    }
}

// ============================================================================
// TRAIT
// ============================================================================

/// Backend for encrypted get/set/delete of named secrets.
pub trait KeystoreProvider: Send + Sync {
    /// Decrypt and return a UTF-8 string secret.
    fn get(&self, key: &str) -> Result<String, KeystoreError>;

    /// Decrypt and return raw bytes.
    fn get_bytes(&self, key: &str) -> Result<Vec<u8>, KeystoreError>;

    /// Encrypt and store a string secret; persists to disk for SecureStore.
    fn set(&self, key: &str, value: &str) -> Result<(), KeystoreError>;

    /// Encrypt and store binary secret.
    fn set_bytes(&self, key: &str, value: &[u8]) -> Result<(), KeystoreError>;

    /// Remove a secret. Idempotent: missing key is OK.
    fn delete(&self, key: &str) -> Result<(), KeystoreError>;

    fn exists(&self, key: &str) -> Result<bool, KeystoreError>;

    fn list_keys(&self) -> Result<Vec<String>, KeystoreError>;
}

pub(crate) fn validate_key(key: &str) -> Result<(), KeystoreError> {
    if key.trim().is_empty() {
        return Err(KeystoreError::InvalidKey("empty key".to_string()));
    }
    Ok(())
}
