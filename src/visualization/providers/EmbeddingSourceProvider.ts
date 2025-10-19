import type { VectorSourceProvider } from "./VectorSourceProvider";
import type { VectorSourceConfig, VectorDataPoint, EmbeddingSourceConfig } from "../types";
import { VectorSourceType as VectorSourceTypeEnum } from "../types";
import type HelloWorldPlugin from "../../../main";

// Qdrant API response types (external API uses snake_case)
interface QdrantCollection {
	name: string;
	// eslint-disable-next-line @typescript-eslint/naming-convention
	vectors_count?: number;
}

interface QdrantCollectionsResponse {
	result?: {
		collections?: QdrantCollection[];
	};
}

interface QdrantPoint {
	id: string;
	vector: number[];
	payload?: {
		title?: string;
		[key: string]: unknown;
	};
}

interface QdrantScrollResponse {
	result?: {
		points?: QdrantPoint[];
	};
}

export class EmbeddingSourceProvider implements VectorSourceProvider {
	constructor(_plugin: HelloWorldPlugin) {
		// Reserved for future use (e.g., settings access)
	}

	/**
	 * Auto-discover available Qdrant collections.
	 */
	async discoverSources(): Promise<VectorSourceConfig[]> {
		try {
			const endpoint = "http://localhost:6333"; // TODO: Make configurable
			const response = await fetch(`${endpoint}/collections`);

			if (!response.ok) {
				throw new Error(`Failed to fetch collections: ${response.statusText}`);
			}

			const data = (await response.json()) as QdrantCollectionsResponse;
			const collections = data.result?.collections || [];

			return collections.map((collection) => ({
				type: VectorSourceTypeEnum.EMBEDDING_MODEL,
				id: collection.name,
				name: `${collection.name} (Embedding)`,
				dimensionality: collection.vectors_count || 1536,
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
				headers: {
					// eslint-disable-next-line @typescript-eslint/naming-convention
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					limit: 10000, // Adjust as needed
					// eslint-disable-next-line @typescript-eslint/naming-convention
					with_vector: true,
					// eslint-disable-next-line @typescript-eslint/naming-convention
					with_payload: true,
				}),
			}
		);

		if (!response.ok) {
			throw new Error(`Failed to fetch vectors: ${response.statusText}`);
		}

		const data = (await response.json()) as QdrantScrollResponse;
		const points = data.result?.points || [];

		return points.map((point) => ({
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
