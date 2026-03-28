//! SA (Super Admin) identity loaded from a file. Used only for authentication/verification.
//! File is created by an external process; this module only reads and parses.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Default filename for the SA identity file under the server root.
pub const SA_IDENTITY_FILENAME: &str = ".sa-identity.json";

/// In-memory representation of the SA identity file.
/// Username is stored in plaintext for login; no plaintext password is ever stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaIdentity {
    /// Unique SA account id (e.g. UUID v4/v7).
    pub id: String,
    /// SA login name (plaintext).
    pub username: String,
    /// Argon2id hash of the password (base64 or hex).
    pub password_hash: String,
    /// Random salt used for the Argon2id hash (base64).
    pub password_salt: String,
    /// When the SA was created (ISO 8601), for audit.
    pub created_at: String,
    /// Optional Argon2id params. If absent, use default profile.
    #[serde(default)]
    pub kdf_params: Option<KdfParams>,
    /// Base64-encoded secret used to sign/verify JWT Bearer tokens. Required for token issuance.
    pub jwt_signing_key: String,
}

/// Argon2id parameters (memory, iterations, parallelism).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    #[serde(default = "default_memory")]
    pub memory_kib: u32,
    #[serde(default = "default_iterations")]
    pub iterations: u32,
    #[serde(default = "default_parallelism")]
    pub parallelism: u32,
}

fn default_memory() -> u32 {
    65536
}
fn default_iterations() -> u32 {
    3
}
fn default_parallelism() -> u32 {
    4
}

/// Resolves the absolute path to the SA identity file under the given server root.
pub fn sa_identity_path(server_root: &str) -> std::path::PathBuf {
    Path::new(server_root).join(SA_IDENTITY_FILENAME)
}

/// Load and parse the SA identity file from the server root.
/// Returns `Ok(SaIdentity)` if the file exists and is valid, or an error message.
pub fn load_sa_identity(server_root: &str) -> Result<SaIdentity, String> {
    let path = sa_identity_path(server_root);
    let contents = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "SA identity file not found. Run provisioning to create .sa-identity.json.".to_string()
        } else {
            format!("Failed to read SA identity file: {}", e)
        }
    })?;
    let identity: SaIdentity = serde_json::from_str(&contents)
        .map_err(|e| format!("Invalid SA identity file format: {}", e))?;
    if identity.jwt_signing_key.is_empty() {
        return Err("SA identity file must set jwt_signing_key for token issuance.".to_string());
    }
    Ok(identity)
}
