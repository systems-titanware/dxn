//! SA (Super Admin) file-based authentication: load identity from .sa-identity.json,
//! verify username (hash) + password (Argon2id), issue/verify JWT Bearer tokens.

pub mod controller;
pub mod middleware;
pub mod sa_identity;
pub mod token;
pub mod verify;
