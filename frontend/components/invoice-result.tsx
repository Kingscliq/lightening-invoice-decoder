"use client";

import { useState, type ReactNode } from "react";

import type { DecodedInvoice } from "@/types/invoice";

type Props = {
  invoice: DecodedInvoice;
  onReset: () => void;
};

export function InvoiceResult({ invoice, onReset }: Props) {
  const amount = formatAmount(invoice.amount_msat);
  const description = invoice.description?.trim() || "Not included";

  return (
    <section className="result-card" aria-live="polite">
      <header className="result-header">
        <div>
          <p className="section-kicker">Decode successful</p>
          <h2>Invoice details</h2>
        </div>
        <div className="result-header-actions">
          <span className={`status${invoice.expired ? " expired" : ""}`}>
            {invoice.expired ? "Expired" : "Active"}
          </span>
          <button className="secondary-button" type="button" onClick={onReset}>
            Decode another invoice
          </button>
        </div>
      </header>

      <div className="result-content">
        <div className="section-grid">
          <ResultSection title="Summary">
            <dl className="result-grid">
              <ResultField label="Network" value={invoice.network} />
              <ResultField
                label="Amount"
                value={amount.primary}
                secondary={amount.secondary}
              />
              <ResultField label="Description" value={description} />
            </dl>
          </ResultSection>

          <ResultSection title="Validity">
            <dl className="result-grid">
              <ResultField
                label="Signature"
                value={invoice.signature_valid ? "Valid" : "Invalid"}
                tone={invoice.signature_valid ? "success" : "danger"}
              />
              <ResultField
                label="Created"
                value={formatTimestamp(invoice.created_at_unix)}
              />
              <ResultField
                label="Expires"
                value={formatTimestamp(invoice.expires_at_unix)}
                secondary={formatRelativeExpiry(invoice.expires_at_unix)}
              />
              <ResultField
                label="Duration"
                value={formatDuration(invoice.expiry_seconds)}
                secondary={`${invoice.expiry_seconds.toLocaleString()} seconds`}
              />
            </dl>
          </ResultSection>
        </div>

        <ResultSection title="Payment identifiers">
          <div className="identifier-list">
            <Identifier label="Payment hash" value={invoice.payment_hash} />
            <Identifier label="Payee public key" value={invoice.payee_public_key} />
          </div>
        </ResultSection>

        <details className="advanced-section">
          <summary>Advanced details</summary>
          <div className="advanced-content">
            <dl className="result-grid">
              <ResultField
                label="Description hash"
                value={invoice.description_hash ?? "Not included"}
              />
              <ResultField
                label="Minimum final CLTV delta"
                value={`${invoice.min_final_cltv_expiry_delta} blocks`}
              />
              <ResultField
                label="Fallback addresses"
                value={invoice.fallback_addresses.join(", ") || "None"}
              />
            </dl>

            <div className="route-section">
              <h3>Route hints</h3>
              {invoice.route_hints.length ? (
                <pre>{JSON.stringify(invoice.route_hints, null, 2)}</pre>
              ) : (
                <p className="hint">No private route hints included.</p>
              )}
            </div>
          </div>
        </details>
      </div>
    </section>
  );
}

function ResultSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="result-section">
      <h3>{title}</h3>
      {children}
    </section>
  );
}

function ResultField({
  label,
  value,
  secondary,
  tone,
}: {
  label: string;
  value: string;
  secondary?: string;
  tone?: "success" | "danger";
}) {
  return (
    <div className="result-field">
      <dt>{label}</dt>
      <dd className={tone ? `value-${tone}` : undefined}>{value}</dd>
      {secondary ? <span className="field-secondary">{secondary}</span> : null}
    </div>
  );
}

function Identifier({ label, value }: { label: string; value: string }) {
  return (
    <div className="identifier-row">
      <div>
        <span className="identifier-label">{label}</span>
        <code>{value}</code>
      </div>
      <CopyButton label={label} value={value} />
    </div>
  );
}

function CopyButton({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    await navigator.clipboard.writeText(value);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1_500);
  }

  return (
    <button
      className="copy-button"
      type="button"
      onClick={copy}
      aria-label={`Copy ${label}`}
    >
      {copied ? "Copied" : "Copy"}
    </button>
  );
}

function formatAmount(amountMsat: number | null) {
  if (amountMsat === null) {
    return { primary: "Amount not specified", secondary: undefined };
  }

  return {
    primary: `${(amountMsat / 1_000).toLocaleString()} sats`,
    secondary: `${amountMsat.toLocaleString()} msat`,
  };
}

function formatTimestamp(timestamp: number) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(timestamp * 1_000));
}

function formatDuration(seconds: number) {
  if (seconds % 86_400 === 0) return `${seconds / 86_400} days`;
  if (seconds % 3_600 === 0) return `${seconds / 3_600} hours`;
  if (seconds % 60 === 0) return `${seconds / 60} minutes`;
  return `${seconds} seconds`;
}

function formatRelativeExpiry(timestamp: number) {
  const seconds = timestamp - Math.floor(Date.now() / 1_000);
  const formatter = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

  if (Math.abs(seconds) >= 86_400) {
    return formatter.format(Math.round(seconds / 86_400), "day");
  }
  if (Math.abs(seconds) >= 3_600) {
    return formatter.format(Math.round(seconds / 3_600), "hour");
  }
  return formatter.format(Math.round(seconds / 60), "minute");
}
