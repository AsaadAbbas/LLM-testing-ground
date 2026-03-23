# CrateSync

A package registry sync tool with Rust backend and TypeScript CLI.

## Architecture

- `crates/core` — Shared types (Package, Version, Manifest)
- `crates/resolver` — Dependency resolution (DAG solver)
- `crates/registry` — Registry HTTP client with cache
- `crates/server` — Axum REST API
- `cli/` — TypeScript CLI (Commander.js)

## Development

### Rust Backend
```bash
cargo build
cargo test
```

### TypeScript CLI
```bash
cd cli && npm install && npm run build
```

### Run Server
```bash
cargo run --bin cratesync-server
```

## Known Issues

- Version constraints are not enforced during resolution
- Some tests may fail intermittently
- CLI sync command has data parsing issues
