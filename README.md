# Beacon Server (Rust WASM / WASI)

Ultra-high performance WebAssembly HTTP backend for web analytics and event ingestion, running on Spin / Kubernetes (k3s).

## Features
- **Sub-millisecond Cold Starts**: Compiled to `wasm32-wasip1` / Spin component.
- **In-Memory Web Tag Serving**: Serves dynamic, cryptographically signed client tracker script (`/client.js`, `/tag.js`).
- **Telemetry Ingestion**: Ingests and validates analytics events via `/v1/sync` (`POST`), emitting flattened Parquet-friendly records to `stdout`.
- **Security & Anti-Spam**:
  - Ephemeral HMAC-SHA256 handshake tokens.
  - Origin & Referrer domain whitelisting.
  - Automatic traffic partitioning (`is_quarantined` flag).

## API Endpoints
- `GET /client.js` or `GET /tag.js` - Serves injected JS tag script.
- `OPTIONS /v1/sync` - CORS Preflight response.
- `POST /v1/sync` - Event ingestion endpoint (returns `204 No Content`).
- `GET /healthz` - Health check endpoint.

## Build & Run Locally
```bash
# Build WASM component
spin build

# Run locally
spin up
```
