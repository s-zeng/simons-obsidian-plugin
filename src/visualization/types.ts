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
