//! Vector source abstraction for polymorphic vector handling.
//!
//! This module provides a trait-based abstraction for different vector sources,
//! enabling unified handling of embeddings, adjacency matrices, and future sources.

use crate::PluginError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for polymorphic vector source handling.
///
/// Implementations can provide vectors from different sources (embeddings, graphs, etc.)
pub trait VectorSource {
    /// Unique identifier for this source (e.g., "openai-ada-002", "forward-links").
    fn source_id(&self) -> String;

    /// Dimensionality of vectors from this source.
    fn dimensionality(&self) -> usize;

    /// Fetch vectors from this source.
    ///
    /// # Errors
    /// Returns error if vectors cannot be fetched or processed.
    fn fetch_vectors(&self) -> Result<Vec<VectorWithMetadata>, PluginError>;
}

/// Vector data with associated metadata.
///
/// Represents a single vector point with its identifying information and metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorWithMetadata {
    /// Note file path or unique ID.
    pub id: String,
    /// Note title or display name.
    pub label: String,
    /// The actual vector data.
    pub vector: Vec<f64>,
    /// Which source generated this vector.
    pub source_id: String,
    /// Additional metadata (tags, dates, etc.).
    pub metadata: HashMap<String, String>,
}

impl VectorWithMetadata {
    /// Create a new vector with metadata.
    ///
    /// # Arguments
    /// * `id` - Unique identifier
    /// * `label` - Display label
    /// * `vector` - Vector data
    /// * `source_id` - Source identifier
    #[must_use]
    pub fn new(id: String, label: String, vector: Vec<f64>, source_id: String) -> Self {
        Self { id, label, vector, source_id, metadata: HashMap::new() }
    }

    /// Create a new vector with metadata and additional metadata fields.
    ///
    /// # Arguments
    /// * `id` - Unique identifier
    /// * `label` - Display label
    /// * `vector` - Vector data
    /// * `source_id` - Source identifier
    /// * `metadata` - Additional metadata
    #[must_use]
    pub const fn with_metadata(
        id: String,
        label: String,
        vector: Vec<f64>,
        source_id: String,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self { id, label, vector, source_id, metadata }
    }

    /// Add a metadata field.
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get the dimensionality of this vector.
    #[must_use]
    pub fn dimensionality(&self) -> usize {
        self.vector.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_with_metadata_new() {
        let vec = VectorWithMetadata::new(
            "test.md".to_string(),
            "Test Note".to_string(),
            vec![1.0, 2.0, 3.0],
            "test-source".to_string(),
        );

        assert_eq!(vec.id, "test.md");
        assert_eq!(vec.label, "Test Note");
        assert_eq!(vec.vector, vec![1.0, 2.0, 3.0]);
        assert_eq!(vec.source_id, "test-source");
        assert_eq!(vec.dimensionality(), 3);
    }

    #[test]
    fn test_vector_with_metadata_add_metadata() {
        let mut vec = VectorWithMetadata::new(
            "test.md".to_string(),
            "Test Note".to_string(),
            vec![1.0, 2.0],
            "test-source".to_string(),
        );

        vec.add_metadata("tag".to_string(), "important".to_string());
        assert_eq!(vec.metadata.get("tag"), Some(&"important".to_string()));
    }
}
