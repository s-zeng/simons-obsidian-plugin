use std::fmt;

/// Custom error types for the plugin
/// Follows functional programming principles with ADT pattern
#[derive(Debug, Clone, PartialEq)]
pub enum PluginError {
    /// Validation error with context about which field and why it failed
    ValidationError {
        field: String,
        value: String,
        reason: String,
    },
    /// Serialization/deserialization error
    SerializationError {
        context: String,
        source: String,
    },
    /// Unknown setting key
    UnknownSetting {
        key: String,
    },
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::ValidationError { field, value, reason } => {
                write!(
                    f,
                    "Validation failed for field '{}' with value '{}': {}",
                    field, value, reason
                )
            }
            PluginError::SerializationError { context, source } => {
                write!(f, "Serialization error in {}: {}", context, source)
            }
            PluginError::UnknownSetting { key } => {
                write!(f, "Unknown setting key: '{}'", key)
            }
        }
    }
}

impl std::error::Error for PluginError {}

/// Convert PluginError to JsValue for WASM boundary
impl From<PluginError> for wasm_bindgen::JsValue {
    fn from(err: PluginError) -> Self {
        wasm_bindgen::JsValue::from_str(&err.to_string())
    }
}
