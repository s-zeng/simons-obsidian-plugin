use rust::{AdjacencyMatrixBuilder, NoteLink};

#[test]
fn test_adjacency_matrix_basic() {
    let note_paths = vec!["note1.md".to_string(), "note2.md".to_string(), "note3.md".to_string()];

    let links = vec![
        NoteLink { from_id: 0, to_id: 1 },
        NoteLink { from_id: 0, to_id: 2 },
        NoteLink { from_id: 1, to_id: 2 },
    ];

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder.build(links).expect("Failed to build matrix");
    let vectors = builder.matrix_to_vectors(&matrix);

    let snapshot = serde_json::to_string_pretty(&vectors).expect("Failed to serialize");
    insta::assert_snapshot!(snapshot);
}

#[test]
fn test_adjacency_matrix_self_links() {
    let note_paths = vec!["note1.md".to_string(), "note2.md".to_string()];
    let links = vec![NoteLink { from_id: 0, to_id: 0 }]; // Self-link

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder.build(links).expect("Failed to build matrix");
    let vectors = builder.matrix_to_vectors(&matrix);

    let snapshot = serde_json::to_string_pretty(&vectors).expect("Failed to serialize");
    insta::assert_snapshot!(snapshot);
}

#[test]
fn test_adjacency_matrix_no_links() {
    let note_paths = vec!["note1.md".to_string(), "note2.md".to_string(), "note3.md".to_string()];
    let links = vec![]; // No links

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder.build(links).expect("Failed to build matrix");
    let vectors = builder.matrix_to_vectors(&matrix);

    let snapshot = serde_json::to_string_pretty(&vectors).expect("Failed to serialize");
    insta::assert_snapshot!(snapshot);
}

#[test]
fn test_adjacency_matrix_multiple_links() {
    let note_paths = vec!["note1.md".to_string(), "note2.md".to_string()];

    let links = vec![
        NoteLink { from_id: 0, to_id: 1 },
        NoteLink { from_id: 0, to_id: 1 }, // Duplicate link - should count as 2
    ];

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder.build(links).expect("Failed to build matrix");
    let vectors = builder.matrix_to_vectors(&matrix);

    let snapshot = serde_json::to_string_pretty(&vectors).expect("Failed to serialize");
    insta::assert_snapshot!(snapshot);
}

#[test]
fn test_adjacency_matrix_empty_notes_error() {
    let builder = AdjacencyMatrixBuilder::new(vec![]);
    let snapshot = match builder.build(vec![]) {
        Ok(_) => "ok".to_string(),
        Err(err) => err.to_string(),
    };
    insta::assert_snapshot!(snapshot, @"Insufficient data: required 1, provided 0");
}

#[test]
fn test_laplacian_matrix_basic() {
    let note_paths = vec!["note1.md".to_string(), "note2.md".to_string(), "note3.md".to_string()];

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

    let snapshot = serde_json::to_string_pretty(&vectors).expect("Failed to serialize");
    insta::assert_snapshot!(
        snapshot,
        @r#"[
  [
    2.0,
    -1.0,
    -1.0
  ],
  [
    0.0,
    1.0,
    -1.0
  ],
  [
    0.0,
    0.0,
    0.0
  ]
]"#
    );
}
