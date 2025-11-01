//! Adjacency matrix builder for link-graph vector representation.
//!
//! This module constructs sparse adjacency matrices from note links,
//! where M[i][j] = number of forward links from note i to note j.

use crate::PluginError;
use serde::{Deserialize, Serialize};
use sprs::{CsMat, TriMat};
use std::collections::HashMap;

/// Represents a link between two notes in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteLink {
    /// Source note index.
    #[serde(rename = "fromId")]
    pub from_id: usize,
    /// Target note index.
    #[serde(rename = "toId")]
    pub to_id: usize,
}

/// Builds adjacency matrices from note links.
///
/// The matrix M[i][j] represents the number of forward links from note i to note j.
/// Uses sparse matrix representation for efficiency with large vaults.
pub struct AdjacencyMatrixBuilder {
    /// Total number of notes.
    num_notes: usize,
    /// Map from note paths to indices.
    note_id_map: HashMap<String, usize>,
}

impl AdjacencyMatrixBuilder {
    /// Create a new adjacency matrix builder.
    ///
    /// # Arguments
    /// * `note_paths` - List of note paths in the vault
    ///
    /// # Returns
    /// A new builder configured for the given notes
    #[must_use]
    pub fn new(note_paths: Vec<String>) -> Self {
        let num_notes = note_paths.len();
        let note_id_map = note_paths
            .into_iter()
            .enumerate()
            .map(|(i, path)| (path, i))
            .collect();

        Self { num_notes, note_id_map }
    }

    /// Build the sparse adjacency matrix from a list of links.
    ///
    /// # Arguments
    /// * `links` - List of note links
    ///
    /// # Returns
    /// Sparse adjacency matrix in CSR format
    ///
    /// # Errors
    /// Returns error if link indices are out of bounds
    #[allow(clippy::cast_precision_loss)]
    pub fn build(&self, links: Vec<NoteLink>) -> Result<CsMat<f64>, PluginError> {
        let mut triplets = TriMat::new((self.num_notes, self.num_notes));

        // Count links between each pair of notes
        let mut link_counts: HashMap<(usize, usize), usize> = HashMap::new();
        for link in links {
            if link.from_id >= self.num_notes || link.to_id >= self.num_notes {
                return Err(PluginError::InvalidLinkIndex {
                    from: link.from_id,
                    to: link.to_id,
                    max: self.num_notes - 1,
                });
            }
            *link_counts.entry((link.from_id, link.to_id)).or_insert(0) += 1;
        }

        // Build sparse matrix from link counts
        for ((from, to), count) in link_counts {
            triplets.add_triplet(from, to, count as f64);
        }

        Ok(triplets.to_csr())
    }

    /// Convert adjacency matrix to vector representation.
    ///
    /// Each row becomes a vector representing the outgoing link pattern for that note.
    ///
    /// # Arguments
    /// * `matrix` - Sparse adjacency matrix
    ///
    /// # Returns
    /// Dense vector representation, one vector per note
    #[must_use]
    pub fn matrix_to_vectors(&self, matrix: &CsMat<f64>) -> Vec<Vec<f64>> {
        (0..self.num_notes)
            .map(|i| {
                let mut vec = vec![0.0; self.num_notes];
                if let Some(row) = matrix.outer_view(i) {
                    for (col, &val) in row.iter() {
                        vec[col] = val;
                    }
                }
                vec
            })
            .collect()
    }

    /// Get the number of notes in the builder.
    #[must_use]
    pub const fn num_notes(&self) -> usize {
        self.num_notes
    }

    /// Get note index from path.
    ///
    /// # Arguments
    /// * `path` - Note path
    ///
    /// # Returns
    /// Note index if found
    #[must_use]
    pub fn get_note_index(&self, path: &str) -> Option<usize> {
        self.note_id_map.get(path).copied()
    }

    /// Build the graph Laplacian matrix from a list of links.
    ///
    /// The Laplacian matrix L = D - A, where:
    /// - D is the degree matrix (diagonal, D[i][i] = out-degree of node i)
    /// - A is the adjacency matrix
    ///
    /// # Arguments
    /// * `links` - List of note links
    ///
    /// # Returns
    /// Sparse Laplacian matrix in CSR format
    ///
    /// # Errors
    /// Returns error if link indices are out of bounds
    pub fn build_laplacian(&self, links: Vec<NoteLink>) -> Result<CsMat<f64>, PluginError> {
        // First build the adjacency matrix
        let adjacency = self.build(links)?;

        // Compute degree matrix (diagonal with out-degrees)
        let mut degree_triplets = TriMat::new((self.num_notes, self.num_notes));
        for i in 0..self.num_notes {
            let row_sum: f64 = adjacency
                .outer_view(i)
                .map_or(0.0, |row| row.iter().map(|(_, &val)| val).sum());
            if row_sum > 0.0 {
                degree_triplets.add_triplet(i, i, row_sum);
            }
        }
        let degree = degree_triplets.to_csr();

        // Compute L = D - A
        let laplacian = &degree - &adjacency;

        Ok(laplacian)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacency_matrix_builder_new() {
        let note_paths =
            vec!["note1.md".to_string(), "note2.md".to_string(), "note3.md".to_string()];

        let builder = AdjacencyMatrixBuilder::new(note_paths);
        assert_eq!(builder.num_notes(), 3);
        assert_eq!(builder.get_note_index("note1.md"), Some(0));
        assert_eq!(builder.get_note_index("note2.md"), Some(1));
        assert_eq!(builder.get_note_index("note3.md"), Some(2));
    }

    #[test]
    fn test_adjacency_matrix_simple() {
        let note_paths =
            vec!["note1.md".to_string(), "note2.md".to_string(), "note3.md".to_string()];

        let links = vec![
            NoteLink { from_id: 0, to_id: 1 },
            NoteLink { from_id: 0, to_id: 2 },
            NoteLink { from_id: 1, to_id: 2 },
        ];

        let builder = AdjacencyMatrixBuilder::new(note_paths);
        let matrix = builder.build(links).expect("Failed to build matrix");
        let vectors = builder.matrix_to_vectors(&matrix);

        assert_eq!(vectors.len(), 3);
        assert_eq!(vectors[0], vec![0.0, 1.0, 1.0]); // note1 links to note2 and note3
        assert_eq!(vectors[1], vec![0.0, 0.0, 1.0]); // note2 links to note3
        assert_eq!(vectors[2], vec![0.0, 0.0, 0.0]); // note3 has no outgoing links
    }

    #[test]
    fn test_adjacency_matrix_multiple_links() {
        let note_paths = vec!["note1.md".to_string(), "note2.md".to_string()];

        let links = vec![
            NoteLink { from_id: 0, to_id: 1 },
            NoteLink { from_id: 0, to_id: 1 }, // Duplicate link
        ];

        let builder = AdjacencyMatrixBuilder::new(note_paths);
        let matrix = builder.build(links).expect("Failed to build matrix");
        let vectors = builder.matrix_to_vectors(&matrix);

        assert_eq!(vectors[0], vec![0.0, 2.0]); // note1 has 2 links to note2
    }

    #[test]
    fn test_adjacency_matrix_invalid_index() {
        let note_paths = vec!["note1.md".to_string(), "note2.md".to_string()];

        let links = vec![NoteLink { from_id: 0, to_id: 5 }]; // Invalid index

        let builder = AdjacencyMatrixBuilder::new(note_paths);
        let result = builder.build(links);

        assert!(result.is_err());
        match result {
            Err(PluginError::InvalidLinkIndex { from, to, max }) => {
                assert_eq!(from, 0);
                assert_eq!(to, 5);
                assert_eq!(max, 1);
            },
            _ => panic!("Expected InvalidLinkIndex error"),
        }
    }

    #[test]
    fn test_laplacian_simple() {
        let note_paths =
            vec!["note1.md".to_string(), "note2.md".to_string(), "note3.md".to_string()];

        let links = vec![
            NoteLink { from_id: 0, to_id: 1 },
            NoteLink { from_id: 0, to_id: 2 },
            NoteLink { from_id: 1, to_id: 2 },
        ];

        let builder = AdjacencyMatrixBuilder::new(note_paths);
        let matrix = builder
            .build_laplacian(links)
            .expect("Failed to build Laplacian");
        let vectors = builder.matrix_to_vectors(&matrix);

        // Verify Laplacian properties
        // For a directed graph: L[i][i] = out-degree(i), L[i][j] = -A[i][j]
        // node1: out-degree=2, links to node2 and node3
        // node2: out-degree=1, links to node3
        // node3: out-degree=0, no outgoing links
        assert_eq!(vectors[0], vec![2.0, -1.0, -1.0]); // L[0] = [2, -1, -1]
        assert_eq!(vectors[1], vec![0.0, 1.0, -1.0]); // L[1] = [0, 1, -1]
        assert_eq!(vectors[2], vec![0.0, 0.0, 0.0]); // L[2] = [0, 0, 0]
    }

    #[test]
    fn test_laplacian_isolated_node() {
        let note_paths = vec!["note1.md".to_string(), "note2.md".to_string()];

        let links = vec![]; // No links, all nodes isolated

        let builder = AdjacencyMatrixBuilder::new(note_paths);
        let matrix = builder
            .build_laplacian(links)
            .expect("Failed to build Laplacian");
        let vectors = builder.matrix_to_vectors(&matrix);

        // Isolated nodes should have all zeros
        assert_eq!(vectors[0], vec![0.0, 0.0]);
        assert_eq!(vectors[1], vec![0.0, 0.0]);
    }

    #[test]
    fn test_laplacian_self_loop() {
        let note_paths = vec!["note1.md".to_string(), "note2.md".to_string()];

        let links = vec![
            NoteLink { from_id: 0, to_id: 0 }, // Self-loop
            NoteLink { from_id: 0, to_id: 1 },
        ];

        let builder = AdjacencyMatrixBuilder::new(note_paths);
        let matrix = builder
            .build_laplacian(links)
            .expect("Failed to build Laplacian");
        let vectors = builder.matrix_to_vectors(&matrix);

        // node1 has out-degree=2 (including self-loop)
        // L[0][0] = 2, L[0][0] -= A[0][0] = 2 - 1 = 1
        // L[0][1] = -A[0][1] = -1
        assert_eq!(vectors[0], vec![1.0, -1.0]);
        assert_eq!(vectors[1], vec![0.0, 0.0]);
    }
}
