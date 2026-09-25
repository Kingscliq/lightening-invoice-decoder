"use client";

import { FormEvent, useState } from "react";

import { InvoiceResult } from "@/components/invoice-result";
import { decodeInvoice } from "@/lib/api";
import type { DecodedInvoice } from "@/types/invoice";

export function InvoiceDecoder() {
  const [invoice, setInvoice] = useState("");
  const [result, setResult] = useState<DecodedInvoice | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setResult(null);
    setIsLoading(true);

    try {
      setResult(await decodeInvoice(invoice));
    } catch (caught) {
      setError(
        caught instanceof Error
          ? caught.message
          : "The invoice could not be decoded.",
      );
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <>
      <form className="decoder-card" onSubmit={handleSubmit} aria-busy={isLoading}>
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
          required
        />

        <div className="actions">
          <p className="hint">The invoice is decoded by the Rust API.</p>
          <button type="submit" disabled={isLoading || !invoice.trim()}>
            {isLoading ? "Decoding…" : "Decode invoice"}
          </button>
        </div>

        {error ? (
          <p className="error" role="alert">
            {error}
          </p>
        ) : null}
      </form>

      {result ? <InvoiceResult invoice={result} /> : null}
    </>
  );
}
