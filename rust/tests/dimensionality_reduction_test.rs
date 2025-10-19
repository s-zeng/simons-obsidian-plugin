use rust::{DimensionalityReducer, SVDReducer};

#[test]
fn test_svd_reduction_3d_to_2d() {
    let vectors =
        vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0], vec![7.0, 8.0, 9.0], vec![2.0, 3.0, 4.0]];

    let reducer = SVDReducer::new();
    let result = reducer.reduce(&vectors, 2).expect("SVD reduction failed");

    // Normalize results for deterministic snapshots (scale-invariant)
    let normalized: Vec<Vec<String>> = result
        .iter()
        .map(|v| v.iter().map(|x| format!("{x:.6}")).collect())
        .collect();

    let snapshot = serde_json::to_string_pretty(&normalized).expect("Failed to serialize");
    insta::assert_snapshot!(snapshot);
}

#[test]
fn test_svd_reduction_preserves_count() {
    let vectors = vec![vec![1.0, 2.0, 3.0, 4.0], vec![5.0, 6.0, 7.0, 8.0]];

    let reducer = SVDReducer::new();
    let result = reducer.reduce(&vectors, 2).expect("SVD reduction failed");

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].len(), 2);
    assert_eq!(result[1].len(), 2);
}
