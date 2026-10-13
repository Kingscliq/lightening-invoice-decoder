import type { Metadata } from "next";
import Link from "next/link";
import { guides } from "./guides";

export const metadata: Metadata = {
  title: "Docs | Lightening Decoder",
  description: "Guides for the Lightning invoice decoder: web, terminal, QR images, validation, API, local setup, and Docker.",
};

export default function DocsPage() {
  const allGuides = [
    ...guides.slice(0, 1),
    { slug: "offline", title: "Terminal and offline usage", description: "Build the CLI, explore Ratatui, decode locally, and export JSON." },
    ...guides.slice(1),
  ];
  return (
    <main className="page-shell docs-shell">
      <nav className="app-nav" aria-label="Main navigation">
        <Link className="app-brand" href="/">Lightening Decoder</Link>
        <div><Link href="/">Decoder</Link><Link href="/docs" aria-current="page">Docs</Link></div>
      </nav>
      <header className="docs-hero">
        <p className="eyebrow">Documentation</p>
        <h1>From first invoice<br />to your own setup.</h1>
        <p className="lede">Learn the web app, work in your terminal, connect to the API, and understand every check. Find step-by-step guides and commands you can copy.</p>
      </header>
      <section className="docs-start" aria-labelledby="start-title">
        <div><p className="section-kicker">Start here</p><h2 id="start-title">Inspect your first invoice</h2><p>Paste a payment request or try a sample. Read the summary, explore validation, and export the result.</p></div>
        <Link href="/docs/web">Web decoder guide <span aria-hidden="true">↗</span></Link>
      </section>
      <section aria-labelledby="guides-title">
        <h2 id="guides-title" className="docs-grid-title">Explore the guides</h2>
        <div className="docs-guide-grid">
          {allGuides.map((guide, index) => (
            <Link key={guide.slug} href={`/docs/${guide.slug}`} className="docs-guide-card">
              <div className="docs-card-meta"><span>{String(index + 1).padStart(2, "0")}</span><span aria-hidden="true">↗</span></div>
              <h3>{guide.title}</h3><p>{guide.description}</p>
            </Link>
          ))}
        </div>
      </section>
      <div className="docs-note docs-home-note"><strong>Commands run in your terminal.</strong><p>The guides provide copy buttons and explain where each command runs. They do not execute commands on your computer. You can run HTTP requests through Swagger UI once your API is available.</p></div>
    </main>
  );
}
