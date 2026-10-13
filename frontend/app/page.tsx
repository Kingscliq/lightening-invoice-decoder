import { InvoiceDecoder } from "@/components/invoice-decoder";
import Link from "next/link";

export default function Home() {
  return (
    <main className="page-shell">
      <nav className="app-nav" aria-label="Main navigation">
        <Link className="app-brand" href="/">Lightening Decoder</Link>
        <div><Link href="/" aria-current="page">Decoder</Link><Link href="/docs">Docs</Link></div>
      </nav>
      <section className="hero">
        <p className="eyebrow">Lightening Decoder</p>
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
