//! Obsidian plugin powered by Rust and WebAssembly.
//!
//! This library provides high-performance computational functions for an Obsidian plugin,
//! leveraging Rust's safety and performance through WebAssembly compilation.

#![allow(clippy::multiple_crate_versions)]

use wasm_bindgen::prelude::*;

// Module declarations
mod adjacency_matrix;
mod commands;
mod dimensionality_reduction;
mod error;
mod settings;
mod utils;
mod vector_ops;
mod vector_source;

// Re-export all public functions from modules
pub use adjacency_matrix::*;
pub use commands::*;
pub use dimensionality_reduction::*;
pub use error::*;
pub use settings::*;
pub use utils::*;
pub use vector_ops::*;
pub use vector_source::*;

/// Generate a greeting message (legacy compatibility function).
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A greeting string
#[wasm_bindgen]
#[must_use]
pub fn greet(name: &str) -> String {
    generate_greeting(name)
}

/// Calculate the nth Fibonacci number (legacy compatibility function).
///
/// # Arguments
/// * `n` - The position in the Fibonacci sequence
///
/// # Returns
/// The Fibonacci number at position n
#[wasm_bindgen]
#[must_use]
pub fn fibonacci(n: u32) -> u32 {
    calculate_fibonacci(n)
}

/// Build adjacency matrix from note links.
///
/// # Arguments
/// * `note_paths_json` - JSON array of note paths
/// * `links_json` - JSON array of links (objects with from_id and to_id)
///
/// # Returns
/// JSON string of vectors (adjacency matrix rows)
///
/// # Errors
/// Returns error if parsing fails or link indices are invalid
#[wasm_bindgen]
pub fn build_adjacency_matrix(note_paths_json: &str, links_json: &str) -> Result<String, JsValue> {
    let note_paths: Vec<String> = serde_json::from_str(note_paths_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse note paths: {e}")))?;

    let links: Vec<NoteLink> = serde_json::from_str(links_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse links: {e}")))?;

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder
        .build(links)
        .map_err(|e| JsValue::from_str(&format!("Failed to build matrix: {e}")))?;

    let vectors = builder.matrix_to_vectors(&matrix);

    serde_json::to_string(&vectors)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {e}")))
}

/// Build graph Laplacian matrix from note links.
///
/// # Arguments
/// * `note_paths_json` - JSON array of note paths
/// * `links_json` - JSON array of links (objects with from_id and to_id)
///
/// # Returns
/// JSON string of vectors (Laplacian matrix rows)
///
/// # Errors
/// Returns error if parsing fails or link indices are invalid
#[wasm_bindgen]
pub fn build_laplacian_matrix(note_paths_json: &str, links_json: &str) -> Result<String, JsValue> {
    let note_paths: Vec<String> = serde_json::from_str(note_paths_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse note paths: {e}")))?;

    let links: Vec<NoteLink> = serde_json::from_str(links_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse links: {e}")))?;

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder
        .build_laplacian(links)
        .map_err(|e| JsValue::from_str(&format!("Failed to build Laplacian: {e}")))?;

    let vectors = builder.matrix_to_vectors(&matrix);

    serde_json::to_string(&vectors)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {e}")))
}

/// Reduce dimensionality using SVD.
///
/// # Arguments
/// * `vectors_json` - JSON array of vectors
/// * `target_dims` - Target dimensionality (typically 2 or 3)
///
/// # Returns
/// JSON string of reduced vectors
///
/// # Errors
/// Returns error if parsing fails or reduction fails
#[wasm_bindgen]
pub fn reduce_dimensions_svd(vectors_json: &str, target_dims: usize) -> Result<String, JsValue> {
    let vectors: Vec<Vec<f64>> = serde_json::from_str(vectors_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;

    let reducer = SVDReducer::new();
    let result = reducer
        .reduce(&vectors, target_dims)
        .map_err(|e| JsValue::from_str(&format!("Reduction error: {e}")))?;

    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))
}

/// Cluster vectors using k-means.
///
/// # Arguments
/// * `vectors_json` - JSON array of vectors
/// * `num_clusters` - Number of clusters
///
/// # Returns
/// JSON string of cluster assignments (one per vector)
///
/// # Errors
/// Returns error if parsing fails or clustering fails
#[wasm_bindgen]
pub fn cluster_vectors(vectors_json: &str, num_clusters: usize) -> Result<String, JsValue> {
    let vectors: Vec<Vec<f64>> = serde_json::from_str(vectors_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;

    let clusters = simple_kmeans_clustering(&vectors, num_clusters)
        .map_err(|e| JsValue::from_str(&format!("Clustering error: {e}")))?;

    serde_json::to_string(&clusters)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))
}
