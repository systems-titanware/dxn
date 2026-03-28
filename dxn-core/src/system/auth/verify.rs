//! Verification of SA credentials: username (plaintext) and password (Argon2id).

use argon2::Argon2;
use subtle::ConstantTimeEq;

use super::sa_identity::{KdfParams, SaIdentity};

/// Verifies that the submitted username matches the stored username (constant-time on bytes).
pub fn verify_username(submitted_username: &str, stored_username: &str) -> bool {
    let submitted_bytes = submitted_username.as_bytes();
    let stored_bytes = stored_username.as_bytes();
    if submitted_bytes.len() != stored_bytes.len() {
        return false;
    }
    submitted_bytes.ct_eq(stored_bytes).into()
}

fn decode_b64(s: &str) -> Option<Vec<u8>> {
    base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        s.trim().as_bytes(),
    )
    .ok()
}

/// Verifies the submitted password against the SA identity using Argon2id.
/// Uses password_salt and kdf_params from the identity; password_hash is the Argon2 raw hash (base64).
pub fn verify_password(submitted_password: &str, identity: &SaIdentity) -> Result<bool, String> {
    let salt = decode_b64(&identity.password_salt)
        .ok_or_else(|| "Invalid password_salt encoding in SA identity.".to_string())?;

    let (memory_kib, iterations, parallelism) = match &identity.kdf_params {
        Some(p) => (p.memory_kib, p.iterations, p.parallelism),
        None => (65536, 3, 4),
    };

    let params = argon2::Params::new(memory_kib, iterations, parallelism, None)
        .map_err(|e| format!("Argon2 params error: {}", e))?;
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );

    // Argon2 output is typically 32 bytes
    let mut out = [0u8; 32];
    argon2
        .hash_password_into(submitted_password.as_bytes(), &salt, &mut out)
        .map_err(|e| format!("Argon2 hash error: {}", e))?;

    let computed_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &out[..],
    );
    let stored = identity.password_hash.trim();
    if computed_b64.len() != stored.len() {
        return Ok(false);
    }
    Ok(computed_b64.as_bytes().ct_eq(stored.as_bytes()).into())
}
