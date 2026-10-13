# Lightening Decoder

Lightening Decoder parses and validates BOLT11 Lightning Network invoices. It
extracts payment metadata, verifies the signature, and reports expiration
without paying the invoice or connecting to a Lightning node.

The project provides a Next.js web client, an Axum HTTP API, a Ratatui terminal
client, and a reusable Rust decoder library.

## Live API

- Web client: [lightening-invoice-decoder.vercel.app](https://lightening-invoice-decoder.vercel.app/)
- API: [lightening-decoder-api.onrender.com](https://lightening-decoder-api.onrender.com)
- Swagger UI: [lightening-decoder-api.onrender.com/swagger-ui/](https://lightening-decoder-api.onrender.com/swagger-ui/)
- Health check: [lightening-decoder-api.onrender.com/health](https://lightening-decoder-api.onrender.com/health)
- OpenAPI JSON: [lightening-decoder-api.onrender.com/api-docs/openapi.json](https://lightening-decoder-api.onrender.com/api-docs/openapi.json)

## Demo

[Watch the Lightening Decoder demo on Loom](https://www.loom.com/share/f4c26f9deb1a4ff180a2290410bad10d)

## Features

- Decode mainnet, testnet, signet, and regtest invoices
- Display amounts, descriptions, timestamps, payment hashes, and payee keys
- Verify signatures and report invoice expiration
- Decode fallback addresses and private route hints
- Use the terminal client locally or through the HTTP API
- Access an OpenAPI specification and Swagger UI
- Import BOLT11 QR images in the browser or terminal (PNG, JPEG, WebP)
- Explore shared, clearly labeled sample invoices
- Export decoded results as JSON; inspect JSON directly in the terminal
- Read detailed validation checks for format, signature, required fields, expiry, network, and amount

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

Open the interactive TUI:

```bash
cargo run -p lightning-cli --bin bolt11-decoder
```

Decode an invoice directly and print JSON. When using Polar, copy the complete
`payment_request` value:

```bash
cargo run -p lightning-cli --bin bolt11-decoder -- \
  decode 'paste-complete-BOLT11-payment_request-here'
```

Use a terminal at least 72 columns by 24 rows (100 × 32 recommended).

| Key | Action |
| --- | --- |
| `Enter` | Decode the pasted invoice or confirm a dialog |
| `F2` | Open the sample gallery; choose with arrow keys and press Enter |
| `F3` | Import a QR image by entering its local file path |
| `F4` | Cycle expected network before decoding (default: detect only) |
| `v` | Open or close the validation report after decoding |
| `Tab` | Switch between invoice details and JSON after decoding |
| `e` | Export JSON to a new file after decoding |
| `↑` / `↓`, `j` / `k`, Page Up / Down | Scroll the result |
| `Home` | Return to the top of the result |
| `Ctrl+U` | Clear input or the dialog's file path |
| `n` / `r` | Start another invoice after decoding |
| `Esc` | Cancel a dialog; otherwise quit |
| `Ctrl+C` | Quit from any screen |

QR import reads the image from disk, so it works without terminal image-display
support. Paste a plain path, a quoted path with spaces, or a path starting with
`~/`. Bracketed paste is supported. Images are processed locally, even in remote
mode; only the extracted invoice is sent to the API. Use one QR code per image,
up to 10 MB and 4096 × 4096 pixels. No image preview or clipboard-image import
is required. Export refuses to overwrite existing files.

For a noninteractive QR-to-JSON command:

```bash
cargo run -p lightning-cli --bin bolt11-decoder -- \
  decode --image frontend/public/samples/coffee.png
```

## Web input and exports

Choose or drop a PNG, JPEG, or WebP image (up to 10 MB and 16 million pixels).
The QR is read in the browser; review the extracted invoice and click Decode.
The image is not uploaded to the API. A `lightning:` prefix is accepted in both
clients. The result's **Export JSON** button downloads the decoded API response.

The coffee and open-donation samples are expired public examples from the
[BOLT11 specification](https://github.com/lightning/bolts/blob/master/11-payment-encoding.md#examples),
intended for inspection only. Both clients use `frontend/data/samples.json`.
Matching QR images are in `frontend/public/samples/`; regenerate them with:

```bash
cargo run -p lightning-cli --example sample_qr
```

## Validation reports

Select an optional **Expected network** in the web app, or press `F4` before
terminal decoding. Use `v` on the terminal result to read the report. Failed
terminal decoding opens the diagnostic report automatically when available;
`Esc` returns to the input. Exported JSON includes the report.

Checks use `passed`, `warning`, `failed`, `info`, or `not_checked` statuses.
Expired invoices and network mismatches produce warnings without hiding invoice
details. An invalid encoding leaves dependent checks unperformed. Signature
failures and missing required fields are distinguished when the encoding can be
parsed. The report records the time it was checked; expiry status is a snapshot.
Wallet feature compatibility, settlement status, recipient identity, and route
availability are not established by these checks.

The CLI accepts `--expected-network mainnet|testnet|signet|regtest` in local and
remote modes. Its shared Rust inspector validates input locally, including in
remote mode. Invalid CLI input prints a JSON diagnostic report and exits nonzero.
For example:

```bash
cargo run -p lightning-cli --bin bolt11-decoder -- \
  --expected-network testnet decode --image frontend/public/samples/coffee.png
```

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
| `PUBLIC_BASE_URL` | `http://localhost:<PORT>` | Public API URL shown in startup logs |
| `ALLOWED_ORIGINS` | `http://localhost:3000` | Comma-separated permitted browser origins |
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
  -e PUBLIC_BASE_URL=http://localhost:3001 \
  -e ALLOWED_ORIGINS=http://localhost:3000 \
  lightening-decoder-api
```

Set `ALLOWED_ORIGINS` to the local and deployed frontend origins and
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
