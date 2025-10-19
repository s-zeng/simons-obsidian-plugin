import type { VectorSourceConfig, VectorDataPoint } from "../types";

export interface VectorSourceProvider {
	fetchVectors(config: VectorSourceConfig): Promise<VectorDataPoint[]>;
}
