# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hyperlit is a tool for extracting, viewing and searching developer documentation.

## Build Commands

### Backend (Rust)

```bash
cargo build          # Build project
cargo test           # Run tests
cargo fmt            # Format code
cargo clippy         # Lint (use: cargo clippy -- -D warnings)
```

Run a single test:
```bash
cargo test test_name
```

### Frontend (TypeScript/React)

The frontend is located in the `web/` directory.

**IMPORTANT:** Use `./tool-tool` to ensure the correct Node.js and pnpm versions are used. tool-tool automatically downloads and runs the correct versions specified in the project configuration.

```bash
cd web
../tool-tool pnpm install         # Install dependencies
../tool-tool pnpm build           # Build for production
../tool-tool pnpm dev             # Start development server
```

## Git Hooks Setup

Install hooks before committing:
```bash
./scripts/git-hooks/install-hooks.sh
```

Pre-commit hook runs: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` with `RUSTFLAGS='-D warnings'`.

## Commit Message Format

Uses Conventional Commits. Format: `type(scope)?: description`

Valid types: `wip`, `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`

## Documentation Strategy

When writing code, document the "Why" directly in the source code using hyperlit comment markers ("📖"). This ensures that:

- **Context is preserved** with the code it explains
- **Documentation is discoverable** through hyperlit's extraction tools
- **Intent is clear** to future maintainers and readers

Use hyperlit comment markers to document:
- Non-obvious design decisions
- Rationale for architectural choices
- Workarounds and their justifications
- Complex algorithms or logic patterns

Format these comments as markdown.

Always use a heading as the first line of the comment.

Prefer to formulate the heading as a question ("Why ..."). This makes it easier to search for specific documentation.

Example:
```rust
/* 📖 # Why use Arc<Mutex<T>> for the app state?
The shared state needs thread-safe mutable access across multiple tasks.
Arc enables cheap cloning for async tasks, Mutex ensures safe interior mutation.
*/
let state = Arc::new(Mutex::new(data));
```

Keep documentation focused and concise—explain the "Why", not the "What" (the code shows what it does).