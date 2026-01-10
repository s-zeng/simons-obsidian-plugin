use rust::VectorWithMetadata;

fn format_vector(vec: &VectorWithMetadata) -> String {
    let mut metadata: Vec<(String, String)> = vec
        .metadata
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    metadata.sort_by(|a, b| a.0.cmp(&b.0));
    format!(
        "id={}|label={}|vector={:?}|source_id={}|metadata={:?}|dims={}",
        vec.id,
        vec.label,
        vec.vector,
        vec.source_id,
        metadata,
        vec.dimensionality()
    )
}

#[test]
fn test_vector_with_metadata_new() {
    let vec = VectorWithMetadata::new(
        "test.md".to_string(),
        "Test Note".to_string(),
        vec![1.0, 2.0, 3.0],
        "test-source".to_string(),
    );

    let snapshot = format_vector(&vec);
    insta::assert_snapshot!(
        snapshot,
        @"id=test.md|label=Test Note|vector=[1.0, 2.0, 3.0]|source_id=test-source|metadata=[]|dims=3"
    );
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
    let snapshot = format_vector(&vec);
    insta::assert_snapshot!(
        snapshot,
        @"id=test.md|label=Test Note|vector=[1.0, 2.0]|source_id=test-source|metadata=[(\"tag\", \"important\")]|dims=2"
    );
}
