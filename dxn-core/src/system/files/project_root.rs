use std::env;
use std::path::{Path, PathBuf};
use serde_json::Value;

const DXN_FILES_DIR: &str = "dxn-files";
const PROJECT_ROOT_ENV: &str = "DXN_PROJECT_ROOT";
const AUTH_ROOT_ENV: &str = "DXN_AUTH_ROOT";

/// Resolve project root once at startup.
///
/// Precedence:
/// 1) DXN_PROJECT_ROOT env var (must contain dxn-files/)
/// 2) Walk up from cwd until dxn-files/ is found
/// 3) Fallback to cwd
pub fn resolve_project_root() -> Result<String, String> {
    if let Ok(env_root) = env::var(PROJECT_ROOT_ENV) {
        let root = PathBuf::from(env_root.trim());
        if root.join(DXN_FILES_DIR).is_dir() {
            return root
                .canonicalize()
                .map(|p| p.to_string_lossy().into_owned())
                .map_err(|e| format!("Failed to canonicalize DXN_PROJECT_ROOT: {e}"));
        }
        return Err(format!(
            "Invalid DXN_PROJECT_ROOT: '{}' does not contain '{}/'",
            root.display(),
            DXN_FILES_DIR
        ));
    }

    let cwd = env::current_dir().map_err(|e| format!("Failed to read current dir: {e}"))?;
    let discovered = find_root_with_dxn_files(&cwd).unwrap_or(cwd);
    discovered
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| format!("Failed to canonicalize resolved project root: {e}"))
}

/// Resolve project root once at startup, with optional config-based override.
///
/// Precedence:
/// 1) DXN_PROJECT_ROOT env var (must contain dxn-files/)
/// 2) config.json settings.projectRoot (must contain dxn-files/)
/// 3) Walk up from cwd until dxn-files/ is found
/// 4) Fallback to cwd
pub fn resolve_project_root_with_config(config_path: &str) -> Result<String, String> {
    if let Ok(env_root) = env::var(PROJECT_ROOT_ENV) {
        let root = PathBuf::from(env_root.trim());
        if root.join(DXN_FILES_DIR).is_dir() {
            return root
                .canonicalize()
                .map(|p| p.to_string_lossy().into_owned())
                .map_err(|e| format!("Failed to canonicalize DXN_PROJECT_ROOT: {e}"));
        }
        return Err(format!(
            "Invalid DXN_PROJECT_ROOT: '{}' does not contain '{}/'",
            root.display(),
            DXN_FILES_DIR
        ));
    }

    if let Some(config_root) = read_project_root_from_config(config_path)? {
        if config_root.join(DXN_FILES_DIR).is_dir() {
            return config_root
                .canonicalize()
                .map(|p| p.to_string_lossy().into_owned())
                .map_err(|e| format!("Failed to canonicalize config projectRoot: {e}"));
        }
        return Err(format!(
            "Invalid config settings.projectRoot: '{}' does not contain '{}/'",
            config_root.display(),
            DXN_FILES_DIR
        ));
    }

    resolve_project_root()
}

/// Returns absolute `project_root/dxn-files` path for the already-resolved project root.
pub fn dxn_files_root(project_root: &str) -> String {
    Path::new(project_root)
        .join(DXN_FILES_DIR)
        .to_string_lossy()
        .into_owned()
}

/// Resolve auth root once at startup.
///
/// Precedence:
/// 1) DXN_AUTH_ROOT env var
/// 2) config.json settings.authRoot (or settings.auth_root)
/// 3) project_root
pub fn resolve_auth_root_with_config(config_path: &str, project_root: &str) -> Result<String, String> {
    if let Ok(env_root) = env::var(AUTH_ROOT_ENV) {
        let root = PathBuf::from(env_root.trim());
        return root
            .canonicalize()
            .map(|p| p.to_string_lossy().into_owned())
            .map_err(|e| format!("Failed to canonicalize DXN_AUTH_ROOT: {e}"));
    }

    if let Some(config_root) = read_auth_root_from_config(config_path)? {
        return config_root
            .canonicalize()
            .map(|p| p.to_string_lossy().into_owned())
            .map_err(|e| format!("Failed to canonicalize config authRoot: {e}"));
    }

    PathBuf::from(project_root)
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| format!("Failed to canonicalize project root for auth fallback: {e}"))
}

fn find_root_with_dxn_files(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(DXN_FILES_DIR).is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn read_project_root_from_config(config_path: &str) -> Result<Option<PathBuf>, String> {
    let config_path_buf = PathBuf::from(config_path);
    if !config_path_buf.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path_buf)
        .map_err(|e| format!("Failed to read config file '{}': {e}", config_path_buf.display()))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config JSON '{}': {e}", config_path_buf.display()))?;

    let from_camel = json
        .get("settings")
        .and_then(|s| s.get("projectRoot"))
        .and_then(|v| v.as_str());
    let from_snake = json
        .get("settings")
        .and_then(|s| s.get("project_root"))
        .and_then(|v| v.as_str());
    let project_root_raw = match from_camel.or(from_snake) {
        Some(v) if !v.trim().is_empty() => v.trim(),
        _ => return Ok(None),
    };

    let configured = PathBuf::from(project_root_raw);
    if configured.is_absolute() {
        return Ok(Some(configured));
    }

    let config_dir = config_path_buf
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(Some(config_dir.join(configured)))
}

fn read_auth_root_from_config(config_path: &str) -> Result<Option<PathBuf>, String> {
    let config_path_buf = PathBuf::from(config_path);
    if !config_path_buf.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path_buf)
        .map_err(|e| format!("Failed to read config file '{}': {e}", config_path_buf.display()))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config JSON '{}': {e}", config_path_buf.display()))?;

    let from_camel = json
        .get("settings")
        .and_then(|s| s.get("authRoot"))
        .and_then(|v| v.as_str());
    let from_snake = json
        .get("settings")
        .and_then(|s| s.get("auth_root"))
        .and_then(|v| v.as_str());
    let auth_root_raw = match from_camel.or(from_snake) {
        Some(v) if !v.trim().is_empty() => v.trim(),
        _ => return Ok(None),
    };

    let configured = PathBuf::from(auth_root_raw);
    if configured.is_absolute() {
        return Ok(Some(configured));
    }

    let config_dir = config_path_buf
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(Some(config_dir.join(configured)))
}
