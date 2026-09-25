# HTTP API

## Health check

```http
GET /health
```

Returns `200 OK` when the process is ready to accept requests.

## Decode a BOLT11 invoice

```http
GET /api/v1/invoices/decode/{invoice}
Accept: application/json
```

`invoice` is the complete URL-encoded BOLT11 string, not a payment hash or database ID. This is the planned contract. At the skeleton stage, non-empty input receives `501 Not Implemented` until the decoder lessons are complete.

Example:

```bash
curl "http://localhost:3001/api/v1/invoices/decode/$(printf %s 'lnbcrt...' | jq -sRr @uri)"
```

Once implemented, successful responses will contain the decoded network, amount, description, timestamps, payment hash, payee key, route hints, and validation status. Unix timestamps are expressed in seconds, and amounts are expressed in millisatoshis.

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
  "route_hints": [],
  "features": null
}
```

Errors use a stable envelope:

```json
{
  "error": {
    "code": "INVALID_INVOICE",
    "message": "The supplied value is not a valid BOLT11 invoice."
  }
}
```

The skeleton returns `501` for decoding that has not been implemented. The finished endpoint will also use `400` for empty or invalid input, `404` for unknown routes, `414` when the invoice exceeds the configured limit, and `422` for a decoded value that cannot be represented safely.

TODO for the API lesson: responses should use `Cache-Control: no-store`. Clients should avoid storing or logging invoice URLs unless users explicitly consent.
