//! Vector operations for normalization, distance, and clustering.
//!
//! This module provides utilities for vector manipulation and analysis.

use crate::PluginError;

/// Normalize vectors to unit length.
///
/// # Arguments
/// * `vectors` - Input vectors to normalize
///
/// # Returns
/// Normalized vectors with unit length
///
/// # Errors
/// Returns error if any vector has zero norm
pub fn normalize_vectors(vectors: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, PluginError> {
    vectors
        .iter()
        .map(|vec| {
            let norm = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < 1e-10 {
                return Err(PluginError::ZeroNormVector);
            }
            Ok(vec.iter().map(|x| x / norm).collect())
        })
        .collect()
}

/// Compute Euclidean distance between two vectors.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
/// Euclidean distance
///
/// # Errors
/// Returns error if vectors have different dimensions
pub fn euclidean_distance(a: &[f64], b: &[f64]) -> Result<f64, PluginError> {
    if a.len() != b.len() {
        return Err(PluginError::InvalidVectorDimensions {
            expected: a.len(),
            got: b.len(),
            vector_index: 0,
        });
    }

    let distance = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f64>()
        .sqrt();

    Ok(distance)
}

/// Simple k-means clustering for vector assignment.
///
/// # Arguments
/// * `vectors` - Input vectors to cluster
/// * `k` - Number of clusters
///
/// # Returns
/// Cluster assignment for each vector
///
/// # Errors
/// Returns error if k is invalid or vectors have mismatched dimensions
pub fn simple_kmeans_clustering(vectors: &[Vec<f64>], k: usize) -> Result<Vec<usize>, PluginError> {
    if vectors.is_empty() {
        return Err(PluginError::InsufficientData { required: 1, provided: 0 });
    }

    if k == 0 {
        return Err(PluginError::InsufficientData { required: 1, provided: 0 });
    }

    if k > vectors.len() {
        return Err(PluginError::InsufficientData { required: k, provided: vectors.len() });
    }

    let dim = vectors[0].len();

    // Validate all vectors have same dimensionality
    for (i, vec) in vectors.iter().enumerate() {
        if vec.len() != dim {
            return Err(PluginError::InvalidVectorDimensions {
                expected: dim,
                got: vec.len(),
                vector_index: i,
            });
        }
    }

    // Initialize centroids using k-means++ strategy
    let mut centroids = initialize_centroids_kmeanspp(vectors, k)?;

    // Run k-means iterations
    let max_iterations = 100;
    let mut assignments = vec![0; vectors.len()];

    for _ in 0..max_iterations {
        let mut changed = false;

        // Assignment step: assign each point to nearest centroid
        for (i, vec) in vectors.iter().enumerate() {
            let mut min_dist = f64::MAX;
            let mut best_cluster = 0;

            for (j, centroid) in centroids.iter().enumerate() {
                let dist = euclidean_distance(vec, centroid)?;
                if dist < min_dist {
                    min_dist = dist;
                    best_cluster = j;
                }
            }

            if assignments[i] != best_cluster {
                assignments[i] = best_cluster;
                changed = true;
            }
        }

        // If no assignments changed, we've converged
        if !changed {
            break;
        }

        // Update step: recompute centroids
        centroids = compute_centroids(vectors, &assignments, k, dim);
    }

    Ok(assignments)
}

/// Initialize centroids using k-means++ strategy.
fn initialize_centroids_kmeanspp(
    vectors: &[Vec<f64>],
    k: usize,
) -> Result<Vec<Vec<f64>>, PluginError> {
    let mut centroids = Vec::with_capacity(k);

    // Choose first centroid randomly (use first point for determinism)
    centroids.push(vectors[0].clone());

    // Choose remaining centroids with probability proportional to distance squared
    for _ in 1..k {
        let mut distances = vec![0.0; vectors.len()];

        for (i, vec) in vectors.iter().enumerate() {
            let mut min_dist = f64::MAX;
            for centroid in &centroids {
                let dist = euclidean_distance(vec, centroid)?;
                min_dist = min_dist.min(dist);
            }
            distances[i] = min_dist * min_dist;
        }

        // Select point with highest distance (deterministic, simpler than random)
        let max_idx = distances
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0, |(idx, _)| idx);

        centroids.push(vectors[max_idx].clone());
    }

    Ok(centroids)
}

/// Compute new centroids from current assignments.
fn compute_centroids(
    vectors: &[Vec<f64>],
    assignments: &[usize],
    k: usize,
    dim: usize,
) -> Vec<Vec<f64>> {
    let mut centroids = vec![vec![0.0; dim]; k];
    let mut counts = vec![0; k];

    // Sum all vectors in each cluster
    for (vec, &cluster) in vectors.iter().zip(assignments.iter()) {
        for (i, &val) in vec.iter().enumerate() {
            centroids[cluster][i] += val;
        }
        counts[cluster] += 1;
    }

    // Compute means (avoid division by zero)
    for (cluster, count) in counts.iter().enumerate() {
        if *count > 0 {
            for val in &mut centroids[cluster] {
                *val /= f64::from(*count);
            }
        }
    }

    centroids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_vectors() {
        let vectors = vec![vec![3.0, 4.0], vec![1.0, 0.0]];

        let normalized = normalize_vectors(&vectors).expect("Normalization failed");

        assert_eq!(normalized.len(), 2);
        // First vector: [3, 4] -> [0.6, 0.8]
        assert!((normalized[0][0] - 0.6).abs() < 1e-10);
        assert!((normalized[0][1] - 0.8).abs() < 1e-10);
        // Second vector: [1, 0] -> [1, 0]
        assert!((normalized[1][0] - 1.0).abs() < 1e-10);
        assert!((normalized[1][1] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let vectors = vec![vec![0.0, 0.0]];
        let result = normalize_vectors(&vectors);

        assert!(result.is_err());
        match result {
            Err(PluginError::ZeroNormVector) => {},
            _ => panic!("Expected ZeroNormVector error"),
        }
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        let dist = euclidean_distance(&a, &b).expect("Distance calculation failed");

        // sqrt((4-1)^2 + (5-2)^2 + (6-3)^2) = sqrt(9+9+9) = sqrt(27) ≈ 5.196
        assert!((dist - 5.196152422706632).abs() < 1e-10);
    }

    #[test]
    fn test_simple_kmeans_clustering() {
        let vectors = vec![
            vec![1.0, 1.0],
            vec![1.5, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 7.0],
            vec![3.5, 5.0],
            vec![4.5, 5.0],
            vec![3.5, 4.5],
        ];

        let assignments = simple_kmeans_clustering(&vectors, 2).expect("Clustering failed");

        assert_eq!(assignments.len(), 7);
        // All assignments should be valid cluster IDs (0 or 1)
        for &cluster in &assignments {
            assert!(cluster < 2);
        }
    }

    #[test]
    fn test_kmeans_invalid_k() {
        let vectors = vec![vec![1.0, 2.0], vec![3.0, 4.0]];

        let result = simple_kmeans_clustering(&vectors, 5); // k > num vectors

        assert!(result.is_err());
    }

    #[test]
    fn test_kmeans_empty_vectors() {
        let vectors: Vec<Vec<f64>> = vec![];

        let result = simple_kmeans_clustering(&vectors, 2);

        assert!(result.is_err());
    }
}
