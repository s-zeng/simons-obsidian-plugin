import { ItemView } from "obsidian";
import type { WorkspaceLeaf } from "obsidian";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { VectorDataManager } from "./VectorDataManager";
import type { VectorDataPoint, VectorSourceConfig } from "./types";
import { DEFAULT_COLOR_SCHEMES } from "./ColorSchemes";
import type HelloWorldPlugin from "../../main";

export const VECTOR_VIEW_TYPE = "vector-3d-view";

export class VectorVisualizationView extends ItemView {
	private scene!: THREE.Scene;
	private camera!: THREE.PerspectiveCamera;
	private renderer!: THREE.WebGLRenderer;
	private controls!: OrbitControls;

	// Data
	private vectorData: VectorDataPoint[] = [];
	private dataManager: VectorDataManager;

	// UI Controls
	private controlPanel!: HTMLElement;
	private sourceSelector!: HTMLSelectElement;
	private canvasContainer!: HTMLElement;
	private statusPanel!: HTMLElement;

	// State
	private currentSource: VectorSourceConfig | null = null;
	private availableSources: VectorSourceConfig[] = [];
	private pointCloud: THREE.Points | null = null;

	constructor(leaf: WorkspaceLeaf, _plugin: HelloWorldPlugin) {
		super(leaf);
		this.dataManager = new VectorDataManager(_plugin);
	}

	override getViewType(): string {
		return VECTOR_VIEW_TYPE;
	}

	override getDisplayText(): string {
		return "3D Vector Visualization";
	}

	override getIcon(): string {
		return "cube";
	}

	override async onOpen(): Promise<void> {
		console.log("[VectorVisualizationView] onOpen called");
		const container = this.containerEl.children[1];
		if (!container) {
			console.error("[VectorVisualizationView] Container element not found!");
			return;
		}

		container.empty();
		container.addClass("vector-visualization-container");

		try {
			// Discover available sources
			console.log("[VectorVisualizationView] Loading available sources...");
			this.updateStatus("Loading available sources...");
			await this.loadAvailableSources();

			// Build UI
			console.log("[VectorVisualizationView] Building control panel...");
			this.buildControlPanel(container);
			this.buildStatusPanel(container);

			// Create canvas container
			console.log("[VectorVisualizationView] Creating canvas container...");
			this.canvasContainer = container.createDiv({ cls: "vector-canvas-container" });

			// Initialize Three.js scene
			console.log("[VectorVisualizationView] Initializing Three.js scene...");
			this.updateStatus("Initializing 3D scene...");
			this.initializeScene(this.canvasContainer);
			this.setupControls();
			this.setupEventListeners();

			// Load default source
			if (this.availableSources.length > 0) {
				console.log(
					"[VectorVisualizationView] Loading default source:",
					this.availableSources[0]?.id
				);
				this.currentSource = this.availableSources[0] || null;
				if (this.currentSource) {
					this.updateStatus(`Loading vectors from ${this.currentSource.name}`);
				}
				await this.loadAndVisualize();
			} else {
				console.warn("[VectorVisualizationView] No sources available!");
				this.updateStatus("Error: No sources available", true);
			}
		} catch (error) {
			console.error("[VectorVisualizationView] Error in onOpen:", error);
			this.updateStatus(`Error: ${String(error)}`, true);
		}
	}

	private async loadAvailableSources(): Promise<void> {
		this.availableSources = await this.dataManager.discoverAvailableSources();
	}

	private buildControlPanel(container: Element): void {
		this.controlPanel = container.createDiv({ cls: "vector-control-panel" });

		// Source selector
		const sourceGroup = this.controlPanel.createDiv({ cls: "control-group" });
		sourceGroup.createEl("label", { text: "Vector Source:" });
		this.sourceSelector = sourceGroup.createEl("select");

		for (const source of this.availableSources) {
			this.sourceSelector.createEl("option", {
				value: source.id,
				text: `${source.name} (${source.dimensionality}D)`,
			});
		}

		this.sourceSelector.addEventListener("change", () => {
			void this.onSourceChanged();
		});

		// Refresh button
		const refreshBtn = this.controlPanel.createEl("button", { text: "🔄 Refresh Sources" });
		refreshBtn.addEventListener("click", () => {
			void this.loadAvailableSources().then(() => {
				this.rebuildSourceSelector();
			});
		});

		// Export button
		const exportBtn = this.controlPanel.createEl("button", { text: "Export PNG" });
		exportBtn.addEventListener("click", () => this.exportAsImage());
	}

	private rebuildSourceSelector(): void {
		this.sourceSelector.empty();
		for (const source of this.availableSources) {
			this.sourceSelector.createEl("option", {
				value: source.id,
				text: `${source.name} (${source.dimensionality}D)`,
			});
		}
	}

	private async onSourceChanged(): Promise<void> {
		const sourceId = this.sourceSelector.value;
		const foundSource = this.availableSources.find((s) => s.id === sourceId);
		this.currentSource = foundSource || null;

		if (this.currentSource) {
			await this.loadAndVisualize();
		}
	}

	private async loadAndVisualize(): Promise<void> {
		if (!this.currentSource) {
			console.warn("[VectorVisualizationView] loadAndVisualize called with no current source");
			this.updateStatus("No source selected", true);
			return;
		}

		console.log("[VectorVisualizationView] Loading and visualizing source:", this.currentSource.id);
		this.updateStatus(`Fetching vectors from ${this.currentSource.name}...`);

		try {
			// Fetch vectors from selected source
			console.log("[VectorVisualizationView] Fetching vectors...");
			this.vectorData = await this.dataManager.fetchVectors(this.currentSource);
			console.log("[VectorVisualizationView] Received", this.vectorData.length, "data points");

			if (this.vectorData.length === 0) {
				this.updateStatus("No data available from source", true);
				return;
			}

			// Render visualization
			console.log("[VectorVisualizationView] Rendering points...");
			this.updateStatus(`Rendering ${this.vectorData.length} points...`);
			this.renderPoints();
			console.log("[VectorVisualizationView] Rendering complete");
			this.updateStatus(
				`Loaded ${this.vectorData.length} points from ${this.currentSource.name} | Canvas: ${this.renderer.domElement.width}x${this.renderer.domElement.height}`
			);
		} catch (error) {
			console.error("[VectorVisualizationView] Failed to load vectors:", error);
			this.updateStatus(`Error loading vectors: ${String(error)}`, true);
		}
	}

	private initializeScene(container: HTMLElement): void {
		const width = container.clientWidth;
		const height = container.clientHeight;

		console.log(
			"[VectorVisualizationView] Initializing scene, container size:",
			width,
			"x",
			height
		);

		// Defensive check: ensure container has non-zero dimensions
		if (width === 0 || height === 0) {
			console.error(
				"[VectorVisualizationView] Container has zero dimensions! Width:",
				width,
				"Height:",
				height
			);
			this.updateStatus("Error: Canvas container has zero size", true);
			return;
		}

		// Three.js scene setup
		this.scene = new THREE.Scene();
		this.scene.background = new THREE.Color(0x1e1e1e);
		console.log("[VectorVisualizationView] Scene created");

		// Camera
		this.camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
		this.camera.position.z = 5;
		console.log("[VectorVisualizationView] Camera created at position:", this.camera.position);

		// Renderer
		this.renderer = new THREE.WebGLRenderer({ antialias: true });
		this.renderer.setSize(width, height);
		container.appendChild(this.renderer.domElement);
		console.log(
			"[VectorVisualizationView] Renderer created, canvas size:",
			this.renderer.domElement.width,
			"x",
			this.renderer.domElement.height
		);

		// Defensive check: verify canvas was created with non-zero size
		if (this.renderer.domElement.width === 0 || this.renderer.domElement.height === 0) {
			console.error("[VectorVisualizationView] Canvas created with zero size!");
			this.updateStatus("Error: Canvas has zero size", true);
			return;
		}

		// Lighting
		const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
		this.scene.add(ambientLight);
		console.log("[VectorVisualizationView] Lighting added");

		// Animation loop
		const animate = (): void => {
			requestAnimationFrame(animate);
			if (this.controls) {
				this.controls.update();
			}
			if (this.renderer && this.scene && this.camera) {
				this.renderer.render(this.scene, this.camera);
			}
		};
		animate();
		console.log("[VectorVisualizationView] Animation loop started");
	}

	private setupControls(): void {
		this.controls = new OrbitControls(this.camera, this.renderer.domElement);
		this.controls.enableDamping = true;
		this.controls.dampingFactor = 0.05;
	}

	private setupEventListeners(): void {
		// TODO: Add click detection for point selection
	}

	private renderPoints(): void {
		console.log(
			"[VectorVisualizationView] renderPoints called, data points:",
			this.vectorData.length
		);

		// Defensive check: ensure scene exists
		if (!this.scene) {
			console.error("[VectorVisualizationView] Cannot render: scene not initialized");
			this.updateStatus("Error: Scene not initialized", true);
			return;
		}

		// Remove old point cloud
		if (this.pointCloud) {
			console.log("[VectorVisualizationView] Removing old point cloud");
			this.scene.remove(this.pointCloud);
			this.pointCloud.geometry.dispose();
			(this.pointCloud.material as THREE.Material).dispose();
		}

		if (this.vectorData.length === 0) {
			console.warn("[VectorVisualizationView] No data points to render!");
			this.updateStatus("No data points to render", true);
			return;
		}

		// Create point cloud from vectorData
		const geometry = new THREE.BufferGeometry();
		const positions = new Float32Array(this.vectorData.length * 3);
		const colors = new Float32Array(this.vectorData.length * 3);

		const colorScheme = DEFAULT_COLOR_SCHEMES[1]; // Golden Angle
		const maxCluster = Math.max(...this.vectorData.map((p) => p.cluster));
		console.log("[VectorVisualizationView] Max cluster:", maxCluster);

		let validPoints = 0;
		let allZeroPositions = true;

		for (let i = 0; i < this.vectorData.length; i++) {
			const point = this.vectorData[i];
			if (point && point.position3d && colorScheme) {
				const x = point.position3d[0] || 0;
				const y = point.position3d[1] || 0;
				const z = point.position3d[2] || 0;

				positions[i * 3] = x;
				positions[i * 3 + 1] = y;
				positions[i * 3 + 2] = z;

				// Check if at least one point has non-zero position
				if (x !== 0 || y !== 0 || z !== 0) {
					allZeroPositions = false;
				}

				// Color based on cluster
				const color = colorScheme.getColor(point.cluster || 0, maxCluster + 1);
				colors[i * 3] = color.r;
				colors[i * 3 + 1] = color.g;
				colors[i * 3 + 2] = color.b;
				validPoints++;
			}
		}

		console.log("[VectorVisualizationView] Valid points rendered:", validPoints);
		console.log("[VectorVisualizationView] Sample position:", [
			positions[0],
			positions[1],
			positions[2],
		]);

		// Defensive check: warn if all positions are at origin
		if (allZeroPositions) {
			console.warn(
				"[VectorVisualizationView] WARNING: All points are at origin (0,0,0)! Check SVD reduction."
			);
			this.updateStatus("Warning: All points at origin - check data processing", true);
		}

		geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
		geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));

		const material = new THREE.PointsMaterial({
			size: 0.05,
			vertexColors: true,
		});

		this.pointCloud = new THREE.Points(geometry, material);
		this.scene.add(this.pointCloud);
		console.log(
			"[VectorVisualizationView] Point cloud added to scene, children count:",
			this.scene.children.length
		);
	}

	private exportAsImage(): void {
		// Export canvas as PNG
		const dataURL = this.renderer.domElement.toDataURL("image/png");
		const link = document.createElement("a");
		link.download = "vector-visualization.png";
		link.href = dataURL;
		link.click();
	}

	private buildStatusPanel(container: Element): void {
		this.statusPanel = container.createDiv({ cls: "vector-status-panel" });
		this.updateStatus("Initializing...");
	}

	private updateStatus(message: string, isError = false): void {
		if (!this.statusPanel) {
			// Status panel not yet created, just log
			console.log("[VectorVisualizationView] Status:", message);
			return;
		}

		this.statusPanel.empty();
		this.statusPanel.createEl("span", {
			text: message,
			cls: isError ? "status-error" : "status-normal",
		});

		console.log("[VectorVisualizationView] Status updated:", message);
	}

	override onClose(): Promise<void> {
		// Cleanup Three.js resources
		if (this.renderer) {
			this.renderer.dispose();
		}
		if (this.pointCloud) {
			this.pointCloud.geometry.dispose();
			(this.pointCloud.material as THREE.Material).dispose();
		}
		return Promise.resolve();
	}
}
