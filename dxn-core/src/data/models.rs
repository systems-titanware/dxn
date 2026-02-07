use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemData { 
    pub(crate) public: Option<Vec<SystemDataModel>>, 
    pub(crate) private: Option<Vec<SystemDataModel>>,
}

/// Schema status for soft delete support
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaStatus {
    Active,
    Deleted,
}

impl Default for SchemaStatus {
    fn default() -> Self {
        SchemaStatus::Active
    }
}

impl SchemaStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SchemaStatus::Active => "active",
            SchemaStatus::Deleted => "deleted",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "deleted" => SchemaStatus::Deleted,
            _ => SchemaStatus::Active,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemDataModel {
    pub(crate) name: String,
    pub(crate) version: u32,
    #[serde(default = "default_db")]
    pub(crate) db: String,
    #[serde(default)]
    pub(crate) public: bool,
    #[serde(default)]
    pub(crate) source: Option<String>,
    /// Icon for the schema (emoji or icon identifier, e.g., "📦", "mdi-account")
    #[serde(default)]
    pub(crate) icon: Option<String>,
    /// Schema status: "active" or "deleted" (soft delete)
    #[serde(default)]
    pub(crate) status: SchemaStatus,
    /// Timestamp when schema was soft-deleted (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) deleted_at: Option<String>,
    pub(crate) fields: Vec<SystemDataModelField>
}

fn default_db() -> String {
    "public".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemDataModelField {
    pub(crate) name: String,
    pub(crate) datatype: String,
    pub(crate) value: String,
    pub(crate) primary: Option<bool>,
    pub(crate) secondary: Option<bool>
}


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemDataModelMetadata {
    pub(crate) name: String,
    pub(crate) version: u32,
    pub(crate) fields: String
}

/// Deserialize Option<u8> from either a number or a string (e.g. query params are always strings).
fn deserialize_option_u8_from_str_or_num<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNum {
        S(String),
        N(u64),
    }
    let opt = Option::<StringOrNum>::deserialize(deserializer)?;
    let value = match opt {
        None => return Ok(None),
        Some(StringOrNum::S(s)) => s.parse().map_err(serde::de::Error::custom)?,
        Some(StringOrNum::N(n)) => u8::try_from(n).map_err(serde::de::Error::custom)?,
    };
    Ok(Some(value))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryParams {
    #[serde(default, deserialize_with = "deserialize_option_u8_from_str_or_num")]
    pub(crate) page: Option<u8>,
    #[serde(default, deserialize_with = "deserialize_option_u8_from_str_or_num")]
    pub(crate) page_size: Option<u8>,
    pub(crate) query: Option<String>,
}

// ============================================================================
// EVENT SOURCING MODELS
// ============================================================================

/// Represents a domain event in the event store.
/// Events are immutable records of state changes.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Unique identifier for the event (UUID v7)
    pub id: String,
    /// The entity this event relates to (e.g., order ID, profile ID)
    pub aggregate_id: String,
    /// The schema/entity type (e.g., "order", "profile")
    pub schema_name: String,
    /// The type of event that occurred
    pub event_type: EventType,
    /// The data associated with this event (JSON)
    pub payload: serde_json::Value,
    /// The state before this change (optional, for debugging/rollback)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<serde_json::Value>,
    /// Version number for optimistic concurrency control
    pub version: u32,
    /// Who triggered this event (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// When the event occurred
    pub timestamp: String,
}

/// The type of event that occurred
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    /// Entity was created
    Created,
    /// Entity was updated
    Updated,
    /// Entity was deleted
    Deleted,
    /// Custom domain event (e.g., "shipped", "cancelled")
    #[serde(untagged)]
    Custom(String),
}

impl EventType {
    /// Convert EventType to a string representation
    pub fn as_str(&self) -> &str {
        match self {
            EventType::Created => "created",
            EventType::Updated => "updated",
            EventType::Deleted => "deleted",
            EventType::Custom(s) => s.as_str(),
        }
    }
    
    /// Create EventType from a string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "created" => EventType::Created,
            "updated" => EventType::Updated,
            "deleted" => EventType::Deleted,
            other => EventType::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Query parameters for event retrieval
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventQueryParams {
    /// Filter events after this timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// Filter events before this timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    /// Filter by event type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    /// Maximum number of events to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Number of events to skip (for pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

// ============================================================================
// FILE STORAGE MODELS
// ============================================================================

/// Root files configuration from config.json
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemFiles {
    pub directories: Option<Vec<SystemFileDirectory>>,
}

/// A configured file directory
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemFileDirectory {
    /// Unique name for this directory configuration
    pub name: String,
    /// Provider type: "local", "sftp", "s3", etc.
    #[serde(default = "default_provider")]
    pub provider: String,
    /// Base path for this directory (relative to provider root)
    pub path: String,
    /// Icon for the directory (emoji or icon identifier)
    #[serde(default)]
    pub icon: Option<String>,
    /// Source of the config ("config" or "runtime")
    #[serde(default)]
    pub source: Option<String>,
    /// Provider-specific configuration (credentials, options, etc.)
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

fn default_provider() -> String {
    "local".to_string()
}

/// Represents a file or folder entry in a directory listing
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// File or directory name
    pub name: String,
    /// Full path relative to the directory root
    pub path: String,
    /// True if this is a directory, false if it's a file
    pub is_directory: bool,
    /// File size in bytes (None for directories)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Last modified timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// MIME type (for files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}