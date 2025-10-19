use rust::{normalize_vectors, simple_kmeans_clustering};

#[test]
fn test_normalize_vectors_snapshot() {
    let vectors = vec![vec![3.0, 4.0], vec![1.0, 0.0], vec![5.0, 12.0]];

    let normalized = normalize_vectors(&vectors).expect("Normalization failed");

    // Format with fixed precision for deterministic snapshots
    let formatted: Vec<Vec<String>> = normalized
        .iter()
        .map(|v| v.iter().map(|x| format!("{x:.10}")).collect())
        .collect();

    let snapshot = serde_json::to_string_pretty(&formatted).expect("Failed to serialize");
    insta::assert_snapshot!(snapshot);
}

#[test]
fn test_kmeans_clustering_deterministic() {
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

    let snapshot = serde_json::to_string_pretty(&assignments).expect("Failed to serialize");
    insta::assert_snapshot!(snapshot);
}
