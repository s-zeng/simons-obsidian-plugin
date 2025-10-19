//! Dimensionality reduction algorithms for vector visualization.
//!
//! This module provides trait-based abstractions for reducing high-dimensional
//! vectors to lower dimensions for visualization purposes.

use crate::PluginError;
use nalgebra::{DMatrix, DVector};

/// Trait for dimensionality reduction algorithms.
pub trait DimensionalityReducer {
    /// Reduce vectors to target dimensionality.
    ///
    /// # Arguments
    /// * `vectors` - Input high-dimensional vectors
    /// * `target_dims` - Target dimensionality (typically 2 or 3)
    ///
    /// # Returns
    /// Reduced vectors with target dimensionality
    ///
    /// # Errors
    /// Returns error if reduction fails
    fn reduce(
        &self,
        vectors: &[Vec<f64>],
        target_dims: usize,
    ) -> Result<Vec<Vec<f64>>, PluginError>;

    /// Get the name of this reduction method.
    fn method_name(&self) -> &str;
}

/// SVD-based dimensionality reduction.
///
/// Uses Singular Value Decomposition to project high-dimensional data
/// onto lower-dimensional subspaces that preserve maximum variance.
pub struct SVDReducer {
    /// Whether to center data (subtract mean).
    center: bool,
    /// Whether to scale data (divide by std dev).
    scale: bool,
}

impl SVDReducer {
    /// Create a new SVD reducer with default settings.
    ///
    /// Default: center=true, scale=false
    #[must_use]
    pub const fn new() -> Self {
        Self { center: true, scale: false }
    }

    /// Create a new SVD reducer with custom settings.
    ///
    /// # Arguments
    /// * `center` - Whether to center the data
    /// * `scale` - Whether to scale the data
    #[must_use]
    pub const fn with_options(center: bool, scale: bool) -> Self {
        Self { center, scale }
    }

    /// Center the data matrix (subtract column means).
    fn center_data(matrix: &DMatrix<f64>) -> (DMatrix<f64>, DVector<f64>) {
        let means = matrix.column_mean();
        let nrows = matrix.nrows();
        let ncols = matrix.ncols();

        let centered = DMatrix::from_fn(nrows, ncols, |i, j| matrix[(i, j)] - means[j]);

        (centered, means)
    }

    /// Scale the data matrix (divide by column standard deviations).
    #[allow(clippy::cast_precision_loss)]
    fn scale_data(matrix: &DMatrix<f64>) -> (DMatrix<f64>, DVector<f64>) {
        let ncols = matrix.ncols();
        let mut std_devs = DVector::zeros(ncols);

        for col in 0..ncols {
            let column = matrix.column(col);
            let variance = column.iter().map(|x| x * x).sum::<f64>() / (matrix.nrows() as f64);
            std_devs[col] = variance.sqrt().max(1e-10); // Avoid division by zero
        }

        let scaled =
            DMatrix::from_fn(matrix.nrows(), matrix.ncols(), |i, j| matrix[(i, j)] / std_devs[j]);

        (scaled, std_devs)
    }
}

impl Default for SVDReducer {
    fn default() -> Self {
        Self::new()
    }
}

impl DimensionalityReducer for SVDReducer {
    fn reduce(
        &self,
        vectors: &[Vec<f64>],
        target_dims: usize,
    ) -> Result<Vec<Vec<f64>>, PluginError> {
        if vectors.is_empty() {
            return Err(PluginError::InsufficientData { required: 1, provided: 0 });
        }

        // Validate all vectors have same dimensionality
        let dim = vectors[0].len();
        for (i, vec) in vectors.iter().enumerate() {
            if vec.len() != dim {
                return Err(PluginError::InvalidVectorDimensions {
                    expected: dim,
                    got: vec.len(),
                    vector_index: i,
                });
            }
        }

        if target_dims > dim {
            return Err(PluginError::DimensionalityReductionError {
                method: "SVD".to_string(),
                reason: format!(
                    "Target dimensions ({target_dims}) cannot exceed input dimensions ({dim})"
                ),
            });
        }

        // Convert to matrix (rows = data points, cols = dimensions)
        let mut matrix = DMatrix::from_fn(vectors.len(), dim, |i, j| vectors[i][j]);

        // Center and/or scale if requested
        if self.center {
            (matrix, _) = Self::center_data(&matrix);
        }
        if self.scale {
            (matrix, _) = Self::scale_data(&matrix);
        }

        // Perform SVD
        let svd = matrix.svd(true, true);

        // Project onto top `target_dims` singular vectors
        let u = svd
            .u
            .ok_or_else(|| PluginError::DimensionalityReductionError {
                method: "SVD".to_string(),
                reason: "SVD failed to compute U matrix".to_string(),
            })?;

        let sigma = &svd.singular_values;

        // Reduced representation: U * Sigma (first target_dims components)
        let reduced_matrix =
            DMatrix::from_fn(vectors.len(), target_dims, |i, j| u[(i, j)] * sigma[j]);

        // Convert back to Vec<Vec<f64>>
        let result = (0..vectors.len())
            .map(|i| (0..target_dims).map(|j| reduced_matrix[(i, j)]).collect())
            .collect();

        Ok(result)
    }

    fn method_name(&self) -> &'static str {
        "SVD"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svd_reducer_simple() {
        let vectors = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0], vec![7.0, 8.0, 9.0]];

        let reducer = SVDReducer::new();
        let result = reducer.reduce(&vectors, 2).expect("SVD reduction failed");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].len(), 2);
        assert_eq!(result[1].len(), 2);
        assert_eq!(result[2].len(), 2);
    }

    #[test]
    fn test_svd_reducer_invalid_target_dims() {
        let vectors = vec![vec![1.0, 2.0], vec![3.0, 4.0]];

        let reducer = SVDReducer::new();
        let result = reducer.reduce(&vectors, 5); // Target > source dims

        assert!(result.is_err());
        match result {
            Err(PluginError::DimensionalityReductionError { method, reason: _ }) => {
                assert_eq!(method, "SVD");
            },
            _ => panic!("Expected DimensionalityReductionError"),
        }
    }

    #[test]
    fn test_svd_reducer_mismatched_dimensions() {
        let vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0], // Wrong dimension
        ];

        let reducer = SVDReducer::new();
        let result = reducer.reduce(&vectors, 2);

        assert!(result.is_err());
        match result {
            Err(PluginError::InvalidVectorDimensions { expected, got, vector_index }) => {
                assert_eq!(expected, 3);
                assert_eq!(got, 2);
                assert_eq!(vector_index, 1);
            },
            _ => panic!("Expected InvalidVectorDimensions error"),
        }
    }

    #[test]
    fn test_svd_reducer_empty_input() {
        let vectors: Vec<Vec<f64>> = vec![];

        let reducer = SVDReducer::new();
        let result = reducer.reduce(&vectors, 2);

        assert!(result.is_err());
        match result {
            Err(PluginError::InsufficientData { required, provided }) => {
                assert_eq!(required, 1);
                assert_eq!(provided, 0);
            },
            _ => panic!("Expected InsufficientData error"),
        }
    }
}
