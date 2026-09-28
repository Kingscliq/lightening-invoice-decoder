# Lightening Decoder

Lightening Decoder parses and validates BOLT11 Lightning Network invoices. It
extracts payment metadata, verifies the signature, and reports expiration
without paying the invoice or connecting to a Lightning node.

The project provides a Next.js web client, an Axum HTTP API, a Ratatui terminal
client, and a reusable Rust decoder library.

## Features

- Decode mainnet, testnet, signet, and regtest invoices
- Display amounts, descriptions, timestamps, payment hashes, and payee keys
- Verify signatures and report invoice expiration
- Decode fallback addresses and private route hints
- Use the terminal client locally or through the HTTP API
- Access an OpenAPI specification and Swagger UI

## Architecture

```text
Next.js web client ────────HTTP────┐
Ratatui remote mode ───────HTTP────┤
                                   ▼
                              Axum API
                                   │
Ratatui local mode ────────────────┤
                                   ▼
                          invoice-decoder
                                   │
                                   ▼
                          lightning-invoice
```

| Component | Location | Responsibility |
| --- | --- | --- |
| Decoder library | `crates/invoice-decoder` | BOLT11 parsing, field extraction, and validation |
| HTTP API | `bin/api` | Routing, CORS, errors, tracing, and OpenAPI |
| Terminal client | `bin/cli` | Interactive and JSON local/remote decoding |
| Web client | `frontend` | Invoice input and result presentation |

The application is stateless and requires no database, Bitcoin Core instance,
or Lightning node.

## Run locally

Requirements: Rust 1.91+, Node.js compatible with Next.js 16, and npm.

Start the API from the repository root:

```bash
cp .env.example .env
cargo run -p lightning-api
```

Start the web client in a second terminal:

```bash
cd frontend
cp .env.example .env.local
npm install
npm run dev
```

Local services:

- Web client: `http://localhost:3000`
- API: `http://localhost:3001`
- Swagger UI: `http://localhost:3001/swagger-ui/`
- OpenAPI JSON: `http://localhost:3001/api-docs/openapi.json`

## Terminal client

Interactive local mode:

```bash
cargo run -p lightning-cli --bin bolt11-decoder
```

Interactive remote mode:

```bash
cargo run -p lightning-cli --bin bolt11-decoder -- \
  --api-url http://localhost:3001
```

JSON output:

```bash
# Local
cargo run -p lightning-cli --bin bolt11-decoder -- \
  decode 'lnbcrt...'

# Remote
cargo run -p lightning-cli --bin bolt11-decoder -- \
  decode 'lnbcrt...' --api-url http://localhost:3001
```

Use `Enter` to decode, arrow keys or `j`/`k` to scroll advanced details, `n` or
`r` to start again, and `q`, `Esc`, or `Ctrl+C` to exit.

## HTTP API

```http
GET /health
GET /api/v1/invoices/decode/{url-encoded-invoice}
```

Example:

```bash
curl "http://localhost:3001/api/v1/invoices/decode/$(printf %s 'lnbcrt...' | jq -sRr @uri)"
```

The decode response includes the network, amount, description, timestamps,
payment hash, payee key, signature status, expiration status, fallback
addresses, and route hints. Errors use a stable JSON envelope with `code` and
`message` fields.

See [docs/api.md](docs/api.md) for the complete contract and status codes.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `3001` | API listening port |
| `ALLOWED_ORIGIN` | `http://localhost:3000` | Permitted browser origin |
| `RUST_LOG` | Application default | API tracing filter |
| `NEXT_PUBLIC_API_BASE_URL` | `http://localhost:3001` | Web client API origin |
| `LIGHTNING_API_URL` | Unset | Enables CLI remote mode |
| `LIGHTNING_TUI_DEMO_DELAY_MS` | `0` | Optional TUI loading-state delay |

## Verification

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

cd frontend
npm run typecheck
npm run build
```

## Deployment

The API ships as a multi-stage Docker image and includes `render.yaml` for
Render. The frontend can be deployed separately to Vercel or another Node.js
host.

```bash
docker build -t lightening-decoder-api .
docker run --rm -p 3001:10000 \
  -e PORT=10000 \
  -e ALLOWED_ORIGIN=http://localhost:3000 \
  lightening-decoder-api
```

Set `ALLOWED_ORIGIN` to the deployed frontend origin and
`NEXT_PUBLIC_API_BASE_URL` to the deployed API origin. See
[docs/docker.md](docs/docker.md) for container details.

## Security and scope

- Invoice input is limited to 8,192 characters.
- Decode responses use `Cache-Control: no-store`.
- Request tracing excludes invoice URLs.
- The decoder does not persist invoice data.
- The container runs as an unprivileged user.

The API accepts invoices in a GET path, so upstream proxies and analytics must
not log request URLs. Use a POST contract if invoice metadata must never appear
in URLs.

This application decodes invoices only. It does not create or pay invoices,
manage channels, discover payment routes, or query settlement state.

## Documentation

- [Architecture](docs/architecture.md)
- [HTTP API](docs/api.md)
- [Docker deployment](docs/docker.md)

## License

MIT
