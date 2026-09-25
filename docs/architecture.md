# Architecture

The planned MVP is a stateless BOLT11 decoder. It does not connect to a Lightning node. The structure below exists, while the arrows involving actual decoding are intentionally TODOs.

```text
Browser (Next.js)
    |
    | GET /api/v1/invoices/decode/{url-encoded-invoice}
    v
Axum API
    |
    | Rust function call
    v
invoice-decoder library
    |
    v
lightning-invoice
```

## Components

- `frontend/` owns input, loading and error states, and result presentation.
- `bin/api/` owns HTTP concerns: routing, CORS, limits, and error responses.
- `bin/cli/` will decode locally by default and later call the API in remote mode.
- `crates/invoice-decoder/` is where framework-independent parsing and validation will be implemented.

Keeping the decoder in a library gives the HTTP API and CLI the same behavior without duplicating protocol logic.

## Runtime boundaries

The frontend may be deployed to Vercel and the backend Docker image to Render. Render supplies `PORT`; the API listens on `0.0.0.0:$PORT`. `ALLOWED_ORIGIN` must be the deployed frontend origin.

The decoder is designed to be stateless and needs neither a database nor persistent storage. Invoice strings can reveal payment metadata, so the tracing TODO must exclude complete request URLs.

## Future node integration

Creating invoices, paying invoices, opening channels, and reading private payment state require a Lightning node. Those features should be added behind a separate backend-only interface:

```text
Browser -> Axum -> authenticated node adapter -> LND/CLN/LDK node
```

Node credentials, wallet seeds, TLS keys, and macaroons must never be sent to the browser.
