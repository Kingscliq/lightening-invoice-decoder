import type { ValidationReport as Report } from "@/types/invoice";

const statuses = {
  passed: { label: "Passed", symbol: "✓" },
  warning: { label: "Warning", symbol: "!" },
  failed: { label: "Failed", symbol: "×" },
  info: { label: "Info", symbol: "i" },
  not_checked: { label: "Not checked", symbol: "—" },
};

export function ValidationReport({ report }: { report: Report }) {
  const warnings = report.checks.filter((check) => check.status === "warning").length;
  const failures = report.checks.filter((check) => check.status === "failed").length;
  return (
    <section className="validation-report" aria-label="Validation report">
      <header className="validation-heading">
        <div><p className="section-kicker">Understand the checks</p><h3>Validation report</h3></div>
        <span className="hint">{failures ? `${failures} failed` : warnings ? `${warnings} warning${warnings === 1 ? "" : "s"}` : "No failures or warnings"}</span>
      </header>
      <ul className="validation-checks">
        {report.checks.map((check) => {
          const status = statuses[check.status];
          return (
            <li key={check.id} className={`validation-check check-${check.status}`}>
              <span className="check-symbol" aria-hidden="true">{status.symbol}</span>
              <div><div className="check-heading"><strong>{check.label}</strong><span className="check-status">{status.label}</span></div>
                <p>{check.message}</p>
              </div>
            </li>
          );
        })}
      </ul>
      <p className="validation-scope">Checked {new Date(report.checked_at_unix * 1000).toLocaleString()}. These checks do not establish payment status, recipient identity, or route availability.</p>
    </section>
  );
}
