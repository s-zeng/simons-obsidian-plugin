declare module "bun:test" {
	export interface MatcherResult {
		pass: boolean;
		message: () => string;
	}

	export interface Matchers<T> {
		toBe(expected: T): void;
		toBeGreaterThanOrEqual(expected: number): void;
		toBeLessThanOrEqual(expected: number): void;
		toMatchSnapshot(): void;
	}

	export type Expect = <T>(value: T) => Matchers<T>;

	export interface TestAPI {
		(name: string, fn: () => void | Promise<void>): void;
	}

	export const describe: TestAPI;
	export const test: TestAPI;
	export const expect: Expect;
}
