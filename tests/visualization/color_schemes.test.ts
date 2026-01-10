import { describe, expect, test } from "bun:test";
import { DEFAULT_COLOR_SCHEMES } from "../../src/visualization/ColorSchemes";

describe("color schemes", () => {
	test("sample palette snapshots", () => {
		const samples = DEFAULT_COLOR_SCHEMES.map((scheme) => {
			const colors = [0, 1, 2, 3, 5, 8].map((index) => scheme.getColor(index, 12).getHexString());
			return { name: scheme.name, colors };
		});

		expect(samples).toMatchSnapshot();
	});

	test("colors stay within valid RGB bounds", () => {
		const swatches = 32;
		for (const scheme of DEFAULT_COLOR_SCHEMES) {
			for (let index = 0; index < swatches; index += 1) {
				const color = scheme.getColor(index, swatches);
				expect(Number.isFinite(color.r)).toBe(true);
				expect(Number.isFinite(color.g)).toBe(true);
				expect(Number.isFinite(color.b)).toBe(true);
				expect(color.r).toBeGreaterThanOrEqual(0);
				expect(color.g).toBeGreaterThanOrEqual(0);
				expect(color.b).toBeGreaterThanOrEqual(0);
				expect(color.r).toBeLessThanOrEqual(1);
				expect(color.g).toBeLessThanOrEqual(1);
				expect(color.b).toBeLessThanOrEqual(1);
			}
		}
	});
});
