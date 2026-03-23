# Nexus

A plugin-based API gateway / middleware pipeline framework in Rust with a TypeScript SDK.

## Architecture

- **Rust Backend** (Cargo workspace, 5 crates)
  - `nexus-core` — Core types: Request, Response, Middleware trait, Handler trait, errors
  - `nexus-pipeline` — Middleware pipeline engine with route matching
  - `nexus-plugins` — Built-in plugins: auth, rate-limit, CORS, transform, logging
  - `nexus-server` — Server runtime that wires config → pipeline → handlers
  - `nexus-config` — Configuration system with validation

- **TypeScript SDK** (`sdk/`)
  - HTTP client for the gateway
  - Admin API methods (metrics, config reload, route management)
  - SSE event subscription

## Quick Start

```bash
cargo build
cargo test
cd sdk && npm install && npm run build
```

## Middleware Pipeline

Requests flow through middleware in order:
1. Logging → CORS → Auth → Rate Limit → Transform
2. Route matching
3. Handler execution
4. Response flows back through middleware in reverse order

Each middleware can:
- Inspect/modify the request (`on_request`)
- Short-circuit with a response (e.g., auth rejection)
- Inspect/modify the response (`on_response`)

## Configuration

Configure via JSON. See `nexus-config` for the schema.

## Plugins

| Plugin | Purpose |
|--------|---------|
| `logging` | Request/response logging with request IDs |
| `cors` | CORS headers and preflight handling |
| `auth` | API key authentication |
| `rate-limit` | Token bucket rate limiting |
| `transform` | Response body envelope wrapping |
