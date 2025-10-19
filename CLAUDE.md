# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

This is an Obsidian plugin that leverages Rust/WebAssembly for high-performance computations.
The plugin is designed to provide machine learning and statistical analysis tools directly within Obsidian,
with the heavy computational work handled by Rust code compiled to WebAssembly.

The architecture follows a clean separation:

- **Rust (rust/src/)**: Core logic, data processing, ML/statistics algorithms, and business logic
- **TypeScript (main.ts)**: Obsidian API integration, UI components, and plugin lifecycle management
- **WebAssembly**: Bridge between TypeScript and Rust for near-native performance in the browser

## Style

Try to keep as much code as possible in rust, leaving typescript solely for interfacing with the Obsidian API.

Try to keep the style as functional as possible ("Ocaml with manual garbage
collection", as opposed to "C++ with borrow checker"). Use features like
Algebraic Data Types and Traits liberally, with an algebra-oriented design
mindset

When writing new documentation files, ensure to clarify that "Documentation written
by Claude Code" somewhere in the file.

ALL tests should be in the `tests/` or `rust/tests` directory, and should follow the snapshot
testing instructions in the `## Testing` section.

This project is in heavy development. Whenever you make a change, make sure to
check `CLAUDE.md` and update it if necessary to reflect any newly added/changed
features or structures

All code changes must pass the pre-commit hook of `bun run lint`. After any change, please ensure `bun run lint` still passes.

## 3D Vector Visualization Architecture

The plugin includes a comprehensive 3D vector visualization system that supports multiple vector sources:

### Architecture Overview

- **Rust Layer (Performance-Critical)**:
  - `vector_source.rs`: Trait-based abstraction for polymorphic vector sources
  - `adjacency_matrix.rs`: Sparse adjacency matrix builder for link-graph vectors (M[i][j] = # links i→j)
  - `dimensionality_reduction.rs`: SVD-based dimensionality reduction using nalgebra
  - `vector_ops.rs`: Vector normalization, distance computation, and k-means clustering

- **TypeScript Layer (UI & Obsidian Integration)**:
  - `src/visualization/types.ts`: Core type definitions for vector data and configurations
  - `src/visualization/providers/`: Provider pattern for extensible vector sources
    - `VectorSourceProvider.ts`: Base interface
    - `AdjacencyMatrixProvider.ts`: Link-graph vector provider
    - `EmbeddingSourceProvider.ts`: Qdrant embedding provider with auto-discovery
  - `src/visualization/VectorDataManager.ts`: Multi-source data fetching and caching
  - `src/visualization/VectorVisualizationView.ts`: Three.js-based 3D visualization view
  - `src/visualization/ColorSchemes.ts`: Color scheme implementations

### Key Features

- **Multi-Source Support**: Qdrant embeddings (auto-discovered) + adjacency matrix (link graphs)
- **Source Switching**: UI dropdown to switch between vector sources
- **Graceful Degradation**: Works offline with adjacency matrix even if Qdrant unavailable
- **Performance**: Rust handles heavy computation (SVD, clustering), TypeScript only for rendering
- **Caching**: In-memory cache with source-aware keys
- **Interactive Visualization**: Three.js point cloud with OrbitControls, cluster-based coloring

### Usage

1. Open the vector visualization view via ribbon icon (cube) or command palette
2. Select a vector source from the dropdown (embedding models or link graph)
3. Interact with the 3D visualization (rotate, zoom, pan)
4. Export visualization as PNG

See `vector_plan.md` for detailed architecture documentation

## Error Handling & Safety Guidelines

Based on comprehensive bug audits, follow these critical safety practices:

### Never Use `unwrap()` in Production Code

- **NEVER** use `.unwrap()` on `Option` or `Result` types in production paths
- Use proper error handling with `?`, `.ok_or()`, `.map_err()`, or pattern matching
- Example: Replace `tag_name.chars().nth(1).unwrap()` with proper error handling
- Exception: Only use `unwrap()` in tests or when preceded by explicit checks that guarantee safety

### Error Message Quality

- Include contextual information in error messages
- Use structured error types instead of plain strings where possible
- Provide actionable information for debugging

## Development Environment

This project uses Nix for reproducible builds and development environments.

## Testing

The project will use **snapshot testing** via the `insta` crate for all test assertions. This testing paradigm provides deterministic, maintainable tests that capture expected behavior through literal value snapshots.

### Snapshot Testing Approach

All tests follow these principles:

- **Single assertion per test**: Each test has exactly one `insta::assert_snapshot!()` or `insta::assert_json_snapshot!()` call
- **Deterministic snapshots**: Dynamic data (timestamps, file sizes, temp paths) is normalized to ensure reproducible results
- **Literal value snapshots**: Snapshots contain only concrete, expected values without variables
- **Offline resilience**: All tests must pass in offline environments (CI, restricted networks) by using dual-snapshot patterns or graceful degradation

### Snapshot Management

- Snapshots are stored in `src/snapshots/` (unit tests) and `tests/snapshots/` (integration tests)
- When test behavior changes, run `cargo insta review` to inspect differences
- Accept valid changes with `cargo insta accept` or reject with `cargo insta reject`
- Never commit `.snap.new` files - these are pending snapshot updates

### Deterministic Test Strategies

To ensure reproducible snapshots, the tests employ several normalization techniques:

- **Timestamp normalization**: Replace dynamic timestamps with fixed values
- **File size ranges**: Use `size > min && size < max` instead of exact sizes
- **Path abstraction**: Extract relevant path components, ignore temp directories
- **Content summarization**: Focus on structural properties rather than exact values

This approach makes tests resilient to environmental differences while maintaining strict behavioral validation.
