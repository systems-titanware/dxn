//! Encrypted key-value keystore (manager + providers).
//!
//! See `examples/z.documents/keystore.md` for design.

pub mod manager;
pub mod providers;

pub use manager::KeystoreManager;
pub use providers::securestore::SecureStoreKeystoreProvider;
pub use providers::{KeystoreError, KeystoreProvider};
