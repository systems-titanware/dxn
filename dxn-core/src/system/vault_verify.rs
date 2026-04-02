//! Shared-secret verification against `.dxn-setup-lock.json` (Argon2id + HMAC-SHA256 fingerprint).
//!
//! Must match dxn-setup: `key_fingerprint = HMAC-SHA256(derived_key, "DXN_VAULT")` with derived key
//! from Argon2id using the provisioning shared secret and lock salt/params.

use argon2::{Argon2, Params};
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use super::setup_lock::LockKdf;

type HmacSha256 = Hmac<Sha256>;

const VAULT_FINGERPRINT_MSG: &[u8] = b"DXN_VAULT";
const DERIVED_KEY_LEN: usize = 32;

#[derive(Debug)]
pub enum VaultVerifyError {
    MissingSalt,
    Argon2Params(String),
    Argon2Hash,
    FingerprintMismatch,
    InvalidFingerprintEncoding,
}

impl std::fmt::Display for VaultVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultVerifyError::MissingSalt => write!(f, "KDF salt is missing or invalid"),
            VaultVerifyError::Argon2Params(s) => write!(f, "Argon2 params: {}", s),
            VaultVerifyError::Argon2Hash => write!(f, "Argon2 key derivation failed"),
            VaultVerifyError::FingerprintMismatch => {
                write!(f, "invalid shared secret for this server instance (fingerprint mismatch)")
            }
            VaultVerifyError::InvalidFingerprintEncoding => {
                write!(f, "stored key fingerprint is not valid base64")
            }
        }
    }
}

impl std::error::Error for VaultVerifyError {}

fn decode_salt_b64(salt_b64: &str) -> Result<Vec<u8>, VaultVerifyError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(salt_b64.trim().as_bytes())
        .map_err(|_| VaultVerifyError::MissingSalt)?;
    if bytes.is_empty() {
        return Err(VaultVerifyError::MissingSalt);
    }
    Ok(bytes)
}

/// Derive the vault key bytes (same length as Argon2 output) from the shared secret and lock KDF block.
pub fn derive_vault_key(secret: &str, kdf: &LockKdf) -> Result<[u8; DERIVED_KEY_LEN], VaultVerifyError> {
    let salt = decode_salt_b64(&kdf.salt)?;
    let params = Params::new(
        kdf.memory_kib,
        kdf.iterations,
        kdf.parallelism,
        Some(DERIVED_KEY_LEN),
    )
    .map_err(|e| VaultVerifyError::Argon2Params(e.to_string()))?;
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );
    let mut out = [0u8; DERIVED_KEY_LEN];
    argon2
        .hash_password_into(secret.as_bytes(), &salt, &mut out)
        .map_err(|_| VaultVerifyError::Argon2Hash)?;
    Ok(out)
}

/// HMAC-SHA256(derived_key, "DXN_VAULT") as raw bytes (32 bytes).
pub fn fingerprint_bytes_from_derived_key(derived_key: &[u8; DERIVED_KEY_LEN]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(derived_key.as_slice())
        .expect("HMAC key length is valid for sha256");
    mac.update(VAULT_FINGERPRINT_MSG);
    let out = mac.finalize().into_bytes();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out[..32]);
    arr
}

/// Verify `DXN_CORE_SECRET` against the lock file fingerprint (base64).
pub fn verify_vault_fingerprint(
    secret: &str,
    kdf: &LockKdf,
    expected_fingerprint_b64: &str,
) -> Result<(), VaultVerifyError> {
    let derived = derive_vault_key(secret, kdf)?;
    let computed = fingerprint_bytes_from_derived_key(&derived);
    let expected_raw = base64::engine::general_purpose::STANDARD
        .decode(expected_fingerprint_b64.trim().as_bytes())
        .map_err(|_| VaultVerifyError::InvalidFingerprintEncoding)?;
    if expected_raw.len() != 32 {
        return Err(VaultVerifyError::InvalidFingerprintEncoding);
    }
    let mut expected_arr = [0u8; 32];
    expected_arr.copy_from_slice(&expected_raw[..32]);
    if computed.ct_eq(&expected_arr).into() {
        Ok(())
    } else {
        Err(VaultVerifyError::FingerprintMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::setup_lock::LockKdf;

    #[test]
    fn fingerprint_round_trip_self_consistent() {
        let kdf = LockKdf {
            algorithm: "argon2id".to_string(),
            profile: Some("standard".to_string()),
            salt: base64::engine::general_purpose::STANDARD.encode(b"testsalt-16bytes"),
            memory_kib: 65536,
            iterations: 3,
            parallelism: 4,
        };
        let secret = "my-shared-secret";
        let derived = derive_vault_key(secret, &kdf).expect("derive");
        let fp = fingerprint_bytes_from_derived_key(&derived);
        let b64 = base64::engine::general_purpose::STANDARD.encode(fp);
        verify_vault_fingerprint(secret, &kdf, &b64).expect("verify");
        assert!(verify_vault_fingerprint("wrong", &kdf, &b64).is_err());
    }
}
