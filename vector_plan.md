Architecture Design for 3D Vector Visualization with Multi-Source Support

This document outlines the architecture for a 3D vector visualization feature that supports multiple vector sources including different embedding models and link-graph adjacency matrices.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      TypeScript Layer                        │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  VectorVisualizationView (extends ItemView)            │  │
│  │  - Three.js scene management                           │  │
│  │  - User interaction (rotate/pan/zoom)                  │  │
│  │  - Point selection & metadata display                  │  │
│  │  - Source selector UI                                  │  │
│  │  - Controls (method selector, color scheme, etc.)      │  │
│  │  - Export functionality                                │  │
│  └────────────────────────────────────────────────────────┘  │
│                            ↕                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  VectorDataManager                                     │  │
│  │  - Multi-source vector fetching                        │  │
│  │  - Provider pattern for extensibility                  │  │
│  │  - Source-aware cache management (in-memory + IndexedDB)│ │
│  │  - Data invalidation strategy                          │  │
│  │  - Interface with Rust computation layer               │  │
│  └────────────────────────────────────────────────────────┘  │
│                            ↕                                 │
│  ┌───────────────────────────────────────────────────────┐   │
│  │  Vector Source Providers                              │   │
│  │  ┌─────────────────┐  ┌──────────────────────────┐   │   │
│  │  │ Embedding       │  │ Adjacency Matrix         │   │   │
│  │  │ Provider        │  │ Provider                 │   │   │
│  │  │ (Qdrant auto-   │  │ (Link-graph vectors)     │   │   │
│  │  │  discovery)     │  │                          │   │   │
│  │  └─────────────────┘  └──────────────────────────┘   │   │
│  └───────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ↕
                ┌─────────────────────┐
                │   WASM Boundary     │
                └─────────────────────┘
                            ↕
┌─────────────────────────────────────────────────────────────┐
│                        Rust Layer                            │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  vector_source.rs                                      │  │
│  │  - Trait: VectorSource                                 │  │
│  │  - VectorWithMetadata struct                           │  │
│  │  - Source abstraction for polymorphic handling         │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  adjacency_matrix.rs                                   │  │
│  │  - Build forward-link matrix M[i][j] = # links i→j    │  │
│  │  - Sparse matrix representation                        │  │
│  │  - Convert matrix rows to vectors                      │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  dimensionality_reduction.rs                           │  │
│  │  - Trait: DimensionalityReducer                        │  │
│  │  - SVD implementation (using nalgebra)                 │  │
│  │  - Future: UMAP, t-SNE, PCA variants                   │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  vector_ops.rs                                         │  │
│  │  - Vector normalization                                │  │
│  │  - Distance computations                               │  │
│  │  - Clustering utilities (for color assignment)         │  │
│  └────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Detailed Component Breakdown

#### 1. **Rust Components** (Performance-Critical Computation)

**File: `rust/src/vector_source.rs` (NEW)**

```rust
use std::collections::HashMap;

/// Trait for polymorphic vector source handling
pub trait VectorSource {
    /// Unique identifier for this source (e.g., "openai-ada-002", "forward-links")
    fn source_id(&self) -> String;

    /// Dimensionality of vectors from this source
    fn dimensionality(&self) -> usize;

    /// Fetch vectors from this source
    fn fetch_vectors(&self) -> Result<Vec<VectorWithMetadata>, PluginError>;
}

/// Vector data with associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorWithMetadata {
    pub id: String,           // Note file path or unique ID
    pub label: String,        // Note title or display name
    pub vector: Vec<f64>,     // The actual vector data
    pub source_id: String,    // Which source generated this vector
    pub metadata: HashMap<String, String>,  // Additional metadata
}

// Concrete implementations can be added as needed
// (Most implementations will be on TypeScript side for Obsidian API access)
```

**File: `rust/src/adjacency_matrix.rs` (NEW)**

```rust
use std::collections::HashMap;
use sprs::{CsMat, TriMat};  // Sparse matrix support

/// Represents a link in the vault
#[derive(Debug, Clone)]
pub struct NoteLink {
    pub from_id: usize,
    pub to_id: usize,
}

/// Build adjacency matrix from note links
/// M[i][j] = number of forward links from note i to note j
pub struct AdjacencyMatrixBuilder {
    num_notes: usize,
    note_id_map: HashMap<String, usize>,
}

impl AdjacencyMatrixBuilder {
    pub fn new(note_paths: Vec<String>) -> Self {
        let num_notes = note_paths.len();
        let note_id_map = note_paths
            .into_iter()
            .enumerate()
            .map(|(i, path)| (path, i))
            .collect();

        Self {
            num_notes,
            note_id_map,
        }
    }

    /// Build the adjacency matrix from a list of links
    pub fn build(&self, links: Vec<NoteLink>) -> Result<CsMat<f64>, PluginError> {
        let mut triplets = TriMat::new((self.num_notes, self.num_notes));

        // Count links
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

        // Build sparse matrix
        for ((from, to), count) in link_counts {
            triplets.add_triplet(from, to, count as f64);
        }

        Ok(triplets.to_csr())
    }

    /// Convert adjacency matrix to vector representation
    /// Each row becomes a vector (outgoing link pattern for that note)
    pub fn matrix_to_vectors(&self, matrix: &CsMat<f64>) -> Vec<Vec<f64>> {
        (0..self.num_notes)
            .map(|i| {
                let mut vec = vec![0.0; self.num_notes];
                for (col, &val) in matrix.outer_view(i).unwrap().iter() {
                    vec[col] = val;
                }
                vec
            })
            .collect()
    }
}

#[wasm_bindgen]
pub fn build_adjacency_matrix(
    note_paths_json: &str,
    links_json: &str,
) -> Result<String, JsValue> {
    let note_paths: Vec<String> = serde_json::from_str(note_paths_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse note paths: {}", e)))?;

    let links: Vec<NoteLink> = serde_json::from_str(links_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse links: {}", e)))?;

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder.build(links)
        .map_err(|e| JsValue::from_str(&format!("Failed to build matrix: {}", e)))?;

    let vectors = builder.matrix_to_vectors(&matrix);

    serde_json::to_string(&vectors)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
}
```

**File: `rust/src/dimensionality_reduction.rs`**

```rust
pub trait DimensionalityReducer {
    fn reduce(&self, vectors: &[Vec<f64>], target_dims: usize) -> Result<Vec<Vec<f64>>, PluginError>;
    fn method_name(&self) -> &str;
}

pub struct SVDReducer {
    center: bool,
    scale: bool,
}

impl SVDReducer {
    pub fn new() -> Self {
        Self {
            center: true,
            scale: false,
        }
    }
}

impl DimensionalityReducer for SVDReducer {
    fn reduce(&self, vectors: &[Vec<f64>], target_dims: usize) -> Result<Vec<Vec<f64>>, PluginError> {
        // Implementation using nalgebra SVD
        // 1. Convert to matrix
        // 2. Center if needed
        // 3. Perform SVD
        // 4. Project to target_dims
        // 5. Return reduced vectors

        todo!("SVD implementation")
    }

    fn method_name(&self) -> &str {
        "SVD"
    }
}

// Future implementations:
// pub struct UMAPReducer { ... }
// pub struct TSNEReducer { ... }
```

**File: `rust/src/vector_ops.rs`**

```rust
// Vector utilities
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

pub fn compute_distances(vectors: &[Vec<f64>]) -> Result<Vec<f64>, PluginError> {
    // Pairwise distances or distances from mean
    todo!("Distance computation")
}

pub fn simple_kmeans_clustering(vectors: &[Vec<f64>], k: usize) -> Result<Vec<usize>, PluginError> {
    // Simple k-means for cluster assignment
    todo!("K-means implementation")
}
```

**WASM Bindings in `rust/src/lib.rs`**

```rust
#[wasm_bindgen]
pub fn reduce_dimensions_svd(
    vectors_json: &str,
    target_dims: usize
) -> Result<String, JsValue> {
    let vectors: Vec<Vec<f64>> = serde_json::from_str(vectors_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let reducer = SVDReducer::new();
    let reduced = reducer.reduce(&vectors, target_dims)
        .map_err(|e| JsValue::from_str(&format!("Reduction error: {}", e)))?;

    serde_json::to_string(&reduced)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

#[wasm_bindgen]
pub fn cluster_vectors(
    vectors_json: &str,
    num_clusters: usize
) -> Result<String, JsValue> {
    let vectors: Vec<Vec<f64>> = serde_json::from_str(vectors_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let clusters = simple_kmeans_clustering(&vectors, num_clusters)
        .map_err(|e| JsValue::from_str(&format!("Clustering error: {}", e)))?;

    serde_json::to_string(&clusters)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}
```

**New Dependencies in `Cargo.toml`**

```toml
[dependencies]
nalgebra = { version = "0.33", features = ["serde-serialize"] }
sprs = "0.11"  # Sparse matrix support
```

#### 2. **TypeScript Components** (UI & Obsidian Integration)

**File: `src/visualization/types.ts` (NEW)**

```typescript
export enum VectorSourceType {
	EMBEDDING_MODEL = "embedding_model",
	ADJACENCY_MATRIX = "adjacency_matrix",
}

export interface VectorSourceConfig {
	type: VectorSourceType;
	id: string; // Unique identifier (e.g., "openai-ada-002", "forward-links")
	name: string; // Human-readable name
	dimensionality: number;
	config: EmbeddingSourceConfig | AdjacencySourceConfig;
}

export interface EmbeddingSourceConfig {
	modelName: string;
	qdrantCollection: string;
	endpoint?: string;
}

export interface AdjacencySourceConfig {
	graphType: "forward-links" | "backlinks" | "bidirectional";
	normalize: boolean;
}

export interface VectorDataPoint {
	id: string; // Note file path or unique ID
	label: string; // Note title
	vector: number[]; // Original high-dim vector
	position3d: [number, number, number]; // Reduced 3D coords
	cluster: number; // Cluster assignment

	// Source tracking
	sourceId: string;
	sourceName: string;
	sourceType: VectorSourceType;

	metadata?: Record<string, unknown>;
}
```

**File: `src/visualization/providers/VectorSourceProvider.ts` (NEW)**

```typescript
import { VectorSourceConfig, VectorDataPoint } from "../types";

export interface VectorSourceProvider {
	fetchVectors(config: VectorSourceConfig): Promise<VectorDataPoint[]>;
}
```

**File: `src/visualization/providers/EmbeddingSourceProvider.ts` (NEW)**

```typescript
import { VectorSourceProvider } from "./VectorSourceProvider";
import {
	VectorSourceConfig,
	VectorDataPoint,
	EmbeddingSourceConfig,
	VectorSourceType,
} from "../types";

export class EmbeddingSourceProvider implements VectorSourceProvider {
	constructor(private plugin: HelloWorldPlugin) {}

	/**
	 * Auto-discover available Qdrant collections
	 */
	async discoverSources(): Promise<VectorSourceConfig[]> {
		try {
			const endpoint = this.plugin.settings.qdrantEndpoint || "http://localhost:6333";
			const response = await fetch(`${endpoint}/collections`);

			if (!response.ok) {
				throw new Error(`Failed to fetch collections: ${response.statusText}`);
			}

			const data = await response.json();
			const collections = data.result.collections || [];

			return collections.map((collection: any) => ({
				type: VectorSourceType.EMBEDDING_MODEL,
				id: collection.name,
				name: `${collection.name} (Embedding)`,
				dimensionality: collection.vectors_count || 1536, // Get from collection metadata
				config: {
					modelName: collection.name,
					qdrantCollection: collection.name,
					endpoint,
				} as EmbeddingSourceConfig,
			}));
		} catch (error) {
			console.warn("Failed to auto-discover Qdrant collections:", error);
			return []; // Graceful degradation
		}
	}

	async fetchVectors(config: VectorSourceConfig): Promise<VectorDataPoint[]> {
		const embeddingConfig = config.config as EmbeddingSourceConfig;
		const endpoint = embeddingConfig.endpoint || "http://localhost:6333";

		// Fetch vectors from Qdrant
		const response = await fetch(
			`${endpoint}/collections/${embeddingConfig.qdrantCollection}/points/scroll`,
			{
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					limit: 10000, // Adjust as needed
					with_vector: true,
					with_payload: true,
				}),
			}
		);

		if (!response.ok) {
			throw new Error(`Failed to fetch vectors: ${response.statusText}`);
		}

		const data = await response.json();
		const points = data.result.points || [];

		return points.map((point: any) => ({
			id: point.id,
			label: point.payload?.title || point.id,
			vector: point.vector,
			position3d: [0, 0, 0], // Will be computed later
			cluster: 0, // Will be assigned later
			sourceId: config.id,
			sourceName: config.name,
			sourceType: config.type,
			metadata: point.payload,
		}));
	}
}
```

**File: `src/visualization/providers/AdjacencyMatrixProvider.ts` (NEW)**

```typescript
import { VectorSourceProvider } from "./VectorSourceProvider";
import {
	VectorSourceConfig,
	VectorDataPoint,
	AdjacencySourceConfig,
	VectorSourceType,
} from "../types";
import { TFile } from "obsidian";
import { build_adjacency_matrix } from "../../../rust/pkg";

interface NoteLink {
	from_id: number;
	to_id: number;
}

export class AdjacencyMatrixProvider implements VectorSourceProvider {
	constructor(private plugin: HelloWorldPlugin) {}

	async fetchVectors(config: VectorSourceConfig): Promise<VectorDataPoint[]> {
		const adjConfig = config.config as AdjacencySourceConfig;

		// Get all markdown files in vault
		const files = this.plugin.app.vault.getMarkdownFiles();
		const notePaths = files.map((f) => f.path);
		const pathToId = new Map(notePaths.map((path, i) => [path, i]));

		// Extract links from each file
		const links: NoteLink[] = [];

		for (const file of files) {
			const fromId = pathToId.get(file.path);
			if (fromId === undefined) continue;

			const content = await this.plugin.app.vault.read(file);
			const linkedPaths = this.extractLinks(content, file);

			for (const linkedPath of linkedPaths) {
				const toId = pathToId.get(linkedPath);
				if (toId !== undefined) {
					links.push({ from_id: fromId, to_id: toId });
				}
			}
		}

		// Build adjacency matrix using Rust/WASM
		const vectorsJson = build_adjacency_matrix(JSON.stringify(notePaths), JSON.stringify(links));
		const vectors: number[][] = JSON.parse(vectorsJson);

		// Convert to VectorDataPoint format
		return files.map((file, i) => ({
			id: file.path,
			label: file.basename,
			vector: vectors[i],
			position3d: [0, 0, 0], // Will be computed later
			cluster: 0, // Will be assigned later
			sourceId: config.id,
			sourceName: config.name,
			sourceType: config.type,
			metadata: {
				path: file.path,
				linkCount: links.filter((l) => l.from_id === i).length,
			},
		}));
	}

	/**
	 * Extract linked file paths from note content
	 */
	private extractLinks(content: string, sourceFile: TFile): string[] {
		const links: string[] = [];

		// Extract [[wiki-links]]
		const wikiLinkRegex = /\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/g;
		let match;
		while ((match = wikiLinkRegex.exec(content)) !== null) {
			const linkText = match[1];
			const linkedFile = this.plugin.app.metadataCache.getFirstLinkpathDest(
				linkText,
				sourceFile.path
			);
			if (linkedFile) {
				links.push(linkedFile.path);
			}
		}

		// Extract [markdown](links.md)
		const mdLinkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
		while ((match = mdLinkRegex.exec(content)) !== null) {
			const linkPath = match[2];
			if (linkPath.endsWith(".md")) {
				const linkedFile = this.plugin.app.metadataCache.getFirstLinkpathDest(
					linkPath,
					sourceFile.path
				);
				if (linkedFile) {
					links.push(linkedFile.path);
				}
			}
		}

		return links;
	}
}
```

**File: `src/visualization/VectorDataManager.ts`**

```typescript
import { VectorDataPoint, VectorSourceConfig, VectorSourceType } from "./types";
import { VectorSourceProvider } from "./providers/VectorSourceProvider";
import { EmbeddingSourceProvider } from "./providers/EmbeddingSourceProvider";
import { AdjacencyMatrixProvider } from "./providers/AdjacencyMatrixProvider";
import { reduce_dimensions_svd, cluster_vectors } from "../../rust/pkg";

export class VectorDataManager {
	private cache: Map<string, VectorDataPoint[]>;
	private sourceProviders: Map<VectorSourceType, VectorSourceProvider>;
	private plugin: HelloWorldPlugin;

	constructor(plugin: HelloWorldPlugin) {
		this.cache = new Map();
		this.plugin = plugin;
		this.sourceProviders = new Map();

		// Register providers
		this.registerSourceProvider(
			VectorSourceType.EMBEDDING_MODEL,
			new EmbeddingSourceProvider(plugin)
		);
		this.registerSourceProvider(
			VectorSourceType.ADJACENCY_MATRIX,
			new AdjacencyMatrixProvider(plugin)
		);
	}

	registerSourceProvider(type: VectorSourceType, provider: VectorSourceProvider) {
		this.sourceProviders.set(type, provider);
	}

	/**
	 * Fetch vectors from a specific source
	 */
	async fetchVectors(sourceConfig: VectorSourceConfig): Promise<VectorDataPoint[]> {
		const cacheKey = this.getCacheKey(sourceConfig);

		// Check in-memory cache
		if (this.cache.has(cacheKey)) {
			return this.cache.get(cacheKey)!;
		}

		// Check IndexedDB
		const cached = await this.loadFromIndexedDB(cacheKey);
		if (cached) {
			this.cache.set(cacheKey, cached);
			return cached;
		}

		// Fetch from provider
		const provider = this.sourceProviders.get(sourceConfig.type);
		if (!provider) {
			throw new Error(`No provider registered for source type: ${sourceConfig.type}`);
		}

		const rawVectors = await provider.fetchVectors(sourceConfig);

		// Compute 3D reduction
		const vectors = rawVectors.map((v) => v.vector);
		const reduced3d = await this.computeReduction(vectors, "svd");

		// Compute clusters
		const clusterCount = Math.min(10, Math.max(3, Math.floor(vectors.length / 50)));
		const clusters = await this.computeClusters(vectors, clusterCount);

		// Merge results
		const vectorsWithReduction = rawVectors.map((v, i) => ({
			...v,
			position3d: reduced3d[i] as [number, number, number],
			cluster: clusters[i],
		}));

		// Cache results
		this.cache.set(cacheKey, vectorsWithReduction);
		await this.persistToIndexedDB(cacheKey, vectorsWithReduction);

		return vectorsWithReduction;
	}

	/**
	 * Compute 3D reduction using Rust/WASM
	 */
	private async computeReduction(vectors: number[][], method: "svd" | "umap"): Promise<number[][]> {
		if (method === "svd") {
			const result = reduce_dimensions_svd(JSON.stringify(vectors), 3);
			return JSON.parse(result);
		}
		// Future: handle UMAP
		throw new Error(`Unsupported reduction method: ${method}`);
	}

	/**
	 * Compute cluster assignments
	 */
	private async computeClusters(vectors: number[][], numClusters: number): Promise<number[]> {
		const result = cluster_vectors(JSON.stringify(vectors), numClusters);
		return JSON.parse(result);
	}

	/**
	 * Generate cache key that includes source information
	 */
	private getCacheKey(sourceConfig: VectorSourceConfig): string {
		const vaultName = this.plugin.app.vault.getName();
		return `${sourceConfig.type}:${sourceConfig.id}:${vaultName}`;
	}

	/**
	 * Auto-discover available vector sources
	 */
	async discoverAvailableSources(): Promise<VectorSourceConfig[]> {
		const sources: VectorSourceConfig[] = [];

		// Discover embedding sources from Qdrant
		const embeddingProvider = this.sourceProviders.get(
			VectorSourceType.EMBEDDING_MODEL
		) as EmbeddingSourceProvider;
		if (embeddingProvider) {
			const embeddingSources = await embeddingProvider.discoverSources();
			sources.push(...embeddingSources);
		}

		// Add adjacency matrix source
		const files = this.plugin.app.vault.getMarkdownFiles();
		sources.push({
			type: VectorSourceType.ADJACENCY_MATRIX,
			id: "forward-links",
			name: "Forward Links Graph",
			dimensionality: files.length,
			config: {
				graphType: "forward-links",
				normalize: true,
			},
		});

		return sources;
	}

	private async persistToIndexedDB(key: string, data: VectorDataPoint[]) {
		// IndexedDB persistence for long-term caching
		// Implementation depends on IndexedDB setup
	}

	private async loadFromIndexedDB(key: string): Promise<VectorDataPoint[] | null> {
		// Load from IndexedDB
		// Implementation depends on IndexedDB setup
		return null;
	}
}
```

**File: `src/visualization/VectorVisualizationView.ts`**

```typescript
import { ItemView, WorkspaceLeaf } from "obsidian";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import { VectorDataManager } from "./VectorDataManager";
import { VectorDataPoint, VectorSourceConfig } from "./types";

export const VECTOR_VIEW_TYPE = "vector-3d-view";

export class VectorVisualizationView extends ItemView {
	private scene: THREE.Scene;
	private camera: THREE.PerspectiveCamera;
	private renderer: THREE.WebGLRenderer;
	private controls: OrbitControls;
	private raycaster: THREE.Raycaster;

	// Data
	private vectorData: VectorDataPoint[];
	private dataManager: VectorDataManager;

	// UI Controls
	private controlPanel: HTMLElement;
	private sourceSelector: HTMLSelectElement;
	private reductionMethodSelector: HTMLSelectElement;
	private clusterCountInput: HTMLInputElement;

	// State
	private currentSource: VectorSourceConfig | null = null;
	private availableSources: VectorSourceConfig[] = [];

	constructor(
		leaf: WorkspaceLeaf,
		private plugin: HelloWorldPlugin
	) {
		super(leaf);
		this.dataManager = new VectorDataManager(plugin);
	}

	getViewType(): string {
		return VECTOR_VIEW_TYPE;
	}
	getDisplayText(): string {
		return "3D Vector Visualization";
	}
	getIcon(): string {
		return "cube";
	}

	async onOpen() {
		const container = this.containerEl.children[1];
		container.empty();

		// Discover available sources
		await this.loadAvailableSources();

		// Build UI
		this.buildControlPanel(container);

		// Initialize Three.js scene
		this.initializeScene(container);
		this.setupControls();
		this.setupEventListeners();

		// Load default source
		if (this.availableSources.length > 0) {
			this.currentSource = this.availableSources[0];
			await this.loadAndVisualize();
		}
	}

	private async loadAvailableSources() {
		this.availableSources = await this.dataManager.discoverAvailableSources();
	}

	private buildControlPanel(container: Element) {
		this.controlPanel = container.createDiv({ cls: "vector-control-panel" });

		// Source selector
		const sourceGroup = this.controlPanel.createDiv({ cls: "control-group" });
		sourceGroup.createEl("label", { text: "Vector Source:" });
		this.sourceSelector = sourceGroup.createEl("select");

		for (const source of this.availableSources) {
			const option = this.sourceSelector.createEl("option", {
				value: source.id,
				text: `${source.name} (${source.dimensionality}D)`,
			});
		}

		this.sourceSelector.addEventListener("change", async () => {
			await this.onSourceChanged();
		});

		// Refresh button
		const refreshBtn = this.controlPanel.createEl("button", { text: "🔄 Refresh Sources" });
		refreshBtn.addEventListener("click", async () => {
			await this.loadAvailableSources();
			this.rebuildSourceSelector();
		});

		// Method selector
		const methodGroup = this.controlPanel.createDiv({ cls: "control-group" });
		methodGroup.createEl("label", { text: "Method:" });
		this.reductionMethodSelector = methodGroup.createEl("select");
		this.reductionMethodSelector.createEl("option", { value: "svd", text: "SVD" });
		// Future: UMAP, t-SNE

		// Export button
		const exportBtn = this.controlPanel.createEl("button", { text: "Export PNG" });
		exportBtn.addEventListener("click", () => this.exportAsImage());
	}

	private rebuildSourceSelector() {
		this.sourceSelector.empty();
		for (const source of this.availableSources) {
			this.sourceSelector.createEl("option", {
				value: source.id,
				text: `${source.name} (${source.dimensionality}D)`,
			});
		}
	}

	private async onSourceChanged() {
		const sourceId = this.sourceSelector.value;
		this.currentSource = this.availableSources.find((s) => s.id === sourceId) || null;

		if (this.currentSource) {
			await this.loadAndVisualize();
		}
	}

	private async loadAndVisualize() {
		if (!this.currentSource) return;

		// Show loading indicator
		// TODO: Add loading UI

		// Fetch vectors from selected source
		this.vectorData = await this.dataManager.fetchVectors(this.currentSource);

		// Render visualization
		this.renderPoints();
	}

	private initializeScene(container: Element) {
		// Three.js scene setup
		this.scene = new THREE.Scene();
		this.scene.background = new THREE.Color(0x1e1e1e);

		// Camera
		this.camera = new THREE.PerspectiveCamera(
			75,
			container.clientWidth / container.clientHeight,
			0.1,
			1000
		);
		this.camera.position.z = 5;

		// Renderer
		this.renderer = new THREE.WebGLRenderer({ antialias: true });
		this.renderer.setSize(container.clientWidth, container.clientHeight);
		container.appendChild(this.renderer.domElement);

		// Lighting
		const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
		this.scene.add(ambientLight);

		// Animation loop
		const animate = () => {
			requestAnimationFrame(animate);
			this.controls.update();
			this.renderer.render(this.scene, this.camera);
		};
		animate();
	}

	private setupControls() {
		this.controls = new OrbitControls(this.camera, this.renderer.domElement);
		this.controls.enableDamping = true;
		this.controls.dampingFactor = 0.05;
	}

	private setupEventListeners() {
		this.raycaster = new THREE.Raycaster();
		// TODO: Add click detection for point selection
	}

	private renderPoints() {
		// Clear existing points
		// TODO: Remove old point cloud

		// Create point cloud from vectorData
		const geometry = new THREE.BufferGeometry();
		const positions = new Float32Array(this.vectorData.length * 3);
		const colors = new Float32Array(this.vectorData.length * 3);

		for (let i = 0; i < this.vectorData.length; i++) {
			const point = this.vectorData[i];
			positions[i * 3] = point.position3d[0];
			positions[i * 3 + 1] = point.position3d[1];
			positions[i * 3 + 2] = point.position3d[2];

			// Color based on cluster
			const color = this.getClusterColor(point.cluster);
			colors[i * 3] = color.r;
			colors[i * 3 + 1] = color.g;
			colors[i * 3 + 2] = color.b;
		}

		geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
		geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));

		const material = new THREE.PointsMaterial({
			size: 0.05,
			vertexColors: true,
		});

		const points = new THREE.Points(geometry, material);
		this.scene.add(points);
	}

	private getClusterColor(cluster: number): THREE.Color {
		const hue = (cluster * 137.508) % 360; // Golden angle for better distribution
		return new THREE.Color().setHSL(hue / 360, 0.8, 0.6);
	}

	private onPointClick(point: VectorDataPoint) {
		// Show metadata in info panel
		// TODO: Implement info panel
	}

	private exportAsImage() {
		// Export canvas as PNG
		const dataURL = this.renderer.domElement.toDataURL("image/png");
		const link = document.createElement("a");
		link.download = "vector-visualization.png";
		link.href = dataURL;
		link.click();
	}

	async onClose() {
		// Cleanup Three.js resources
		this.renderer.dispose();
	}
}
```

**File: `src/visualization/ColorSchemes.ts`**

```typescript
export interface ColorScheme {
	name: string;
	getColor(clusterIndex: number, totalClusters: number): THREE.Color;
}

export const DEFAULT_COLOR_SCHEMES: ColorScheme[] = [
	{
		name: "Rainbow",
		getColor: (index, total) => {
			const hue = (index / total) * 360;
			return new THREE.Color().setHSL(hue / 360, 0.8, 0.6);
		},
	},
	{
		name: "Golden Angle",
		getColor: (index, total) => {
			const hue = (index * 137.508) % 360;
			return new THREE.Color().setHSL(hue / 360, 0.8, 0.6);
		},
	},
	{
		name: "Categorical",
		getColor: (index, total) => {
			const palette = [
				0xff6b6b, 0x4ecdc4, 0x45b7d1, 0xf7b731, 0x5f27cd, 0x00d2d3, 0xff9ff3, 0x54a0ff,
			];
			return new THREE.Color(palette[index % palette.length]);
		},
	},
];
```

**Integration in `main.ts`**

```typescript
import { VectorVisualizationView, VECTOR_VIEW_TYPE } from "./visualization/VectorVisualizationView";

export default class HelloWorldPlugin extends Plugin {
	override async onload(): Promise<void> {
		await init(wasmData);

		// Register the view
		this.registerView(VECTOR_VIEW_TYPE, (leaf) => new VectorVisualizationView(leaf, this));

		// Add ribbon icon
		this.addRibbonIcon("cube", "Open Vector Visualization", async () => {
			await this.activateView();
		});

		// Add command
		this.addCommand({
			id: "open-vector-visualization",
			name: "Open 3D Vector Visualization",
			callback: async () => {
				await this.activateView();
			},
		});
	}

	async activateView() {
		const { workspace } = this.app;

		let leaf = workspace.getLeavesOfType(VECTOR_VIEW_TYPE)[0];

		if (!leaf) {
			const rightLeaf = workspace.getRightLeaf(false);
			await rightLeaf?.setViewState({
				type: VECTOR_VIEW_TYPE,
				active: true,
			});
			leaf = workspace.getLeavesOfType(VECTOR_VIEW_TYPE)[0];
		}

		if (leaf) {
			workspace.revealLeaf(leaf);
		}
	}
}
```

#### 3. **Caching & Performance Strategy**

**Multi-Level Cache with Source Awareness:**

1. **In-Memory Cache** (Map in `VectorDataManager`): Fast access for current session
2. **IndexedDB Cache**: Persistent across Obsidian restarts
3. **Cache Keys**: Include source type and ID: `${sourceType}:${sourceId}:${vaultName}`

**Cache Invalidation Strategy:**

- **Embedding vectors**: Manual refresh or time-based expiration
- **Adjacency matrix**: Invalidate when vault modification time changes
- **Reduced 3D coordinates**: Cache separately per source+method combination

**Computation Strategy:**

- **Initial Load**: Check cache → If miss, fetch from source → Compute reduction → Cache
- **Source Switching**: Load from cache if available, otherwise compute
- **Background Refresh**: Option to recompute when vault changes detected
- **Progressive Rendering**: For 1200+ points, use instanced rendering in Three.js

#### 4. **UI/UX Design**

**View Layout:**

```
┌─────────────────────────────────────────────────────────┐
│  [Controls Panel]                                       │
│  Source: [Select Vector Source ▼] [🔄 Refresh]         │
│    Options:                                             │
│    - OpenAI Ada-002 (1536D) [Embedding]                │
│    - OpenAI 3-Large (3072D) [Embedding]                │
│    - Forward Links Graph (1200D) [Adjacency Matrix]    │
│  Method: [SVD ▼] | Colors: [Rainbow ▼]                 │
│  [Export PNG] [Settings]                                │
├─────────────────────────────────────────────────────────┤
│                                                         │
│                                                         │
│              [3D Visualization Canvas]                  │
│                                                         │
│                   (Three.js Scene)                      │
│                                                         │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  [Info Panel - Shows metadata on point click]           │
│  Note: "My Note Title.md"                               │
│  Source: OpenAI Ada-002 (Embedding) | Cluster: 2       │
│  Original Dimensions: 1536 | Position: (0.45, -0.23, 0.67) │
└─────────────────────────────────────────────────────────┘
```

**Interactions:**

- **Mouse drag**: Rotate camera (OrbitControls)
- **Mouse wheel**: Zoom in/out
- **Click point**: Highlight + show metadata in info panel
- **Right-click point**: Quick actions (open note, copy link)
- **Double-click point**: Open the associated note in Obsidian

#### 5. **File Structure**

```
simons-obsidian-plugin/
├── main.ts                           # Updated with view registration
├── src/
│   └── visualization/
│       ├── VectorVisualizationView.ts
│       ├── VectorDataManager.ts
│       ├── providers/                # NEW directory
│       │   ├── VectorSourceProvider.ts
│       │   ├── EmbeddingSourceProvider.ts
│       │   └── AdjacencyMatrixProvider.ts
│       ├── ColorSchemes.ts
│       ├── ThreeJsUtils.ts          # Three.js helpers
│       └── types.ts                 # TypeScript interfaces (NEW)
├── rust/
│   └── src/
│       ├── lib.rs                    # Updated with new exports
│       ├── vector_source.rs          # NEW - Trait abstraction
│       ├── adjacency_matrix.rs       # NEW - Link-graph vectors
│       ├── dimensionality_reduction.rs  # NEW
│       ├── vector_ops.rs             # NEW
│       └── error.rs                  # Updated with new error variants
├── rust/tests/
│   ├── vector_source_test.rs         # NEW (snapshot tests)
│   ├── adjacency_matrix_test.rs      # NEW (snapshot tests)
│   ├── dimensionality_reduction_test.rs  # NEW (snapshot tests)
│   └── vector_ops_test.rs            # NEW (snapshot tests)
├── styles.css                        # Updated with visualization styles
└── package.json                      # Updated with three dependencies
```

#### 6. **Dependencies to Add**

**TypeScript (`package.json`):**

```json
{
	"dependencies": {
		"three": "^0.160.0",
		"@types/three": "^0.160.0"
	}
}
```

**Rust (`Cargo.toml`):**

```toml
[dependencies]
nalgebra = { version = "0.33", features = ["serde-serialize"] }
sprs = "0.11"  # Sparse matrix support
```

#### 7. **Testing Strategy**

Following snapshot testing paradigm from `CLAUDE.md`:

**Rust Tests (`rust/tests/adjacency_matrix_test.rs`):**

```rust
use simons_obsidian_plugin::adjacency_matrix::*;

#[test]
fn test_adjacency_matrix_basic() {
    let note_paths = vec![
        "note1.md".to_string(),
        "note2.md".to_string(),
        "note3.md".to_string(),
    ];

    let links = vec![
        NoteLink { from_id: 0, to_id: 1 },
        NoteLink { from_id: 0, to_id: 2 },
        NoteLink { from_id: 1, to_id: 2 },
    ];

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder.build(links).unwrap();
    let vectors = builder.matrix_to_vectors(&matrix);

    // Serialize for deterministic snapshot
    let snapshot = serde_json::to_string_pretty(&vectors).unwrap();
    insta::assert_snapshot!(snapshot);
}

#[test]
fn test_adjacency_matrix_self_links() {
    let note_paths = vec!["note1.md".to_string()];
    let links = vec![NoteLink { from_id: 0, to_id: 0 }];

    let builder = AdjacencyMatrixBuilder::new(note_paths);
    let matrix = builder.build(links).unwrap();
    let vectors = builder.matrix_to_vectors(&matrix);

    let snapshot = serde_json::to_string_pretty(&vectors).unwrap();
    insta::assert_snapshot!(snapshot);
}
```

#### 8. **Error Handling**

**New Error Variants in `rust/src/error.rs`:**

```rust
pub enum PluginError {
    // Existing variants...

    DimensionalityReductionError {
        method: String,
        reason: String,
    },
    InvalidVectorDimensions {
        expected: usize,
        got: usize,
        vector_index: usize,
    },
    InsufficientData {
        required: usize,
        provided: usize,
    },
    InvalidLinkIndex {
        from: usize,
        to: usize,
        max: usize,
    },
    ZeroNormVector,
}
```

### Implementation Plan

The implementation will proceed in these phases:

1. **Phase 1: Rust Foundation** (Core computation layer)
   - Add nalgebra and sprs dependencies
   - Implement `vector_source.rs` with trait abstraction
   - Implement `adjacency_matrix.rs` with sparse matrix support
     - M[i][j] = # of forward links from note i to note j
     - Efficient sparse representation
     - Matrix-to-vectors conversion
   - Implement `dimensionality_reduction.rs` with SVD
   - Implement `vector_ops.rs` with normalization and clustering
   - Write snapshot tests for all Rust functions
   - Update error handling
   - Add WASM bindings

2. **Phase 2: TypeScript Type System & Provider Pattern**
   - Create `types.ts` with multi-source types
   - Create `VectorSourceProvider` interface
   - Create `AdjacencyMatrixProvider` (can work offline)
   - Test adjacency matrix provider independently

3. **Phase 3: Data Manager with Multi-Source Support**
   - Update `VectorDataManager` with provider pattern
   - Implement source-aware caching
   - Add source discovery mechanism
   - Test source switching

4. **Phase 4: Qdrant Integration**
   - Create `EmbeddingSourceProvider`
   - Implement auto-discovery of Qdrant collections
   - Add graceful degradation if Qdrant unavailable
   - Test with real Qdrant instance

5. **Phase 5: Three.js Visualization**
   - Create `VectorVisualizationView` extending ItemView
   - Set up Three.js scene, camera, renderer
   - Implement OrbitControls for interaction
   - Create basic point cloud rendering
   - Add raycasting for point selection

6. **Phase 6: UI Controls & Source Selection**
   - Build control panel with source selector
   - Implement source switching UI
   - Add "Refresh Sources" functionality
   - Implement color schemes
   - Add info panel for selected points
   - Implement export functionality

7. **Phase 7: Integration & Polish**
   - Register view in `main.ts`
   - Add ribbon icon and command
   - Style with CSS
   - Performance optimization (instanced rendering if needed)
   - Add loading indicators
   - Handle errors gracefully

8. **Phase 8: Settings & Configuration**
   - Add settings tab for Qdrant configuration
   - Add default source selection
   - Add cache expiration settings

9. **Phase 9: Testing & Documentation**
   - Write comprehensive snapshot tests
   - Test all source types
   - Test cache invalidation
   - Test source switching
   - Update `CLAUDE.md` with new architecture

10. **Phase 10: Future Enhancements** (Post-MVP)
    - UMAP implementation
    - Additional dimensionality reduction methods
    - Advanced clustering algorithms
    - Multi-source comparison view
    - Animation/transitions

### Key Design Decisions & Rationale

1. **Provider Pattern**: Extensible architecture for adding new vector sources without modifying core logic

2. **Adjacency Matrix as Primary Feature**: Treating link-graph vectors as a first-class source enables comparison of semantic (embedding) vs structural (links) clustering

3. **Auto-Discovery**: Qdrant collections are auto-discovered on startup, reducing manual configuration burden

4. **Source-Specific Caching**: Each source gets its own cache namespace to avoid conflicts and enable efficient switching

5. **Graceful Degradation**: If Qdrant is unavailable, adjacency matrix source still works. System never completely fails.

6. **Sparse Matrix Representation**: Most notes link to <10 other notes out of 1200+, so sparse representation saves memory (~99% sparsity)

7. **Trait-Based Architecture**: Using `VectorSource` and `DimensionalityReducer` traits allows easy addition of new implementations

8. **Multi-Level Caching**: Ensures fast view opening after initial computation (requirement: performant after Obsidian has been open)

9. **Separation of Concerns**: Data fetching, computation, and visualization are distinct layers for maintainability

10. **Performance-First**: Rust handles heavy computation (matrix building, SVD, clustering), TypeScript only for rendering and interaction

11. **Functional Style**: Follows "OCaml with manual GC" philosophy - immutable data structures, ADTs for errors, trait-based polymorphism

This architecture is extensible, performant, and aligns with your project's functional programming philosophy while keeping TypeScript purely for Obsidian API integration.

---

**Documentation written by Claude Code**
