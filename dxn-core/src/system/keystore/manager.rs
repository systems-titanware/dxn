//! Thin facade over a [`KeystoreProvider`](super::providers::KeystoreProvider).

use std::sync::Arc;

use super::providers::{KeystoreError, KeystoreProvider};

/// Application-facing keystore handle (swap provider for tests or alternate backends).
pub struct KeystoreManager {
    provider: Arc<dyn KeystoreProvider>,
}

impl std::fmt::Debug for KeystoreManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeystoreManager").finish_non_exhaustive()
    }
}

impl KeystoreManager {
    pub fn new(provider: Arc<dyn KeystoreProvider>) -> Self {
        Self { provider }
    }

    pub fn provider(&self) -> Arc<dyn KeystoreProvider> {
        Arc::clone(&self.provider)
    }

    pub fn get(&self, key: &str) -> Result<String, KeystoreError> {
        self.provider.get(key)
    }

    pub fn get_bytes(&self, key: &str) -> Result<Vec<u8>, KeystoreError> {
        self.provider.get_bytes(key)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), KeystoreError> {
        self.provider.set(key, value)
    }

    pub fn set_bytes(&self, key: &str, value: &[u8]) -> Result<(), KeystoreError> {
        self.provider.set_bytes(key, value)
    }

    pub fn delete(&self, key: &str) -> Result<(), KeystoreError> {
        self.provider.delete(key)
    }

    pub fn exists(&self, key: &str) -> Result<bool, KeystoreError> {
        self.provider.exists(key)
    }

    pub fn list_keys(&self) -> Result<Vec<String>, KeystoreError> {
        self.provider.list_keys()
    }
}
