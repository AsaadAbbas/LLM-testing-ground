# ChronoKV

A time-series key-value store with versioned access, time-range queries, and leader-follower replication.

## Architecture

- **Rust Backend** (Cargo workspace with 5 crates)
  - `chronokv-core` — Core types, traits, error types
  - `chronokv-engine` — Storage engine (WAL, memtable, compaction, snapshots)
  - `chronokv-query` — Query engine (range queries, aggregation)
  - `chronokv-server` — HTTP API (Axum) + WebSocket subscriptions
  - `chronokv-replication` — Leader-follower replication protocol

- **TypeScript SDK** (`sdk/`)
  - HTTP client for the REST API
  - Fluent query builder
  - WebSocket subscription client

- **TypeScript CLI** (`cli/`)
  - Command-line interface using the SDK

## Quick Start

```bash
# Build Rust
cargo build

# Run tests
cargo test

# Start server
cargo run -p chronokv-server

# TypeScript
cd sdk && npm install && npm run build
cd ../cli && npm install && npm run build
```

## API

See `shared/protocol.json` for the full API contract.

### Key Operations
- `PUT /api/v1/kv/:key` — Store a value
- `GET /api/v1/kv/:key` — Get latest value
- `DELETE /api/v1/kv/:key` — Delete (tombstone)

### Queries
- `GET /api/v1/query?start=&end=&prefix=&limit=` — Time-range queries

### WebSocket
- `ws://localhost:3000/ws` — Subscribe to key changes

## Features

- Versioned entries with timestamp ordering
- Write-ahead log for crash recovery
- Tombstone-based deletion with configurable retention
- Time-range queries with key prefix filtering
- Leader-follower replication
- WebSocket subscriptions for real-time updates
