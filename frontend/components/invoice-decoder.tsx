"use client";

import { FormEvent, useState } from "react";

export function InvoiceDecoder() {
  const [invoice, setInvoice] = useState("");
  const [message, setMessage] = useState<string | null>(null);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    // TODO(lesson 9): Validate the input, call `decodeInvoice`, and model the
    // loading, success, and error states explicitly.
    setMessage(
      "Frontend submission is intentionally not implemented yet. See the lesson 9 TODOs.",
    );
  }

  return (
    <form className="decoder-card" onSubmit={handleSubmit}>
      <label className="field-label" htmlFor="invoice">
        BOLT11 invoice
      </label>
      <textarea
        id="invoice"
        name="invoice"
        placeholder="lnbc... or lnbcrt..."
        value={invoice}
        onChange={(event) => setInvoice(event.target.value)}
        spellCheck={false}
        autoCapitalize="none"
        autoCorrect="off"
      />

      <div className="actions">
        <p className="hint">Learning skeleton: API integration is a TODO.</p>
        <button type="submit">Decode invoice</button>
      </div>

      {message ? <p className="error">{message}</p> : null}
    </form>
  );
}
