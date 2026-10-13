import type { Metadata } from "next";
import Link from "next/link";
import { CommandBlock } from "@/components/command-block";

export const metadata: Metadata = {
  title: "Offline guide | Lightening Decoder",
  description: "Build and use the Lightning invoice decoder offline with terminal commands, QR import, JSON export, and Ratatui shortcuts.",
};

const sections = [
  ["setup", "Prepare your terminal"],
  ["build", "Build once"],
  ["tui", "Open the terminal UI"],
  ["decode", "Decode an invoice"],
  ["qr", "Read a QR image"],
  ["network", "Check the network"],
  ["export", "Export JSON"],
  ["help", "Command help"],
] as const;

const shortcuts = [
  ["Enter", "Decode the invoice or confirm a dialog"],
  ["F2", "Choose a built-in sample invoice"],
  ["F3", "Import a QR image using its local file path"],
  ["F4", "Cycle the expected network before decoding"],
  ["v", "Open or close the validation report after decoding"],
  ["Tab", "Switch between invoice details and JSON"],
  ["e", "Export JSON to a new file after decoding"],
  ["↑ / ↓", "Scroll through the result"],
  ["n / r", "Start another invoice after decoding"],
  ["Ctrl+U", "Clear the input or file path"],
  ["Esc", "Cancel a dialog; otherwise quit (returns from the validation report)"],
  ["Ctrl+C", "Quit from any screen"],
];

export default function DocsPage() {
  return (
    <main className="page-shell docs-shell">
      <nav className="app-nav" aria-label="Main navigation">
        <Link className="app-brand" href="/">Lightening Decoder</Link>
        <div><Link href="/">Decoder</Link><Link href="/docs">All guides</Link></div>
      </nav>
      <header className="docs-hero">
        <p className="eyebrow">The offline guide</p>
        <h1>Your terminal.<br />Your invoice.</h1>
        <p className="lede">Inspect Lightning invoices on your own computer. Build once, then decode text and QR images without an API or internet connection.</p>
        <div className="docs-badges"><span>Local decoding</span><span>QR import</span><span>JSON export</span></div>
      </header>
      <div className="docs-layout">
        <aside className="docs-sidebar">
          <nav aria-label="On this page">
            <p className="section-kicker">On this page</p>
            {sections.map(([id, title], index) => <a key={id} href={`#${id}`}><span>{String(index + 1).padStart(2, "0")}</span>{title}</a>)}
          </nav>
        </aside>
        <div className="docs-content">
          <div className="docs-note"><strong>Run these commands in your local terminal.</strong><p>Copy a command below, paste it into Terminal, and press Enter. This page is a guide; it does not execute commands on your computer. You need the project source and Rust with Cargo installed. The web decoder still uses the API.</p></div>

          <section id="setup" className="docs-section">
            <p className="section-kicker">01 · Setup</p><h2>Prepare your terminal</h2>
            <p>Open your project folder. Replace this example path with the location of your checkout.</p>
            <CommandBlock command={'cd /path/to/lightening-tool\nunset LIGHTNING_API_URL'} />
            <p><code>unset</code> removes a configured API URL from this terminal session. Omit <code>--api-url</code> to keep decoding local. These examples use macOS / Linux shell syntax.</p>
          </section>

          <section id="build" className="docs-section">
            <p className="section-kicker">02 · Build</p><h2>Build once. Use offline.</h2>
            <p>Build the terminal application from the project root. The first build may need internet access to download Rust dependencies.</p>
            <CommandBlock command="cargo build --release -p lightning-cli --bin bolt11-decoder" />
            <p>If all dependencies are already cached, you can build without network access:</p>
            <CommandBlock command="cargo build --offline --release -p lightning-cli --bin bolt11-decoder" />
            <p>Cargo’s <code>--offline</code> controls dependency downloads. The built decoder runs locally by default; no Docker, backend, or Polar instance is needed.</p>
          </section>

          <section id="tui" className="docs-section">
            <p className="section-kicker">03 · Interactive mode</p><h2>Open the terminal UI</h2>
            <p>Launch Ratatui, paste a complete invoice, and press Enter. For a quick demo, press F2 to select a built-in sample.</p>
            <CommandBlock command="./target/release/bolt11-decoder" />
            <div className="shortcut-table"><table><caption>Ratatui keyboard shortcuts</caption><thead><tr><th scope="col">Key</th><th scope="col">Action</th></tr></thead><tbody>{shortcuts.map(([key, action]) => <tr key={key}><td><kbd>{key}</kbd></td><td>{action}</td></tr>)}</tbody></table></div>
            <p>Use a terminal at least 72 × 24 characters; 100 × 32 is recommended. On a Mac, you may need to hold Fn while pressing a function key.</p>
          </section>

          <section id="decode" className="docs-section">
            <p className="section-kicker">04 · Invoice text</p><h2>Decode an invoice directly</h2>
            <p>Replace the placeholder with the complete BOLT11 string, keeping the quotes. From Polar’s invoice JSON, copy the <code>payment_request</code> value.</p>
            <CommandBlock command="./target/release/bolt11-decoder decode 'PASTE_COMPLETE_INVOICE_HERE'" />
            <p>The command prints invoice details and the validation report as JSON. Invalid invoices produce a diagnostic report and a nonzero exit status.</p>
          </section>

          <section id="qr" className="docs-section">
            <p className="section-kicker">05 · QR images</p><h2>Read an image from disk</h2>
            <p>Try the included coffee QR. The sample is expired, so an expiry warning is expected.</p>
            <CommandBlock command="./target/release/bolt11-decoder decode --image frontend/public/samples/coffee.png" />
            <p>Or replace the path below with your own image:</p>
            <CommandBlock command="./target/release/bolt11-decoder decode --image '/absolute/path/to/my-invoice.png'" />
            <p>Use one QR code per PNG, JPEG, or WebP image, up to 10 MB and 4096 × 4096 pixels. The image is read locally.</p>
          </section>

          <section id="network" className="docs-section">
            <p className="section-kicker">06 · Validation</p><h2>Check the expected network</h2>
            <p>For an invoice from Polar, expect regtest. A network mismatch adds a warning while keeping the invoice details available.</p>
            <CommandBlock command="./target/release/bolt11-decoder --expected-network regtest decode 'PASTE_COMPLETE_INVOICE_HERE'" />
            <p>Accepted values: <code>mainnet</code>, <code>testnet</code>, <code>signet</code>, and <code>regtest</code>. Leave the option out to detect the network without an expectation.</p>
          </section>

          <section id="export" className="docs-section">
            <p className="section-kicker">07 · Export</p><h2>Save the result as JSON</h2>
            <p>Redirect command output to a file in your current folder.</p>
            <CommandBlock command="./target/release/bolt11-decoder decode --image frontend/public/samples/coffee.png > decoded-invoice.json" />
            <p><code>&gt;</code> replaces an existing file with the same name. For an export that refuses to overwrite files, press <kbd>e</kbd> on a Ratatui result.</p>
          </section>

          <section id="help" className="docs-section">
            <p className="section-kicker">08 · Reference</p><h2>Explore command help</h2>
            <CommandBlock command={'./target/release/bolt11-decoder --help\n./target/release/bolt11-decoder decode --help'} />
            <div className="docs-note"><strong>Try a fully offline demo</strong><p>After building, disconnect from the internet and launch Ratatui. Use F2 for a sample or F3 for a saved QR. Signature and network checks still work. Expiry uses your computer’s clock; decoding cannot establish payment status, recipient identity, or route availability.</p></div>
          </section>
          <Link className="docs-back-link" href="/">← Back to the decoder</Link>
        </div>
      </div>
    </main>
  );
}
