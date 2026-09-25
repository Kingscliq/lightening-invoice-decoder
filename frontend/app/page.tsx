import { InvoiceDecoder } from "@/components/invoice-decoder";

export default function Home() {
  return (
    <main className="page-shell">
      <section className="hero">
        <p className="eyebrow">Lightning Tool</p>
        <h1>Understand any BOLT11 invoice.</h1>
        <p className="lede">
          Paste a Lightning payment request to inspect its network, amount,
          expiry, payment hash, payee, signature, and private route hints.
        </p>
      </section>

      <InvoiceDecoder />
    </main>
  );
}

