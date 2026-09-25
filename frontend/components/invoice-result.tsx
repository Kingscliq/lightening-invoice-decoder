import type { DecodedInvoice } from "@/types/invoice";

type Props = {
  invoice: DecodedInvoice;
};

export function InvoiceResult({ invoice }: Props) {
  // TODO(lesson 9): Design the result view field by field after the API
  // response has been implemented and understood.
  return (
    <section className="result-card" aria-live="polite">
      <h2>Decoded invoice</h2>
      <p className="hint">
        Result presentation is not implemented yet ({invoice.network}).
      </p>
    </section>
  );
}
