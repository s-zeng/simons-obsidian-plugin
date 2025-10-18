# Linting and Code Quality Setup

Documentation written by Claude Code

This project uses comprehensive linting and formatting tools for both TypeScript and Rust to ensure maximum code quality and catch errors before they reach production.

## Overview

The project enforces strict linting rules through:

- **TypeScript**: ESLint with strict type-checking rules + Prettier for formatting
- **Rust**: Clippy with pedantic lints + rustfmt for formatting
- **Pre-commit hooks**: Automated checks before every commit using Husky

## TypeScript Configuration

### Strict TypeScript Compiler Options (`tsconfig.json`)

The following strict compiler options are enabled:

- **`strict: true`** - Umbrella flag for all strict checks
- **`noUnusedLocals`** - Error on unused local variables
- **`noUnusedParameters`** - Error on unused parameters
- **`noImplicitReturns`** - Error when not all code paths return a value
- **`noFallthroughCasesInSwitch`** - Error on fallthrough switch cases
- **`noUncheckedIndexedAccess`** - Adds `undefined` to index signature results for safety
- **`noImplicitOverride`** - Require explicit `override` keyword
- **`allowUnreachableCode: false`** - Error on unreachable code

### ESLint Rules (`.eslintrc`)

Strict type-aware linting with:

- **Type safety**: No `any`, no unsafe assignments/calls/returns, explicit return types
- **Async safety**: No floating promises, proper await usage
- **Code quality**: Consistent naming conventions, type imports, interface definitions
- **Type-aware checks**: Full type checking during linting via `@typescript-eslint/recommended-requiring-type-checking`

### Prettier Configuration

Code formatting is enforced with:

- 100 character line width
- Tabs for indentation
- Semicolons required
- LF line endings

## Rust Configuration

### Clippy Lints (`rust/Cargo.toml`)

Maximum lint levels enabled:

- **`clippy::all`** - All default Clippy lints
- **`clippy::pedantic`** - Pedantic lints for extra code quality
- **`clippy::nursery`** - Experimental but useful lints
- **`unwrap_used`**, **`expect_used`**, **`panic`** - Warnings for unsafe unwrapping (aligns with project's error handling guidelines)

### Compiler Lints

- **`unsafe_code = "forbid"`** - No unsafe code allowed
- **`missing_docs = "warn"`** - Documentation required
- **`unused_must_use = "deny"`** - Results must be handled

### Rustfmt Configuration (`rust/rustfmt.toml`)

Strict formatting rules including:

- 100 character max width
- Reorder imports and impl items
- Consistent trailing commas
- Comprehensive comment and code formatting

## Pre-commit Hooks

The pre-commit hook (`.husky/pre-commit`) automatically runs on every commit:

### For TypeScript files:

1. ESLint with auto-fix
2. Prettier formatting
3. TypeScript type checking

### For Rust files:

1. `cargo fmt` - Format code
2. `cargo clippy` - Lint with strict warnings treated as errors

## Available Scripts

Run these commands manually:

```bash
# Lint everything
npm run lint

# Lint TypeScript only
npm run lint:ts

# Lint Rust only
npm run lint:rust

# Format everything
npm run format

# Format TypeScript only
npm run format:ts

# Format Rust only
npm run format:rust
```

## Development Workflow

1. **Write code** - Your editor should show linting errors in real-time
2. **Before committing** - The pre-commit hook will automatically:
   - Format your code
   - Check for linting errors
   - Run type checking
   - Block the commit if there are issues
3. **Fix issues** - If the commit is blocked, fix the reported issues
4. **Commit again** - Once all checks pass, your commit will succeed

## Bypassing Hooks (Not Recommended)

If you absolutely must commit without running hooks:

```bash
git commit --no-verify
```

**Warning**: This bypasses all quality checks and is strongly discouraged.

## Integration with Editors

### VS Code

Install these extensions for the best experience:

- ESLint
- Prettier
- rust-analyzer

Add to `.vscode/settings.json`:

```json
{
	"editor.formatOnSave": true,
	"editor.codeActionsOnSave": {
		"source.fixAll.eslint": true
	},
	"rust-analyzer.check.command": "clippy"
}
```

## Troubleshooting

### "ESLint couldn't find tsconfig.json"

Make sure you're running commands from the project root directory.

### "Clippy is not installed"

Install with: `rustup component add clippy`

### "Cargo fmt check failed"

Run `npm run format:rust` to auto-format Rust code.

### Pre-commit hook not running

Ensure husky is installed: `npm run prepare`

## Philosophy

This strict linting setup aligns with the project's philosophy:

- **TypeScript** handles Obsidian API integration only
- **Rust** contains the core logic with functional programming principles
- **Maximum safety** through strict type checking and linting catches bugs early
- **Consistency** through automated formatting reduces cognitive load
