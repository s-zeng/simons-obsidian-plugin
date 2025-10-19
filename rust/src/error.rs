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
    /// Dimensionality reduction error.
    DimensionalityReductionError {
        /// The reduction method that failed
        method: String,
        /// Explanation of why it failed
        reason: String,
    },
    /// Invalid vector dimensions.
    InvalidVectorDimensions {
        /// Expected dimensionality
        expected: usize,
        /// Actual dimensionality received
        got: usize,
        /// Index of the problematic vector
        vector_index: usize,
    },
    /// Insufficient data for operation.
    InsufficientData {
        /// Required minimum data points
        required: usize,
        /// Actual data points provided
        provided: usize,
    },
    /// Invalid link index in adjacency matrix.
    InvalidLinkIndex {
        /// Source note index
        from: usize,
        /// Target note index
        to: usize,
        /// Maximum valid index
        max: usize,
    },
    /// Zero norm vector encountered.
    ZeroNormVector,
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
            Self::DimensionalityReductionError { method, reason } => {
                write!(f, "Dimensionality reduction failed for method '{method}': {reason}")
            },
            Self::InvalidVectorDimensions { expected, got, vector_index } => {
                write!(
                    f,
                    "Invalid vector dimensions at index {vector_index}: expected {expected}, got {got}"
                )
            },
            Self::InsufficientData { required, provided } => {
                write!(f, "Insufficient data: required {required}, provided {provided}")
            },
            Self::InvalidLinkIndex { from, to, max } => {
                write!(f, "Invalid link index: from={from}, to={to}, max={max}")
            },
            Self::ZeroNormVector => {
                write!(f, "Cannot normalize vector with zero norm")
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
