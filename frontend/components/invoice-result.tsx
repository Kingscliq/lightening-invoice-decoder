import type { DecodedInvoice } from "@/types/invoice";

type Props = {
  invoice: DecodedInvoice;
};

export function InvoiceResult({ invoice }: Props) {
  const amount = invoice.amount_msat?.toLocaleString() ?? "Amount not specified";

  return (
    <section className="result-card" aria-live="polite">
      <header className="result-header">
        <h2>Decoded invoice</h2>
        <span className={`status${invoice.expired ? " expired" : ""}`}>
          {invoice.expired ? "Expired" : "Active"}
        </span>
      </header>

      <dl className="result-grid">
        <ResultField label="Network" value={invoice.network} />
        <ResultField label="Amount" value={`${amount} msat`} />
        <ResultField
          label="Description"
          value={invoice.description ?? "Not included"}
        />
        <ResultField
          label="Description hash"
          value={invoice.description_hash ?? "Not included"}
        />
        <ResultField label="Payment hash" value={invoice.payment_hash} />
        <ResultField label="Payee public key" value={invoice.payee_public_key} />
        <ResultField
          label="Created"
          value={formatTimestamp(invoice.created_at_unix)}
        />
        <ResultField
          label="Expires"
          value={formatTimestamp(invoice.expires_at_unix)}
        />
        <ResultField
          label="Expiry duration"
          value={`${invoice.expiry_seconds.toLocaleString()} seconds`}
        />
        <ResultField
          label="Signature"
          value={invoice.signature_valid ? "Valid" : "Invalid"}
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
    </section>
  );
}

function ResultField({ label, value }: { label: string; value: string }) {
  return (
    <div className="result-field">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function formatTimestamp(timestamp: number) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
  }).format(new Date(timestamp * 1_000));
}
