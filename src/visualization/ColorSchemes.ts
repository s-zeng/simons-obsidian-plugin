import * as THREE from "three";

export interface ColorScheme {
	name: string;
	getColor(clusterIndex: number, totalClusters: number): THREE.Color;
}

export const DEFAULT_COLOR_SCHEMES: ColorScheme[] = [
	{
		name: "Rainbow",
		getColor: (index: number, total: number): THREE.Color => {
			const hue = (index / total) * 360;
			return new THREE.Color().setHSL(hue / 360, 0.8, 0.6);
		},
	},
	{
		name: "Golden Angle",
		getColor: (index: number, _total: number): THREE.Color => {
			const hue = (index * 137.508) % 360;
			return new THREE.Color().setHSL(hue / 360, 0.8, 0.6);
		},
	},
	{
		name: "Categorical",
		getColor: (index: number, _total: number): THREE.Color => {
			const palette = [
				0xff6b6b, 0x4ecdc4, 0x45b7d1, 0xf7b731, 0x5f27cd, 0x00d2d3, 0xff9ff3, 0x54a0ff,
			];
			return new THREE.Color(palette[index % palette.length]);
		},
	},
];
