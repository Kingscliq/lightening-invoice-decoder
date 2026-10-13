"use client";

import { useEffect, useRef, useState } from "react";

export function CommandBlock({ command }: { command: string }) {
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => () => { if (timer.current) clearTimeout(timer.current); }, []);

  async function copy() {
    if (timer.current) clearTimeout(timer.current);
    try {
      await navigator.clipboard.writeText(command);
      setStatus("copied");
    } catch {
      setStatus("failed");
    }
    timer.current = setTimeout(() => setStatus("idle"), 3000);
  }

  return (
    <div className="command-block">
      <div className="command-toolbar">
        <span>Terminal</span>
        <button type="button" className="copy-button" onClick={copy} aria-label="Copy terminal command">
          {status === "copied" ? "Copied" : "Copy command"}
        </button>
      </div>
      <pre tabIndex={0}><code>{command}</code></pre>
      <span className="command-feedback" role="status">
        {status === "failed" ? "Could not copy. Select the command and copy it manually." : status === "copied" ? "Command copied to clipboard." : ""}
      </span>
    </div>
  );
}
