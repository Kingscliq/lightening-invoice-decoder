"use client";

import { FormEvent, useRef, useState } from "react";

import { InvoiceResult } from "@/components/invoice-result";
import { decodeInvoice } from "@/lib/api";
import type { DecodedInvoice } from "@/types/invoice";
import samples from "@/data/samples.json";
import { readQrImage } from "@/lib/qr";

export function InvoiceDecoder() {
  const [invoice, setInvoice] = useState("");
  const [result, setResult] = useState<DecodedInvoice | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isReading, setIsReading] = useState(false);
  const [notice, setNotice] = useState("");
  const [isDragging, setIsDragging] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const busyRef = useRef(false);
  const busy = isLoading || isReading;

  async function importImage(file: File) {
    if (busyRef.current) return;
    busyRef.current = true;
    setIsReading(true);
    setError(null);
    setNotice("");
    try {
      setInvoice(await readQrImage(file));
      setNotice(`QR read from ${file.name}. Review the invoice, then decode.`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "The image could not be read.");
    } finally {
      setIsReading(false);
      busyRef.current = false;
    }
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (busyRef.current) return;
    busyRef.current = true;
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
      busyRef.current = false;
    }
  }

  function resetDecoder() {
    setInvoice("");
    setResult(null);
    setError(null);
    setNotice("");
  }

  if (result) {
    return <InvoiceResult invoice={result} onReset={resetDecoder} />;
  }

  return (
    <form className="decoder-card" onSubmit={handleSubmit} aria-busy={busy}>
      <div className="input-heading">
        <div><p className="section-kicker">Your invoice, unpacked</p><h2>Start with an invoice or QR</h2></div>
        <span className="format-badge">BOLT11</span>
      </div>
      <div className={`qr-dropzone${isDragging ? " dragging" : ""}`}
        onDragOver={(event) => { event.preventDefault(); if (!busy) setIsDragging(true); }}
        onDragLeave={() => setIsDragging(false)}
        onDrop={(event) => {
          event.preventDefault(); setIsDragging(false);
          if (event.dataTransfer.files.length !== 1) { setError("Drop one QR image at a time."); return; }
          void importImage(event.dataTransfer.files[0]);
        }}>
        <span className="qr-mark" aria-hidden="true">▦</span>
        <div><strong>{isReading ? "Reading QR image…" : "Drop a QR image here"}</strong>
          <p className="hint">PNG, JPEG or WebP · up to 10 MB · image stays on your device</p></div>
        <button type="button" className="secondary-button" disabled={busy} onClick={() => fileInput.current?.click()}>Choose image</button>
        <input ref={fileInput} type="file" accept=".png,.jpg,.jpeg,.webp" hidden
          onChange={(event) => {
            const file = event.target.files?.[0]; event.target.value = "";
            if (file) void importImage(file);
          }} />
      </div>
      <label className="field-label" htmlFor="invoice">
        BOLT11 invoice
      </label>
      <textarea
        id="invoice"
        name="invoice"
        placeholder="lnbc... or lnbcrt..."
        value={invoice}
        onChange={(event) => { setInvoice(event.target.value); setNotice(""); setError(null); }}
        disabled={busy}
        maxLength={8202}
        spellCheck={false}
        autoCapitalize="none"
        autoCorrect="off"
        required
      />

      <div className="actions">
        <p className="hint">The invoice is decoded by the Rust API.</p>
        <button type="submit" disabled={busy || !invoice.trim()}>
          {isLoading ? "Decoding…" : "Decode invoice"}
        </button>
      </div>

      <div className="sample-section">
        <p className="field-label">Just exploring? Try a sample</p>
        <div className="sample-grid">{samples.map((sample) => (
          <button key={sample.label} className="sample-button" type="button" disabled={busy}
            onClick={() => { setInvoice(sample.invoice); setError(null); setNotice(`${sample.label} loaded. This is an expired public demo invoice.`); }}>
            <strong>{sample.label} <span aria-hidden="true">↗</span></strong><span>{sample.detail}</span>
          </button>
        ))}</div>
        <p className="hint">Public BOLT11 examples. Both are expired and for inspection only.</p>
      </div>
      {notice ? <p className="import-notice" role="status">{notice}</p> : null}

      {error ? (
        <p className="error" role="alert">
          {error}
        </p>
      ) : null}
    </form>
  );
}
