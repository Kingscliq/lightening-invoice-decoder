# Lightning Tool — BOLT11 Invoice Decoder

A learning-focused capstone for decoding and validating Lightning Network BOLT11 invoices without requiring the user to run a Lightning node.

## Current status: decoder complete, integrations in progress

The framework-independent BOLT11 decoder is complete. The Axum API, TUI client,
Next.js frontend, and deployment flow are being completed as separate learning
steps.

### Learning roadmap

1. ✅ Parse a string with `lightning-invoice`.
2. ✅ Extract the basic BOLT11 fields.
3. ✅ Extract route hints and optional fields.
4. ✅ Understand and expose signature and expiry validation.
5. ✅ Complete the decoder's output model and tests.
6. ✅ Test the Axum endpoint, middleware, and error mappings.
7. Build and test the local CLI as a TUI with Ratatui.
8. Implement the CLI's optional remote API mode.
9. ✅ Connect the Next.js form to Axum.
10. Understand and run the Docker and Render deployment flow.

## Overview

A BOLT11 invoice is a signed Lightning Network payment request encoded as a long Bech32 string, for example:

```text
lnbcrt10u1p...
```

The finished decoder will accept that string and display information such as:

- Bitcoin network: mainnet, testnet, signet, or regtest
- Requested amount in millisatoshis
- Description or description hash
- Creation and expiry times
- Whether the invoice has expired
- Payment hash
- Payee public key
- Payment secret and supported feature bits, where applicable
- Fallback addresses
- Private route hints
- Cryptographic signature validity

The planned application will not pay the invoice. It will inspect and validate the information already encoded inside it.

## MVP scope

The MVP will:

1. Accept a complete BOLT11 invoice string.
2. Decode its Bech32 representation.
3. Extract its human-readable and tagged fields.
4. Validate its syntax and semantic requirements.
5. Verify its cryptographic signature.
6. Calculate its expiry time and report whether it has expired.
7. Return a structured JSON response.
8. Display the decoded information in a Next.js interface.

The MVP will not:

- Open or manage Lightning channels
- Find payment routes
- Pay invoices
- Look up whether arbitrary invoices have been paid
- Store users' invoices
- Require LND, Core Lightning, Bitcoin Core, or Polar in production

## Architecture

The target is a stateless application with a Next.js frontend and a Rust/Axum backend.

```text
┌────────────────────────────┐
│ Next.js frontend           │
│                            │
│ • Invoice input            │
│ • Decode action            │
│ • Results and errors       │
└──────────────┬─────────────┘
               │ HTTPS/JSON
               │ GET /api/v1/invoices/decode/{invoice}
               ▼
┌────────────────────────────┐
│ Rust Axum API              │
│                            │
│ • Request validation       │
│ • Response formatting      │
│ • Error handling           │
└──────────────┬─────────────┘
               │ Rust calls
               ▼
┌────────────────────────────┐
│ lightning-invoice crate    │
│                            │
│ • Bech32 decoding          │
│ • Field extraction         │
│ • Semantic validation      │
│ • Signature verification   │
└────────────────────────────┘
```

### Frontend responsibilities

The completed Next.js frontend will:

- Accepts the complete invoice string
- Submits it to the Axum API
- Shows loading, success, and error states
- Presents amounts and timestamps clearly
- Displays route hints in a readable form
- Never receives node credentials, macaroons, or private keys

### Backend responsibilities

The completed Axum backend will:

- Exposes the decode endpoint
- Trims and validates input
- Enforces an input-size limit
- Parses the invoice with `lightning-invoice`
- Converts amounts and timestamps into API-friendly values
- Reports signature and expiry status explicitly
- Maps parsing failures to useful client errors
- Returns structured JSON

### Decoder-library responsibilities

The `lightning-invoice` crate handles the protocol-sensitive operations:

- Bech32 parsing and checksum validation
- Network and optional amount parsing
- Timestamp decoding
- Tagged-field decoding
- Required-field and feature validation
- secp256k1 signature verification
- Payee public-key recovery
- Expiry-related calculations

The crate performs these operations locally. It does not contact the Lightning Network or a Lightning node.

## API design

### Decode an invoice

```http
GET /api/v1/invoices/decode/lnbcrt10u1p...
Accept: application/json
```

The complete BOLT11 invoice is passed as the `{invoice}` route parameter. The request has no body. Clients should URL-encode the value before adding it to the path, even though ordinary BOLT11 strings use a path-safe Bech32 character set.

An Axum route for this contract would look like:

```rust
Router::new().route(
    "/api/v1/invoices/decode/{invoice}",
    get(decode_invoice),
)
```

The handler extracts the path value:

```rust
async fn decode_invoice(
    Path(invoice): Path<String>,
) -> Result<Json<DecodeInvoiceResponse>, ApiError> {
    let decoded = invoice_decoder::decode(&invoice)?;
    Ok(Json(decoded.into()))
}
```

Successful response:

```json
{
  "network": "regtest",
  "amount_msat": 1000000,
  "description": "Capstone test payment",
  "description_hash": null,
  "payment_hash": "82f14a...",
  "payee_public_key": "0384b0...",
  "created_at_unix": 1790244000,
  "expires_at_unix": 1790247600,
  "expiry_seconds": 3600,
  "expired": false,
  "signature_valid": true,
  "min_final_cltv_expiry_delta": 40,
  "fallback_addresses": [],
  "route_hints": []
}
```

Example malformed-invoice response:

```json
{
  "error": {
    "code": "INVALID_INVOICE",
    "message": "The supplied value is not a valid BOLT11 invoice."
  }
}
```

An expired invoice can still be structurally and cryptographically valid. It should normally return a successful decode response with `expired: true`, not a parsing error.

The endpoint accepts an **invoice string**, not an invoice ID or payment hash. A payment hash alone does not contain all the information needed to decode an invoice.

Because this is a `GET` endpoint, the invoice will be part of the URL. Production configuration should:

- Permit URLs long enough for invoices containing large route hints
- Avoid recording complete request paths in proxy, access, analytics, or tracing logs
- Return `Cache-Control: no-store` unless caching decoded invoices is an explicit requirement
- Apply rate limiting and a strict maximum invoice length

If URL-length or metadata-exposure concerns become problematic, a `POST` endpoint with the invoice in a JSON body is the safer fallback.

## How BOLT11 decoding works

The project is conceptually similar to a Bitcoin transaction parser:

```text
Bitcoin transaction parser:
hex → bytes → version/inputs/outputs/scripts/witness/locktime

BOLT11 invoice decoder:
Bech32 text → network/amount/timestamp/tags/signature/checksum
```

A BOLT11 invoice is not a Bitcoin transaction. It is a signed instruction describing a requested Lightning payment.

### Invoice structure

```text
Human-readable prefix | encoded data and signature | Bech32 checksum
```

For an abbreviated invoice:

```text
lnbcrt10u1p...
```

- `ln` identifies a Lightning payment request.
- `bcrt` identifies Bitcoin regtest.
- `10u` represents 10 microbitcoin, or 1,000 sats.
- `1` separates the human-readable prefix from the encoded data.

Common network prefixes include:

| Prefix | Network |
| --- | --- |
| `lnbc` | Bitcoin mainnet |
| `lntb` | Bitcoin testnet |
| `lnbcrt` | Bitcoin regtest |

The amount is optional. When it is omitted, the payer supplies the amount during payment.

Common amount multipliers are:

| Suffix | Unit |
| --- | --- |
| `m` | milli-bitcoin |
| `u` | microbitcoin |
| `n` | nano-bitcoin |
| `p` | pico-bitcoin |

### Tagged fields

After the timestamp, invoice details are represented as tagged fields. Important tags include:

| Tag | Meaning |
| --- | --- |
| `p` | Payment hash |
| `d` | Plain-text description |
| `h` | Hash of a description supplied separately |
| `n` | Payee node public key |
| `x` | Expiry duration |
| `c` | Minimum final CLTV expiry delta |
| `f` | Fallback Bitcoin address |
| `r` | Private route hint |
| `s` | Payment secret |
| `9` | Feature bits |

An invoice normally contains either a direct description (`d`) or a description hash (`h`).

### Signature and checksum

The Bech32 checksum and invoice signature serve different purposes:

```text
Bech32 checksum → detects accidental corruption or typing errors
Signature       → proves that the signed fields were not modified and
                  that the creator controlled the signing key
```

The signature does not prove the real-world identity of the signing-key owner.

### Expiry

Invoice expiration is calculated as:

```text
expiration time = creation time + expiry duration
expired         = current time > expiration time
```

If an invoice does not specify an expiry duration, BOLT11 normally uses a default of one hour.

### Decode pipeline

```text
BOLT11 string
    │
    ▼
Normalize and validate input
    │
    ▼
Verify Bech32 structure and checksum
    │
    ▼
Decode network and optional amount
    │
    ▼
Decode timestamp and tagged fields
    │
    ▼
Verify signature and recover payee key
    │
    ▼
Calculate expiry status
    │
    ▼
Return readable JSON
```

## Local development and Polar

Polar is optional and is used only to generate realistic regtest invoices and observe Lightning payments during development.

```text
Polar
├── Bitcoin Core regtest node
├── Alice LND node
└── Bob LND node
        │
        └── generates lnbcrt... invoices
                         │
                         ▼
                  BOLT11 decoder
```

The decoder does not query Polar. Copy an invoice generated in Polar and paste it into the application.

### Create a test invoice without paying it

1. Start Docker and Polar.
2. Create a network with one Bitcoin Core node and at least one LND node.
3. Start the network.
4. Right-click an LND node and select **Create Invoice**.
5. Enter an amount and optional description.
6. Copy the resulting `lnbcrt...` string.
7. Submit it to the decoder.

A channel is not required merely to create or decode an invoice.

### Create and pay a test invoice

To observe a complete payment:

1. Create a Polar network containing Bitcoin Core, Alice LND, and Bob LND.
2. Fund Alice's on-chain wallet with regtest bitcoin.
3. Open an outgoing channel from Alice to Bob, for example with a capacity of 250,000 sats.
4. Mine blocks until the channel is active.
5. Create a fresh invoice on Bob, for example for 10,000 sats.
6. Pay that invoice from Alice.
7. Inspect the channel to see its balance move from Alice's side to Bob's side.

```text
Before payment:
Alice local balance: approximately 250,000 sats
Bob local balance:                          0 sats

After a 10,000-sat payment:
Alice local balance: approximately 240,000 sats
Bob local balance:                     10,000 sats
```

The balance displayed on a node's main Polar information panel is its **on-chain wallet balance**. Lightning funds received through a channel appear in the **channel balance**, not immediately in the on-chain wallet.

### Useful LND commands in Polar

Create an invoice from Bob's terminal:

```bash
lncli addinvoice --amt=10000 --memo="Capstone test payment"
```

Decode it from either LND terminal:

```bash
lncli decodepayreq --pay_req='lnbcrt...'
```

Decoding only inspects the invoice; it does not move money.

Pay Bob's invoice from Alice's terminal:

```bash
CAPSTONE_INVOICE='lnbcrt...'
lncli sendpayment --pay_req="$CAPSTONE_INVOICE" --fee_limit=10
```

The fee limit is the maximum permitted routing fee. It does not mean that 10 sats will automatically be charged. A direct Alice-to-Bob channel normally has no intermediate routing fee.

Inspect Alice's payments:

```bash
lncli listpayments
```

Inspect Bob's invoices and channel balance:

```bash
lncli listinvoices
lncli channelbalance
lncli listchannels
```

A successfully paid Bob invoice should have the state `SETTLED`.

## Lightning concepts

### Payment channels

A Lightning channel is anchored to Bitcoin by a funding transaction output jointly controlled by its two participants. Traditionally, this is a 2-of-2 multisignature arrangement.

```text
Opening transaction
        │
        ▼
Jointly controlled funding output
        │
  Off-chain balance updates
        │
        ▼
Closing transaction spends funding output
```

The blockchain normally sees the opening and closing transactions, not every Lightning payment.

### Commitment transactions

A commitment transaction is a valid Bitcoin transaction representing an agreed channel state. Alice and Bob create new commitment transactions as balances change; they do not repeatedly edit one mutable transaction.

```text
State 1: Alice 100,000 | Bob      0
State 2: Alice  80,000 | Bob 20,000
State 3: Alice  70,000 | Bob 30,000
```

All states spend the same funding output, so only one can ultimately confirm. Previous states are revoked, and traditional Lightning penalty mechanisms discourage participants from broadcasting an obsolete state.

The nodes store their own channel data locally. There is no central Lightning channel-state database.

### Incoming and outgoing liquidity

Channel direction describes who supplies the initial funds and where the initial liquidity sits.

| Channel from Alice's perspective | Initial Alice balance | Initial Bob balance | Initial direction |
| --- | ---: | ---: | --- |
| Alice opens outgoing channel | Approximately full capacity | 0 | Alice can pay Bob |
| Bob opens incoming channel to Alice | 0 | Approximately full capacity | Alice can receive from Bob |

The channel itself is bidirectional. Payments move liquidity from one side to the other. Bob does not need to deposit money to receive through a channel funded by Alice; after receiving, Bob has outbound balance that can be sent back.

### HTLCs

An HTLC is a **Hash Time-Locked Contract**. It makes a Lightning payment conditional on revealing a secret before a deadline.

Bob creates a random preimage `R` and includes its hash in the invoice:

```text
H = SHA256(R)
```

Each routing hop conditionally forwards the payment to anyone who can reveal `R` before its timelock expires. Bob reveals `R` to claim the payment, and the preimage travels backwards to settle every hop.

```text
Payment: Alice → Carol → Bob
Secret:  Alice ← Carol ← Bob
```

If the secret is not revealed before the deadline, the HTLC expires and the funds return to their previous owners.

### Routing

The sender's Lightning implementation chooses routes. An application normally supplies the invoice, a timeout, and a maximum fee; LND or LDK performs route-finding.

The cheapest advertised route is not always the best route. A route finder also considers:

- Estimated directional liquidity
- Previous successes and failures
- Online and connected peers
- Routing fees
- Timelock requirements
- Number of hops

Exact channel balances are not globally published. A route can therefore appear valid and still fail from insufficient directional liquidity. The node learns from failures and can try another eligible route.

Payments normally complete in under a second to several seconds, although repeated attempts or poor connectivity can take longer. Applications should use a timeout and must not start a duplicate payment while the first remains `IN_FLIGHT`.

### Routing fees

Intermediate Lightning nodes may charge a base fee and a proportional fee for forwarding payments.

```text
Alice → Router 1 → Router 2 → Bob
            forwarding fees
```

A fee limit is a maximum, not a fixed charge. If the limit is 10 sats and the successful route costs 3 sats, only 3 sats is charged. If all viable routes cost more than the limit, the payment fails without partially paying the invoice.

Routing fees are separate from Bitcoin mining fees. Mining fees are incurred when channels are opened, closed, or otherwise resolved on-chain.

## Bitcoin UTXO background

Every Bitcoin transaction input identifies the output it spends using an outpoint:

```text
outpoint = previous transaction ID + output index
```

Each Bitcoin full node independently maintains a local UTXO set containing all currently spendable outputs. When validating a transaction or block, the node checks that every referenced outpoint exists and satisfies its spending conditions.

When an output is spent and confirmed:

1. The spent output is removed from the UTXO set.
2. Newly created outputs are added.
3. Any later attempt to spend the removed output is rejected.

Bitcoin Core's chainstate database is local and is not a central SQL database. Block explorers may create their own SQL or indexed databases for convenient searching, but those services are not part of Bitcoin consensus.

Useful Bitcoin Core queries include:

```bash
bitcoin-cli gettxout <txid> <output-index>
bitcoin-cli gettxoutsetinfo
```

## BOLT11 and other payment mechanisms

BOLT means **Basis of Lightning Technology**. BOLT 11 specifies the traditional signed Lightning invoice format.

Other payment mechanisms include:

- **BOLT12:** Offers, invoice requests, and invoices with reusable offers and improved privacy capabilities. Common prefixes include `lno`, `lnr`, and `lni`.
- **LNURL-pay:** A protocol that contacts a service to obtain a dynamically generated invoice.
- **Lightning Address:** A human-readable identifier, such as `alice@example.com`, normally backed by LNURL-pay.
- **Keysend:** A spontaneous payment that does not require the recipient to create an invoice first.
- **BIP21:** Primarily an on-chain Bitcoin payment URI, not a BOLT11 invoice.

This project's MVP will support BOLT11 only.

## Invoice decoding versus node lookup

These are separate operations:

```text
Decode invoice contents       → no Lightning node required
Look up private invoice state → issuing Lightning node required
Pay an invoice                → funded Lightning node required
```

The payment state—such as `OPEN`, `ACCEPTED`, `SETTLED`, or `CANCELED`—is private data stored by the node that created the invoice. There is no global public invoice-status database.

If a future version manages invoices created by its own LND node, the architecture would become:

```text
Next.js → Axum → authenticated private LND connection → Bitcoin backend
```

Node credentials must remain on the backend. Never expose an admin macaroon, TLS credential, seed phrase, or private key to the browser.

## Suggested repository structure

```text
lightning-tool/
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── .env.example
├── bin/
│   ├── api/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── router.rs
│   │       ├── logger.rs
│   │       ├── config.rs
│   │       └── http/
│   │           ├── mod.rs
│   │           ├── error.rs
│   │           ├── dto/
│   │           │   ├── mod.rs
│   │           │   └── invoice.rs
│   │           └── routes/
│   │               ├── mod.rs
│   │               ├── health.rs
│   │               └── decode_invoice.rs
│   └── cli/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── cli.rs
│           └── remote.rs
├── crates/
│   └── invoice-decoder/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── decoder.rs
│       │   ├── error.rs
│       │   └── types.rs
│       └── tests/
│           ├── decode_invoices.rs
│           └── fixtures/
├── frontend/
│   ├── package.json
│   ├── app/
│   │   └── page.tsx
│   ├── components/
│   ├── lib/
│   │   └── api.ts
│   └── types/
│       └── invoice.ts
├── docs/
│   ├── architecture.md
│   └── api.md
└── README.md
```

The root Cargo workspace contains two thin executables and one reusable library:

```toml
[workspace]
members = [
    "bin/api",
    "bin/cli",
    "crates/invoice-decoder",
]
resolver = "3"
```

`crates/invoice-decoder` is the home for framework-independent BOLT11 logic. `bin/api` adapts that library to Axum, while `bin/cli` exposes it from a terminal.

## Getting started

The backend and CLI require a current stable Rust toolchain. Start the API from the repository root:

```bash
cp .env.example .env
cargo run -p lightning-api
```

The API listens on `http://localhost:3001` when `PORT` is not set. The working endpoint at this stage is:

```http
GET /health
```

Interactive Swagger documentation is available at
`http://localhost:3001/swagger-ui/`, and the generated OpenAPI JSON is available
at `http://localhost:3001/api-docs/openapi.json`.

The decode route uses the completed decoder library. You can also invoke the
current command-line output with:

```bash
cargo run -p lightning-cli --bin bolt11-decoder -- \
  decode 'lnbcrt...'
```

The following remote mode command shape is reserved, but its HTTP implementation is also a TODO:

```bash
cargo run -p lightning-cli --bin bolt11-decoder -- \
  decode 'lnbcrt...' --api-url http://localhost:3001
```

Start the frontend separately:

```bash
cd frontend
cp .env.example .env.local
npm install
npm run dev
```

The frontend is then available at `http://localhost:3000`. Its form is present, but the API call is a TODO. See [`docs/architecture.md`](docs/architecture.md), [`docs/api.md`](docs/api.md), and [`docs/docker.md`](docs/docker.md) for the boundaries and planned contracts.

## Command-line interface

The optional CLI will use Ratatui for an interactive terminal interface while
supporting both local and remote decoding. Local decoding will call the shared
library directly; remote decoding will call the Axum API.

Local mode will call `invoice-decoder` directly and work without a running API:

```bash
bolt11-decoder decode 'lnbcrt...'
```

Remote mode will call the deployed Axum endpoint and will be useful for testing the public API:

```bash
bolt11-decoder decode 'lnbcrt...' \
  --api-url https://lightning-tool-api.onrender.com
```

```text
Local mode:  CLI → invoice-decoder
Remote mode: CLI → Render API → invoice-decoder
```

Local mode is the planned default because it will be faster and work offline. Remote mode will depend on network access and API availability.

## Error-handling strategy

Use typed errors where callers need to distinguish failure cases, and flexible application errors at the executable boundary:

```text
invoice-decoder library → thiserror
Axum API errors         → thiserror
CLI main/run function   → anyhow
```

`thiserror` is used for variants such as `EmptyInvoice`, `InvalidInvoice`, and `InvalidSignature`, allowing Axum to map them to stable HTTP error responses. `anyhow` is used by the CLI to add operational context and propagate failures to the executable entry point without exposing an error type as a library contract. The CLI entry point is responsible for printing the error and returning a non-zero exit status.

## Testing strategy

At minimum, test:

- Valid mainnet, testnet, signet, and regtest invoices
- Invoices with and without amounts
- Direct descriptions and description hashes
- Valid and expired invoices
- Valid and corrupted signatures
- Invalid Bech32 checksums
- Route hints containing one and multiple hops
- Fallback addresses
- Unknown optional feature bits
- Unsupported required feature bits
- Missing mandatory fields
- Excessively long or empty input
- Uppercase and lowercase representations where permitted

Invoices produced by Polar/LND can be decoded with `lncli decodepayreq` and compared against the application's response as a reference test.

## Deployment

The frontend and backend are deployed independently:

```text
Next.js frontend on Vercel
            │ HTTPS
            ▼
Rust/Axum API in a Render Docker web service
```

Only the Rust backend requires a Docker image. The root `Dockerfile` is a documented two-stage learning example: one image compiles Rust and a much smaller image runs the resulting executable. We will walk through and run it during lesson 10; see [`docs/docker.md`](docs/docker.md).

### Render port binding

Render provides the public web-service port through the `PORT` environment variable and currently defaults it to `10000`. The Axum server must listen on that port and bind to `0.0.0.0`, not only `127.0.0.1`:

```rust
let port: u16 = std::env::var("PORT")
    .unwrap_or_else(|_| "3001".to_owned())
    .parse()?;

let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
axum::serve(listener, app).await?;
```

The local fallback of `3001` is used only when `PORT` is absent. The deployment should also expose a lightweight health endpoint, such as:

```http
GET /health
```

Because Vercel and Render use different origins, the Axum CORS policy should explicitly allow the deployed frontend origin. A custom domain and reverse proxy can be added later if both services need to appear under one public origin.

Recommended production controls include:

- HTTPS
- API rate limiting
- Strict invoice-length and URL-size limits
- Explicit CORS configuration
- Structured error responses
- Avoiding invoice contents in application logs
- Dependency and container vulnerability scanning

Because the MVP is stateless, it does not require a database or persistent volume.

## Future enhancements

- QR-code scanning
- BOLT12 support
- Invoice creation through an owned Lightning node
- Payment-status lookup for invoices created by that node
- Invoice payment through LDK, LND, or Core Lightning
- Channel balance and payment-history views
- Saved decoding history with explicit user consent

## Glossary

| Term | Meaning |
| --- | --- |
| BOLT11 | Standard format for traditional Lightning invoices |
| BOLT12 | Newer offers and invoice protocol |
| Bech32 | Human-readable error-detecting string encoding |
| Payment hash | SHA-256 hash that identifies the payment condition |
| Preimage | Secret whose hash matches the payment hash |
| HTLC | Conditional payment using a hash and a deadline |
| Channel | Jointly controlled Bitcoin output with off-chain balance updates |
| Commitment transaction | Signed Bitcoin transaction representing a channel state |
| Outbound liquidity | Amount a node can currently send through a channel |
| Inbound liquidity | Amount a node can currently receive through a channel |
| Route hint | Private routing information embedded in an invoice |
| UTXO | Unspent transaction output |
| LND | Lightning Network Daemon implementation |
| Polar | Local regtest Lightning network development application |
