import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import { guides } from "../guides";

type Props = { params: Promise<{ guide: string }> };

export function generateStaticParams() {
  return guides.map((guide) => ({ guide: guide.slug }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { guide: slug } = await params;
  const guide = guides.find((entry) => entry.slug === slug);
  return guide ? { title: `${guide.title} | Lightening Decoder`, description: guide.description } : { title: "Guide not found" };
}

export default async function GuidePage({ params }: Props) {
  const { guide: slug } = await params;
  const guide = guides.find((entry) => entry.slug === slug);
  if (!guide) notFound();

  return (
    <main className="page-shell docs-shell">
      <nav className="app-nav" aria-label="Main navigation">
        <Link className="app-brand" href="/">Lightening Decoder</Link>
        <div><Link href="/">Decoder</Link><Link href="/docs">All guides</Link></div>
      </nav>
      <header className="docs-hero">
        <p className="eyebrow">Documentation</p>
        <h1>{guide.title}</h1>
        <p className="lede">{guide.description}</p>
      </header>
      <div className="docs-layout">
        <aside className="docs-sidebar">
          <nav aria-label="On this page">
            <p className="section-kicker">On this page</p>
            {guide.sections.map((section, index) => <a key={section.id} href={`#${section.id}`}><span>{String(index + 1).padStart(2, "0")}</span>{section.title}</a>)}
          </nav>
          <nav className="docs-other-guides" aria-label="Other guides">
            <p className="section-kicker">Keep exploring</p>
            <Link href="/docs/offline">Terminal and offline usage</Link>
            {guides.filter((entry) => entry.slug !== slug).map((entry) => <Link key={entry.slug} href={`/docs/${entry.slug}`}>{entry.title}</Link>)}
          </nav>
        </aside>
        <div className="docs-content">
          {guide.sections.map((section, index) => (
            <section key={section.id} id={section.id} className="docs-section">
              <p className="section-kicker">Step {String(index + 1).padStart(2, "0")}</p>
              <h2>{section.title}</h2>
              {section.content}
            </section>
          ))}
          <Link className="docs-back-link" href="/docs">← All guides</Link>
        </div>
      </div>
    </main>
  );
}
