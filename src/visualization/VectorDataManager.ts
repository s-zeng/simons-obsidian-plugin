import type { VectorDataPoint, VectorSourceConfig, VectorSourceType } from "./types";
import { VectorSourceType as VectorSourceTypeEnum } from "./types";
import type { VectorSourceProvider } from "./providers/VectorSourceProvider";
import { EmbeddingSourceProvider } from "./providers/EmbeddingSourceProvider";
import { AdjacencyMatrixProvider } from "./providers/AdjacencyMatrixProvider";
import { reduce_dimensions_svd, cluster_vectors } from "../../pkg/rust";
import type HelloWorldPlugin from "../../main";

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
			VectorSourceTypeEnum.EMBEDDING_MODEL,
			new EmbeddingSourceProvider(plugin)
		);
		this.registerSourceProvider(
			VectorSourceTypeEnum.ADJACENCY_MATRIX,
			new AdjacencyMatrixProvider(plugin)
		);
	}

	registerSourceProvider(type: VectorSourceType, provider: VectorSourceProvider): void {
		this.sourceProviders.set(type, provider);
	}

	/**
	 * Fetch vectors from a specific source.
	 */
	async fetchVectors(sourceConfig: VectorSourceConfig): Promise<VectorDataPoint[]> {
		console.log(
			"[VectorDataManager] Fetching vectors for source:",
			sourceConfig.id,
			sourceConfig.type
		);
		const cacheKey = this.getCacheKey(sourceConfig);

		// Check in-memory cache
		const cached = this.cache.get(cacheKey);
		if (cached) {
			console.log("[VectorDataManager] Using cached data:", cached.length, "points");
			return cached;
		}

		try {
			// Fetch from provider
			const provider = this.sourceProviders.get(sourceConfig.type);
			if (!provider) {
				throw new Error(`No provider registered for source type: ${sourceConfig.type}`);
			}

			console.log("[VectorDataManager] Fetching from provider...");
			const rawVectors = await provider.fetchVectors(sourceConfig);
			console.log("[VectorDataManager] Fetched", rawVectors.length, "raw vectors");

			if (rawVectors.length === 0) {
				console.warn("[VectorDataManager] No vectors returned from provider!");
				return [];
			}

			// Compute 3D reduction
			const vectors = rawVectors.map((v) => v.vector);
			console.log(
				"[VectorDataManager] Computing 3D reduction for",
				vectors.length,
				"vectors, dimensions:",
				vectors[0]?.length
			);
			const reduced3d = this.computeReduction(vectors, "svd");
			console.log("[VectorDataManager] Reduction complete, result count:", reduced3d.length);

			// Compute clusters
			const clusterCount = Math.min(10, Math.max(3, Math.floor(vectors.length / 50)));
			console.log("[VectorDataManager] Computing", clusterCount, "clusters...");
			const clusters = this.computeClusters(vectors, clusterCount);
			console.log("[VectorDataManager] Clustering complete");

			// Merge results
			const vectorsWithReduction = rawVectors.map((v, i) => ({
				...v,
				position3d: (reduced3d[i] || [0, 0, 0]) as [number, number, number],
				cluster: clusters[i] || 0,
			}));

			console.log("[VectorDataManager] Final data points:", vectorsWithReduction.length);
			console.log("[VectorDataManager] Sample point:", vectorsWithReduction[0]);

			// Cache results
			this.cache.set(cacheKey, vectorsWithReduction);

			return vectorsWithReduction;
		} catch (error) {
			console.error("[VectorDataManager] Error fetching vectors:", error);
			throw error;
		}
	}

	/**
	 * Compute 3D reduction using Rust/WASM.
	 */
	private computeReduction(vectors: number[][], method: "svd" | "umap"): number[][] {
		try {
			if (method === "svd") {
				const inputJson = JSON.stringify(vectors);
				console.log(
					"[VectorDataManager] Calling WASM SVD with",
					vectors.length,
					"vectors, JSON size:",
					inputJson.length
				);
				const result = reduce_dimensions_svd(inputJson, 3);
				const parsed = JSON.parse(result) as number[][];
				console.log(
					"[VectorDataManager] SVD returned",
					parsed.length,
					"vectors with dims:",
					parsed[0]?.length
				);
				return parsed;
			}
			throw new Error(`Unsupported reduction method: ${method}`);
		} catch (error) {
			console.error("[VectorDataManager] Error in computeReduction:", error);
			throw error;
		}
	}

	/**
	 * Compute cluster assignments.
	 */
	private computeClusters(vectors: number[][], numClusters: number): number[] {
		try {
			console.log(
				"[VectorDataManager] Calling WASM clustering with",
				vectors.length,
				"vectors,",
				numClusters,
				"clusters"
			);
			const result = cluster_vectors(JSON.stringify(vectors), numClusters);
			const parsed = JSON.parse(result) as number[];
			console.log("[VectorDataManager] Clustering returned", parsed.length, "assignments");
			return parsed;
		} catch (error) {
			console.error("[VectorDataManager] Error in computeClusters:", error);
			throw error;
		}
	}

	/**
	 * Generate cache key that includes source information.
	 */
	private getCacheKey(sourceConfig: VectorSourceConfig): string {
		const vaultName = this.plugin.app.vault.getName();
		return `${sourceConfig.type}:${sourceConfig.id}:${vaultName}`;
	}

	/**
	 * Auto-discover available vector sources.
	 */
	async discoverAvailableSources(): Promise<VectorSourceConfig[]> {
		console.log("[VectorDataManager] Discovering available sources...");
		const sources: VectorSourceConfig[] = [];

		// Discover embedding sources from Qdrant
		const embeddingProvider = this.sourceProviders.get(
			VectorSourceTypeEnum.EMBEDDING_MODEL
		) as EmbeddingSourceProvider;
		if (embeddingProvider) {
			const embeddingSources = await embeddingProvider.discoverSources();
			console.log("[VectorDataManager] Found", embeddingSources.length, "embedding sources");
			sources.push(...embeddingSources);
		}

		// Add adjacency matrix source
		const files = this.plugin.app.vault.getMarkdownFiles();
		console.log("[VectorDataManager] Found", files.length, "markdown files for adjacency matrix");
		sources.push({
			type: VectorSourceTypeEnum.ADJACENCY_MATRIX,
			id: "forward-links",
			name: "Forward Links Graph",
			dimensionality: files.length,
			config: {
				graphType: "forward-links",
				normalize: true,
			},
		});

		console.log("[VectorDataManager] Total sources discovered:", sources.length);
		return sources;
	}
}
