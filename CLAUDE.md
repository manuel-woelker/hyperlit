# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hyperlit is a tool for extracting, viewing and searching developer documentation.

## Build Commands

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

## Git Hooks Setup

Install hooks before committing:
```bash
./scripts/git-hooks/install-hooks.sh
```

Pre-commit hook runs: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` with `RUSTFLAGS='-D warnings'`.

## Commit Message Format

Uses Conventional Commits. Format: `type(scope)?: description`

Valid types: `wip`, `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`
