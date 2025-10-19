import type { App, Editor } from "obsidian";
import { MarkdownView, Modal, Notice, Plugin, PluginSettingTab, Setting } from "obsidian";
import init, {
	// Settings functions
	get_default_settings,
	merge_settings,
	// Command functions
	process_editor_text,
	generate_demo_message,
	// Utility functions
	word_count,
	reverse_string,
	to_title_case,
} from "./pkg/rust";
import wasmData from "./pkg/rust_bg.wasm";
import {
	VectorVisualizationView,
	VECTOR_VIEW_TYPE,
} from "./src/visualization/VectorVisualizationView";

// Remember to rename these classes and interfaces!

interface HelloWorldPluginSettings {
	mySetting: string;
}

export default class HelloWorldPlugin extends Plugin {
	settings!: HelloWorldPluginSettings;

	override async onload(): Promise<void> {
		// Initialize WebAssembly module FIRST (before calling any Rust functions)
		await init(wasmData);

		// Now we can load settings using Rust functions
		await this.loadSettings();

		// Register the 3D vector visualization view
		this.registerView(VECTOR_VIEW_TYPE, (leaf) => new VectorVisualizationView(leaf, this));

		// This creates an icon in the left ribbon.
		const ribbonIconEl = this.addRibbonIcon("dice", "Sample Plugin", (_evt: MouseEvent) => {
			// Called when the user clicks the icon.
			new Notice("This is a notice!");
		});
		// Perform additional things with the ribbon
		ribbonIconEl.addClass("my-plugin-ribbon-class");

		// This adds a status bar item to the bottom of the app. Does not work on mobile apps.
		const statusBarItemEl = this.addStatusBarItem();
		statusBarItemEl.setText("Status Bar Text");

		// This adds a simple command that can be triggered anywhere
		this.addCommand({
			id: "open-sample-modal-simple",
			name: "Open sample modal (simple)",
			callback: () => {
				new SampleModal(this.app).open();
			},
		});
		// This adds an editor command that can perform some operation on the current editor instance
		this.addCommand({
			id: "sample-editor-command",
			name: "Sample editor command",
			editorCallback: (editor: Editor) => {
				const selection = editor.getSelection();
				console.log(selection);
				// Process text in Rust
				const processed = process_editor_text(selection);
				editor.replaceSelection(processed);
			},
		});
		// This adds a complex command that can check whether the current state of the app allows execution of the command
		this.addCommand({
			id: "open-sample-modal-complex",
			name: "Open sample modal (complex)",
			checkCallback: (checking: boolean): boolean => {
				// Conditions to check
				const markdownView = this.app.workspace.getActiveViewOfType(MarkdownView);
				if (markdownView) {
					// If checking is true, we're simply "checking" if the command can be run.
					// If checking is false, then we want to actually perform the operation.
					if (!checking) {
						new SampleModal(this.app).open();
					}

					// This command will only show up in Command Palette when the check function returns true
					return true;
				}
				return false;
			},
		});

		// This adds a settings tab so the user can configure various aspects of the plugin
		this.addSettingTab(new SampleSettingTab(this.app, this));

		// If the plugin hooks up any global DOM events (on parts of the app that doesn't belong to this plugin)
		// Using this function will automatically remove the event listener when this plugin is disabled.
		this.registerDomEvent(document, "click", (evt: MouseEvent) => {
			console.log("click", evt);
		});

		// When registering intervals, this function will automatically clear the interval when the plugin is disabled.
		this.registerInterval(window.setInterval(() => console.log("setInterval"), 5 * 60 * 1000));

		this.addRibbonIcon("dice", "Greet", () => {
			new Notice("Hello, world!");
		});

		// Demo command using Rust/WebAssembly - all logic in Rust
		this.addCommand({
			id: "rust-wasm-demo",
			name: "Rust WASM Demo",
			callback: () => {
				// All logic in Rust - direct string generation, no JSON round-trip
				const message = generate_demo_message("Obsidian", 5, 7, 10);
				new Notice(message);
			},
		});

		// Text utility commands powered by Rust
		this.addCommand({
			id: "reverse-selection",
			name: "Reverse selected text",
			editorCallback: (editor: Editor) => {
				const selection = editor.getSelection();
				const reversed = reverse_string(selection);
				editor.replaceSelection(reversed);
			},
		});

		this.addCommand({
			id: "title-case-selection",
			name: "Title case selected text",
			editorCallback: (editor: Editor) => {
				const selection = editor.getSelection();
				const titled = to_title_case(selection);
				editor.replaceSelection(titled);
			},
		});

		this.addCommand({
			id: "word-count",
			name: "Count words in selection",
			editorCallback: (editor: Editor) => {
				const selection = editor.getSelection();
				const count = word_count(selection);
				new Notice(`Word count: ${count}`);
			},
		});

		// Add ribbon icon for 3D vector visualization
		this.addRibbonIcon("cube", "Open Vector Visualization", () => {
			void this.activateVectorView();
		});

		// Add command for 3D vector visualization
		this.addCommand({
			id: "open-vector-visualization",
			name: "Open 3D Vector Visualization",
			callback: () => {
				void this.activateVectorView();
			},
		});
	}

	async activateVectorView(): Promise<void> {
		const { workspace } = this.app;

		let leaf = workspace.getLeavesOfType(VECTOR_VIEW_TYPE)[0];

		if (!leaf) {
			const rightLeaf = workspace.getRightLeaf(false);
			if (rightLeaf) {
				await rightLeaf.setViewState({
					type: VECTOR_VIEW_TYPE,
					active: true,
				});
			}
			leaf = workspace.getLeavesOfType(VECTOR_VIEW_TYPE)[0];
		}

		if (leaf) {
			await workspace.revealLeaf(leaf);
		}
	}

	override onunload(): void {
		// Cleanup resources if needed
	}

	async loadSettings(): Promise<void> {
		// Use Rust to merge default and loaded settings
		const loadedData = (await this.loadData()) as HelloWorldPluginSettings | null;
		const defaultsJson = get_default_settings();
		const loadedJson = loadedData ? JSON.stringify(loadedData) : "{}";
		const mergedJson = merge_settings(defaultsJson, loadedJson);
		this.settings = JSON.parse(mergedJson) as HelloWorldPluginSettings;
	}

	async saveSettings(): Promise<void> {
		await this.saveData(this.settings);
	}
}

class SampleModal extends Modal {
	constructor(app: App) {
		super(app);
	}

	override onOpen(): void {
		const { contentEl } = this;
		contentEl.setText("Woah!");
	}

	override onClose(): void {
		const { contentEl } = this;
		contentEl.empty();
	}
}

class SampleSettingTab extends PluginSettingTab {
	plugin: HelloWorldPlugin;

	constructor(app: App, plugin: HelloWorldPlugin) {
		super(app, plugin);
		this.plugin = plugin;
	}

	display(): void {
		const { containerEl } = this;

		containerEl.empty();

		new Setting(containerEl)
			.setName("Setting #1")
			.setDesc("It's a secret")
			.addText((text) =>
				text
					.setPlaceholder("Enter your secret")
					.setValue(this.plugin.settings.mySetting)
					.onChange(async (value) => {
						this.plugin.settings.mySetting = value;
						await this.plugin.saveSettings();
					})
			);
	}
}
