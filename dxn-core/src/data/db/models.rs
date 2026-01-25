
pub struct DbColumn {
    pub(crate) name: String,
    pub(crate) value: Option<String>,
    pub(crate) col_type: String,
    pub(crate) primary_key: Option<bool>,
    pub(crate) secondary_key: Option<bool>,
    pub(crate) nullable: bool,
    pub(crate) unique: Option<bool>,
    /// Default value for the column (e.g., "CURRENT_TIMESTAMP", "0", "''")
    pub(crate) default: Option<String>,
    /// Whether to use AUTOINCREMENT for INTEGER PRIMARY KEY columns
    pub(crate) autoincrement: Option<bool>,
    /// CHECK constraint expression (e.g., "age > 0", "email LIKE '%@%'")
    pub(crate) check: Option<String>,
}