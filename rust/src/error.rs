use std::fmt;

/// Custom error types for the plugin.
///
/// Follows functional programming principles with ADT pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    /// Validation error with context about which field and why it failed.
    ValidationError {
        /// The field name that failed validation
        field: String,
        /// The invalid value
        value: String,
        /// Explanation of why validation failed
        reason: String,
    },
    /// Serialization/deserialization error.
    SerializationError {
        /// Context where the error occurred
        context: String,
        /// The underlying error message
        source: String,
    },
    /// Unknown setting key.
    UnknownSetting {
        /// The unrecognized setting key
        key: String,
    },
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationError { field, value, reason } => {
                write!(f, "Validation failed for field '{field}' with value '{value}': {reason}")
            },
            Self::SerializationError { context, source } => {
                write!(f, "Serialization error in {context}: {source}")
            },
            Self::UnknownSetting { key } => {
                write!(f, "Unknown setting key: '{key}'")
            },
        }
    }
}

impl std::error::Error for PluginError {}

/// Convert `PluginError` to `JsValue` for WASM boundary
impl From<PluginError> for wasm_bindgen::JsValue {
    fn from(err: PluginError) -> Self {
        Self::from_str(&err.to_string())
    }
}
