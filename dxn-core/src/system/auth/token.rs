//! JWT Bearer token issue and verification for SA authentication.

use std::collections::HashSet;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Claims stored in the JWT. `sub` is the SA account id.
#[derive(Debug, Serialize, Deserialize)]
pub struct SaClaims {
    /// Subject: SA account id (UUID).
    pub sub: String,
    /// Expiration (seconds since epoch).
    pub exp: i64,
    /// Issued at (seconds since epoch).
    pub iat: i64,
}

/// Default token validity duration (e.g. 24 hours).
const DEFAULT_EXPIRY_HOURS: i64 = 24;

/// Issue a JWT for the given SA id. Signing key is base64-encoded.
pub fn issue_token(sa_id: &str, signing_key_b64: &str) -> Result<String, String> {
    let key_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        signing_key_b64.as_bytes(),
    )
    .map_err(|e| format!("Invalid jwt_signing_key base64: {}", e))?;
    let now = Utc::now();
    let claims = SaClaims {
        sub: sa_id.to_string(),
        exp: (now + Duration::hours(DEFAULT_EXPIRY_HOURS)).timestamp(),
        iat: now.timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&key_bytes),
    )
    .map_err(|e| format!("JWT encode error: {}", e))?;
    Ok(token)
}

/// Verify a Bearer token and return the SA id (sub) if valid.
pub fn verify_token(token: &str, signing_key_b64: &str) -> Result<String, String> {
    let key_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        signing_key_b64.as_bytes(),
    )
    .map_err(|e| format!("Invalid jwt_signing_key base64: {}", e))?;
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.required_spec_claims = ["sub".to_string(), "exp".to_string()]
        .into_iter()
        .collect::<HashSet<_>>();
    let data = decode::<SaClaims>(
        token,
        &DecodingKey::from_secret(&key_bytes),
        &validation,
    )
    .map_err(|e| format!("Invalid or expired token: {}", e))?;
    Ok(data.claims.sub)
}
