import type { TFile } from "obsidian";
import type { VectorSourceProvider } from "./VectorSourceProvider";
import type { VectorSourceConfig, VectorDataPoint, AdjacencySourceConfig } from "../types";
import { build_adjacency_matrix, build_laplacian_matrix } from "../../../pkg/rust";
import type HelloWorldPlugin from "../../../main";

interface NoteLink {
	fromId: number;
	toId: number;
}

function isAdjacencySourceConfig(
	config: VectorSourceConfig["config"]
): config is AdjacencySourceConfig {
	return "graphType" in config;
}

export class AdjacencyMatrixProvider implements VectorSourceProvider {
	constructor(private plugin: HelloWorldPlugin) {}

	async fetchVectors(config: VectorSourceConfig): Promise<VectorDataPoint[]> {
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
					links.push({ fromId: fromId, toId: toId });
				}
			}
		}

		// Build matrix using Rust/WASM (adjacency or Laplacian based on config)
		if (!isAdjacencySourceConfig(config.config)) {
			throw new Error("Invalid config for AdjacencyMatrixProvider");
		}
		const isLaplacian = config.config.graphType === "laplacian";
		const vectorsJson: string = isLaplacian
			? build_laplacian_matrix(JSON.stringify(notePaths), JSON.stringify(links))
			: build_adjacency_matrix(JSON.stringify(notePaths), JSON.stringify(links));
		const vectors = JSON.parse(vectorsJson) as number[][];

		// Convert to VectorDataPoint format
		return files.map((file, i) => ({
			id: file.path,
			label: file.basename,
			vector: vectors[i] || [],
			position3d: [0, 0, 0] as [number, number, number],
			cluster: 0,
			sourceId: config.id,
			sourceName: config.name,
			sourceType: config.type,
			metadata: {
				path: file.path,
				linkCount: links.filter((l) => l.fromId === i).length,
			},
		}));
	}

	/**
	 * Extract linked file paths from note content.
	 */
	private extractLinks(content: string, sourceFile: TFile): string[] {
		const links: string[] = [];

		// Extract [[wiki-links]]
		const wikiLinkRegex = /\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/g;
		let match;
		while ((match = wikiLinkRegex.exec(content)) !== null) {
			const linkText = match[1];
			if (linkText) {
				const linkedFile = this.plugin.app.metadataCache.getFirstLinkpathDest(
					linkText,
					sourceFile.path
				);
				if (linkedFile) {
					links.push(linkedFile.path);
				}
			}
		}

		// Extract [markdown](links.md)
		const mdLinkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
		while ((match = mdLinkRegex.exec(content)) !== null) {
			const linkPath = match[2];
			if (linkPath && linkPath.endsWith(".md")) {
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
